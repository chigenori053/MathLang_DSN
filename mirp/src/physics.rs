//! Conditional Newtonian point-particle calculations with exact SI units.
//! Model/frame/assumption checks are declarations and consistency checks, not
//! empirical evidence that a real system obeys the chosen physical model.
use crate::calculation::{CalculationAnswer as Answer, CalculationRequest as Request};
use crate::exact::ExactRational as R;
use num_rational::BigRational as Q;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PhysicalModel {
    NewtonianPointParticle,
    RelativisticParticle,
    RigidBody,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FrameKind {
    Inertial,
    Accelerating,
    Rotating,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BasisKind {
    OrthonormalCartesian,
    Oblique,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceFrame {
    pub id: String,
    pub origin: String,
    pub dimension: usize,
    pub axes: Vec<String>,
    pub kind: FrameKind,
    pub basis: BasisKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Assumption {
    Nonrelativistic,
    ConstantMass,
    ConstantAcceleration,
    ConstantForce,
    NetForce,
    NetWork,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalContext {
    pub model: PhysicalModel,
    pub frame: ReferenceFrame,
    pub assumptions: Vec<Assumption>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarQuantity {
    pub value: R,
    pub unit: String,
    #[serde(default)]
    pub frame_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorQuantity {
    pub values: Vec<R>,
    pub unit: String,
    pub frame_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionResult {
    pub position: VectorQuantity,
    pub velocity: VectorQuantity,
    pub displacement: VectorQuantity,
    pub elapsed: ScalarQuantity,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyBalance {
    pub initial_kinetic_energy: ScalarQuantity,
    pub final_kinetic_energy: ScalarQuantity,
    pub kinetic_energy_change: ScalarQuantity,
    pub net_work: ScalarQuantity,
    pub residual: ScalarQuantity,
    pub consistent: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum PhysicsOutcome {
    ConstantAcceleration(Box<MotionResult>),
    NewtonForce { net_force: VectorQuantity },
    NewtonAcceleration { acceleration: VectorQuantity },
    ConstantForceWork { work: ScalarQuantity },
    KineticEnergy { kinetic_energy: ScalarQuantity },
    WorkEnergy(Box<EnergyBalance>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PhysicalApplicability {
    ConditionalOnDeclaredAssumptions,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsResult {
    pub context: PhysicalContext,
    pub applicability: PhysicalApplicability,
    pub required_assumptions: Vec<Assumption>,
    pub outcome: PhysicsOutcome,
}

// Exponents are length, mass, time. Scales are exact multiples of SI units.
type Dimension = [i32; 3];
const LENGTH: Dimension = [1, 0, 0];
const MASS: Dimension = [0, 1, 0];
const TIME: Dimension = [0, 0, 1];
const VELOCITY: Dimension = [1, 0, -1];
const ACCELERATION: Dimension = [1, 0, -2];
const FORCE: Dimension = [1, 1, -2];
const ENERGY: Dimension = [2, 1, -2];

fn invalid(s: &str) -> String {
    format!("CAPABILITY_INPUT_INVALID: {s}")
}
fn bad() -> String {
    "CAPABILITY_OUTPUT_INVALID: physical result verification failed".into()
}
fn q(n: i64) -> Q {
    Q::from_integer(n.into())
}
fn base_unit(symbol: &str) -> Result<(Dimension, Q), String> {
    let (dimension, n, d) = match symbol {
        "1" => ([0, 0, 0], 1, 1),
        "m" => (LENGTH, 1, 1),
        "cm" => (LENGTH, 1, 100),
        "mm" => (LENGTH, 1, 1000),
        "km" => (LENGTH, 1000, 1),
        "kg" => (MASS, 1, 1),
        "g" => (MASS, 1, 1000),
        "mg" => (MASS, 1, 1_000_000),
        "s" => (TIME, 1, 1),
        "ms" => (TIME, 1, 1000),
        "min" => (TIME, 60, 1),
        "h" => (TIME, 3600, 1),
        "N" => (FORCE, 1, 1),
        "kN" => (FORCE, 1000, 1),
        "mN" => (FORCE, 1, 1000),
        "J" => (ENERGY, 1, 1),
        "kJ" => (ENERGY, 1000, 1),
        "mJ" => (ENERGY, 1, 1000),
        _ => return Err(invalid("unknown unit symbol; units are case-sensitive")),
    };
    Ok((dimension, Q::new(n.into(), d.into())))
}
fn unit(text: &str) -> Result<(Dimension, Q), String> {
    let text = text.trim();
    if text.is_empty() || text.len() > 80 || !text.is_ascii() {
        return Err(invalid("invalid unit expression"));
    }
    let mut dimension = [0; 3];
    let mut scale = Q::one();
    let mut start = 0;
    let mut direction = 1;
    let mut count = 0;
    for end in text
        .bytes()
        .enumerate()
        .filter_map(|(i, c)| {
            if c == b'*' || c == b'/' {
                Some(i)
            } else {
                None
            }
        })
        .chain(std::iter::once(text.len()))
    {
        count += 1;
        if count > 8 || start == end {
            return Err(invalid("unit expression requires 1..=8 nonempty factors"));
        }
        let factor = &text[start..end];
        let (symbol, exponent) = match factor.split_once('^') {
            Some((s, e)) => (
                s,
                e.parse::<i32>()
                    .map_err(|_| invalid("unit exponent must be an integer"))?,
            ),
            None => (factor, 1),
        };
        if !(-4..=4).contains(&exponent) {
            return Err(invalid("unit exponent must be in -4..=4"));
        }
        let exponent = exponent * direction;
        let (d, s) = base_unit(symbol)?;
        for i in 0..3 {
            dimension[i] += d[i] * exponent;
        }
        scale *= s.pow(exponent);
        if end < text.len() {
            direction = if text.as_bytes()[end] == b'/' { -1 } else { 1 };
            start = end + 1;
        }
    }
    Ok((dimension, scale))
}
fn valid_label(s: &str, limit: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= limit
}
fn required(request: &Request) -> Vec<Assumption> {
    use Assumption::*;
    match request {
        Request::ConstantAcceleration { .. } => vec![Nonrelativistic, ConstantAcceleration],
        Request::NewtonForce { .. } | Request::KineticEnergy { .. } => {
            vec![Nonrelativistic, ConstantMass]
        }
        Request::NewtonAcceleration { .. } => vec![Nonrelativistic, ConstantMass, NetForce],
        Request::ConstantForceWork { .. } => vec![Nonrelativistic, ConstantForce],
        Request::WorkEnergy { .. } => vec![Nonrelativistic, ConstantMass, NetWork],
        _ => vec![],
    }
}
fn context(request: &Request) -> Result<&PhysicalContext, String> {
    match request {
        Request::ConstantAcceleration { context, .. }
        | Request::NewtonForce { context, .. }
        | Request::NewtonAcceleration { context, .. }
        | Request::ConstantForceWork { context, .. }
        | Request::KineticEnergy { context, .. }
        | Request::WorkEnergy { context, .. } => Ok(context),
        _ => Err(invalid("not a classical physics operation")),
    }
}
fn check_context(c: &PhysicalContext, needed: &[Assumption]) -> Result<(), String> {
    if c.model != PhysicalModel::NewtonianPointParticle {
        return Err(invalid(
            "only the Newtonian point-particle model is supported",
        ));
    }
    let f = &c.frame;
    if f.kind != FrameKind::Inertial {
        return Err(invalid(
            "an inertial frame is required; fictitious forces are not modeled",
        ));
    }
    if f.basis != BasisKind::OrthonormalCartesian {
        return Err(invalid("an orthonormal Cartesian basis is required"));
    }
    if !(1..=3).contains(&f.dimension) || f.axes.len() != f.dimension {
        return Err(invalid(
            "frame dimension/axis count must agree and be 1..=3",
        ));
    }
    if !valid_label(&f.id, 128)
        || !valid_label(&f.origin, 128)
        || f.axes.iter().any(|a| !valid_label(a, 32))
        || f.axes.iter().collect::<BTreeSet<_>>().len() != f.axes.len()
    {
        return Err(invalid(
            "frame identifiers, origin and distinct axis labels are required",
        ));
    }
    let declared: BTreeSet<_> = c.assumptions.iter().copied().collect();
    if declared.len() != c.assumptions.len() {
        return Err(invalid("duplicate physical assumption"));
    }
    if needed.iter().any(|a| !declared.contains(a)) {
        return Err(invalid(
            "required physical model assumptions were not declared",
        ));
    }
    Ok(())
}
fn check_frame(id: Option<&str>, c: &PhysicalContext, required: bool) -> Result<(), String> {
    if (required && id.is_none()) || id.is_some_and(|id| id != c.frame.id) {
        return Err(invalid(
            "quantity reference frame differs or is missing; no frame transform is inferred",
        ));
    }
    Ok(())
}
fn scalar(
    v: &ScalarQuantity,
    dimension: Dimension,
    c: &PhysicalContext,
    framed: bool,
) -> Result<Q, String> {
    check_frame(v.frame_id.as_deref(), c, framed)?;
    let (d, scale) = unit(&v.unit)?;
    if d != dimension {
        return Err(invalid("scalar unit has the wrong physical dimension"));
    }
    Ok(v.value.read(80)? * scale)
}
fn vector(v: &VectorQuantity, dimension: Dimension, c: &PhysicalContext) -> Result<Vec<Q>, String> {
    check_frame(Some(&v.frame_id), c, true)?;
    if v.values.len() != c.frame.dimension {
        return Err(invalid("vector dimension differs from reference frame"));
    }
    let (d, scale) = unit(&v.unit)?;
    if d != dimension {
        return Err(invalid("vector unit has the wrong physical dimension"));
    }
    v.values.iter().map(|x| Ok(x.read(80)? * &scale)).collect()
}
fn positive_mass(v: &ScalarQuantity, c: &PhysicalContext) -> Result<Q, String> {
    let m = scalar(v, MASS, c, false)?;
    if !m.is_positive() {
        return Err(invalid("Newtonian particle mass must be positive"));
    }
    Ok(m)
}
fn norm(v: &[Q]) -> Q {
    v.iter().map(|v| v * v).sum()
}
fn speed_allowed(v: &[Q]) -> bool {
    norm(v) < q(299_792_458).pow(2)
}
fn check_speed(v: &[Q]) -> Result<(), String> {
    if !speed_allowed(v) {
        return Err(invalid(
            "velocity contradicts the declared nonrelativistic regime (speed >= c)",
        ));
    }
    Ok(())
}

enum Data {
    Motion {
        position: Vec<Q>,
        velocity: Vec<Q>,
        acceleration: Vec<Q>,
        time: Q,
    },
    Force {
        mass: Q,
        acceleration: Vec<Q>,
    },
    Acceleration {
        mass: Q,
        force: Vec<Q>,
    },
    Work {
        force: Vec<Q>,
        displacement: Vec<Q>,
    },
    Energy {
        mass: Q,
        velocity: Vec<Q>,
    },
    Balance {
        mass: Q,
        initial_velocity: Vec<Q>,
        final_velocity: Vec<Q>,
        work: Q,
    },
}
fn prepare(request: &Request) -> Result<(&PhysicalContext, Data), String> {
    let c = context(request)?;
    check_context(c, &required(request))?;
    let data = match request {
        Request::ConstantAcceleration {
            initial_position,
            initial_velocity,
            acceleration,
            duration,
            ..
        } => {
            let position = vector(initial_position, LENGTH, c)?;
            let velocity = vector(initial_velocity, VELOCITY, c)?;
            let acceleration = vector(acceleration, ACCELERATION, c)?;
            let time = scalar(duration, TIME, c, false)?;
            if time.is_negative() {
                return Err(invalid("elapsed duration must be nonnegative"));
            }
            check_speed(&velocity)?;
            check_speed(
                &velocity
                    .iter()
                    .zip(&acceleration)
                    .map(|(v, a)| v + a * &time)
                    .collect::<Vec<_>>(),
            )?;
            Data::Motion {
                position,
                velocity,
                acceleration,
                time,
            }
        }
        Request::NewtonForce {
            mass, acceleration, ..
        } => Data::Force {
            mass: positive_mass(mass, c)?,
            acceleration: vector(acceleration, ACCELERATION, c)?,
        },
        Request::NewtonAcceleration {
            mass, net_force, ..
        } => Data::Acceleration {
            mass: positive_mass(mass, c)?,
            force: vector(net_force, FORCE, c)?,
        },
        Request::ConstantForceWork {
            force,
            displacement,
            ..
        } => Data::Work {
            force: vector(force, FORCE, c)?,
            displacement: vector(displacement, LENGTH, c)?,
        },
        Request::KineticEnergy { mass, velocity, .. } => {
            let mass = positive_mass(mass, c)?;
            let velocity = vector(velocity, VELOCITY, c)?;
            check_speed(&velocity)?;
            Data::Energy { mass, velocity }
        }
        Request::WorkEnergy {
            mass,
            initial_velocity,
            final_velocity,
            net_work,
            ..
        } => {
            let mass = positive_mass(mass, c)?;
            let initial_velocity = vector(initial_velocity, VELOCITY, c)?;
            let final_velocity = vector(final_velocity, VELOCITY, c)?;
            check_speed(&initial_velocity)?;
            check_speed(&final_velocity)?;
            Data::Balance {
                mass,
                initial_velocity,
                final_velocity,
                work: scalar(net_work, ENERGY, c, true)?,
            }
        }
        _ => return Err(invalid("not a classical physics operation")),
    };
    Ok((c, data))
}
pub(crate) fn validate(request: &Request) -> Result<(), String> {
    prepare(request).map(|_| ())
}
fn out_scalar(value: &Q, unit: &str, c: &PhysicalContext, framed: bool) -> ScalarQuantity {
    ScalarQuantity {
        value: value.into(),
        unit: unit.into(),
        frame_id: if framed {
            Some(c.frame.id.clone())
        } else {
            None
        },
    }
}
fn out_vector(values: &[Q], unit: &str, c: &PhysicalContext) -> VectorQuantity {
    VectorQuantity {
        values: values.iter().map(R::from).collect(),
        unit: unit.into(),
        frame_id: c.frame.id.clone(),
    }
}
fn kinetic(mass: &Q, velocity: &[Q]) -> Q {
    mass * norm(velocity) / q(2)
}

pub(crate) fn execute(request: &Request) -> Result<(Answer, Json), String> {
    let (c, data) = prepare(request)?;
    let outcome = match data {
        Data::Motion {
            position,
            velocity,
            acceleration,
            time,
        } => {
            let displacement: Vec<_> = velocity
                .iter()
                .zip(&acceleration)
                .map(|(v, a)| v * &time + a * &time * &time / q(2))
                .collect();
            let final_position: Vec<_> = position
                .iter()
                .zip(&displacement)
                .map(|(x, d)| x + d)
                .collect();
            let final_velocity: Vec<_> = velocity
                .iter()
                .zip(&acceleration)
                .map(|(v, a)| v + a * &time)
                .collect();
            PhysicsOutcome::ConstantAcceleration(Box::new(MotionResult {
                position: out_vector(&final_position, "m", c),
                velocity: out_vector(&final_velocity, "m/s", c),
                displacement: out_vector(&displacement, "m", c),
                elapsed: out_scalar(&time, "s", c, false),
            }))
        }
        Data::Force { mass, acceleration } => PhysicsOutcome::NewtonForce {
            net_force: out_vector(
                &acceleration.iter().map(|a| a * &mass).collect::<Vec<_>>(),
                "N",
                c,
            ),
        },
        Data::Acceleration { mass, force } => PhysicsOutcome::NewtonAcceleration {
            acceleration: out_vector(
                &force.iter().map(|f| f / &mass).collect::<Vec<_>>(),
                "m/s^2",
                c,
            ),
        },
        Data::Work {
            force,
            displacement,
        } => {
            let work: Q = force.iter().zip(&displacement).map(|(f, d)| f * d).sum();
            PhysicsOutcome::ConstantForceWork {
                work: out_scalar(&work, "J", c, true),
            }
        }
        Data::Energy { mass, velocity } => PhysicsOutcome::KineticEnergy {
            kinetic_energy: out_scalar(&kinetic(&mass, &velocity), "J", c, true),
        },
        Data::Balance {
            mass,
            initial_velocity,
            final_velocity,
            work,
        } => {
            let initial = kinetic(&mass, &initial_velocity);
            let final_energy = kinetic(&mass, &final_velocity);
            let change = &final_energy - &initial;
            let residual = &work - &change;
            PhysicsOutcome::WorkEnergy(Box::new(EnergyBalance {
                initial_kinetic_energy: out_scalar(&initial, "J", c, true),
                final_kinetic_energy: out_scalar(&final_energy, "J", c, true),
                kinetic_energy_change: out_scalar(&change, "J", c, true),
                net_work: out_scalar(&work, "J", c, true),
                residual: out_scalar(&residual, "J", c, true),
                consistent: residual.is_zero(),
            }))
        }
    };
    let result = PhysicsResult {
        context: c.clone(),
        applicability: PhysicalApplicability::ConditionalOnDeclaredAssumptions,
        required_assumptions: required(request),
        outcome,
    };
    let metadata = json!({"executor":"Rust/num-rational classical physics","arithmetic":"exact rational SI conversion and residual checks","frame_id":c.frame.id,"dimension":c.frame.dimension,"physical_applicability":"CONDITIONAL_ON_DECLARED_ASSUMPTIONS","required_assumptions":result.required_assumptions});
    Ok((
        Answer::Physics {
            result: Box::new(result),
        },
        metadata,
    ))
}

fn result_scalar(
    v: &ScalarQuantity,
    unit: &str,
    c: &PhysicalContext,
    framed: bool,
) -> Result<Q, String> {
    if v.unit != unit
        || v.frame_id
            != if framed {
                Some(c.frame.id.clone())
            } else {
                None
            }
    {
        return Err(bad());
    }
    v.value.canonical()
}
fn result_vector(v: &VectorQuantity, unit: &str, c: &PhysicalContext) -> Result<Vec<Q>, String> {
    if v.unit != unit || v.frame_id != c.frame.id || v.values.len() != c.frame.dimension {
        return Err(bad());
    }
    v.values.iter().map(R::canonical).collect()
}
pub(crate) fn verify(request: &Request, answer: &Answer) -> Result<Vec<String>, String> {
    let (c, data) = prepare(request)?;
    let Answer::Physics { result } = answer else {
        return Err(bad());
    };
    if result.context != *c
        || result.required_assumptions != required(request)
        || result.applicability != PhysicalApplicability::ConditionalOnDeclaredAssumptions
    {
        return Err(bad());
    }
    let mut checks = vec![
        "EXACT_UNIT_DIMENSIONS_AND_SI_CONVERSION",
        "COMMON_INERTIAL_ORTHONORMAL_FRAME",
        "DECLARED_MODEL_AND_REQUIRED_ASSUMPTIONS",
    ];
    match (data, &result.outcome) {
        (
            Data::Motion {
                position,
                velocity,
                acceleration,
                time,
            },
            PhysicsOutcome::ConstantAcceleration(out),
        ) => {
            let x = result_vector(&out.position, "m", c)?;
            let v = result_vector(&out.velocity, "m/s", c)?;
            let d = result_vector(&out.displacement, "m", c)?;
            if result_scalar(&out.elapsed, "s", c, false)? != time || !speed_allowed(&v) {
                return Err(bad());
            }
            for i in 0..c.frame.dimension {
                if &v[i] - &velocity[i] != &acceleration[i] * &time
                    || &x[i] - &position[i] != d[i]
                    || q(2) * &d[i] != (&velocity[i] + &v[i]) * &time
                    || &v[i] * &v[i] - &velocity[i] * &velocity[i]
                        != q(2) * &acceleration[i] * &d[i]
                {
                    return Err(bad());
                }
            }
            checks.extend([
                "VELOCITY_CHANGE_AND_AVERAGE_VELOCITY_DISPLACEMENT",
                "COMPONENT_ENERGY_IDENTITY_AND_SPEED_DOMAIN",
            ]);
        }
        (Data::Force { mass, acceleration }, PhysicsOutcome::NewtonForce { net_force }) => {
            let force = result_vector(net_force, "N", c)?;
            if force
                .iter()
                .zip(&acceleration)
                .any(|(f, a)| f / &mass != *a)
            {
                return Err(bad());
            }
            checks.push("FORCE_OVER_MASS_RECOVERS_ACCELERATION");
        }
        (
            Data::Acceleration { mass, force },
            PhysicsOutcome::NewtonAcceleration { acceleration },
        ) => {
            let acceleration = result_vector(acceleration, "m/s^2", c)?;
            if acceleration
                .iter()
                .zip(&force)
                .any(|(a, f)| a * &mass != *f)
            {
                return Err(bad());
            }
            checks.push("MASS_TIMES_ACCELERATION_RECOVERS_NET_FORCE");
        }
        (
            Data::Work {
                force,
                displacement,
            },
            PhysicsOutcome::ConstantForceWork { work },
        ) => {
            let work = result_scalar(work, "J", c, true)?;
            if !force
                .iter()
                .zip(&displacement)
                .fold(work, |residual, (f, d)| residual - f * d)
                .is_zero()
            {
                return Err(bad());
            }
            checks.push("COMPONENT_WORK_SUM_AND_ENERGY_DIMENSION");
        }
        (Data::Energy { mass, velocity }, PhysicsOutcome::KineticEnergy { kinetic_energy }) => {
            let energy = result_scalar(kinetic_energy, "J", c, true)?;
            if energy.is_negative() || q(2) * energy / &mass != norm(&velocity) {
                return Err(bad());
            }
            checks.push("NONNEGATIVE_KINETIC_ENERGY_AND_SPEED_IDENTITY");
        }
        (
            Data::Balance {
                mass,
                initial_velocity,
                final_velocity,
                work,
            },
            PhysicsOutcome::WorkEnergy(out),
        ) => {
            let initial = result_scalar(&out.initial_kinetic_energy, "J", c, true)?;
            let final_energy = result_scalar(&out.final_kinetic_energy, "J", c, true)?;
            let change = result_scalar(&out.kinetic_energy_change, "J", c, true)?;
            let supplied = result_scalar(&out.net_work, "J", c, true)?;
            let residual = result_scalar(&out.residual, "J", c, true)?;
            if initial.is_negative()
                || final_energy.is_negative()
                || q(2) * &initial / &mass != norm(&initial_velocity)
                || q(2) * &final_energy / &mass != norm(&final_velocity)
                || change != final_energy - initial
                || supplied != work
                || &residual + change != work
                || out.consistent != residual.is_zero()
            {
                return Err(bad());
            }
            checks.extend([
                "INITIAL_AND_FINAL_KINETIC_IDENTITIES",
                "NET_WORK_MINUS_KINETIC_CHANGE_RESIDUAL",
                "CONSISTENCY_ASSESSMENT_DISTINCT_FROM_VERIFICATION",
            ]);
        }
        _ => return Err(bad()),
    }
    Ok(checks.into_iter().map(str::to_owned).collect())
}
pub(crate) fn explain(request: &Request, answer: &Answer) -> Vec<String> {
    let Answer::Physics { result } = answer else {
        unreachable!()
    };
    let mut steps=vec![format!("モデルはNewtonian point particle、慣性系{}、{}次元の直交Cartesian座標です。単位を厳密にSIへ換算し、必要な仮定{:?}の宣言を確認しました。",result.context.frame.id,result.context.frame.dimension,result.required_assumptions)];
    steps.push(match request {
        Request::ConstantAcceleration{..}=>"一定加速度としてv=v₀+at、x=x₀+v₀t+at²/2を計算しました。平均速度による変位とv²−v₀²=2aΔxを各軸で検算しました。".into(),
        Request::NewtonForce{..}=>"一定の正の質量と座標加速度から合力F=maを計算し、F/mが元の加速度になることを検算しました。".into(),
        Request::NewtonAcceleration{..}=>"入力が合力であるという宣言を要求し、a=F合力/mを計算しました。maを再構成して合力と照合しました。".into(),
        Request::ConstantForceWork{..}=>"一定力の仕事W=F·Δxを計算しました。負の仕事と直交する変位の仕事0も保持します。この力だけの仕事を総仕事とは自動認定しません。".into(),
        Request::KineticEnergy{..}=>"この座標系での運動エネルギーK=m||v||²/2を求め、非負性と2K/m=||v||²を検算しました。".into(),
        Request::WorkEnergy{..}=>{
            let PhysicsOutcome::WorkEnergy(balance)=&result.outcome else{unreachable!()};
            format!("入力が総仕事であるという宣言のもとでΔK=K後−K前と残差W合力−ΔKを計算しました。consistent={}。VERIFIEDはこの計算・不整合判定の検算成功を表し、consistent=falseのデータが物理法則に適合することは意味しません。",balance.consistent)
        }
        _=>unreachable!(),
    });
    steps.push("モデルの適用は宣言された仮定に条件付きです。仮定が現実に成立することや近似精度は、この計算からは証明していません。与えた速度と運動の端点が光速以上なら宣言との矛盾として拒否します。".into());
    steps
}
