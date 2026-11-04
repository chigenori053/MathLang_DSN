//! Bounded exact calculations selected by Knowledge and executed by registered
//! ReasonScript/rational-arithmetic capabilities before recording checked evidence.
use crate::capability::{
    CapabilityExecutor, CapabilityRequest, CapabilityResult, CapabilityStatus,
    RuntimeCapabilityRegistry,
};
use crate::exact::{ExactComparison, ExactExpression, ExactRational};
use crate::geometry::{
    Circle2, IntersectionResult, Line2, Point2, QuadraticSurd, TriangleDegeneracy,
};
use crate::knowledge::{
    KnowledgeActivation, KnowledgeDecision, KnowledgeQuery, KnowledgeSpace, KnowledgeUnit,
};
use crate::physics::{PhysicalContext, PhysicsResult, ScalarQuantity, VectorQuantity};
use crate::{
    stable_id, Common, Evidence, Object, Provenance, SemanticDelta, SemanticState, Status, Value,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Operation {
    SimplifySqrt,
    DescribeData,
    DataVariance,
    DataCovariance,
    NormalizeWeights,
    DiscreteExpectation,
    NormalizeExact,
    ExactEqual,
    ExactCompare,
    PlaneDistance,
    PlaneDotProduct,
    PlaneOrthogonal,
    TriangleArea,
    LineLineIntersections,
    LineCircleIntersections,
    CircleCircleIntersections,
    ConstantAcceleration,
    NewtonForce,
    NewtonAcceleration,
    ConstantForceWork,
    KineticEnergy,
    WorkEnergy,
}

impl Operation {
    pub(crate) const ALL: [Self; 22] = [
        Self::SimplifySqrt,
        Self::DescribeData,
        Self::DataVariance,
        Self::DataCovariance,
        Self::NormalizeWeights,
        Self::DiscreteExpectation,
        Self::NormalizeExact,
        Self::ExactEqual,
        Self::ExactCompare,
        Self::PlaneDistance,
        Self::PlaneDotProduct,
        Self::PlaneOrthogonal,
        Self::TriangleArea,
        Self::LineLineIntersections,
        Self::LineCircleIntersections,
        Self::CircleCircleIntersections,
        Self::ConstantAcceleration,
        Self::NewtonForce,
        Self::NewtonAcceleration,
        Self::ConstantForceWork,
        Self::KineticEnergy,
        Self::WorkEnergy,
    ];
    fn physical(self) -> bool {
        matches!(
            self,
            Self::ConstantAcceleration
                | Self::NewtonForce
                | Self::NewtonAcceleration
                | Self::ConstantForceWork
                | Self::KineticEnergy
                | Self::WorkEnergy
        )
    }
    fn geometric(self) -> bool {
        matches!(
            self,
            Self::PlaneDistance
                | Self::PlaneDotProduct
                | Self::PlaneOrthogonal
                | Self::TriangleArea
                | Self::LineLineIntersections
                | Self::LineCircleIntersections
                | Self::CircleCircleIntersections
        )
    }
    fn extended(self) -> bool {
        !matches!(
            self,
            Self::SimplifySqrt | Self::DescribeData | Self::DataVariance
        )
    }
    fn capability(self) -> &'static str {
        match self {
            Self::SimplifySqrt => "SIMPLIFY_RADICAL",
            Self::DescribeData => "DESCRIBE_DATA",
            Self::DataVariance => "DATA_VARIANCE",
            Self::DataCovariance => "DATA_COVARIANCE",
            Self::NormalizeWeights => "NORMALIZE_WEIGHTS",
            Self::DiscreteExpectation => "DISCRETE_EXPECTATION",
            Self::NormalizeExact => "NORMALIZE_EXACT",
            Self::ExactEqual => "EXACT_EQUAL",
            Self::ExactCompare => "EXACT_COMPARE",
            Self::PlaneDistance => "PLANE_DISTANCE",
            Self::PlaneDotProduct => "PLANE_DOT_PRODUCT",
            Self::PlaneOrthogonal => "PLANE_ORTHOGONAL",
            Self::TriangleArea => "TRIANGLE_AREA",
            Self::LineLineIntersections => "LINE_LINE_INTERSECTIONS",
            Self::LineCircleIntersections => "LINE_CIRCLE_INTERSECTIONS",
            Self::CircleCircleIntersections => "CIRCLE_CIRCLE_INTERSECTIONS",
            Self::ConstantAcceleration => "CONSTANT_ACCELERATION",
            Self::NewtonForce => "NEWTON_FORCE",
            Self::NewtonAcceleration => "NEWTON_ACCELERATION",
            Self::ConstantForceWork => "CONSTANT_FORCE_WORK",
            Self::KineticEnergy => "KINETIC_ENERGY",
            Self::WorkEnergy => "WORK_ENERGY",
        }
    }
    fn domain(self) -> &'static str {
        if self.physical() {
            return "CLASSICAL_PHYSICS";
        }
        if self.geometric() {
            return "COORDINATE_GEOMETRY";
        }
        if matches!(
            self,
            Self::SimplifySqrt | Self::NormalizeExact | Self::ExactEqual | Self::ExactCompare
        ) {
            "NUMBER_SYSTEM"
        } else {
            "STATISTICS"
        }
    }
    fn goal(self) -> &'static str {
        self.capability()
    }
    fn constraints(self) -> Vec<String> {
        if self.physical() {
            return vec!["VALID_CLASSICAL_PHYSICS_INPUT".into()];
        }
        if self.geometric() {
            return vec!["VALID_RATIONAL_PLANE_GEOMETRY".into()];
        }
        vec![match self {
            Self::SimplifySqrt => "RADICAND_IN_RANGE",
            Self::DescribeData | Self::DataVariance => "BOUNDED_NONEMPTY_INTEGER_DATA",
            Self::DataCovariance => "VALID_PAIRED_RATIONAL_DATA",
            Self::NormalizeWeights | Self::DiscreteExpectation => "VALID_EXACT_DISTRIBUTION",
            _ => "VALID_FINITE_RADICAL_SUM",
        }
        .into()]
    }

    fn entity_type(self) -> &'static str {
        if self.extended() {
            "EXACT_CALCULATION_INPUT"
        } else if self == Self::SimplifySqrt {
            "RADICAND"
        } else {
            "INTEGER_DATA"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "SCREAMING_SNAKE_CASE",
    deny_unknown_fields
)]
pub enum CalculationRequest {
    SimplifySqrt {
        radicand: i64,
    },
    DescribeData {
        values: Vec<i64>,
    },
    DataVariance {
        values: Vec<i64>,
    },
    DataCovariance {
        x: Vec<ExactRational>,
        y: Vec<ExactRational>,
    },
    NormalizeWeights {
        weights: Vec<ExactRational>,
    },
    DiscreteExpectation {
        values: Vec<ExactRational>,
        weights: Vec<ExactRational>,
        #[serde(default)]
        normalize_weights: bool,
    },
    NormalizeExact {
        expression: ExactExpression,
    },
    ExactEqual {
        left: ExactExpression,
        right: ExactExpression,
    },
    ExactCompare {
        left: ExactExpression,
        right: ExactExpression,
    },
    PlaneDistance {
        a: Point2,
        b: Point2,
    },
    PlaneDotProduct {
        u: Point2,
        v: Point2,
    },
    PlaneOrthogonal {
        u: Point2,
        v: Point2,
    },
    TriangleArea {
        a: Point2,
        b: Point2,
        c: Point2,
    },
    LineLineIntersections {
        first: Line2,
        second: Line2,
    },
    LineCircleIntersections {
        line: Line2,
        circle: Circle2,
    },
    CircleCircleIntersections {
        first: Circle2,
        second: Circle2,
    },
    ConstantAcceleration {
        context: PhysicalContext,
        initial_position: VectorQuantity,
        initial_velocity: VectorQuantity,
        acceleration: VectorQuantity,
        duration: ScalarQuantity,
    },
    NewtonForce {
        context: PhysicalContext,
        mass: ScalarQuantity,
        acceleration: VectorQuantity,
    },
    NewtonAcceleration {
        context: PhysicalContext,
        mass: ScalarQuantity,
        net_force: VectorQuantity,
    },
    ConstantForceWork {
        context: PhysicalContext,
        force: VectorQuantity,
        displacement: VectorQuantity,
    },
    KineticEnergy {
        context: PhysicalContext,
        mass: ScalarQuantity,
        velocity: VectorQuantity,
    },
    WorkEnergy {
        context: PhysicalContext,
        mass: ScalarQuantity,
        initial_velocity: VectorQuantity,
        final_velocity: VectorQuantity,
        net_work: ScalarQuantity,
    },
}

impl CalculationRequest {
    fn operation(&self) -> Operation {
        match self {
            Self::SimplifySqrt { .. } => Operation::SimplifySqrt,
            Self::DescribeData { .. } => Operation::DescribeData,
            Self::DataVariance { .. } => Operation::DataVariance,
            Self::DataCovariance { .. } => Operation::DataCovariance,
            Self::NormalizeWeights { .. } => Operation::NormalizeWeights,
            Self::DiscreteExpectation { .. } => Operation::DiscreteExpectation,
            Self::NormalizeExact { .. } => Operation::NormalizeExact,
            Self::ExactEqual { .. } => Operation::ExactEqual,
            Self::ExactCompare { .. } => Operation::ExactCompare,
            Self::PlaneDistance { .. } => Operation::PlaneDistance,
            Self::PlaneDotProduct { .. } => Operation::PlaneDotProduct,
            Self::PlaneOrthogonal { .. } => Operation::PlaneOrthogonal,
            Self::TriangleArea { .. } => Operation::TriangleArea,
            Self::LineLineIntersections { .. } => Operation::LineLineIntersections,
            Self::LineCircleIntersections { .. } => Operation::LineCircleIntersections,
            Self::CircleCircleIntersections { .. } => Operation::CircleCircleIntersections,
            Self::ConstantAcceleration { .. } => Operation::ConstantAcceleration,
            Self::NewtonForce { .. } => Operation::NewtonForce,
            Self::NewtonAcceleration { .. } => Operation::NewtonAcceleration,
            Self::ConstantForceWork { .. } => Operation::ConstantForceWork,
            Self::KineticEnergy { .. } => Operation::KineticEnergy,
            Self::WorkEnergy { .. } => Operation::WorkEnergy,
        }
    }
    fn value(&self) -> Value {
        if self.operation().extended() {
            return Value::String(serde_json::to_string(self).expect("typed request serializes"));
        }
        match self {
            Self::SimplifySqrt { radicand } => Value::Integer(*radicand),
            Self::DescribeData { values } | Self::DataVariance { values } => {
                Value::Vector(values.iter().copied().map(Value::Integer).collect())
            }
            _ => unreachable!(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.operation().physical() {
            return crate::physics::validate(self);
        }
        if self.operation().geometric() {
            return crate::geometry::validate(self);
        }
        if self.operation().extended() {
            return crate::exact::validate(self);
        }
        match self {
            Self::SimplifySqrt { radicand } if !(0..=200_000_000).contains(radicand) => {
                Err("CAPABILITY_INPUT_INVALID: real radicand must be in 0..=200000000".into())
            }
            Self::DescribeData { values } | Self::DataVariance { values }
                if values.is_empty()
                    || values.len() > 64
                    || values.iter().any(|x| x.unsigned_abs() > 1_000_000) =>
            {
                Err(
                    "CAPABILITY_INPUT_INVALID: require 1..=64 integers with magnitude <=1000000"
                        .into(),
                )
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

impl Rational {
    fn matches(&self, numerator: i128, denominator: i128) -> bool {
        self.denominator > 0
            && gcd(self.numerator as i128, self.denominator as i128) == 1
            && self.numerator as i128 * denominator == numerator * self.denominator as i128
    }
}
fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum CalculationAnswer {
    Physics {
        result: Box<PhysicsResult>,
    },
    Radical {
        coefficient: i64,
        radicand: i64,
    },
    Summary {
        count: usize,
        mean: Rational,
        median: Rational,
        lower_quartile: Rational,
        upper_quartile: Rational,
        range: i64,
    },
    Variance {
        count: usize,
        mean: Rational,
        population_variance: Rational,
        sample_variance: Option<Rational>,
    },
    Covariance {
        count: usize,
        mean_x: ExactRational,
        mean_y: ExactRational,
        population_covariance: ExactRational,
        sample_covariance: Option<ExactRational>,
    },
    Weights {
        total_weight: ExactRational,
        probabilities: Vec<ExactRational>,
    },
    Expectation {
        value: ExactRational,
        total_weight: ExactRational,
        probabilities: Vec<ExactRational>,
        normalized: bool,
    },
    ExactExpression {
        expression: ExactExpression,
    },
    ExactComparison {
        comparison: ExactComparison,
    },
    PlaneDistance {
        squared_distance: ExactRational,
        distance: QuadraticSurd,
        coincident: bool,
    },
    PlaneDotProduct {
        value: ExactRational,
    },
    PlaneOrthogonality {
        dot_product: ExactRational,
        orthogonal: bool,
        zero_vector_present: bool,
        angle_defined: bool,
    },
    TriangleArea {
        area: ExactRational,
        signed_double_area: ExactRational,
        degeneracy: TriangleDegeneracy,
    },
    GeometryIntersections {
        intersection: IntersectionResult,
    },
}

/// A runtime arithmetic check, not a Lean proof or a floating-point comparison.
pub fn verify_answer(
    request: &CalculationRequest,
    answer: &CalculationAnswer,
) -> Result<Vec<String>, String> {
    request.validate()?;
    if request.operation().physical() {
        return crate::physics::verify(request, answer);
    }
    if request.operation().geometric() {
        return crate::geometry::verify(request, answer);
    }
    if request.operation().extended() {
        return crate::exact::verify(request, answer);
    }
    let valid = match (request, answer) {
        (
            CalculationRequest::SimplifySqrt { radicand: n },
            CalculationAnswer::Radical {
                coefficient: a,
                radicand: b,
            },
        ) => {
            if *n == 0 {
                *a == 0 && *b == 1
            } else if *a <= 0 || *a > *n || *b < 1 || *b > *n {
                false
            } else {
                let mut factor = 2_i64;
                let mut square_free = true;
                while factor * factor <= *b {
                    if b % (factor * factor) == 0 {
                        square_free = false;
                        break;
                    }
                    factor += 1;
                }
                *a as i128 * *a as i128 * *b as i128 == *n as i128 && square_free
            }
        }
        (
            CalculationRequest::DescribeData { values },
            CalculationAnswer::Summary {
                count,
                mean,
                median,
                lower_quartile,
                upper_quartile,
                range,
            },
        ) => {
            let mut sorted = values.clone();
            sorted.sort();
            let n = sorted.len();
            let middle = |part: &[i64], value: &Rational| {
                let m = part.len();
                if m % 2 == 1 {
                    value.matches(part[m / 2] as i128, 1)
                } else {
                    value.matches(part[m / 2 - 1] as i128 + part[m / 2] as i128, 2)
                }
            };
            let half = (n / 2).max(1);
            *count == n
                && mean.matches(values.iter().map(|x| *x as i128).sum(), n as i128)
                && middle(&sorted, median)
                && middle(&sorted[..half], lower_quartile)
                && middle(&sorted[n - half..], upper_quartile)
                && *range == sorted[n - 1] - sorted[0]
        }
        (
            CalculationRequest::DataVariance { values },
            CalculationAnswer::Variance {
                count,
                mean,
                population_variance,
                sample_variance,
            },
        ) => {
            let n = values.len() as i128;
            // Pairwise differences independently check ReasonScript's n*sum(x²)-sum(x)².
            let mut differences = 0_i128;
            for (i, x) in values.iter().enumerate() {
                for y in &values[..i] {
                    differences += (*x as i128 - *y as i128).pow(2);
                }
            }
            *count == values.len()
                && mean.matches(values.iter().map(|x| *x as i128).sum(), n)
                && population_variance.matches(differences, n * n)
                && if n == 1 {
                    sample_variance.is_none()
                } else {
                    sample_variance
                        .as_ref()
                        .is_some_and(|v| v.matches(differences, n * (n - 1)))
                }
        }
        _ => false,
    };
    if !valid {
        return Err("CAPABILITY_OUTPUT_INVALID: independent exact check failed".into());
    }
    Ok(match request {
        CalculationRequest::SimplifySqrt { .. } => vec![
            "NONNEGATIVE_PRINCIPAL_ROOT",
            "SQUARE_RECONSTRUCTION",
            "SQUARE_FREE_RADICAND",
        ],
        CalculationRequest::DescribeData { .. } => vec![
            "COUNT_AND_EXACT_MEAN",
            "SORTED_MEDIAN_AND_QUARTILES",
            "RANGE",
        ],
        CalculationRequest::DataVariance { .. } => vec![
            "COUNT_AND_EXACT_MEAN",
            "PAIRWISE_DIFFERENCE_VARIANCE",
            "SAMPLE_DENOMINATOR_AND_DEFINEDNESS",
        ],
        _ => unreachable!(),
    }
    .into_iter()
    .map(str::to_owned)
    .collect())
}

pub(crate) struct CalculationExecutor(pub Operation);
impl CapabilityExecutor for CalculationExecutor {
    fn capability_id(&self) -> &'static str {
        self.0.capability()
    }
    fn supports_goal(&self, goal: &str) -> bool {
        goal == self.0.goal()
    }
    fn validate(
        &self,
        activation: &KnowledgeActivation,
        state: &SemanticState,
    ) -> Result<(), String> {
        if activation.capability_id != self.capability_id()
            || activation.source_state
                != stable_id("state", &[&state.canonical().map_err(|e| e.to_string())?])
        {
            return Err("RUO_STATE_MISMATCH".into());
        }
        Ok(())
    }
    fn execute(
        &self,
        request: &CapabilityRequest,
        state: &SemanticState,
    ) -> Result<CapabilityResult, String> {
        if request.capability_id != self.capability_id()
            || request.semantic_state_refs.len() != 1
            || !request.root_constraints.is_empty()
            || request.constraints != self.0.constraints()
        {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        let id = &request.semantic_state_refs[0];
        if !matches!(state.get(id), Some(Object::Entity(e)) if e.entity_type == self.0.entity_type())
        {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        let (value, _) = state
            .value_of(id)
            .map_err(|e| e.to_string())?
            .ok_or("CAPABILITY_INPUT_INVALID")?;
        let calculation = match (self.0, value) {
            (Operation::SimplifySqrt, Value::Integer(radicand)) => {
                CalculationRequest::SimplifySqrt {
                    radicand: *radicand,
                }
            }
            (op, Value::String(payload)) if op.extended() => {
                let parsed: CalculationRequest =
                    serde_json::from_str(payload).map_err(|_| "CAPABILITY_INPUT_INVALID")?;
                if parsed.operation() != op {
                    return Err("CAPABILITY_INPUT_INVALID".into());
                }
                parsed
            }
            (op, Value::Vector(items))
                if matches!(op, Operation::DescribeData | Operation::DataVariance) =>
            {
                let values = items
                    .iter()
                    .map(|item| {
                        if let Value::Integer(n) = item {
                            Ok(*n)
                        } else {
                            Err("CAPABILITY_INPUT_INVALID")
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if op == Operation::DescribeData {
                    CalculationRequest::DescribeData { values }
                } else {
                    CalculationRequest::DataVariance { values }
                }
            }
            _ => return Err("CAPABILITY_INPUT_INVALID".into()),
        };
        calculation.validate()?;
        let (answer, execution_metadata) = if self.0.extended() {
            let (answer, metadata) = if self.0.physical() {
                crate::physics::execute(&calculation)?
            } else if self.0.geometric() {
                crate::geometry::execute(&calculation)?
            } else {
                crate::exact::execute(&calculation)?
            };
            (
                serde_json::to_value(answer).map_err(|_| "CAPABILITY_OUTPUT_INVALID")?,
                metadata,
            )
        } else {
            let (source, extra) = match &calculation {
                CalculationRequest::SimplifySqrt { radicand } => {
                    (format!("JuniorHigh::JSimplifySqrt({radicand})"), vec![])
                }
                CalculationRequest::DescribeData { values } => (
                    format!("Descriptive::SummaryOf({})", json!(values)),
                    vec![("descriptive", "Descriptive")],
                ),
                CalculationRequest::DataVariance { values } => (
                    format!("Descriptive::VarianceOf({})", json!(values)),
                    vec![("descriptive", "Descriptive")],
                ),
                _ => unreachable!(),
            };
            let output = crate::session::run_reason_source(&source, &extra)
                .map_err(|e| format!("CAPABILITY_EXECUTION_FAILED: {e}"))?;
            if output["status"] != "CALCULATED" {
                return Err("CAPABILITY_OUTPUT_INVALID: ReasonScript did not calculate".into());
            }
            let answer = match self.0 {
                Operation::SimplifySqrt => {
                    json!({"kind":"RADICAL", "coefficient":output["outside"], "radicand":output["inside"]})
                }
                Operation::DescribeData => {
                    json!({"kind":"SUMMARY", "count":output["count"], "mean":output["mean"], "median":output["median"], "lower_quartile":output["lower_quartile"], "upper_quartile":output["upper_quartile"], "range":output["range"]})
                }
                Operation::DataVariance => {
                    let defined = output["sample_defined"]
                        .as_bool()
                        .ok_or("CAPABILITY_OUTPUT_INVALID")?;
                    json!({"kind":"VARIANCE", "count":output["count"], "mean":output["mean"], "population_variance":output["population_variance"], "sample_variance":if defined { output["sample_variance"].clone() } else { Json::Null }})
                }
                _ => unreachable!(),
            };
            (
                answer,
                json!({"executor":"ReasonScript", "expression":source, "verification":"independent exact integer arithmetic"}),
            )
        };
        let typed: CalculationAnswer = serde_json::from_value(answer.clone())
            .map_err(|e| format!("CAPABILITY_OUTPUT_INVALID: {e}"))?;
        let checks = verify_answer(&calculation, &typed)?;
        Ok(CapabilityResult {
            status: CapabilityStatus::Calculated,
            output: json!({"answer": answer, "checks":checks}),
            ru_ref: stable_id(
                "ru",
                &[
                    &request.knowledge_id,
                    self.capability_id(),
                    &stable_id("state", &[&state.canonical().map_err(|e| e.to_string())?]),
                ],
            ),
            rus_ref: request.rus_ref.clone(),
            execution_metadata,
        })
    }
}

pub fn calculation_knowledge() -> KnowledgeSpace {
    let mut space = KnowledgeSpace::legacy();
    let units: Vec<KnowledgeUnit> =
        serde_json::from_str(include_str!("../knowledge/exact_statistics_v0_1.json"))
            .expect("bundled calculation knowledge is JSON");
    space
        .register_batch(units)
        .expect("bundled calculation knowledge validates");
    let units = serde_json::from_str(include_str!("../knowledge/plane_geometry_v0_1.json"))
        .expect("bundled plane geometry knowledge is JSON");
    space
        .register_batch(units)
        .expect("bundled plane geometry knowledge validates");
    let units = serde_json::from_str(include_str!("../knowledge/classical_physics_v0_1.json"))
        .expect("bundled physics JSON");
    space
        .register_batch(units)
        .expect("bundled physics validates");
    space
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationResult {
    pub request: CalculationRequest,
    pub answer: CalculationAnswer,
    pub verification: Json,
    pub explanation: Vec<String>,
    pub knowledge_decision: KnowledgeDecision,
    pub trace: Json,
    pub semantic_state: SemanticState,
}

pub fn compute(request: &CalculationRequest) -> Result<CalculationResult, String> {
    compute_with_knowledge(request, &calculation_knowledge())
}

pub fn compute_with_knowledge(
    request: &CalculationRequest,
    space: &KnowledgeSpace,
) -> Result<CalculationResult, String> {
    request.validate()?;
    let operation = request.operation();
    let source = serde_json::to_string(request).map_err(|e| e.to_string())?;
    let problem_id = stable_id("calculation", &[&source]);
    let provenance = Provenance::input("calculation", &source);
    let mut state = SemanticState::default();
    let input_id = state
        .bind(
            "input",
            "calculation",
            &problem_id,
            operation.entity_type(),
            provenance.clone(),
        )
        .map_err(|e| e.to_string())?;
    let input_entity = state.get(&input_id).unwrap().occurrence_id().to_owned();
    let input_occurrence = state
        .assert_value(
            &input_id,
            request.value(),
            provenance.derived(vec![input_entity], "structured input"),
            Status::Known,
        )
        .map_err(|e| e.to_string())?;
    let registry = RuntimeCapabilityRegistry::calculation();
    let query = KnowledgeQuery {
        domain: operation.domain().into(),
        goal: operation.goal().into(),
        symbols: vec![if operation.physical() {
            "physical_quantity"
        } else if operation.geometric() {
            "plane2"
        } else if operation.domain() == "NUMBER_SYSTEM" {
            "integer"
        } else {
            "data"
        }
        .into()],
        relations: vec![],
        required_properties: vec![],
        constraints: operation.constraints(),
        semantic_state_refs: vec![input_id],
        // Check all matching units so a conflict cannot be hidden by truncation.
        limit: space.len(),
    };
    let mut decision = KnowledgeDecision::query(space, &state, query);
    let mut activation = decision
        .activate(space, &state, &registry)?
        .ok_or("NO_APPLICABLE_KNOWLEDGE")?;
    let executor = registry
        .get(&activation.capability_id)
        .ok_or("CAPABILITY_NOT_FOUND")?;
    executor.validate(&activation, &state)?;
    let execution = executor.execute(
        &CapabilityRequest {
            problem_id: problem_id.clone(),
            goal_ref: operation.goal().into(),
            knowledge_id: activation.knowledge_id.clone(),
            knowledge_occurrence: activation.knowledge_occurrence.clone(),
            capability_id: activation.capability_id.clone(),
            rus_ref: activation.selected_rus.clone(),
            semantic_state_refs: decision.query.semantic_state_refs.clone(),
            constraints: decision.query.constraints.clone(),
            root_constraints: vec![],
        },
        &state,
    )?;
    if execution.status != CapabilityStatus::Calculated
        || execution.ru_ref
            != stable_id(
                "ru",
                &[
                    &activation.knowledge_id,
                    &activation.capability_id,
                    &activation.source_state,
                ],
            )
        || execution.rus_ref != activation.selected_rus
    {
        return Err("CAPABILITY_OUTPUT_INVALID".into());
    }
    let answer: CalculationAnswer = serde_json::from_value(execution.output["answer"].clone())
        .map_err(|e| format!("CAPABILITY_OUTPUT_INVALID: {e}"))?;
    let checks = verify_answer(request, &answer)?;
    let verification = json!({"status":"VERIFIED", "method":"EXACT_INTEGER_CHECK", "checks":checks, "formal_status":"NOT_CHECKED"});
    let unit = space
        .get(&activation.knowledge_id)
        .ok_or("KNOWLEDGE_NOT_FOUND")?;
    let explanation = explain(request, &answer, unit);
    let result_id = state
        .bind(
            "result",
            "calculation",
            &problem_id,
            "EXACT_CALCULATION_RESULT",
            provenance.clone(),
        )
        .map_err(|e| e.to_string())?;
    let result_entity = state.get(&result_id).unwrap().occurrence_id().to_owned();
    let result_occurrence = state
        .assert_value(
            &result_id,
            semantic_value(&serde_json::to_value(&answer).unwrap()),
            provenance.derived(
                vec![input_occurrence, result_entity],
                "verified exact calculation",
            ),
            Status::Derived,
        )
        .map_err(|e| e.to_string())?;
    let mut trace = json!({
        "source":"MathLang_DSN/knowledge-runtime", "knowledge_id":unit.knowledge_id,
        "knowledge_occurrence":activation.knowledge_occurrence, "activation_id":activation.activation_id,
        "capability_id":activation.capability_id, "ru_ref":execution.ru_ref, "rus_ref":execution.rus_ref,
        "source_state":activation.source_state, "input_refs":decision.query.semantic_state_refs,
        "result_ref":result_id, "result":answer, "verification":verification, "explanation":explanation,
        "dependency_knowledge_refs":unit.dependencies, "provenance":unit.provenance,
        "execution_metadata":execution.execution_metadata,
    });
    let ruo_ref = stable_id("ruo", &[&trace.to_string()]);
    trace["ruo_ref"] = json!(ruo_ref);
    let mut evidence = Object::Evidence(Evidence {
        common: Common::unidentified(
            Status::Derived,
            provenance.derived(
                vec![result_occurrence],
                "independent calculation verification",
            ),
        ),
        evidence_type: "EXACT_CALCULATION_CHECK".into(),
        source: format!(
            "{} + MIRP exact checker",
            trace["execution_metadata"]["executor"]
                .as_str()
                .unwrap_or("calculation")
        ),
        supports: vec![result_id],
        contradicts: vec![],
    });
    evidence
        .common_mut()
        .metadata
        .insert("knowledge_runtime".into(), trace.clone());
    evidence.normalize_identity(&ruo_ref);
    let occurrence = evidence.occurrence_id().to_owned();
    let parents = evidence.common().provenance.parent_occurrence_ids.clone();
    state
        .apply(SemanticDelta {
            added: vec![evidence],
            dependencies: BTreeMap::from([(occurrence, parents)]),
            ..Default::default()
        })
        .map_err(|e| e.to_string())?;
    activation.ruo_ref = Some(ruo_ref);
    activation.result_state = Some(stable_id(
        "state",
        &[&state.canonical().map_err(|e| e.to_string())?],
    ));
    trace["result_state"] = json!(activation.result_state);
    decision.activation = Some(activation);
    state.validate().map_err(|e| e.to_string())?;
    Ok(CalculationResult {
        request: request.clone(),
        answer,
        verification,
        explanation,
        knowledge_decision: decision,
        trace,
        semantic_state: state,
    })
}

fn semantic_value(value: &Json) -> Value {
    match value {
        Json::Number(n) => Value::Integer(n.as_i64().expect("bounded exact integer")),
        Json::String(s) => Value::String(s.clone()),
        Json::Bool(b) => Value::Boolean(*b),
        Json::Array(items) => Value::Vector(items.iter().map(semantic_value).collect()),
        Json::Object(fields) => Value::Vector(
            fields
                .iter()
                .map(|(key, value)| {
                    Value::Vector(vec![Value::String(key.clone()), semantic_value(value)])
                })
                .collect(),
        ),
        Json::Null => Value::Symbol("UNDEFINED".into()),
    }
}

fn explain(
    request: &CalculationRequest,
    answer: &CalculationAnswer,
    unit: &KnowledgeUnit,
) -> Vec<String> {
    let mut steps = vec![format!(
        "使用Knowledge: {}。{}",
        unit.knowledge_id, unit.conclusion
    )];
    if request.operation().physical() {
        steps.extend(crate::physics::explain(request, answer));
    } else if request.operation().geometric() {
        steps.extend(crate::geometry::explain(request, answer));
    } else if request.operation().extended() {
        steps.extend(crate::exact::explain(request, answer));
    } else {
        match (request, answer) {
            (
                CalculationRequest::SimplifySqrt { radicand: n },
                CalculationAnswer::Radical {
                    coefficient: a,
                    radicand: b,
                },
            ) => {
                steps.push(format!(
                    "{n} = {a}² × {b} より、主平方根は {a}√{b}。小数への丸めは行いません。"
                ));
                steps.push(
                "非負性、二乗による元の値の復元、根号内に平方因子が残らないことを検算しました。"
                    .into(),
            );
            }
            (CalculationRequest::DescribeData { .. }, _) => {
                steps.push(
                    "平均は合計÷個数。中央値は昇順データの中央、偶数個なら中央2値の平均です。"
                        .into(),
                );
                steps.push("四分位数は上下半分の中央値です。奇数個では全体の中央値を除外し、1個の場合は全四分位数をその値とします。範囲は最大値−最小値です。".into());
                steps.push(
                    "独立にソートしたデータと整数の等式で、すべての値を検算しました。".into(),
                );
            }
            (CalculationRequest::DataVariance { values }, _) => {
                steps.push("nを個数、S=Σx、Q=Σx²として、平均=S/n、母分散=(nQ−S²)/n²、不偏分散=(nQ−S²)/(n(n−1))を計算します。".into());
                if values.len() == 1 {
                    steps.push("標本数1の母分散は0、不偏分散は未定義のためnullです。".into());
                }
                steps.push(
                "別の式 Σ(i<j)(xᵢ−xⱼ)² を用いて両分散を検算しました。有理数は既約分数で返します。"
                    .into(),
            );
            }
            _ => unreachable!(),
        }
    }
    steps.push("これは実行結果の厳密検算です。Leanによる形式証明の実行結果ではありません。".into());
    steps
}
