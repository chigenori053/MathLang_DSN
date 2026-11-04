//! Exact affine-plane calculations. Membership, rank and root counts are checked
//! independently before the common Knowledge runtime records results/evidence.
use crate::calculation::{CalculationAnswer as Answer, CalculationRequest as Request};
use crate::exact::{radical_factors, ExactRational as R};
use num_rational::BigRational as Q;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::cmp::Ordering;

pub type Point2 = Vec<R>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line2 {
    pub a: R,
    pub b: R,
    pub c: R,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circle2 {
    pub center: Point2,
    pub radius: R,
}

/// rational + coefficient * sqrt(radicand), with a nonnegative rational radicand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuadraticSurd {
    pub rational: R,
    pub coefficient: R,
    pub radicand: R,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactPoint2 {
    pub coordinates: Vec<QuadraticSurd>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriangleDegeneracy {
    None,
    RepeatedVertex,
    Collinear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntersectionKind {
    Crossing,
    Parallel,
    Secant,
    Tangent,
    PointIntersection,
    Disjoint,
    Coincident,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntersectionResult {
    pub relation: IntersectionKind,
    pub points: Vec<ExactPoint2>,
    pub infinite: bool,
    pub input_degeneracies: Vec<String>,
}

type V2 = [Q; 2];
type Coefficients = [Q; 3];
type Circle = (V2, Q);

fn invalid(message: &str) -> String {
    format!("CAPABILITY_INPUT_INVALID: {message}")
}
fn bad() -> String {
    "CAPABILITY_OUTPUT_INVALID: geometry verification failed".into()
}
fn q(n: i64) -> Q {
    Q::from_integer(n.into())
}
fn point(v: &[R]) -> Result<V2, String> {
    if v.len() != 2 {
        return Err(invalid(
            "plane coordinates/vectors require exactly two components",
        ));
    }
    Ok([v[0].read(80)?, v[1].read(80)?])
}
fn line(v: &Line2) -> Result<Coefficients, String> {
    let a = v.a.read(80)?;
    let b = v.b.read(80)?;
    let c = v.c.read(80)?;
    if a.is_zero() && b.is_zero() {
        return Err(invalid("degenerate line: a and b cannot both be zero"));
    }
    Ok([a, b, c])
}
fn circle(v: &Circle2) -> Result<Circle, String> {
    let center = point(&v.center)?;
    let radius = v.radius.read(80)?;
    if radius.is_negative() {
        return Err(invalid(
            "circle radius must be nonnegative; zero denotes a point",
        ));
    }
    Ok((center, radius))
}
fn canonical_line(v: &Coefficients) -> Coefficients {
    let pivot = if !v[0].is_zero() { &v[0] } else { &v[1] };
    [&v[0] / pivot, &v[1] / pivot, &v[2] / pivot]
}
fn dot(a: &V2, b: &V2) -> Q {
    &a[0] * &b[0] + &a[1] * &b[1]
}
fn sub(a: &V2, b: &V2) -> V2 {
    [&a[0] - &b[0], &a[1] - &b[1]]
}
fn norm(a: &V2) -> Q {
    dot(a, a)
}
fn line_value(l: &Coefficients, p: &V2) -> Q {
    &l[0] * &p[0] + &l[1] * &p[1] + &l[2]
}
fn square_root(v: &Q) -> Option<Q> {
    if v.is_negative() {
        return None;
    }
    let n = v.numer().sqrt();
    let d = v.denom().sqrt();
    if &n * &n == *v.numer() && &d * &d == *v.denom() {
        Some(Q::new(n, d))
    } else {
        None
    }
}
fn root_parts(v: &Q) -> (Q, Q) {
    if let Some(root) = square_root(v) {
        return (root, Q::one());
    }
    if let (Some(n), Some(d)) = (v.numer().to_i64(), v.denom().to_i64()) {
        if (1..=200_000_000).contains(&n) && (1..=200_000_000).contains(&d) {
            let (a, b) = radical_factors(n);
            let (c, e) = radical_factors(d);
            return (Q::new(a.into(), (c * e).into()), q(b * e));
        }
    }
    (Q::one(), v.clone())
}
impl QuadraticSurd {
    fn make(mut a: Q, mut b: Q, mut r: Q) -> Self {
        if !b.is_zero() && !r.is_zero() {
            let (factor, rad) = root_parts(&r);
            b *= factor;
            r = rad;
            if r.is_one() {
                a += &b;
                b = Q::zero();
            }
        } else {
            b = Q::zero();
        }
        if b.is_zero() {
            r = Q::zero();
        }
        Self {
            rational: (&a).into(),
            coefficient: (&b).into(),
            radicand: (&r).into(),
        }
    }
    fn read(&self, field: &Q) -> Result<(Q, Q), String> {
        let a = self.rational.canonical()?;
        let b = self.coefficient.canonical()?;
        let r = self.radicand.canonical()?;
        if b.is_zero() {
            if !r.is_zero() {
                return Err(bad());
            }
        } else if r != *field || !r.is_positive() || square_root(&r).is_some() {
            return Err(bad());
        }
        Ok((a, b))
    }
}
fn exact_point(p: &V2) -> ExactPoint2 {
    ExactPoint2 {
        coordinates: p
            .iter()
            .map(|v| QuadraticSurd::make(v.clone(), Q::zero(), Q::zero()))
            .collect(),
    }
}
fn intersection(
    relation: IntersectionKind,
    points: Vec<ExactPoint2>,
    flags: Vec<String>,
) -> IntersectionResult {
    let infinite = relation == IntersectionKind::Coincident;
    IntersectionResult {
        relation,
        points,
        infinite,
        input_degeneracies: flags,
    }
}
fn circle_flags(c: &Circle, name: &str) -> Vec<String> {
    if c.1.is_zero() {
        vec![format!("{name}:ZERO_RADIUS_POINT")]
    } else {
        vec![]
    }
}
fn line_line(a: &Coefficients, b: &Coefficients) -> IntersectionResult {
    let det = &a[0] * &b[1] - &b[0] * &a[1];
    if det.is_zero() {
        let same = &a[0] * &b[2] == &b[0] * &a[2] && &a[1] * &b[2] == &b[1] * &a[2];
        return intersection(
            if same {
                IntersectionKind::Coincident
            } else {
                IntersectionKind::Parallel
            },
            vec![],
            vec![],
        );
    }
    let p = [
        (&a[1] * &b[2] - &b[1] * &a[2]) / &det,
        (&a[2] * &b[0] - &b[2] * &a[0]) / det,
    ];
    intersection(IntersectionKind::Crossing, vec![exact_point(&p)], vec![])
}
fn line_circle(l: &Coefficients, c: &Circle) -> IntersectionResult {
    let l = canonical_line(l);
    let n = &l[0] * &l[0] + &l[1] * &l[1];
    let s = line_value(&l, &c.0);
    let h = &c.1 * &c.1 * &n - &s * &s;
    let flags = circle_flags(c, "circle");
    if h.is_negative() {
        return intersection(IntersectionKind::Disjoint, vec![], flags);
    }
    let base = [&c.0[0] - &s * &l[0] / &n, &c.0[1] - &s * &l[1] / &n];
    if h.is_zero() {
        return intersection(
            if c.1.is_zero() {
                IntersectionKind::PointIntersection
            } else {
                IntersectionKind::Tangent
            },
            vec![exact_point(&base)],
            flags,
        );
    }
    let direction = [-&l[1] / &n, &l[0] / &n];
    // The first nonzero direction component fixes lexicographic point order.
    let positive = direction
        .iter()
        .find(|v| !v.is_zero())
        .unwrap()
        .is_positive();
    let signs = if positive { [-1, 1] } else { [1, -1] };
    let points = signs
        .into_iter()
        .map(|sign| ExactPoint2 {
            coordinates: (0..2)
                .map(|i| QuadraticSurd::make(base[i].clone(), &direction[i] * q(sign), h.clone()))
                .collect(),
        })
        .collect();
    intersection(IntersectionKind::Secant, points, flags)
}
fn radical_axis(a: &Circle, b: &Circle) -> Coefficients {
    [
        q(2) * (&b.0[0] - &a.0[0]),
        q(2) * (&b.0[1] - &a.0[1]),
        norm(&a.0) - norm(&b.0) - &a.1 * &a.1 + &b.1 * &b.1,
    ]
}
fn circle_circle(a: &Circle, b: &Circle) -> IntersectionResult {
    let mut flags = circle_flags(a, "first_circle");
    flags.extend(circle_flags(b, "second_circle"));
    if a.0 == b.0 {
        return if a.1 == b.1 {
            if a.1.is_zero() {
                intersection(
                    IntersectionKind::PointIntersection,
                    vec![exact_point(&a.0)],
                    flags,
                )
            } else {
                intersection(IntersectionKind::Coincident, vec![], flags)
            }
        } else {
            intersection(IntersectionKind::Disjoint, vec![], flags)
        };
    }
    let mut result = line_circle(&radical_axis(a, b), a);
    result.input_degeneracies = flags;
    if result.points.len() == 1 && (a.1.is_zero() || b.1.is_zero()) {
        result.relation = IntersectionKind::PointIntersection;
    }
    result
}
fn triangle_degeneracy(a: &V2, b: &V2, c: &V2, area: &Q) -> TriangleDegeneracy {
    if a == b || b == c || c == a {
        TriangleDegeneracy::RepeatedVertex
    } else if area.is_zero() {
        TriangleDegeneracy::Collinear
    } else {
        TriangleDegeneracy::None
    }
}

pub(crate) fn validate(request: &Request) -> Result<(), String> {
    match request {
        Request::PlaneDistance { a, b } => {
            point(a)?;
            point(b)?;
        }
        Request::PlaneDotProduct { u, v } | Request::PlaneOrthogonal { u, v } => {
            point(u)?;
            point(v)?;
        }
        Request::TriangleArea { a, b, c } => {
            point(a)?;
            point(b)?;
            point(c)?;
        }
        Request::LineLineIntersections { first, second } => {
            line(first)?;
            line(second)?;
        }
        Request::LineCircleIntersections { line: l, circle: c } => {
            line(l)?;
            circle(c)?;
        }
        Request::CircleCircleIntersections { first, second } => {
            circle(first)?;
            circle(second)?;
        }
        _ => return Err(invalid("not a plane geometry operation")),
    }
    Ok(())
}

pub(crate) fn execute(request: &Request) -> Result<(Answer, Json), String> {
    validate(request)?;
    let answer = match request {
        Request::PlaneDistance { a, b } => {
            let (a, b) = (point(a)?, point(b)?);
            let squared = norm(&sub(&a, &b));
            Answer::PlaneDistance {
                squared_distance: (&squared).into(),
                distance: QuadraticSurd::make(Q::zero(), Q::one(), squared.clone()),
                coincident: squared.is_zero(),
            }
        }
        Request::PlaneDotProduct { u, v } => Answer::PlaneDotProduct {
            value: (&dot(&point(u)?, &point(v)?)).into(),
        },
        Request::PlaneOrthogonal { u, v } => {
            let (u, v) = (point(u)?, point(v)?);
            let value = dot(&u, &v);
            let zero = norm(&u).is_zero() || norm(&v).is_zero();
            Answer::PlaneOrthogonality {
                dot_product: (&value).into(),
                orthogonal: value.is_zero(),
                zero_vector_present: zero,
                angle_defined: !zero,
            }
        }
        Request::TriangleArea { a, b, c } => {
            let (a, b, c) = (point(a)?, point(b)?, point(c)?);
            let u = sub(&b, &a);
            let v = sub(&c, &a);
            let signed = &u[0] * &v[1] - &u[1] * &v[0];
            Answer::TriangleArea {
                area: (&(signed.abs() / q(2))).into(),
                signed_double_area: (&signed).into(),
                degeneracy: triangle_degeneracy(&a, &b, &c, &signed),
            }
        }
        Request::LineLineIntersections { first, second } => Answer::GeometryIntersections {
            intersection: line_line(&line(first)?, &line(second)?),
        },
        Request::LineCircleIntersections { line: l, circle: c } => Answer::GeometryIntersections {
            intersection: line_circle(&line(l)?, &circle(c)?),
        },
        Request::CircleCircleIntersections { first, second } => Answer::GeometryIntersections {
            intersection: circle_circle(&circle(first)?, &circle(second)?),
        },
        _ => return Err(invalid("not a plane geometry operation")),
    };
    Ok((
        answer,
        json!({"executor":"Rust/num-rational plane geometry","arithmetic":"exact rationals and quadratic surds","dimension":2}),
    ))
}

fn polarization(a: &V2, b: &V2) -> Q {
    let sum = [&a[0] + &b[0], &a[1] + &b[1]];
    (norm(&sum) - norm(a) - norm(b)) / q(2)
}
fn field_for(l: &Coefficients, c: &Circle) -> Q {
    let l = canonical_line(l);
    let n = &l[0] * &l[0] + &l[1] * &l[1];
    let s = line_value(&l, &c.0);
    let h = &c.1 * &c.1 * n - &s * &s;
    if !h.is_positive() {
        return Q::zero();
    }
    let r = root_parts(&h).1;
    if r.is_one() {
        Q::zero()
    } else {
        r
    }
}
type FieldPoint = [(Q, Q); 2];
fn read_point(p: &ExactPoint2, field: &Q) -> Result<FieldPoint, String> {
    if p.coordinates.len() != 2 {
        return Err(bad());
    }
    Ok([p.coordinates[0].read(field)?, p.coordinates[1].read(field)?])
}
fn on_line(p: &FieldPoint, l: &Coefficients) -> bool {
    (&l[0] * &p[0].0 + &l[1] * &p[1].0 + &l[2]).is_zero()
        && (&l[0] * &p[0].1 + &l[1] * &p[1].1).is_zero()
}
fn on_circle(p: &FieldPoint, c: &Circle, r: &Q) -> bool {
    let x = &p[0].0 - &c.0[0];
    let y = &p[1].0 - &c.0[1];
    (&x * &x + &y * &y + (&p[0].1 * &p[0].1 + &p[1].1 * &p[1].1) * r - &c.1 * &c.1).is_zero()
        && (x * &p[0].1 + y * &p[1].1).is_zero()
}
fn sign(a: Q, b: Q, r: &Q) -> Ordering {
    if b.is_zero() {
        return a.cmp(&Q::zero());
    }
    if a.is_zero() || a.is_positive() == b.is_positive() {
        return b.cmp(&Q::zero());
    }
    let magnitude = (&a * &a).cmp(&(&b * &b * r));
    if a.is_positive() {
        magnitude
    } else {
        magnitude.reverse()
    }
}
fn verify_intersection(request: &Request, result: &IntersectionResult) -> Result<(), String> {
    let (kind, count, field, flags) = match request {
        Request::LineLineIntersections { first, second } => {
            let a = line(first)?;
            let b = line(second)?;
            let det = &a[0] * &b[1] - &a[1] * &b[0];
            let kind = if !det.is_zero() {
                IntersectionKind::Crossing
            } else if &a[0] * &b[2] == &b[0] * &a[2] && &a[1] * &b[2] == &b[1] * &a[2] {
                IntersectionKind::Coincident
            } else {
                IntersectionKind::Parallel
            };
            let count = usize::from(kind == IntersectionKind::Crossing);
            (kind, count, Q::zero(), vec![])
        }
        Request::LineCircleIntersections { line: l, circle: c } => {
            let l = line(l)?;
            let c = circle(c)?;
            let discriminant =
                &c.1 * &c.1 * (&l[0] * &l[0] + &l[1] * &l[1]) - line_value(&l, &c.0).pow(2);
            let (kind, count) = if discriminant.is_negative() {
                (IntersectionKind::Disjoint, 0)
            } else if discriminant.is_zero() {
                (
                    if c.1.is_zero() {
                        IntersectionKind::PointIntersection
                    } else {
                        IntersectionKind::Tangent
                    },
                    1,
                )
            } else {
                (IntersectionKind::Secant, 2)
            };
            (kind, count, field_for(&l, &c), circle_flags(&c, "circle"))
        }
        Request::CircleCircleIntersections { first, second } => {
            let (a, b) = (circle(first)?, circle(second)?);
            let mut flags = circle_flags(&a, "first_circle");
            flags.extend(circle_flags(&b, "second_circle"));
            let d2 = norm(&sub(&a.0, &b.0));
            let (kind, count) = if d2.is_zero() {
                if a.1 != b.1 {
                    (IntersectionKind::Disjoint, 0)
                } else if a.1.is_zero() {
                    (IntersectionKind::PointIntersection, 1)
                } else {
                    (IntersectionKind::Coincident, 0)
                }
            } else {
                let sum = (&a.1 + &b.1).pow(2);
                let difference = (&a.1 - &b.1).pow(2);
                if d2 > sum || d2 < difference {
                    (IntersectionKind::Disjoint, 0)
                } else if d2 == sum || d2 == difference {
                    (
                        if a.1.is_zero() || b.1.is_zero() {
                            IntersectionKind::PointIntersection
                        } else {
                            IntersectionKind::Tangent
                        },
                        1,
                    )
                } else {
                    (IntersectionKind::Secant, 2)
                }
            };
            let field = if count > 0 && !d2.is_zero() {
                field_for(&radical_axis(&a, &b), &a)
            } else {
                Q::zero()
            };
            (kind, count, field, flags)
        }
        _ => return Err(bad()),
    };
    if result.relation != kind
        || result.points.len() != count
        || result.infinite != (kind == IntersectionKind::Coincident)
        || result.input_degeneracies != flags
    {
        return Err(bad());
    }
    let points: Vec<_> = result
        .points
        .iter()
        .map(|p| read_point(p, &field))
        .collect::<Result<_, _>>()?;
    for p in &points {
        let member = match request {
            Request::LineLineIntersections { first, second } => {
                on_line(p, &line(first)?) && on_line(p, &line(second)?)
            }
            Request::LineCircleIntersections { line: l, circle: c } => {
                on_line(p, &line(l)?) && on_circle(p, &circle(c)?, &field)
            }
            Request::CircleCircleIntersections { first, second } => {
                on_circle(p, &circle(first)?, &field) && on_circle(p, &circle(second)?, &field)
            }
            _ => false,
        };
        if !member {
            return Err(bad());
        }
    }
    if points.len() == 2 {
        let first = &points[0];
        let last = &points[1];
        let mut order = sign(&first[0].0 - &last[0].0, &first[0].1 - &last[0].1, &field);
        if order == Ordering::Equal {
            order = sign(&first[1].0 - &last[1].0, &first[1].1 - &last[1].1, &field);
        }
        if order != Ordering::Less {
            return Err(bad());
        }
    }
    Ok(())
}

pub(crate) fn verify(request: &Request, answer: &Answer) -> Result<Vec<String>, String> {
    validate(request)?;
    let checks = match (request, answer) {
        (
            Request::PlaneDistance { a, b },
            Answer::PlaneDistance {
                squared_distance,
                distance,
                coincident,
            },
        ) => {
            let (a, b) = (point(a)?, point(b)?);
            let expected = norm(&a) + norm(&b) - q(2) * polarization(&a, &b);
            if squared_distance.canonical()? != expected || *coincident != expected.is_zero() {
                return Err(bad());
            }
            let rational = distance.rational.canonical()?;
            let coefficient = distance.coefficient.canonical()?;
            let radicand = distance.radicand.canonical()?;
            if coefficient.is_zero() {
                if !radicand.is_zero()
                    || rational.is_negative()
                    || &rational * &rational != expected
                {
                    return Err(bad());
                }
            } else if !rational.is_zero()
                || !coefficient.is_positive()
                || !radicand.is_positive()
                || square_root(&radicand).is_some()
                || &coefficient * &coefficient * radicand != expected
            {
                return Err(bad());
            }
            vec![
                "EXPANDED_SQUARED_DISTANCE",
                "NONNEGATIVE_PRINCIPAL_ROOT",
                "COINCIDENT_POINT_CLASSIFICATION",
            ]
        }
        (Request::PlaneDotProduct { u, v }, Answer::PlaneDotProduct { value }) => {
            if value.canonical()? != polarization(&point(u)?, &point(v)?) {
                return Err(bad());
            }
            vec!["TWO_DIMENSIONAL_COORDINATES", "POLARIZATION_IDENTITY"]
        }
        (
            Request::PlaneOrthogonal { u, v },
            Answer::PlaneOrthogonality {
                dot_product,
                orthogonal,
                zero_vector_present,
                angle_defined,
            },
        ) => {
            let (u, v) = (point(u)?, point(v)?);
            let expected = polarization(&u, &v);
            let zero = norm(&u).is_zero() || norm(&v).is_zero();
            if dot_product.canonical()? != expected
                || *orthogonal != expected.is_zero()
                || *zero_vector_present != zero
                || *angle_defined == zero
            {
                return Err(bad());
            }
            vec![
                "POLARIZATION_ZERO_TEST",
                "ZERO_VECTOR_AND_ANGLE_DEFINEDNESS",
            ]
        }
        (
            Request::TriangleArea { a, b, c },
            Answer::TriangleArea {
                area,
                signed_double_area,
                degeneracy,
            },
        ) => {
            let (a, b, c) = (point(a)?, point(b)?, point(c)?);
            let shoelace =
                &a[0] * (&b[1] - &c[1]) + &b[0] * (&c[1] - &a[1]) + &c[0] * (&a[1] - &b[1]);
            if signed_double_area.canonical()? != shoelace
                || area.canonical()? * q(2) != shoelace.abs()
                || *degeneracy != triangle_degeneracy(&a, &b, &c, &shoelace)
            {
                return Err(bad());
            }
            vec![
                "SHOELACE_AREA_IDENTITY",
                "REPEATED_VERTEX_AND_COLLINEAR_CLASSIFICATION",
            ]
        }
        (_, Answer::GeometryIntersections { intersection }) => {
            verify_intersection(request, intersection)?;
            vec![
                "EXACT_EQUATION_SUBSTITUTION",
                "RANK_OR_DISCRIMINANT_COMPLETE_ROOT_COUNT",
                "DISTINCT_SORTED_POINTS_OR_EXPLICIT_INFINITY",
                "DEGENERATE_CIRCLE_CLASSIFICATION",
            ]
        }
        _ => return Err(bad()),
    };
    Ok(checks.into_iter().map(str::to_owned).collect())
}

pub(crate) fn explain(request: &Request, answer: &Answer) -> Vec<String> {
    match (request,answer) {
        (Request::PlaneDistance{..},_)=>vec!["2次元の有理数座標を検査し、距離の二乗=(Δx)²+(Δy)²と、その非負の主平方根を厳密に計算しました。".into(),"展開した内積の恒等式で距離の二乗を再計算し、返した距離の二乗と非負性を検算しました。".into()],
        (Request::PlaneDotProduct{..},_)=>vec!["2成分ベクトルの内積uₓvₓ+uᵧvᵧを有理数で計算しました。".into(),"||u+v||²−||u||²−||v||²=2(u·v)で独立に検算しました。".into()],
        (Request::PlaneOrthogonal{..},Answer::PlaneOrthogonality{zero_vector_present,..})=>vec!["内積が厳密に0であることを代数的な直交の条件としました。".into(),if *zero_vector_present{"零ベクトルを含むため内積は0ですが、方向のなす角は未定義です。angle_defined=falseとして区別しました。".into()}else{"両ベクトルは非零です。内積とノルムから直交判定・角の定義可能性を検算しました。".into()}],
        (Request::TriangleArea{..},Answer::TriangleArea{degeneracy,..})=>vec!["辺ベクトルの行列式の絶対値÷2で面積を求め、靴紐公式で独立に検算しました。".into(),format!("退化判定は{degeneracy:?}。重複頂点と、相異なる共線頂点を区別しています。")],
        (_,Answer::GeometryIntersections{intersection})=> {
            let reasoning=match request {
                Request::LineLineIntersections{first,second}=>{
                    let a=line(first).expect("validated line");let b=line(second).expect("validated line");
                    format!("係数行列の行列式は{}です。0なら定数項との整合性から平行と一致を区別します。",&a[0]*&b[1]-&a[1]*&b[0])
                }
                Request::LineCircleIntersections{line:l,circle:c}=>{
                    let l=line(l).expect("validated line");let c=circle(c).expect("validated circle");
                    let discriminant=&c.1*&c.1*(&l[0]*&l[0]+&l[1]*&l[1])-line_value(&l,&c.0).pow(2);
                    format!("判別量r²(a²+b²)−(a cx+b cy+c)²は{discriminant}です。負なら0点、0なら1点、正なら2点です。")
                }
                Request::CircleCircleIntersections{first,second}=>{
                    let a=circle(first).expect("validated circle");let b=circle(second).expect("validated circle");
                    format!("中心間距離²={}、半径の和²={}、半径の差²={}を厳密に比較し、根軸で交点を求めました。同心の場合は半径0の点と正半径の一致円を区別します。",norm(&sub(&a.0,&b.0)),(&a.1+&b.1).pow(2),(&a.1-&b.1).pow(2))
                }
                _=>unreachable!(),
            };
            let enumeration=if intersection.infinite {
                "図形が一致するため交点は無限個です。有限の点列では表現せず、infinite=true・points=[]を返します。".into()
            }else{format!("位置関係は{:?}。交点{}点を座標順に重複なく全列挙しました。",intersection.relation,intersection.points.len())};
            vec![reasoning,enumeration,"元の方程式への代入、階数または判別式・中心間距離による個数判定を別に行い、交点の欠落・重複を検査しました。半径0の円は点として扱います。".into()]
        },
        _=>unreachable!(),
    }
}
