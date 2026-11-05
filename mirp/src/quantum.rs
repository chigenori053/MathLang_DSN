//! Bounded exact finite-dimensional quantum calculations in a named orthonormal basis.
//! Coefficients are Gaussian rationals; pure states and unitaries may carry one
//! positive common square-root scale, retaining normalization without rounding.
use crate::calculation::{CalculationAnswer as Answer, CalculationRequest as Request};
use crate::exact::ExactRational as R;
use num_rational::BigRational as Q;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactComplex {
    pub real: R,
    pub imaginary: R,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumBasis {
    pub id: String,
    pub labels: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PureState {
    pub basis_id: String,
    pub amplitudes: Vec<ExactComplex>,
    /// The physical ket is sqrt(squared_scale) times amplitudes (positive root).
    pub squared_scale: R,
}
pub type ComplexMatrix = Vec<Vec<ExactComplex>>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumOperator {
    pub basis_id: String,
    pub entries: ComplexMatrix,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum QuantumState {
    Pure(PureState),
    Mixed(QuantumOperator),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumProjector {
    pub label: String,
    pub operator: QuantumOperator,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitaryOperator {
    pub operator: QuantumOperator,
    pub squared_scale: R,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexRadical {
    pub coefficient: ExactComplex,
    pub radicand: R,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PsdCertificate {
    pub lower_triangular: ComplexMatrix,
    pub diagonal: Vec<R>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum QuantumOutcome {
    NormalizedState {
        state: PureState,
        original_norm_squared: R,
        norm_squared: R,
    },
    InnerProduct {
        value: ComplexRadical,
        overlap_probability: R,
    },
    Born {
        labels: Vec<String>,
        probabilities: Vec<R>,
        total_probability: R,
    },
    Expectation {
        value: R,
    },
    DensityMatrix {
        state: QuantumOperator,
        trace: R,
        purity: R,
        pure: bool,
        psd_certificate: PsdCertificate,
    },
    Unitary {
        state: QuantumState,
        normalization: R,
        purity: R,
    },
    Mix {
        state: QuantumOperator,
        alpha: R,
        trace: R,
        purity: R,
        psd_certificate: PsdCertificate,
        effect_changes: Vec<EffectChange>,
    },
    EffectScore {
        value: R,
        effect_certificate: PsdCertificate,
        complement_certificate: PsdCertificate,
    },
    Charpoly {
        coefficients: Vec<R>,
        trace: R,
        determinant: R,
    },
}
/// Score change of one effect E under a QUANTUM_MIX update (Tr(E rho) before and after).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectChange {
    pub before: R,
    pub after: R,
    pub new_state_score: R,
    pub absolute_change: R,
    pub bound_holds: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumResult {
    pub basis: QuantumBasis,
    pub outcome: QuantumOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct C {
    re: Q,
    im: Q,
}
impl C {
    fn one() -> Self {
        Self {
            re: Q::one(),
            im: Q::zero(),
        }
    }
    fn add(&self, b: &Self) -> Self {
        Self {
            re: &self.re + &b.re,
            im: &self.im + &b.im,
        }
    }
    fn sub(&self, b: &Self) -> Self {
        Self {
            re: &self.re - &b.re,
            im: &self.im - &b.im,
        }
    }
    fn mul(&self, b: &Self) -> Self {
        Self {
            re: &self.re * &b.re - &self.im * &b.im,
            im: &self.re * &b.im + &self.im * &b.re,
        }
    }
    fn scale(&self, s: &Q) -> Self {
        Self {
            re: &self.re * s,
            im: &self.im * s,
        }
    }
    fn conj(&self) -> Self {
        Self {
            re: self.re.clone(),
            im: -&self.im,
        }
    }
    fn abs2(&self) -> Q {
        &self.re * &self.re + &self.im * &self.im
    }
    fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
    fn inv(&self) -> Self {
        let n = self.abs2().recip();
        Self {
            re: &self.re * &n,
            im: -(&self.im * &n),
        }
    }
}
/// Maximum supported Hilbert-space dimension (extended from 8 to 32 for USM-QRM-001).
pub const MAX_DIMENSION: usize = 32;
type M = Vec<Vec<C>>;
fn invalid(message: &str) -> String {
    format!("CAPABILITY_INPUT_INVALID: {message}")
}
fn bad() -> String {
    "CAPABILITY_OUTPUT_INVALID: quantum verification failed".into()
}
fn real(c: C) -> Result<Q, String> {
    if !c.im.is_zero() {
        return Err(bad());
    }
    Ok(c.re)
}
fn sum(values: impl Iterator<Item = C>) -> C {
    values.fold(C::default(), |s, v| s.add(&v))
}
fn norm(a: &[C]) -> Q {
    a.iter().map(C::abs2).sum()
}
fn inner(a: &[C], b: &[C]) -> C {
    sum(a.iter().zip(b).map(|(a, b)| a.conj().mul(b)))
}
fn identity(n: usize) -> M {
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { C::one() } else { C::default() })
                .collect()
        })
        .collect()
}
fn adjoint(a: &M) -> M {
    (0..a.len())
        .map(|i| (0..a.len()).map(|j| a[j][i].conj()).collect())
        .collect()
}
fn matmul(a: &M, b: &M) -> M {
    (0..a.len())
        .map(|i| {
            (0..a.len())
                .map(|j| sum((0..a.len()).map(|k| a[i][k].mul(&b[k][j]))))
                .collect()
        })
        .collect()
}
fn matvec(a: &M, b: &[C]) -> Vec<C> {
    a.iter()
        .map(|row| sum(row.iter().zip(b).map(|(a, b)| a.mul(b))))
        .collect()
}
fn scaled(a: &M, s: &Q) -> M {
    a.iter()
        .map(|row| row.iter().map(|c| c.scale(s)).collect())
        .collect()
}
fn trace(a: &M) -> C {
    sum((0..a.len()).map(|i| a[i][i].clone()))
}
fn trace_product(a: &M, b: &M) -> C {
    sum((0..a.len()).flat_map(|i| (0..a.len()).map(move |j| a[i][j].mul(&b[j][i]))))
}
fn outer(a: &[C], scale: &Q) -> M {
    a.iter()
        .map(|x| a.iter().map(|y| x.mul(&y.conj()).scale(scale)).collect())
        .collect()
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.chars().count() <= 128
}
fn check_basis(b: &QuantumBasis) -> Result<usize, String> {
    let n = b.labels.len();
    if !(1..=MAX_DIMENSION).contains(&n)
        || !label(&b.id)
        || b.labels.iter().any(|s| !label(s))
        || b.labels.iter().collect::<BTreeSet<_>>().len() != n
    {
        return Err(invalid(
            "require a named orthonormal complex basis with 1..=32 distinct nonempty labels",
        ));
    }
    Ok(n)
}
fn read_r(r: &R, output: bool) -> Result<Q, String> {
    if output {
        r.canonical()
    } else {
        r.read(80)
    }
}
fn read_c(c: &ExactComplex, output: bool) -> Result<C, String> {
    Ok(C {
        re: read_r(&c.real, output)?,
        im: read_r(&c.imaginary, output)?,
    })
}
fn read_m(a: &ComplexMatrix, n: usize, output: bool) -> Result<M, String> {
    if a.len() != n || a.iter().any(|row| row.len() != n) {
        return Err(invalid(
            "matrix must be square and match the basis dimension",
        ));
    }
    a.iter()
        .map(|row| row.iter().map(|c| read_c(c, output)).collect())
        .collect()
}
fn basis_id(id: &str, b: &QuantumBasis) -> Result<(), String> {
    if id != b.id {
        return Err(invalid(
            "basis IDs must match; no basis transform is inferred",
        ));
    }
    Ok(())
}
fn read_operator(a: &QuantumOperator, b: &QuantumBasis, output: bool) -> Result<M, String> {
    basis_id(&a.basis_id, b)?;
    read_m(&a.entries, b.labels.len(), output)
}
fn read_pure(
    a: &PureState,
    b: &QuantumBasis,
    normalized: bool,
    output: bool,
) -> Result<(Vec<C>, Q), String> {
    basis_id(&a.basis_id, b)?;
    if a.amplitudes.len() != b.labels.len() {
        return Err(invalid("ket dimension differs from the basis"));
    }
    let a_values = a
        .amplitudes
        .iter()
        .map(|c| read_c(c, output))
        .collect::<Result<Vec<_>, _>>()?;
    let scale = read_r(&a.squared_scale, output)?;
    let length = norm(&a_values);
    if !scale.is_positive() || length.is_zero() {
        return Err(invalid(
            "a pure state requires positive squared_scale and a nonzero vector",
        ));
    }
    if normalized && &length * &scale != Q::one() {
        return Err(invalid(
            "state norm must be exactly one; use QUANTUM_NORMALIZE explicitly",
        ));
    }
    Ok((a_values, scale))
}
fn hermitian(a: &M) -> Result<(), String> {
    if *a != adjoint(a) {
        return Err(invalid(
            "matrix must be Hermitian (equal to its conjugate transpose)",
        ));
    }
    Ok(())
}
// A zero PSD pivot requires its entire remaining row/column to vanish.
fn ldl(a: &M) -> Result<(M, Vec<Q>), String> {
    hermitian(a)?;
    let n = a.len();
    let mut residual = a.clone();
    let mut lower = identity(n);
    let mut diagonal = vec![Q::zero(); n];
    for k in 0..n {
        let d = residual[k][k].re.clone();
        if d.is_negative() {
            return Err(invalid("density matrix is not positive semidefinite"));
        }
        diagonal[k] = d.clone();
        if d.is_zero() {
            if (k + 1..n).any(|i| !residual[i][k].is_zero()) {
                return Err(invalid("zero PSD pivot has nonzero coupling"));
            }
            continue;
        }
        let reciprocal = d.recip();
        for i in k + 1..n {
            lower[i][k] = residual[i][k].scale(&reciprocal);
        }
        for i in k + 1..n {
            for j in k + 1..n {
                residual[i][j] =
                    residual[i][j].sub(&lower[i][k].mul(&lower[j][k].conj()).scale(&d));
            }
        }
    }
    Ok((lower, diagonal))
}
fn check_density(a: &M) -> Result<(), String> {
    hermitian(a)?;
    if trace(a) != C::one() {
        return Err(invalid("density matrix trace must be exactly one"));
    }
    ldl(a)?;
    Ok(())
}
#[derive(Clone)]
enum State {
    Pure { a: Vec<C>, scale: Q },
    Mixed { rho: M },
}
impl State {
    fn density(&self) -> M {
        match self {
            Self::Pure { a, scale } => outer(a, scale),
            Self::Mixed { rho } => rho.clone(),
        }
    }
    fn purity(&self) -> Q {
        let rho = self.density();
        trace_product(&rho, &rho).re
    }
}
fn read_state(s: &QuantumState, b: &QuantumBasis, output: bool) -> Result<State, String> {
    match s {
        QuantumState::Pure(s) => {
            let (a, scale) = read_pure(s, b, true, output)?;
            Ok(State::Pure { a, scale })
        }
        QuantumState::Mixed(s) => {
            let rho = read_operator(s, b, output)?;
            check_density(&rho)?;
            Ok(State::Mixed { rho })
        }
    }
}
fn projections(p: &[QuantumProjector], b: &QuantumBasis) -> Result<Vec<(String, M)>, String> {
    let n = b.labels.len();
    if p.is_empty()
        || p.len() > n
        || p.iter().any(|p| !label(&p.label))
        || p.iter().map(|p| &p.label).collect::<BTreeSet<_>>().len() != p.len()
    {
        return Err(invalid(
            "require 1..=dimension distinct labeled nonzero projectors",
        ));
    }
    let mut all = vec![];
    let mut total = vec![vec![C::default(); n]; n];
    for projector in p {
        let a = read_operator(&projector.operator, b, false)?;
        hermitian(&a)?;
        if matmul(&a, &a) != a || !trace(&a).re.is_positive() {
            return Err(invalid(
                "measurement must consist of nonzero Hermitian idempotent projectors",
            ));
        }
        for (_, previous) in &all {
            if matmul(previous, &a).iter().flatten().any(|c| !c.is_zero()) {
                return Err(invalid(
                    "measurement projectors must be mutually orthogonal",
                ));
            }
        }
        for i in 0..n {
            for j in 0..n {
                total[i][j] = total[i][j].add(&a[i][j]);
            }
        }
        all.push((projector.label.clone(), a));
    }
    if total != identity(n) {
        return Err(invalid(
            "measurement projectors must sum to the identity (complete measurement)",
        ));
    }
    Ok(all)
}
fn unitary(u: &UnitaryOperator, b: &QuantumBasis) -> Result<(M, Q), String> {
    let a = read_operator(&u.operator, b, false)?;
    let scale = u.squared_scale.read(80)?;
    if !scale.is_positive() {
        return Err(invalid("unitary squared_scale must be positive"));
    }
    let adj = adjoint(&a);
    let id = identity(a.len());
    if scaled(&matmul(&adj, &a), &scale) != id || scaled(&matmul(&a, &adj), &scale) != id {
        return Err(invalid("unitary must satisfy both U†U=I and UU†=I exactly"));
    }
    Ok((a, scale))
}
fn basis(request: &Request) -> Result<&QuantumBasis, String> {
    match request {
        Request::QuantumNormalize { basis, .. }
        | Request::QuantumInnerProduct { basis, .. }
        | Request::QuantumBorn { basis, .. }
        | Request::QuantumExpectation { basis, .. }
        | Request::QuantumDensityMatrix { basis, .. }
        | Request::QuantumUnitary { basis, .. }
        | Request::QuantumMix { basis, .. }
        | Request::QuantumEffectScore { basis, .. }
        | Request::QuantumCharpoly { basis, .. } => Ok(basis),
        _ => Err(invalid("not a quantum operation")),
    }
}
enum Data {
    Normalize {
        a: Vec<C>,
        scale: Q,
    },
    Inner {
        a: Vec<C>,
        b: Vec<C>,
        sa: Q,
        sb: Q,
    },
    Born {
        state: State,
        projectors: Vec<(String, M)>,
    },
    Expectation {
        state: State,
        observable: M,
    },
    Density {
        state: State,
    },
    Unitary {
        state: State,
        operator: M,
        scale: Q,
    },
    Mix {
        state: State,
        new_state: State,
        alpha: Q,
        effects: Vec<M>,
    },
    EffectScore {
        state: State,
        effect: M,
    },
    Charpoly {
        a: M,
    },
}
fn sub_m(a: &M, b: &M) -> M {
    a.iter()
        .zip(b)
        .map(|(r, s)| r.iter().zip(s).map(|(x, y)| x.sub(y)).collect())
        .collect()
}
fn add_m(a: &M, b: &M) -> M {
    a.iter()
        .zip(b)
        .map(|(r, s)| r.iter().zip(s).map(|(x, y)| x.add(y)).collect())
        .collect()
}
/// An effect satisfies 0 <= E <= I: Hermitian, E PSD and I - E PSD (both by exact LDL).
fn read_effect(e: &QuantumOperator, b: &QuantumBasis) -> Result<M, String> {
    let a = read_operator(e, b, false)?;
    hermitian(&a)?;
    if ldl(&a).is_err() || ldl(&sub_m(&identity(a.len()), &a)).is_err() {
        return Err(invalid(
            "effect must satisfy 0 <= E <= I (E and I-E positive semidefinite)",
        ));
    }
    Ok(a)
}
fn mix(rho: &M, sigma: &M, alpha: &Q) -> M {
    add_m(&scaled(rho, &(Q::one() - alpha)), &scaled(sigma, alpha))
}
/// Faddeev-LeVerrier: coefficients c_0..c_n of det(xI - A), c_n = 1.
fn charpoly(a: &M) -> Vec<C> {
    let n = a.len();
    let mut c = vec![C::default(); n + 1];
    c[n] = C::one();
    let mut mk = vec![vec![C::default(); n]; n];
    for k in 1..=n {
        let mut next = matmul(a, &mk);
        for (i, row) in next.iter_mut().enumerate() {
            row[i] = row[i].add(&c[n - k + 1]);
        }
        mk = next;
        let t = trace(&matmul(a, &mk));
        c[n - k] = t.scale(&-Q::from_integer((k as i64).into()).recip());
    }
    c
}
/// Determinant by exact Gaussian elimination (independent of Faddeev-LeVerrier).
fn determinant(a: &M) -> C {
    let n = a.len();
    let mut m = a.clone();
    let mut det = C::one();
    for k in 0..n {
        let Some(p) = (k..n).find(|&i| !m[i][k].is_zero()) else {
            return C::default();
        };
        if p != k {
            m.swap(p, k);
            det = det.scale(&-Q::one());
        }
        det = det.mul(&m[k][k]);
        let inv = m[k][k].inv();
        for i in k + 1..n {
            let f = m[i][k].mul(&inv);
            if f.is_zero() {
                continue;
            }
            let pivot_row = m[k].clone();
            for (x, p) in m[i].iter_mut().zip(&pivot_row).skip(k) {
                *x = x.sub(&f.mul(p));
            }
        }
    }
    det
}
fn out_certificate(a: &M) -> Result<PsdCertificate, String> {
    let (lower, diagonal) = ldl(a)?;
    Ok(PsdCertificate {
        lower_triangular: out_m(&lower),
        diagonal: diagonal.iter().map(R::from).collect(),
    })
}
fn score(state: &State, e: &M) -> Result<Q, String> {
    match state {
        State::Pure { a, scale } => Ok(real(inner(a, &matvec(e, a)))? * scale),
        State::Mixed { rho } => real(trace_product(e, rho)),
    }
}
fn prepare(request: &Request) -> Result<(&QuantumBasis, Data), String> {
    let b = basis(request)?;
    check_basis(b)?;
    let data = match request {
        Request::QuantumNormalize { state, .. } => {
            let (a, scale) = read_pure(state, b, false, false)?;
            Data::Normalize { a, scale }
        }
        Request::QuantumInnerProduct { left, right, .. } => {
            let (a, sa) = read_pure(left, b, true, false)?;
            let (b, sb) = read_pure(right, b, true, false)?;
            Data::Inner { a, b, sa, sb }
        }
        Request::QuantumBorn {
            state, projectors, ..
        } => Data::Born {
            state: read_state(state, b, false)?,
            projectors: projections(projectors, b)?,
        },
        Request::QuantumExpectation {
            state, observable, ..
        } => {
            let observable = read_operator(observable, b, false)?;
            hermitian(&observable)?;
            Data::Expectation {
                state: read_state(state, b, false)?,
                observable,
            }
        }
        Request::QuantumDensityMatrix { state, .. } => Data::Density {
            state: read_state(state, b, false)?,
        },
        Request::QuantumUnitary {
            state, unitary: u, ..
        } => {
            let (operator, scale) = unitary(u, b)?;
            Data::Unitary {
                state: read_state(state, b, false)?,
                operator,
                scale,
            }
        }
        Request::QuantumMix {
            state,
            new_state,
            alpha,
            effects,
            ..
        } => {
            let alpha = alpha.read(80)?;
            if alpha.is_negative() || alpha > Q::one() {
                return Err(invalid("mixing weight alpha must satisfy 0 <= alpha <= 1"));
            }
            if effects.len() > 64 {
                return Err(invalid("at most 64 effects per QUANTUM_MIX request"));
            }
            Data::Mix {
                state: read_state(state, b, false)?,
                new_state: read_state(new_state, b, false)?,
                alpha,
                effects: effects
                    .iter()
                    .map(|e| read_effect(e, b))
                    .collect::<Result<_, _>>()?,
            }
        }
        Request::QuantumEffectScore { state, effect, .. } => Data::EffectScore {
            state: read_state(state, b, false)?,
            effect: read_effect(effect, b)?,
        },
        Request::QuantumCharpoly { state, .. } => Data::Charpoly {
            a: read_state(state, b, false)?.density(),
        },
        _ => return Err(invalid("not a quantum operation")),
    };
    Ok((b, data))
}
pub(crate) fn validate(request: &Request) -> Result<(), String> {
    prepare(request).map(|_| ())
}
fn out_c(c: &C) -> ExactComplex {
    ExactComplex {
        real: (&c.re).into(),
        imaginary: (&c.im).into(),
    }
}
fn out_m(a: &M) -> ComplexMatrix {
    a.iter()
        .map(|row| row.iter().map(out_c).collect())
        .collect()
}
fn out_operator(a: &M, b: &QuantumBasis) -> QuantumOperator {
    QuantumOperator {
        basis_id: b.id.clone(),
        entries: out_m(a),
    }
}
fn out_pure(a: &[C], scale: &Q, b: &QuantumBasis) -> PureState {
    PureState {
        basis_id: b.id.clone(),
        amplitudes: a.iter().map(out_c).collect(),
        squared_scale: scale.into(),
    }
}
fn probability(q: &Q) -> Result<(), String> {
    if q.is_negative() || *q > Q::one() {
        return Err(bad());
    }
    Ok(())
}

pub(crate) fn execute(request: &Request) -> Result<(Answer, Json), String> {
    let (basis, data) = prepare(request)?;
    let outcome = match data {
        Data::Normalize { a, scale } => {
            let length = norm(&a);
            let original = &length * &scale;
            QuantumOutcome::NormalizedState {
                state: out_pure(&a, &length.recip(), basis),
                original_norm_squared: (&original).into(),
                norm_squared: (&Q::one()).into(),
            }
        }
        Data::Inner { a, b, sa, sb } => {
            let coefficient = inner(&a, &b);
            let radicand = sa * sb;
            let overlap = coefficient.abs2() * &radicand;
            QuantumOutcome::InnerProduct {
                value: ComplexRadical {
                    coefficient: out_c(&coefficient),
                    radicand: (&radicand).into(),
                },
                overlap_probability: (&overlap).into(),
            }
        }
        Data::Born { state, projectors } => {
            let rho = state.density();
            let mut labels = vec![];
            let mut probabilities = vec![];
            let mut total = Q::zero();
            for (label, p) in projectors {
                let probability = real(trace(&matmul(&rho, &p)))?;
                total += &probability;
                labels.push(label);
                probabilities.push((&probability).into());
            }
            QuantumOutcome::Born {
                labels,
                probabilities,
                total_probability: (&total).into(),
            }
        }
        Data::Expectation { state, observable } => {
            let value = real(trace(&matmul(&state.density(), &observable)))?;
            QuantumOutcome::Expectation {
                value: (&value).into(),
            }
        }
        Data::Density { state } => {
            let rho = state.density();
            let (lower, diagonal) = ldl(&rho)?;
            let purity = real(trace(&matmul(&rho, &rho)))?;
            QuantumOutcome::DensityMatrix {
                state: out_operator(&rho, basis),
                trace: (&real(trace(&rho))?).into(),
                pure: purity == Q::one(),
                purity: (&purity).into(),
                psd_certificate: PsdCertificate {
                    lower_triangular: out_m(&lower),
                    diagonal: diagonal.iter().map(R::from).collect(),
                },
            }
        }
        Data::Unitary {
            state,
            operator,
            scale,
        } => {
            let transformed = match state {
                State::Pure { a, scale: s } => {
                    QuantumState::Pure(out_pure(&matvec(&operator, &a), &(s * &scale), basis))
                }
                State::Mixed { rho } => QuantumState::Mixed(out_operator(
                    &scaled(
                        &matmul(&matmul(&operator, &rho), &adjoint(&operator)),
                        &scale,
                    ),
                    basis,
                )),
            };
            let state = read_state(&transformed, basis, true)?;
            let purity = state.purity();
            QuantumOutcome::Unitary {
                state: transformed,
                normalization: (&Q::one()).into(),
                purity: (&purity).into(),
            }
        }
        Data::Mix {
            state,
            new_state,
            alpha,
            effects,
        } => {
            let rho = state.density();
            let sigma = new_state.density();
            let out = mix(&rho, &sigma, &alpha);
            let psd_certificate = out_certificate(&out)?;
            let mut effect_changes = vec![];
            for e in &effects {
                let before = real(trace_product(e, &rho))?;
                let after = real(trace_product(e, &out))?;
                let new_score = real(trace_product(e, &sigma))?;
                let change = (&after - &before).abs();
                effect_changes.push(EffectChange {
                    before: (&before).into(),
                    after: (&after).into(),
                    new_state_score: (&new_score).into(),
                    bound_holds: change <= alpha,
                    absolute_change: (&change).into(),
                });
            }
            QuantumOutcome::Mix {
                trace: (&real(trace(&out))?).into(),
                purity: (&real(trace_product(&out, &out))?).into(),
                state: out_operator(&out, basis),
                alpha: (&alpha).into(),
                psd_certificate,
                effect_changes,
            }
        }
        Data::EffectScore { state, effect } => QuantumOutcome::EffectScore {
            value: (&score(&state, &effect)?).into(),
            effect_certificate: out_certificate(&effect)?,
            complement_certificate: out_certificate(&sub_m(&identity(effect.len()), &effect))?,
        },
        Data::Charpoly { a } => {
            let c = charpoly(&a);
            let n = a.len();
            let coefficients = c.iter().cloned().map(real).collect::<Result<Vec<_>, _>>()?;
            let mut det = coefficients[0].clone();
            if n % 2 == 1 {
                det = -det;
            }
            QuantumOutcome::Charpoly {
                trace: (&-coefficients[n - 1].clone()).into(),
                determinant: (&det).into(),
                coefficients: coefficients.iter().map(R::from).collect(),
            }
        }
    };
    let result = QuantumResult {
        basis: basis.clone(),
        outcome,
    };
    let metadata = json!({"executor":"Rust/num-rational finite quantum mechanics","arithmetic":"exact Gaussian rationals with positive common square-root scale","inner_product_convention":"CONJUGATE_FIRST","basis_id":basis.id,"dimension":basis.labels.len(),"measurement_model":"complete orthogonal projective measurement"});
    Ok((
        Answer::Quantum {
            result: Box::new(result),
        },
        metadata,
    ))
}

fn certify_psd(rho: &M, certificate: &PsdCertificate) -> Result<(), String> {
    let n = rho.len();
    let lower = read_m(&certificate.lower_triangular, n, true)?;
    if certificate.diagonal.len() != n {
        return Err(bad());
    }
    let diagonal = certificate
        .diagonal
        .iter()
        .map(R::canonical)
        .collect::<Result<Vec<_>, _>>()?;
    if diagonal.iter().any(Q::is_negative)
        || (0..n).any(|i| lower[i][i] != C::one() || (i + 1..n).any(|j| !lower[i][j].is_zero()))
    {
        return Err(bad());
    }
    let mut ld = lower.clone();
    for row in &mut ld {
        for (c, d) in row.iter_mut().zip(&diagonal) {
            *c = c.scale(d);
        }
    }
    if matmul(&ld, &adjoint(&lower)) != *rho {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn verify(request: &Request, answer: &Answer) -> Result<Vec<String>, String> {
    let (basis, data) = prepare(request)?;
    let Answer::Quantum { result } = answer else {
        return Err(bad());
    };
    if result.basis != *basis {
        return Err(bad());
    }
    let mut checks = vec![
        "EXACT_COMPLEX_ARITHMETIC_CONJUGATE_FIRST",
        "COMMON_NAMED_ORTHONORMAL_BASIS_AND_DIMENSIONS",
    ];
    let verification = (|| -> Result<(), String> {
        match (data, &result.outcome) {
            (
                Data::Normalize { a, scale },
                QuantumOutcome::NormalizedState {
                    state,
                    original_norm_squared,
                    norm_squared,
                },
            ) => {
                let (out, s) = read_pure(state, basis, true, true)?;
                if out != a
                    || s * norm(&a) != Q::one()
                    || original_norm_squared.canonical()? != scale * norm(&a)
                    || norm_squared.canonical()? != Q::one()
                {
                    return Err(bad());
                }
                checks.push("EXPLICIT_POSITIVE_ROOT_NORMALIZATION_PRESERVES_PHASE_AND_DIRECTION");
            }
            (
                Data::Inner { a, b, sa, sb },
                QuantumOutcome::InnerProduct {
                    value,
                    overlap_probability,
                },
            ) => {
                let coefficient = read_c(&value.coefficient, true)?;
                let radicand = value.radicand.canonical()?;
                let overlap = overlap_probability.canonical()?;
                probability(&overlap)?;
                if coefficient != inner(&b, &a).conj()
                    || radicand != sa * sb
                    || overlap != coefficient.abs2() * radicand
                {
                    return Err(bad());
                }
                checks.push("CONJUGATE_SYMMETRY_AND_CAUCHY_SCHWARZ_OVERLAP_BOUND");
            }
            (
                Data::Born { state, projectors },
                QuantumOutcome::Born {
                    labels,
                    probabilities,
                    total_probability,
                },
            ) => {
                if labels
                    != &projectors
                        .iter()
                        .map(|(s, _)| s.clone())
                        .collect::<Vec<_>>()
                    || probabilities.len() != projectors.len()
                {
                    return Err(bad());
                }
                let mut total = Q::zero();
                for (p, (_, projector)) in probabilities.iter().zip(&projectors) {
                    let p = p.canonical()?;
                    probability(&p)?;
                    let expected = match &state {
                        State::Pure { a, scale } => norm(&matvec(projector, a)) * scale,
                        State::Mixed { rho } => real(trace_product(projector, rho))?,
                    };
                    if p != expected {
                        return Err(bad());
                    }
                    total += p;
                }
                if total != Q::one() || total_probability.canonical()? != total {
                    return Err(bad());
                }
                checks.extend([
                    "HERMITIAN_IDEMPOTENT_ORTHOGONAL_COMPLETE_PROJECTORS",
                    "BORN_PROJECTED_NORMS_OR_TRACE_CONTRACTIONS",
                    "EACH_PROBABILITY_IN_UNIT_INTERVAL_AND_TOTAL_ONE",
                ]);
            }
            (Data::Expectation { state, observable }, QuantumOutcome::Expectation { value }) => {
                let expected = match &state {
                    State::Pure { a, scale } => real(inner(a, &matvec(&observable, a)))? * scale,
                    State::Mixed { rho } => real(trace_product(&observable, rho))?,
                };
                if value.canonical()? != expected {
                    return Err(bad());
                }
                checks.extend([
                    "HERMITIAN_OBSERVABLE",
                    "REAL_EXPECTATION_KET_OR_DENSITY_TRACE_IDENTITY",
                ]);
            }
            (
                Data::Density { state: input },
                QuantumOutcome::DensityMatrix {
                    state,
                    trace: t,
                    purity,
                    pure,
                    psd_certificate,
                },
            ) => {
                let rho = read_operator(state, basis, true)?;
                hermitian(&rho)?;
                certify_psd(&rho, psd_certificate)?;
                match &input {
                    State::Pure { a, .. } => {
                        if matvec(&rho, a) != *a {
                            return Err(bad());
                        }
                    }
                    State::Mixed { rho: original } => {
                        if rho != *original {
                            return Err(bad());
                        }
                    }
                }
                let tr = real(trace(&rho))?;
                let purity_value = real(trace_product(&rho, &rho))?;
                probability(&purity_value)?;
                if tr != Q::one()
                    || t.canonical()? != tr
                    || purity.canonical()? != purity_value
                    || *pure != (purity_value == Q::one())
                {
                    return Err(bad());
                }
                checks.extend([
                    "DENSITY_HERMITICITY_AND_TRACE_ONE",
                    "PSD_CERTIFICATE_L_D_L_ADJOINT_RECONSTRUCTION",
                    "DENSITY_PURITY_AND_PURE_STATE_EIGENVECTOR_IDENTITY",
                ]);
            }
            (
                Data::Unitary {
                    state: input,
                    operator,
                    scale,
                },
                QuantumOutcome::Unitary {
                    state,
                    normalization,
                    purity,
                },
            ) => {
                let output = read_state(state, basis, true)?;
                let adj = adjoint(&operator);
                match (&input, &output) {
                    (State::Pure { a, scale: si }, State::Pure { a: b, scale: so }) => {
                        if *so != si * &scale
                            || matvec(&adj, b)
                                .iter()
                                .map(|c| c.scale(&scale))
                                .collect::<Vec<_>>()
                                != *a
                        {
                            return Err(bad());
                        }
                    }
                    (State::Mixed { rho: a }, State::Mixed { rho: b }) => {
                        if scaled(&matmul(&matmul(&adj, b), &operator), &scale) != *a {
                            return Err(bad());
                        }
                    }
                    _ => return Err(bad()),
                }
                let purity_out = output.purity();
                probability(&purity_out)?;
                if normalization.canonical()? != Q::one()
                    || purity.canonical()? != purity_out
                    || purity_out != input.purity()
                {
                    return Err(bad());
                }
                checks.extend([
                    "BOTH_SIDED_UNITARITY",
                    "ADJOINT_INVERSE_RECOVERS_INPUT",
                    "STATE_NORMALIZATION_TRACE_PSD_AND_PURITY_PRESERVED",
                ]);
            }
            (
                Data::Mix {
                    state: input,
                    new_state,
                    alpha,
                    effects,
                },
                QuantumOutcome::Mix {
                    state,
                    alpha: a_out,
                    trace: t,
                    purity,
                    psd_certificate,
                    effect_changes,
                },
            ) => {
                let rho = input.density();
                let sigma = new_state.density();
                let out = read_operator(state, basis, true)?;
                // independent recomputation entrywise: rho'_ij = rho_ij + alpha (sigma_ij - rho_ij)
                for i in 0..out.len() {
                    for j in 0..out.len() {
                        let expected = rho[i][j].add(&sigma[i][j].sub(&rho[i][j]).scale(&alpha));
                        if out[i][j] != expected {
                            return Err(bad());
                        }
                    }
                }
                hermitian(&out)?;
                certify_psd(&out, psd_certificate)?;
                let tr = real(trace(&out))?;
                let pu = real(trace_product(&out, &out))?;
                probability(&pu)?;
                if tr != Q::one()
                    || t.canonical()? != tr
                    || purity.canonical()? != pu
                    || a_out.canonical()? != alpha
                    || effect_changes.len() != effects.len()
                {
                    return Err(bad());
                }
                for (e, ch) in effects.iter().zip(effect_changes) {
                    let before = score(&input, e)?;
                    let new_score = score(&new_state, e)?;
                    let after = real(trace_product(e, &out))?;
                    let change = ch.absolute_change.canonical()?;
                    probability(&before)?;
                    probability(&after)?;
                    probability(&new_score)?;
                    if ch.before.canonical()? != before
                        || ch.after.canonical()? != after
                        || ch.new_state_score.canonical()? != new_score
                        || &after - &before != (&new_score - &before) * &alpha
                        || change != (&after - &before).abs()
                        || !ch.bound_holds
                        || change > alpha
                    {
                        return Err(bad());
                    }
                }
                checks.extend([
                    "ALPHA_IN_UNIT_INTERVAL_AND_EFFECTS_BETWEEN_ZERO_AND_IDENTITY",
                    "CONVEX_COMBINATION_RECOMPUTED_ENTRYWISE",
                    "MIXED_STATE_HERMITIAN_TRACE_ONE_PSD_CERTIFICATE",
                    "EFFECT_SCORE_CHANGE_IDENTITY_AND_ALPHA_BOUND",
                ]);
            }
            (
                Data::EffectScore { state, effect },
                QuantumOutcome::EffectScore {
                    value,
                    effect_certificate,
                    complement_certificate,
                },
            ) => {
                certify_psd(&effect, effect_certificate)?;
                certify_psd(
                    &sub_m(&identity(effect.len()), &effect),
                    complement_certificate,
                )?;
                let v = value.canonical()?;
                probability(&v)?;
                let expected = match &state {
                    State::Pure { a, scale } => real(inner(a, &matvec(&effect, a)))? * scale,
                    State::Mixed { rho } => real(trace(&matmul(rho, &effect)))?,
                };
                if v != expected {
                    return Err(bad());
                }
                checks.extend([
                    "EFFECT_AND_COMPLEMENT_PSD_CERTIFICATES",
                    "SCORE_KET_OR_DENSITY_TRACE_IN_UNIT_INTERVAL",
                ]);
            }
            (
                Data::Charpoly { a },
                QuantumOutcome::Charpoly {
                    coefficients,
                    trace: t,
                    determinant: d,
                },
            ) => {
                let n = a.len();
                let c = coefficients
                    .iter()
                    .map(R::canonical)
                    .collect::<Result<Vec<_>, _>>()?;
                if c.len() != n + 1 || c[n] != Q::one() {
                    return Err(bad());
                }
                // Cayley-Hamilton: sum_k c_k A^k = 0 (Horner)
                let mut acc = vec![vec![C::default(); n]; n];
                for k in (0..=n).rev() {
                    acc = matmul(&acc, &a);
                    for (i, row) in acc.iter_mut().enumerate() {
                        row[i] = row[i].add(&C {
                            re: c[k].clone(),
                            im: Q::zero(),
                        });
                    }
                }
                if acc.iter().flatten().any(|x| !x.is_zero()) {
                    return Err(bad());
                }
                let tr = real(trace(&a))?;
                let mut det = real(determinant(&a))?;
                if t.canonical()? != tr || c[n - 1] != -tr.clone() {
                    return Err(bad());
                }
                if d.canonical()? != det {
                    return Err(bad());
                }
                if n % 2 == 1 {
                    det = -det;
                }
                if c[0] != det {
                    return Err(bad());
                }
                checks.extend([
                    "CAYLEY_HAMILTON_EXACT",
                    "TRACE_AND_INDEPENDENT_GAUSSIAN_DETERMINANT_MATCH_COEFFICIENTS",
                ]);
            }
            _ => return Err(bad()),
        }
        Ok(())
    })();
    verification.map_err(|_| bad())?;
    Ok(checks.into_iter().map(str::to_owned).collect())
}
pub(crate) fn explain(request: &Request, answer: &Answer) -> Vec<String> {
    let Answer::Quantum { result } = answer else {
        unreachable!()
    };
    let mut steps=vec![format!("{}次元の正規直交複素基底{}で計算しました。内積は第一引数を共役にする規約です。有理数複素係数と正の共通平方根係数を厳密に保持します。",result.basis.labels.len(),result.basis.id)];
    steps.push(match request {
        Request::QuantumNormalize{..}=>"非零状態のノルム二乗を求め、正の平方根で明示的に正規化しました。相対位相と方向を保持し、正規化後のノルムが厳密に1であることを確認しました。".into(),
        Request::QuantumInnerProduct{..}=>"⟨ψ|φ⟩=Σconj(ψᵢ)φᵢを求め、共役対称性と重なりの確率|⟨ψ|φ⟩|²∈[0,1]を検算しました。".into(),
        Request::QuantumBorn{..}=>"完全で互いに直交するエルミート射影Pₖを検査し、Born則pₖ=Tr(ρPₖ)を計算しました。純粋状態では||Pₖψ||²、混合状態では別の成分和で検算し、各確率が0〜1、総和が1であることを確認しました。".into(),
        Request::QuantumExpectation{..}=>"観測量A=A†を検査し、期待値Tr(ρA)を求めました。純粋状態では⟨ψ|A|ψ⟩、混合状態では成分の縮約で照合し、虚部が厳密に0であることを検査しました。".into(),
        Request::QuantumDensityMatrix{..}=>"純粋状態はρ=|ψ⟩⟨ψ|から構成し、混合状態は入力密度行列を検査しました。ρ=ρ†、Trρ=1、ρ=LDL†かつD≥0の証明書、純度Trρ²を検算しました。対角成分の非負性だけでは半正定値と判定しません。".into(),
        Request::QuantumUnitary{..}=>"U†U=UU†=Iを検査し、純粋状態にはU|ψ⟩、混合状態にはUρU†を適用しました。U†による逆変換と正規化・トレース・半正定値性・純度の保存を検算しました。".into(),
        Request::QuantumMix{..}=>"0≤α≤1と各効果演算子0⪯E⪯Iを検査し、ρ'=(1-α)ρ+ασを厳密計算しました。成分ごとの再計算、ρ'=ρ†・Trρ'=1・LDL†証明書、各EについてTr(E(ρ'-ρ))=α(Tr(Eσ)-Tr(Eρ))と|ΔTr(Eρ)|≤αを検算しました。".into(),
        Request::QuantumEffectScore{..}=>"EとI-Eの両方にLDL†証明書を構成して0⪯E⪯Iを検査し、意味観測スコアTr(Eρ)を計算しました。純粋状態では⟨ψ|E|ψ⟩、混合状態では別の積順序で照合し、0〜1であることを確認しました。".into(),
        Request::QuantumCharpoly{..}=>"Faddeev–LeVerrier法で特性多項式det(xI-ρ)の係数を厳密計算し、Cayley–Hamiltonの恒等式p(ρ)=0、トレース、独立なGauss消去による行列式で検算しました。固有値・エントロピー自体は無理数となり得るため計算しません。".into(),
        _=>unreachable!(),
    });
    steps.push("これは有限次元モデル内の厳密算術の検算です。入力された状態・観測量が現実の系を表すことの検証は含みません。".into());
    steps
}
