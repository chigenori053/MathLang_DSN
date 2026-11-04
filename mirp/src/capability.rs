//! Capability dispatch is independent of Knowledge and legacy numeric Rule IDs.
use crate::intent::ConstraintKind;
use crate::knowledge::KnowledgeActivation;
use crate::session::run_reason_source;
use crate::{Object, SemanticState, Value};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KnowledgeRuntimeError {
    KnowledgeNotFound,
    NoApplicableKnowledge,
    KnowledgeConflict,
    KnowledgeDependencyMissing,
    KnowledgeDependencyCycle,
    CapabilityNotFound,
    CapabilityInputInvalid,
    CapabilityExecutionFailed,
    CapabilityOutputInvalid,
    RusNotAvailable,
    RuoKnowledgeMismatch,
    RuoCapabilityMismatch,
    RuoStateMismatch,
}

impl std::fmt::Display for KnowledgeRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().trim_matches('"')
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningCapability {
    pub capability_id: String,
    pub domain: String,
    pub supported_goals: Vec<String>,
    pub input_contract: String,
    pub output_contract: String,
    pub executor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub problem_id: String,
    pub goal_ref: String,
    pub knowledge_id: String,
    pub knowledge_occurrence: String,
    pub capability_id: String,
    pub rus_ref: String,
    pub semantic_state_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub root_constraints: Vec<ConstraintKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResult {
    pub status: CapabilityStatus,
    pub output: Json,
    pub ru_ref: String,
    pub rus_ref: String,
    pub execution_metadata: Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityStatus {
    Calculated,
    Factored,
    Solved,
    NoRealRoots,
    Unsupported,
    ResourceLimit,
    Unknown,
}

pub trait CapabilityExecutor: Send + Sync {
    fn capability_id(&self) -> &'static str;
    fn supports_goal(&self, goal: &str) -> bool;
    fn validate(
        &self,
        activation: &KnowledgeActivation,
        state: &SemanticState,
    ) -> Result<(), String>;
    fn execute(
        &self,
        request: &CapabilityRequest,
        state: &SemanticState,
    ) -> Result<CapabilityResult, String>;
}

#[derive(Default)]
pub struct RuntimeCapabilityRegistry {
    executors: BTreeMap<String, Box<dyn CapabilityExecutor>>,
}

impl RuntimeCapabilityRegistry {
    /// Bounded exact mathematics, classical physics and finite quantum executors.
    pub fn calculation() -> Self {
        let mut registry = Self::quadratic();
        for operation in crate::calculation::Operation::ALL {
            registry
                .register(Box::new(crate::calculation::CalculationExecutor(operation)))
                .unwrap();
        }
        registry
    }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quadratic() -> Self {
        let mut registry = Self::new();
        registry
            .register(Box::new(QuadraticExecutor {
                id: "FACTOR_QUADRATIC_INTEGER",
                goal: "FACTOR_EXPRESSION",
                factorization: true,
            }))
            .unwrap();
        registry
            .register(Box::new(QuadraticExecutor {
                id: "SOLVE_QUADRATIC",
                goal: "SOLVE_EQUATION",
                factorization: false,
            }))
            .unwrap();
        registry
    }

    pub fn register(&mut self, executor: Box<dyn CapabilityExecutor>) -> Result<(), String> {
        let id = executor.capability_id();
        if self.executors.contains_key(id) {
            return Err(format!("DUPLICATE_CAPABILITY: {id}"));
        }
        self.executors.insert(id.into(), executor);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&dyn CapabilityExecutor> {
        self.executors.get(id).map(Box::as_ref)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.executors.keys().map(String::as_str)
    }
}

struct QuadraticExecutor {
    id: &'static str,
    goal: &'static str,
    factorization: bool,
}

impl CapabilityExecutor for QuadraticExecutor {
    fn capability_id(&self) -> &'static str {
        self.id
    }

    fn supports_goal(&self, goal: &str) -> bool {
        goal == self.goal
    }

    fn validate(
        &self,
        activation: &KnowledgeActivation,
        state: &SemanticState,
    ) -> Result<(), String> {
        if activation.capability_id != self.id {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        if state.canonical().is_err() {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        Ok(())
    }

    fn execute(
        &self,
        request: &CapabilityRequest,
        state: &SemanticState,
    ) -> Result<CapabilityResult, String> {
        if request.capability_id != self.id || request.semantic_state_refs.len() != 1 {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        let id = &request.semantic_state_refs[0];
        if !matches!(state.get(id), Some(Object::Entity(entity)) if entity.entity_type == "EQUATION")
        {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        let Some((Value::Vector(items), _)) =
            state.value_of(id).map_err(|_| "CAPABILITY_INPUT_INVALID")?
        else {
            return Err("CAPABILITY_INPUT_INVALID".into());
        };
        let [Value::Integer(c), Value::Integer(b), Value::Integer(a)] = items.as_slice() else {
            return Err("CAPABILITY_INPUT_INVALID".into());
        };
        if *a == 0 {
            return Err("CAPABILITY_INPUT_INVALID".into());
        }
        let (mut kinds, mut lefts, mut rights) = (vec![], vec![], vec![]);
        for constraint in &request.root_constraints {
            let (kind, left, right) = match constraint {
                ConstraintKind::GreaterThan(n) => (1, *n, 0),
                ConstraintKind::GreaterOrEqual(n) => (2, *n, 0),
                ConstraintKind::LessThan(n) => (3, *n, 0),
                ConstraintKind::LessOrEqual(n) => (4, *n, 0),
                ConstraintKind::NotEqual(n) => (5, *n, 0),
                ConstraintKind::ClosedInterval(left, right) => (6, *left, *right),
            };
            kinds.push(kind);
            lefts.push(left);
            rights.push(right);
        }
        let source = format!(
            "JuniorHigh::JProcessQuadratic({}, {a}, {b}, {c}, {}, {}, {})",
            self.factorization,
            serde_json::to_string(&kinds).unwrap(),
            serde_json::to_string(&lefts).unwrap(),
            serde_json::to_string(&rights).unwrap(),
        );
        let output = run_reason_source(&source, &[])
            .map_err(|e| format!("CAPABILITY_EXECUTION_FAILED: {e}"))?;
        if !output["roots"].is_array()
            || !output["integer_roots"].is_array()
            || !output["validated_roots"].is_array()
            || !output["filtered_roots"].is_array()
            || !output["factors"].is_array()
            || !output["verified"].is_boolean()
        {
            return Err("CAPABILITY_OUTPUT_INVALID".into());
        }
        let status = serde_json::from_value(output["status"].clone())
            .map_err(|_| "CAPABILITY_OUTPUT_INVALID")?;
        Ok(CapabilityResult {
            status,
            output: json!({
                "roots": output["roots"],
                "integer_roots": output["integer_roots"],
                "validated_roots": output["validated_roots"],
                "filtered_roots": output["filtered_roots"],
                "factors": output["factors"],
                "verified": output["verified"],
            }),
            ru_ref: crate::stable_id(
                "ru",
                &[
                    &request.knowledge_id,
                    self.id,
                    &crate::stable_id(
                        "state",
                        &[&state.canonical().map_err(|_| "CAPABILITY_INPUT_INVALID")?],
                    ),
                ],
            ),
            rus_ref: request.rus_ref.clone(),
            execution_metadata: json!({"executor": "ReasonScript/JuniorHigh"}),
        })
    }
}
