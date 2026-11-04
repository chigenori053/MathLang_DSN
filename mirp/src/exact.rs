//! Exact rational statistics and certified comparisons of finite radical sums.
use crate::calculation::{CalculationAnswer as Answer, CalculationRequest as Request};
use num_bigint::BigInt;
use num_rational::BigRational as Q;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

const INPUT_DIGITS: usize = 80;
const OUTPUT_DIGITS: usize = 20_000;
const MAX_BITS: u32 = 4096;

fn invalid(message: &str) -> String {
    format!("CAPABILITY_INPUT_INVALID: {message}")
}
fn bad_output() -> String {
    "CAPABILITY_OUTPUT_INVALID: exact verification failed".into()
}

fn decimal<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Decimal {
        Text(String),
        Integer(i64),
    }
    Ok(match Decimal::deserialize(deserializer)? {
        Decimal::Text(s) => s,
        Decimal::Integer(n) => n.to_string(),
    })
}

fn integer(text: &str, limit: usize) -> Result<BigInt, String> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || digits.len() > limit || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("expected a bounded decimal integer"));
    }
    text.parse().map_err(|_| invalid("invalid integer"))
}

/// Decimal strings preserve arbitrarily large exact results through JSON/MIRP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactRational {
    #[serde(deserialize_with = "decimal")]
    pub numerator: String,
    #[serde(deserialize_with = "decimal")]
    pub denominator: String,
}
impl ExactRational {
    pub fn new(numerator: i64, denominator: i64) -> Self {
        Self {
            numerator: numerator.to_string(),
            denominator: denominator.to_string(),
        }
    }
    pub(crate) fn read(&self, limit: usize) -> Result<Q, String> {
        let n = integer(&self.numerator, limit)?;
        let d = integer(&self.denominator, limit)?;
        if d <= BigInt::zero() {
            return Err(invalid("denominator must be positive"));
        }
        Ok(Q::new(n, d))
    }
    pub(crate) fn canonical(&self) -> Result<Q, String> {
        let q = self.read(OUTPUT_DIGITS).map_err(|_| bad_output())?;
        if Self::from(&q) != *self {
            return Err(bad_output());
        }
        Ok(q)
    }
}
impl From<&Q> for ExactRational {
    fn from(value: &Q) -> Self {
        Self {
            numerator: value.numer().to_string(),
            denominator: value.denom().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadicalTerm {
    pub coefficient: ExactRational,
    pub radicand: i64,
}

/// Σ coefficient * sqrt(radicand); a rational term uses radicand=1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactExpression {
    pub terms: Vec<RadicalTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExactOrdering {
    Less,
    Equal,
    Greater,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqrtFloor {
    pub radicand: i64,
    pub floor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonCertificate {
    pub precision_bits: u32,
    pub roots: Vec<SqrtFloor>,
    pub lower: ExactRational,
    pub upper: ExactRational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactComparison {
    pub left: ExactExpression,
    pub right: ExactExpression,
    pub ordering: ExactOrdering,
    pub equal: bool,
    pub certificate: ComparisonCertificate,
}

type Terms = BTreeMap<i64, Q>;
type RootFactors = BTreeMap<i64, (i64, i64)>;

fn data(values: &[ExactRational]) -> Result<Vec<Q>, String> {
    if values.is_empty() || values.len() > 64 {
        return Err(invalid("require 1..=64 rational values"));
    }
    values.iter().map(|v| v.read(INPUT_DIGITS)).collect()
}

fn weights(values: &[ExactRational]) -> Result<(Vec<Q>, Q), String> {
    let values = data(values)?;
    if values.iter().any(Q::is_negative) {
        return Err(invalid("weights must be nonnegative"));
    }
    let sum: Q = values.iter().sum();
    if sum.is_zero() {
        return Err(invalid("total weight must be positive"));
    }
    Ok((values, sum))
}

fn expression(value: &ExactExpression) -> Result<(), String> {
    if value.terms.len() > 32 {
        return Err(invalid("at most 32 radical terms per expression"));
    }
    for term in &value.terms {
        term.coefficient.read(INPUT_DIGITS)?;
        if !(0..=200_000_000).contains(&term.radicand) {
            return Err(invalid("real radicand must be in 0..=200000000"));
        }
    }
    Ok(())
}

pub(crate) fn validate(request: &Request) -> Result<(), String> {
    match request {
        Request::DataCovariance { x, y } => {
            data(x)?;
            data(y)?;
            if x.len() != y.len() {
                return Err(invalid("paired datasets must have equal length"));
            }
        }
        Request::NormalizeWeights { weights: w } => {
            weights(w)?;
        }
        Request::DiscreteExpectation {
            values,
            weights: w,
            normalize_weights,
        } => {
            data(values)?;
            let (_, sum) = weights(w)?;
            if values.len() != w.len() {
                return Err(invalid("values and weights must have equal length"));
            }
            if !normalize_weights && sum != Q::one() {
                return Err(invalid(
                    "probabilities must sum exactly to one; request normalization explicitly",
                ));
            }
        }
        Request::NormalizeExact { expression: e } => expression(e)?,
        Request::ExactEqual { left, right } | Request::ExactCompare { left, right } => {
            expression(left)?;
            expression(right)?;
        }
        _ => return Err(invalid("not an extended calculation")),
    }
    Ok(())
}

fn sum(values: &[Q]) -> Q {
    values.iter().sum()
}
fn mean(values: &[Q]) -> Q {
    sum(values) / BigInt::from(values.len())
}
fn fractions(values: &[Q]) -> Vec<ExactRational> {
    values.iter().map(ExactRational::from).collect()
}

pub(crate) fn radical_factors(n: i64) -> (i64, i64) {
    if n == 0 {
        return (0, 1);
    }
    let (mut remaining, mut outside, mut inside, mut prime) = (n, 1, 1, 2);
    while prime * prime <= remaining {
        let mut exponent = 0;
        while remaining % prime == 0 {
            remaining /= prime;
            exponent += 1;
        }
        outside *= prime.pow(exponent / 2);
        if exponent % 2 == 1 {
            inside *= prime;
        }
        prime += 1;
    }
    (outside, inside * remaining)
}

fn normalize(value: &ExactExpression, roots: &RootFactors) -> Result<Terms, String> {
    let mut terms = Terms::new();
    for t in &value.terms {
        let &(a, b) = roots.get(&t.radicand).ok_or_else(bad_output)?;
        *terms.entry(b).or_insert_with(Q::zero) +=
            t.coefficient.read(INPUT_DIGITS)? * BigInt::from(a);
    }
    terms.retain(|_, q| !q.is_zero());
    Ok(terms)
}
fn as_expression(terms: &Terms) -> ExactExpression {
    ExactExpression {
        terms: terms
            .iter()
            .map(|(&radicand, q)| RadicalTerm {
                coefficient: q.into(),
                radicand,
            })
            .collect(),
    }
}
fn difference(left: &Terms, right: &Terms) -> Terms {
    let mut terms = left.clone();
    for (&r, q) in right {
        *terms.entry(r).or_insert_with(Q::zero) -= q;
    }
    terms.retain(|_, q| !q.is_zero());
    terms
}

fn reason_roots(expressions: &[&ExactExpression]) -> Result<(RootFactors, String), String> {
    let inputs: BTreeMap<_, _> = expressions
        .iter()
        .flat_map(|e| e.terms.iter().map(|t| (t.radicand, ())))
        .collect();
    if inputs.is_empty() {
        return Ok((BTreeMap::new(), "[]".into()));
    }
    let source = format!(
        "[{}]",
        inputs
            .keys()
            .map(|n| format!("JuniorHigh::JSimplifySqrt({n})"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let output = crate::session::run_reason_source(&source, &[])
        .map_err(|e| format!("CAPABILITY_EXECUTION_FAILED: {e}"))?;
    let values = output.as_array().ok_or_else(bad_output)?;
    if values.len() != inputs.len() {
        return Err(bad_output());
    }
    let mut roots = BTreeMap::new();
    for (&n, value) in inputs.keys().zip(values) {
        let a = value["outside"].as_i64().ok_or_else(bad_output)?;
        let b = value["inside"].as_i64().ok_or_else(bad_output)?;
        if value["status"] != "CALCULATED" || (a, b) != radical_factors(n) {
            return Err(bad_output());
        }
        roots.insert(n, (a, b));
    }
    Ok((roots, source))
}

fn interval(terms: &Terms, bits: u32, roots: &[SqrtFloor]) -> Result<(Q, Q), String> {
    if bits > MAX_BITS || roots.len() != terms.keys().filter(|r| **r != 1).count() {
        return Err(bad_output());
    }
    let scale = BigInt::one() << bits;
    let (mut lower, mut upper) = (Q::zero(), Q::zero());
    let mut bounds = roots.iter();
    for (&r, q) in terms {
        if r == 1 {
            lower += q;
            upper += q;
            continue;
        }
        let bound = bounds.next().ok_or_else(bad_output)?;
        if bound.radicand != r {
            return Err(bad_output());
        }
        let floor = integer(&bound.floor, 2000).map_err(|_| bad_output())?;
        let squared = &floor * &floor;
        let target = BigInt::from(r) << (2 * bits);
        let next = &floor + 1;
        if floor.is_negative() || squared > target || &next * &next <= target {
            return Err(bad_output());
        }
        let lo = Q::new(floor, scale.clone());
        let hi = if squared == target {
            lo.clone()
        } else {
            Q::new(next, scale.clone())
        };
        if q.is_negative() {
            lower += q * &hi;
            upper += q * &lo;
        } else {
            lower += q * &lo;
            upper += q * &hi;
        }
    }
    Ok((lower, upper))
}

fn compare(left: Terms, right: Terms) -> Result<ExactComparison, String> {
    let terms = difference(&left, &right);
    let mut certificate = ComparisonCertificate {
        precision_bits: 0,
        roots: vec![],
        lower: ExactRational::new(0, 1),
        upper: ExactRational::new(0, 1),
    };
    let mut ordering = ExactOrdering::Equal;
    if !terms.is_empty() {
        let mut bits: u32 = 8;
        loop {
            let roots: Vec<_> = terms
                .keys()
                .filter(|r| **r != 1)
                .map(|&r| SqrtFloor {
                    radicand: r,
                    floor: (BigInt::from(r) << (2 * bits)).sqrt().to_string(),
                })
                .collect();
            let (lower, upper) = interval(&terms, bits, &roots)?;
            if lower.is_positive() || upper.is_negative() {
                ordering = if lower.is_positive() {
                    ExactOrdering::Greater
                } else {
                    ExactOrdering::Less
                };
                certificate = ComparisonCertificate {
                    precision_bits: bits,
                    roots,
                    lower: (&lower).into(),
                    upper: (&upper).into(),
                };
                break;
            }
            if bits == MAX_BITS {
                return Err("RESOURCE_LIMIT: exact comparison needs more than 4096 bits; no approximate verdict returned".into());
            }
            bits *= 2;
        }
    }
    Ok(ExactComparison {
        left: as_expression(&left),
        right: as_expression(&right),
        equal: ordering == ExactOrdering::Equal,
        ordering,
        certificate,
    })
}

pub(crate) fn execute(request: &Request) -> Result<(Answer, Json), String> {
    validate(request)?;
    let mut metadata =
        json!({"executor":"Rust/num-rational + num-bigint", "arithmetic":"exact rational"});
    let answer = match request {
        Request::DataCovariance { x, y } => {
            let (x, y) = (data(x)?, data(y)?);
            let n = BigInt::from(x.len());
            let numerator =
                x.iter().zip(&y).map(|(a, b)| a * b).sum::<Q>() * &n - sum(&x) * sum(&y);
            Answer::Covariance {
                count: x.len(),
                mean_x: (&mean(&x)).into(),
                mean_y: (&mean(&y)).into(),
                population_covariance: (&(&numerator / (&n * &n))).into(),
                sample_covariance: if x.len() == 1 {
                    None
                } else {
                    Some((&(numerator / (&n * (&n - BigInt::one())))).into())
                },
            }
        }
        Request::NormalizeWeights { weights: w } => {
            let (w, total) = weights(w)?;
            Answer::Weights {
                total_weight: (&total).into(),
                probabilities: fractions(&w.iter().map(|q| q / &total).collect::<Vec<_>>()),
            }
        }
        Request::DiscreteExpectation {
            values,
            weights: w,
            normalize_weights,
        } => {
            let (w, total) = weights(w)?;
            let probabilities: Vec<_> = w.iter().map(|q| q / &total).collect();
            let expectation: Q = data(values)?
                .iter()
                .zip(&probabilities)
                .map(|(x, p)| x * p)
                .sum();
            Answer::Expectation {
                value: (&expectation).into(),
                total_weight: (&total).into(),
                probabilities: fractions(&probabilities),
                normalized: *normalize_weights,
            }
        }
        Request::NormalizeExact { expression } => {
            let (roots, source) = reason_roots(&[expression])?;
            metadata = json!({"executor":"ReasonScript + Rust/num-rational", "expression":source, "arithmetic":"exact radical normal form"});
            Answer::ExactExpression {
                expression: as_expression(&normalize(expression, &roots)?),
            }
        }
        Request::ExactEqual { left, right } | Request::ExactCompare { left, right } => {
            let (roots, source) = reason_roots(&[left, right])?;
            metadata = json!({"executor":"ReasonScript + Rust/num-bigint", "expression":source, "arithmetic":"certified rational bounds; no floating point"});
            Answer::ExactComparison {
                comparison: compare(normalize(left, &roots)?, normalize(right, &roots)?)?,
            }
        }
        _ => return Err(invalid("not an extended calculation")),
    };
    Ok((answer, metadata))
}

fn verified_distribution(
    w: &[ExactRational],
    total: &ExactRational,
    p: &[ExactRational],
) -> Result<Vec<Q>, String> {
    let (w, input_total) = weights(w)?;
    let total = total.canonical()?;
    if total != input_total || w.len() != p.len() {
        return Err(bad_output());
    }
    let p: Vec<_> = p
        .iter()
        .map(ExactRational::canonical)
        .collect::<Result<_, _>>()?;
    if sum(&p) != Q::one()
        || w.iter()
            .zip(&p)
            .any(|(w, p)| p.is_negative() || p * &total != *w)
    {
        return Err(bad_output());
    }
    Ok(p)
}

fn independently_normalize(e: &ExactExpression) -> Result<Terms, String> {
    let roots = e
        .terms
        .iter()
        .map(|t| (t.radicand, radical_factors(t.radicand)))
        .collect();
    normalize(e, &roots)
}

pub(crate) fn verify(request: &Request, answer: &Answer) -> Result<Vec<String>, String> {
    validate(request)?;
    let checks = match (request, answer) {
        (
            Request::DataCovariance { x, y },
            Answer::Covariance {
                count,
                mean_x,
                mean_y,
                population_covariance,
                sample_covariance,
            },
        ) => {
            let (x, y) = (data(x)?, data(y)?);
            let (mx, my) = (mean_x.canonical()?, mean_y.canonical()?);
            if *count != x.len() || mx != mean(&x) || my != mean(&y) {
                return Err(bad_output());
            }
            let centered: Q = x.iter().zip(&y).map(|(a, b)| (a - &mx) * (b - &my)).sum();
            let n = BigInt::from(x.len());
            if population_covariance.canonical()? != &centered / &n {
                return Err(bad_output());
            }
            if x.len() == 1 {
                if sample_covariance.is_some() {
                    return Err(bad_output());
                }
            } else if sample_covariance
                .as_ref()
                .ok_or_else(bad_output)?
                .canonical()?
                != centered / (&n - BigInt::one())
            {
                return Err(bad_output());
            }
            vec![
                "PAIRED_LENGTH_AND_EXACT_MEANS",
                "CENTERED_PRODUCT_COVARIANCE",
                "N_AND_N_MINUS_ONE_DEFINEDNESS",
            ]
        }
        (
            Request::NormalizeWeights { weights },
            Answer::Weights {
                total_weight,
                probabilities,
            },
        ) => {
            verified_distribution(weights, total_weight, probabilities)?;
            vec![
                "NONNEGATIVE_WEIGHTS_AND_POSITIVE_TOTAL",
                "EXACT_PROPORTIONS",
                "PROBABILITY_SUM_EXACTLY_ONE",
            ]
        }
        (
            Request::DiscreteExpectation {
                values,
                weights,
                normalize_weights,
            },
            Answer::Expectation {
                value,
                total_weight,
                probabilities,
                normalized,
            },
        ) => {
            verified_distribution(weights, total_weight, probabilities)?;
            let expectation = value.canonical()?;
            let centered: Q = data(values)?
                .iter()
                .zip(data(weights)?)
                .map(|(x, w)| w * (x - &expectation))
                .sum();
            if !centered.is_zero() || normalized != normalize_weights {
                return Err(bad_output());
            }
            vec![
                "EXACT_PROBABILITY_NORMALIZATION",
                "WEIGHTED_CENTERED_RESIDUAL_ZERO",
                "EXPLICIT_NORMALIZATION_MODE",
            ]
        }
        (
            Request::NormalizeExact { expression },
            Answer::ExactExpression { expression: result },
        ) => {
            if as_expression(&independently_normalize(expression)?) != *result {
                return Err(bad_output());
            }
            vec![
                "INDEPENDENT_PRIME_FACTORIZATION",
                "CANONICAL_COEFFICIENTS_AND_SORTED_RADICALS",
            ]
        }
        (
            Request::ExactEqual { left, right } | Request::ExactCompare { left, right },
            Answer::ExactComparison { comparison: c },
        ) => {
            let (left, right) = (
                independently_normalize(left)?,
                independently_normalize(right)?,
            );
            if c.left != as_expression(&left)
                || c.right != as_expression(&right)
                || c.equal != (c.ordering == ExactOrdering::Equal)
            {
                return Err(bad_output());
            }
            let terms = difference(&left, &right);
            let cert = &c.certificate;
            if terms.is_empty() {
                if c.ordering != ExactOrdering::Equal
                    || cert.precision_bits != 0
                    || !cert.roots.is_empty()
                    || !cert.lower.canonical()?.is_zero()
                    || !cert.upper.canonical()?.is_zero()
                {
                    return Err(bad_output());
                }
            } else {
                let (lower, upper) = interval(&terms, cert.precision_bits, &cert.roots)?;
                if cert.lower.canonical()? != lower
                    || cert.upper.canonical()? != upper
                    || match c.ordering {
                        ExactOrdering::Less => !upper.is_negative(),
                        ExactOrdering::Greater => !lower.is_positive(),
                        ExactOrdering::Equal => true,
                    }
                {
                    return Err(bad_output());
                }
            }
            vec![
                "INDEPENDENT_RADICAL_NORMALIZATION",
                "INTEGER_SQUARE_ROOT_BOUND_CERTIFICATE",
                "EXACT_SIGN_OR_ZERO_WITHOUT_ROUNDING",
            ]
        }
        _ => return Err(bad_output()),
    };
    Ok(checks.into_iter().map(str::to_owned).collect())
}

pub(crate) fn explain(request: &Request, answer: &Answer) -> Vec<String> {
    match (request,answer) {
        (Request::DataCovariance {..},Answer::Covariance {count,..}) => vec!["対応する組の長さを検査し、母共分散=(nΣxy−ΣxΣy)/n²、不偏共分散=(nΣxy−ΣxΣy)/(n(n−1))を既約分数で計算しました。".into(),format!("標本数は{count}。中心化した積の和Σ(x−平均x)(y−平均y)で独立に検算しました。n=1の不偏共分散は未定義でnullです。")],
        (Request::NormalizeWeights {..},_) => vec!["重みが非負で総和Wが正であることを検査し、各確率をpᵢ=wᵢ/Wとして正規化しました。".into(),"Σpᵢ=1とpᵢW=wᵢを有理数で検算しました。丸めや負の重みの切り捨ては行いません。".into()],
        (Request::DiscreteExpectation {normalize_weights,..},_) => vec![if *normalize_weights {"明示指定に従って非負の重みを正規化し、E[X]=Σpᵢxᵢを計算しました。".into()}else{"入力を確率として扱い、合計が厳密に1であることを検査してE[X]=Σpᵢxᵢを計算しました。".into()},"確率の整合性とΣwᵢ(xᵢ−E[X])=0を独立に検算しました。推測による正規化は行いません。".into()],
        (Request::NormalizeExact {..},_) => vec!["各主平方根を既存ReasonScriptで簡約し、有理数係数の同類項を統合して根号内の値順に並べました。空の項列は厳密な0です。".into(),"素因数分解から正規形を別に再構成し、係数・符号・分母を検算しました。".into()],
        (_,Answer::ExactComparison {comparison}) => vec![format!("両辺を正規化した結果、順序は{:?}、等値判定は{}です。",comparison.ordering,comparison.equal),if comparison.equal {"差のすべての係数が厳密に0になることを確認しました。".into()} else {format!("差の符号を{}ビットの有理数区間で確定しました。各平方根の境界は整数の二乗不等式で検証し、小数の近似一致は使っていません。",comparison.certificate.precision_bits)}],
        _=>unreachable!(),
    }
}
