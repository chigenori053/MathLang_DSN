//! Capability dispatch is independent of Knowledge and legacy numeric Rule IDs.
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quadratic() -> Self {
        let mut registry = Self::new();
        registry
            .register(Box::new(QuadraticExecutor {
                id: "FACTOR_QUADRATIC_INTEGER",
                goal: "FACTOR_EXPRESSION",
                function: "JFactorQuadratic",
            }))
            .unwrap();
        registry
            .register(Box::new(QuadraticExecutor {
                id: "SOLVE_QUADRATIC",
                goal: "SOLVE_EQUATION",
                function: "JSolveQuadratic",
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
    function: &'static str,
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
        let output = run_reason_source(
            &format!("JuniorHigh::{}({a}, {b}, {c})", self.function),
            &[],
        )
        .map_err(|e| format!("CAPABILITY_EXECUTION_FAILED: {e}"))?;
        if !output["roots"].is_array() {
            return Err("CAPABILITY_OUTPUT_INVALID".into());
        }
        let status = serde_json::from_value(output["status"].clone())
            .map_err(|_| "CAPABILITY_OUTPUT_INVALID")?;
        Ok(CapabilityResult {
            status,
            output: json!({"roots": output["roots"]}),
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
