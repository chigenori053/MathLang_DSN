//! ProblemIntent to native ReasonScript reasoning, with MIRP as mathematical state.
use crate::capability::{CapabilityRequest, RuntimeCapabilityRegistry};
use crate::intent::{integer_quadratic_roots, ConstraintKind, Method, Polynomial};
use crate::knowledge::{
    Applicability, ApplicabilityEngine, KnowledgeActivation, KnowledgeQuery, KnowledgeQueryResult,
};
use crate::memory::MemorySpace;
use crate::pif::{
    parse_problem, CriterionKind, CriterionStatus, GoalKind, GoalStatus, IntentRuntimeState,
    IntentStatus, MethodEvidence, ProblemDefinition,
};
use crate::{
    stable_id, Common, Evidence as MirpEvidence, Modality, Object, Polarity, Provenance, Relation,
    SemanticDelta, SemanticState, Status, Value,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasoningStatus {
    Complete,
    Partial,
    Invalid,
    Unknown,
    Ambiguous,
    Unsupported,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeGoal {
    Solve,
    Factorization,
    Expansion,
    Verification,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    Factorization,
    QuadraticFormula,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPlan {
    pub problem_id: String,
    pub goal_ref: String,
    pub runtime_goal: RuntimeGoal,
    pub candidate_strategies: Vec<Strategy>,
    pub required_method: Option<Method>,
    pub constraints: Vec<ConstraintKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTrace {
    pub ru_ref: String,
    pub rus_ref: String,
    pub ruo_ref: String,
    pub ruo: Json,
    pub state_ref: String,
    pub knowledge_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentEvidence {
    pub semantic_refs: Vec<String>,
    pub occurrence_refs: Vec<String>,
    pub state_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub criterion_refs: Vec<String>,
    pub ru_refs: Vec<String>,
    pub rus_refs: Vec<String>,
    pub ruo_refs: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemEvaluation {
    pub mathematical_status: String,
    pub problem_relevance: String,
    pub intent_alignment: String,
    pub goal_completion: String,
    pub method_compliance: String,
    pub constraint_status: String,
    pub primary_errors: Vec<String>,
    pub downstream_errors: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathLangResult {
    pub answer: Option<Vec<i64>>,
    pub status: ReasoningStatus,
    pub problem_evaluation: ProblemEvaluation,
    pub trace: Vec<NativeTrace>,
    pub evidence: Vec<IntentEvidence>,
    pub knowledge_decisions: Vec<KnowledgeDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDecision {
    pub query: KnowledgeQuery,
    pub retrieval: KnowledgeQueryResult,
    pub applicability: Vec<(String, Applicability)>,
    pub activation: Option<KnowledgeActivation>,
}

pub struct MathProblemContext {
    definition: ProblemDefinition,
    semantic_state: SemanticState,
    intent_runtime: IntentRuntimeState,
    reasoning_runtime: ReasoningRuntimeState,
    history: Vec<ReasoningTransition>,
    plan: ReasoningPlan,
    trace: Vec<NativeTrace>,
    evidence: Vec<IntentEvidence>,
    memory: MemorySpace,
    knowledge_decisions: Vec<KnowledgeDecision>,
    active_knowledge: Option<KnowledgeActivation>,
    capabilities: RuntimeCapabilityRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningRuntimeState {
    pub selected_ru: Option<String>,
    pub active_rus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTransition {
    pub id: String,
    pub state_version: u64,
    pub status: ReasoningStatus,
    pub ruo_refs: Vec<String>,
}

impl MathProblemContext {
    pub fn parse(text: &str) -> Result<Self, String> {
        let parsed = parse_problem(text)?;
        Self::new(parsed.definition, parsed.state)
    }

    pub fn new(
        definition: ProblemDefinition,
        semantic_state: SemanticState,
    ) -> Result<Self, String> {
        Self::with_memory(definition, semantic_state, MemorySpace::new())
    }

    pub fn with_memory(
        definition: ProblemDefinition,
        mut semantic_state: SemanticState,
        memory: MemorySpace,
    ) -> Result<Self, String> {
        definition.intent().validate(&semantic_state)?;
        let intent = definition.intent();
        let runtime_goal = match intent.primary_goal.as_ref().map(|g| &g.kind) {
            Some(GoalKind::Solve) => RuntimeGoal::Solve,
            Some(GoalKind::Factor) => RuntimeGoal::Factorization,
            Some(GoalKind::Expand) => RuntimeGoal::Expansion,
            Some(GoalKind::Verify) => RuntimeGoal::Verification,
            _ => RuntimeGoal::Unsupported,
        };
        let required_method = intent
            .method_constraints
            .first()
            .map(|m| m.required.clone());
        let plan = ReasoningPlan {
            problem_id: intent.problem_id.clone(),
            goal_ref: intent
                .primary_goal
                .as_ref()
                .map_or(String::new(), |g| g.id.clone()),
            runtime_goal,
            candidate_strategies: if intent.intent_status == IntentStatus::Known
                && intent.operation_intent.kind == Some(crate::intent::OperationIntent::Solve)
            {
                vec![Strategy::Factorization, Strategy::QuadraticFormula]
            } else {
                vec![]
            },
            required_method,
            constraints: intent.constraints.iter().map(|c| c.kind.clone()).collect(),
        };
        if let Some(given) = intent.givens.first() {
            let (name, kind) = if matches!(
                plan.runtime_goal,
                RuntimeGoal::Factorization | RuntimeGoal::Expansion
            ) {
                ("expression", "EXPRESSION")
            } else {
                ("equation", "EQUATION")
            };
            let provenance = given.provenance.derived(
                vec![given.source_occurrence_id.clone()],
                "mathematical recognition",
            );
            let equation_id = semantic_state
                .bind(
                    name,
                    "pini",
                    &intent.problem_id,
                    kind,
                    given.provenance.clone(),
                )
                .map_err(|e| e.to_string())?;
            let entity_occurrence = semantic_state
                .get(&equation_id)
                .unwrap()
                .occurrence_id()
                .to_owned();
            semantic_state
                .assert_value(
                    &equation_id,
                    polynomial_value(&given.equation),
                    provenance.derived(vec![entity_occurrence], "coefficient extraction"),
                    Status::Known,
                )
                .map_err(|e| e.to_string())?;
        }
        let goal_status = intent
            .primary_goal
            .iter()
            .chain(&intent.subgoals)
            .map(|g| (g.id.clone(), GoalStatus::Unresolved))
            .collect();
        let criterion_status = intent
            .completion_criteria
            .iter()
            .map(|c| (c.id.clone(), CriterionStatus::Unresolved))
            .collect();
        let intent_runtime = IntentRuntimeState {
            goal_status,
            criterion_status,
            active_goals: vec![],
            satisfied_goals: vec![],
            unresolved_goals: intent
                .primary_goal
                .iter()
                .chain(&intent.subgoals)
                .map(|g| g.id.clone())
                .collect(),
            method_evidence: vec![],
            evaluation_history: vec![],
        };
        let recognized = intent.givens.first().map(|g| g.equation.coefficients);
        let mut context = Self {
            definition,
            semantic_state,
            intent_runtime,
            reasoning_runtime: ReasoningRuntimeState::default(),
            history: vec![],
            plan,
            trace: vec![],
            evidence: vec![],
            memory,
            knowledge_decisions: vec![],
            active_knowledge: None,
            capabilities: RuntimeCapabilityRegistry::quadratic(),
        };
        if let Some(coefficients) = recognized {
            context.native_step(
                if matches!(
                    context.plan.runtime_goal,
                    RuntimeGoal::Factorization | RuntimeGoal::Expansion
                ) {
                    "PolynomialRecognitionRU"
                } else {
                    "QuadraticEquationRecognitionRU"
                },
                "QuadraticSolveRUS",
                json!({"coefficients": coefficients}),
            );
        }
        Ok(context)
    }

    pub fn definition(&self) -> &ProblemDefinition {
        &self.definition
    }
    pub fn semantic_state(&self) -> &SemanticState {
        &self.semantic_state
    }
    pub fn intent_runtime(&self) -> &IntentRuntimeState {
        &self.intent_runtime
    }
    pub fn reasoning_runtime(&self) -> &ReasoningRuntimeState {
        &self.reasoning_runtime
    }
    pub fn history(&self) -> &[ReasoningTransition] {
        &self.history
    }
    pub fn knowledge_decisions(&self) -> &[KnowledgeDecision] {
        &self.knowledge_decisions
    }

    pub fn capabilities_mut(&mut self) -> &mut RuntimeCapabilityRegistry {
        &mut self.capabilities
    }

    fn select_knowledge(
        &mut self,
        strategy: &Strategy,
        poly: &Polynomial,
    ) -> Result<Option<KnowledgeActivation>, String> {
        let equation_id = self
            .semantic_state
            .resolve("equation", "pini", &self.plan.problem_id)
            .ok_or("CAPABILITY_INPUT_INVALID: equation missing")?
            .common
            .id
            .clone();
        let query = KnowledgeQuery {
            domain: "EQUATION".into(),
            goal: match strategy {
                Strategy::Factorization => "FACTOR_EXPRESSION",
                Strategy::QuadraticFormula => "SOLVE_EQUATION",
            }
            .into(),
            symbols: vec!["quadratic".into()],
            relations: vec![],
            required_properties: vec![],
            constraints: if poly.coefficients[2] != 0
                && poly.coefficients.iter().all(|n| n.unsigned_abs() <= 10_000)
            {
                vec!["INTEGER_COEFFICIENTS".into(), "NONZERO_QUADRATIC".into()]
            } else {
                vec![]
            },
            semantic_state_refs: vec![equation_id],
            limit: 8,
        };
        let retrieval = self.memory.query_knowledge(&query);
        let mut applicability = vec![];
        for candidate in &retrieval.candidates {
            let unit = self
                .memory
                .get_knowledge(&candidate.knowledge_id)
                .ok_or("KNOWLEDGE_NOT_FOUND")?;
            let verdict = ApplicabilityEngine::evaluate(
                unit,
                candidate,
                &query,
                &self.semantic_state,
                |id| self.memory.get_knowledge(id).is_some(),
            );
            applicability.push((candidate.knowledge_id.clone(), verdict));
        }
        let selected_id = applicability
            .iter()
            .find(|(id, verdict)| {
                *verdict == Applicability::Applicable
                    && self
                        .memory
                        .get_knowledge(id)
                        .is_some_and(|unit| !unit.capabilities.is_empty())
            })
            .map(|(id, _)| id.clone());
        self.knowledge_decisions.push(KnowledgeDecision {
            query: query.clone(),
            retrieval,
            applicability,
            activation: None,
        });
        let Some(selected_id) = selected_id else {
            return Ok(None);
        };
        let unit = self
            .memory
            .get_knowledge(&selected_id)
            .ok_or("KNOWLEDGE_NOT_FOUND")?;
        let decision = self.knowledge_decisions.last().unwrap();
        let selected_class = decision
            .retrieval
            .candidates
            .iter()
            .find(|candidate| candidate.knowledge_id == selected_id)
            .unwrap()
            .activation_class
            .clone();
        for (other_id, verdict) in &decision.applicability {
            if *verdict != Applicability::Applicable || *other_id == selected_id {
                continue;
            }
            let other = self
                .memory
                .get_knowledge(other_id)
                .ok_or("KNOWLEDGE_NOT_FOUND")?;
            let other_class = decision
                .retrieval
                .candidates
                .iter()
                .find(|candidate| candidate.knowledge_id == *other_id)
                .unwrap()
                .activation_class
                .clone();
            let conflicts = |a: &crate::knowledge::KnowledgeUnit, b: &str| {
                a.activation_metadata
                    .get("conflicts_with")
                    .and_then(Json::as_array)
                    .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(b)))
            };
            if other.priority == unit.priority
                && other_class == selected_class
                && (conflicts(unit, other_id) || conflicts(other, &selected_id))
            {
                return Err("KNOWLEDGE_CONFLICT".into());
            }
        }
        let capability_id = unit
            .capabilities
            .iter()
            .find(|id| {
                self.capabilities
                    .get(id)
                    .is_some_and(|executor| executor.supports_goal(&query.goal))
            })
            .ok_or_else(|| format!("CAPABILITY_NOT_FOUND: {}", unit.capabilities.join(",")))?;
        let activation =
            KnowledgeActivation::for_capability(unit, &self.semantic_state, capability_id)
                .ok_or("RUS_NOT_AVAILABLE")?;
        self.knowledge_decisions.last_mut().unwrap().activation = Some(activation.clone());
        Ok(Some(activation))
    }

    fn native_step(&mut self, ru: &str, rus: &str, output: Json) -> NativeTrace {
        let ruo = json!({"source": "MathLang_DSN/native", "ru": ru, "rus": rus, "output": output});
        let trace = NativeTrace {
            ru_ref: ru.into(),
            rus_ref: rus.into(),
            ruo_ref: stable_id(
                "ruo",
                &[&self.plan.problem_id, &serde_json::to_string(&ruo).unwrap()],
            ),
            ruo,
            state_ref: self.semantic_state.version.to_string(),
            knowledge_ref: self
                .active_knowledge
                .as_ref()
                .map(|a| a.knowledge_id.clone()),
        };
        self.trace.push(trace.clone());
        trace
    }
    pub fn plan(&self) -> &ReasoningPlan {
        &self.plan
    }

    pub fn evaluate_submission(
        &self,
        target_ref: &str,
        submitted: &[i64],
    ) -> Result<MathLangResult, String> {
        let expected_target = self
            .definition
            .intent()
            .primary_goal
            .as_ref()
            .ok_or("missing goal")?
            .target_refs
            .first()
            .ok_or("missing target")?;
        if target_ref != expected_target {
            let mut result = self.result(
                Some(submitted.to_vec()),
                ReasoningStatus::Invalid,
                "VALID",
                "SATISFIED",
                "UNKNOWN",
            );
            result.problem_evaluation.problem_relevance = "WRONG_TARGET".into();
            result.problem_evaluation.intent_alignment = "VIOLATION".into();
            result
                .problem_evaluation
                .primary_errors
                .push("WRONG_TARGET".into());
            return Ok(result);
        }
        let solution = self
            .semantic_state
            .resolve("solution_set", "pini", &self.plan.problem_id)
            .ok_or("solution set not derived")?;
        let (value, _) = self
            .semantic_state
            .value_of(&solution.common.id)
            .map_err(|e| e.to_string())?
            .ok_or("solution set empty")?;
        let Value::Set(items) = value else {
            return Err("invalid solution set".into());
        };
        let mut expected: Vec<i64> = items
            .iter()
            .map(|item| match item {
                Value::Integer(n) => Ok(*n),
                _ => Err("noninteger root"),
            })
            .collect::<Result<_, _>>()?;
        let mut actual = submitted.to_vec();
        expected.sort();
        expected.dedup();
        actual.sort();
        actual.dedup();
        let status = if actual == expected {
            ReasoningStatus::Complete
        } else if actual.iter().all(|root| expected.contains(root)) {
            ReasoningStatus::Partial
        } else {
            ReasoningStatus::Invalid
        };
        let math = if status == ReasoningStatus::Invalid {
            "INVALID"
        } else {
            "VALID"
        };
        Ok(self.result(Some(actual), status, math, "SATISFIED", "SATISFIED"))
    }

    fn equation(&self) -> Result<Polynomial, String> {
        let id = self
            .semantic_state
            .resolve("equation", "pini", &self.plan.problem_id)
            .ok_or("missing equation")?
            .common
            .id
            .clone();
        let (value, _) = self
            .semantic_state
            .value_of(&id)
            .map_err(|e| e.to_string())?
            .ok_or("missing coefficients")?;
        let Value::Vector(items) = value else {
            return Err("invalid coefficients".into());
        };
        if items.len() != 3 {
            return Err("invalid coefficients".into());
        }
        let mut coefficients = [0; 3];
        for (i, item) in items.iter().enumerate() {
            let Value::Integer(number) = item else {
                return Err("invalid coefficient".into());
            };
            coefficients[i] = *number;
        }
        Ok(Polynomial { coefficients })
    }

    fn record(&mut self, label: &str, value: Value, trace: &NativeTrace) -> Result<(), String> {
        let source = self.definition.intent().provenance.clone();
        let parent = self
            .semantic_state
            .resolve("equation", "pini", &self.plan.problem_id)
            .unwrap()
            .common
            .occurrence_id
            .clone();
        let provenance = source.derived(vec![parent], &trace.ru_ref);
        let id = self
            .semantic_state
            .bind(label, "pini", &self.plan.problem_id, label, source)
            .map_err(|e| e.to_string())?;
        let occurrence = self
            .semantic_state
            .get(&id)
            .unwrap()
            .occurrence_id()
            .to_owned();
        let value_occurrence = self
            .semantic_state
            .assert_value(
                &id,
                value,
                provenance.derived(vec![occurrence], "RUO result"),
                Status::Derived,
            )
            .map_err(|e| e.to_string())?;
        let evidence = IntentEvidence {
            semantic_refs: vec![id.clone()],
            occurrence_refs: vec![value_occurrence.clone()],
            state_refs: vec![trace.state_ref.clone()],
            goal_refs: vec![self.plan.goal_ref.clone()],
            criterion_refs: self
                .definition
                .intent()
                .completion_criteria
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            ru_refs: vec![trace.ru_ref.clone()],
            rus_refs: vec![trace.rus_ref.clone()],
            ruo_refs: vec![trace.ruo_ref.clone()],
            provenance: provenance.clone(),
        };
        let equation_id = self
            .semantic_state
            .resolve("equation", "pini", &self.plan.problem_id)
            .unwrap()
            .common
            .id
            .clone();
        let mut relation = Object::Relation(Relation {
            common: Common::unidentified(
                Status::Derived,
                provenance.derived(vec![value_occurrence.clone()], "state derivation"),
            ),
            relation_type: "MATHEMATICAL.DERIVED_FROM".into(),
            source: id.clone(),
            target: equation_id,
            arguments: vec![],
            polarity: Polarity::Positive,
            modality: Modality::Asserted,
        });
        relation.normalize_identity(&format!("native-relation:{label}"));
        let relation_occurrence = relation.occurrence_id().to_owned();
        let relation_parents = relation.common().provenance.parent_occurrence_ids.clone();
        let mut mirp = Object::Evidence(MirpEvidence {
            common: Common::unidentified(
                Status::Derived,
                provenance.derived(vec![value_occurrence.clone()], "native trace evidence"),
            ),
            evidence_type: "RUNTIME_RESULT".into(),
            source: if trace.ruo["source"] == "MathLang_DSN/native" {
                "MathLang_DSN native"
            } else {
                "ReasonScript/DSN"
            }
            .into(),
            supports: vec![id],
            contradicts: vec![],
        });
        mirp.common_mut()
            .metadata
            .insert("native_trace".into(), json!(evidence));
        if trace.ruo["source"] == "MathLang_DSN/knowledge-runtime" {
            mirp.common_mut().metadata.insert(
                "knowledge_runtime".into(),
                json!({
                    "knowledge_id": trace.ruo["knowledge_id"],
                    "knowledge_occurrence": trace.ruo["knowledge_occurrence"],
                    "activation_id": trace.ruo["activation_id"],
                    "capability_id": trace.ruo["capability_id"],
                    "ru_ref": trace.ru_ref,
                    "rus_ref": trace.rus_ref,
                    "ruo_ref": trace.ruo_ref,
                    "source_state": trace.ruo["source_state"],
                    "result_state": (self.semantic_state.version + 1).to_string(),
                    "dependency_knowledge_refs": trace.ruo["dependency_knowledge_refs"],
                }),
            );
        }
        if let Some(active) = &self.active_knowledge {
            let mut applied = active.clone();
            applied.result_state = Some((self.semantic_state.version + 1).to_string());
            mirp.common_mut()
                .metadata
                .insert("knowledge_activation".into(), json!(applied));
        }
        mirp.normalize_identity(&format!("native-evidence:{label}"));
        let evidence_occurrence = mirp.occurrence_id().to_owned();
        let evidence_parents = mirp.common().provenance.parent_occurrence_ids.clone();
        self.semantic_state
            .apply(SemanticDelta {
                added: vec![mirp, relation],
                dependencies: BTreeMap::from([
                    (evidence_occurrence, evidence_parents),
                    (relation_occurrence, relation_parents),
                ]),
                ..Default::default()
            })
            .map_err(|e| e.to_string())?;
        if let Some(active) = &mut self.active_knowledge {
            active.result_state = Some(self.semantic_state.version.to_string());
            if let Some(decision) = self.knowledge_decisions.last_mut() {
                decision.activation = Some(active.clone());
            }
        }
        self.evidence.push(evidence);
        Ok(())
    }

    pub fn execute(&mut self, selected: Option<Strategy>) -> Result<MathLangResult, String> {
        let intent = self.definition.intent();
        let status = match intent.intent_status {
            IntentStatus::Ambiguous => ReasoningStatus::Ambiguous,
            IntentStatus::Unknown => ReasoningStatus::Unknown,
            _ if self.semantic_state.status == Status::Conflict => ReasoningStatus::Conflict,
            IntentStatus::Known if self.plan.runtime_goal != RuntimeGoal::Solve => {
                ReasoningStatus::Unsupported
            }
            IntentStatus::Known => ReasoningStatus::Unknown,
        };
        if status != ReasoningStatus::Unknown || self.plan.runtime_goal != RuntimeGoal::Solve {
            let result = self.result(None, status, "UNKNOWN", "UNKNOWN", "UNKNOWN");
            return Ok(self.finish(result));
        }
        let poly = self.equation()?;
        let a = poly.coefficients[2];
        let automatic_selection = selected.is_none();
        let strategy = selected.unwrap_or(match self.plan.required_method {
            Some(Method::QuadraticFormula) => Strategy::QuadraticFormula,
            _ => Strategy::Factorization,
        });
        let activation = match self.select_knowledge(&strategy, &poly) {
            Ok(Some(activation)) => activation,
            outcome => {
                let conflict = matches!(&outcome, Err(error) if error == "KNOWLEDGE_CONFLICT");
                let mut result = self.result(
                    None,
                    if conflict {
                        ReasoningStatus::Conflict
                    } else {
                        ReasoningStatus::Unsupported
                    },
                    "UNKNOWN",
                    "UNKNOWN",
                    "UNKNOWN",
                );
                result
                    .problem_evaluation
                    .primary_errors
                    .push(match outcome {
                        Err(error) => error,
                        _ => "NO_APPLICABLE_KNOWLEDGE".into(),
                    });
                return Ok(self.finish(result));
            }
        };
        self.active_knowledge = Some(activation.clone());
        let executor = self
            .capabilities
            .get(&activation.capability_id)
            .ok_or("CAPABILITY_NOT_FOUND")?;
        if stable_id(
            "state",
            &[&self.semantic_state.canonical().map_err(|e| e.to_string())?],
        ) != activation.source_state
        {
            return Err("RUO_STATE_MISMATCH".into());
        }
        executor.validate(&activation, &self.semantic_state)?;
        let query = &self
            .knowledge_decisions
            .last()
            .ok_or("NO_APPLICABLE_KNOWLEDGE")?
            .query;
        let request = CapabilityRequest {
            problem_id: self.plan.problem_id.clone(),
            goal_ref: self.plan.goal_ref.clone(),
            knowledge_id: activation.knowledge_id.clone(),
            knowledge_occurrence: activation.knowledge_occurrence.clone(),
            capability_id: activation.capability_id.clone(),
            rus_ref: activation.selected_rus.clone(),
            semantic_state_refs: query.semantic_state_refs.clone(),
            constraints: query.constraints.clone(),
        };
        let output = executor.execute(&request, &self.semantic_state)?;
        if output.ru_ref
            != stable_id(
                "ru",
                &[
                    &activation.knowledge_id,
                    &activation.capability_id,
                    &activation.source_state,
                ],
            )
        {
            return Err("CAPABILITY_OUTPUT_INVALID".into());
        }
        if output.rus_ref != activation.selected_rus {
            return Err("RUO_STATE_MISMATCH".into());
        }
        let unit = self
            .memory
            .get_knowledge(&activation.knowledge_id)
            .ok_or("KNOWLEDGE_NOT_FOUND")?;
        let mut ruo = json!({
            "source": "MathLang_DSN/knowledge-runtime",
            "knowledge_id": activation.knowledge_id,
            "knowledge_occurrence": activation.knowledge_occurrence,
            "activation_id": activation.activation_id,
            "capability_id": activation.capability_id,
            "ru_ref": output.ru_ref,
            "rus_ref": output.rus_ref,
            "source_state": activation.source_state,
            "result": output.output,
            "status": output.status,
            "execution_metadata": output.execution_metadata,
            "provenance": unit.provenance,
            "dependency_knowledge_refs": unit.dependencies,
        });
        if let Some(rule) = activation.legacy_runtime_rule_id {
            ruo["legacy_runtime_rule_id"] = json!(rule);
        }
        if ruo["knowledge_id"] != request.knowledge_id {
            return Err("RUO_KNOWLEDGE_MISMATCH".into());
        }
        if ruo["capability_id"] != request.capability_id {
            return Err("RUO_CAPABILITY_MISMATCH".into());
        }
        if ruo["source_state"] != activation.source_state {
            return Err("RUO_STATE_MISMATCH".into());
        }
        let ru_ref = ruo["ru_ref"]
            .as_str()
            .ok_or("CAPABILITY_OUTPUT_INVALID")?
            .to_owned();
        let rus_ref = ruo["rus_ref"]
            .as_str()
            .ok_or("CAPABILITY_OUTPUT_INVALID")?
            .to_owned();
        let ruo_ref = stable_id(
            "ruo",
            &[&serde_json::to_string(&ruo).unwrap(), &self.plan.problem_id],
        );
        let trace = NativeTrace {
            ru_ref,
            rus_ref,
            ruo_ref,
            ruo,
            state_ref: self.semantic_state.version.to_string(),
            knowledge_ref: Some(activation.knowledge_id.clone()),
        };
        if let Some(active) = &mut self.active_knowledge {
            active.ruo_ref = Some(trace.ruo_ref.clone());
        }
        if let Some(decision) = self.knowledge_decisions.last_mut() {
            decision.activation = self.active_knowledge.clone();
        }
        self.reasoning_runtime.selected_ru = Some(trace.ru_ref.clone());
        self.reasoning_runtime.active_rus = Some(trace.rus_ref.clone());
        self.trace.push(trace.clone());
        let mut roots = vec![];
        let mut all_integral = true;
        for root in trace.ruo["result"]["roots"]
            .as_array()
            .ok_or("CAPABILITY_OUTPUT_INVALID")?
        {
            let numerator = root["numerator"].as_i64().ok_or("invalid numerator")?;
            let radical = root["radical_coefficient"]
                .as_i64()
                .ok_or("invalid radical")?;
            let denominator = root["denominator"].as_i64().ok_or("invalid denominator")?;
            if radical != 0 || denominator == 0 || numerator % denominator != 0 {
                all_integral = false;
                continue;
            }
            roots.push(numerator / denominator);
        }
        roots.sort();
        roots.dedup();
        let native_status = trace.ruo["status"].as_str().unwrap_or("UNKNOWN");
        if native_status == "UNSUPPORTED" || native_status == "RESOURCE_LIMIT" {
            let result = self.result(
                None,
                ReasoningStatus::Unsupported,
                "UNKNOWN",
                "UNKNOWN",
                "UNKNOWN",
            );
            return Ok(self.finish(result));
        }
        let exact_roots = integer_quadratic_roots(&poly);
        if !all_integral
            || native_status == "UNKNOWN"
            || exact_roots
                .as_ref()
                .is_none_or(|expected| *expected != roots)
        {
            if automatic_selection
                && strategy == Strategy::Factorization
                && self.plan.required_method.is_none()
            {
                return self.execute(Some(Strategy::QuadraticFormula));
            }
            let result = self.result(
                None,
                ReasoningStatus::Unknown,
                "UNKNOWN",
                "UNKNOWN",
                "UNKNOWN",
            );
            return Ok(self.finish(result));
        }
        let is_factorization = activation.capability_id == "FACTOR_QUADRATIC_INTEGER";
        let factor_validation = if is_factorization {
            Some(self.native_step(
                "FactorValidationRU",
                "QuadraticFactorizationRUS",
                json!({"validated_roots": roots}),
            ))
        } else {
            None
        };
        if is_factorization {
            let factors = std::iter::once(a)
                .chain(roots.iter().copied())
                .map(Value::Integer)
                .collect();
            self.record(
                "factored_form",
                Value::Vector(factors),
                factor_validation.as_ref().unwrap(),
            )?;
        }
        self.record("candidate_roots", root_set(&roots), &trace)?;
        let validated: Vec<i64> = roots.into_iter().filter(|x| poly.at(*x) == 0).collect();
        let validation_trace = self.native_step(
            "RootValidationRU",
            "QuadraticSolveRUS",
            json!({"validated_roots": validated}),
        );
        self.record("validated_roots", root_set(&validated), &validation_trace)?;
        let filtered: Vec<i64> = validated
            .iter()
            .copied()
            .filter(|x| self.plan.constraints.iter().all(|c| c.accepts(*x)))
            .collect();
        let filter_trace = self.native_step(
            "ConstraintFilterRU",
            "QuadraticSolveRUS",
            json!({"filtered_roots": filtered}),
        );
        self.record("filtered_roots", root_set(&filtered), &filter_trace)?;
        let answer_trace = self.native_step(
            "AnswerConstructionRU",
            "QuadraticSolveRUS",
            json!({"answer": filtered}),
        );
        self.record("solution_set", root_set(&filtered), &answer_trace)?;
        let method = match (
            &self.plan.required_method,
            activation.capability_id.as_str(),
        ) {
            (None, _)
            | (Some(Method::Factorization), "FACTOR_QUADRATIC_INTEGER")
            | (Some(Method::QuadraticFormula), "SOLVE_QUADRATIC") => "SATISFIED",
            _ => "VIOLATION",
        };
        let complete =
            method == "SATISFIED" && (native_status == "NO_REAL_ROOTS" || !validated.is_empty());
        let status = if complete {
            ReasoningStatus::Complete
        } else {
            ReasoningStatus::Partial
        };
        let result = self.result(Some(filtered), status, "VALID", method, "SATISFIED");
        Ok(self.finish(result))
    }

    fn finish(&mut self, result: MathLangResult) -> MathLangResult {
        let execution = result
            .trace
            .iter()
            .rev()
            .find(|step| step.ruo["source"] == "MathLang_DSN/knowledge-runtime");
        let transition_id = stable_id(
            "native-transition",
            &[
                &self.plan.problem_id,
                execution
                    .or_else(|| result.trace.last())
                    .map_or("", |t| t.ruo_ref.as_str()),
            ],
        );
        if let Some(execution) = execution {
            let strategy = if execution.ruo["capability_id"] == "FACTOR_QUADRATIC_INTEGER" {
                Method::Factorization
            } else {
                Method::QuadraticFormula
            };
            self.intent_runtime.method_evidence.push(MethodEvidence {
                strategy,
                transition_refs: vec![transition_id.clone()],
                ru_refs: vec![execution.ru_ref.clone()],
                rus_refs: vec![execution.rus_ref.clone()],
                ruo_refs: vec![execution.ruo_ref.clone()],
                provenance: self.definition.intent().provenance.clone(),
            });
        }
        let goal = &self.plan.goal_ref;
        if !goal.is_empty() {
            let goal_status = match result.status {
                ReasoningStatus::Complete => GoalStatus::Satisfied,
                ReasoningStatus::Partial => GoalStatus::Partial,
                ReasoningStatus::Conflict => GoalStatus::Conflict,
                ReasoningStatus::Unknown
                | ReasoningStatus::Ambiguous
                | ReasoningStatus::Unsupported => GoalStatus::Unknown,
                ReasoningStatus::Invalid => GoalStatus::Unresolved,
            };
            self.intent_runtime
                .goal_status
                .insert(goal.clone(), goal_status);
            if result.status == ReasoningStatus::Complete {
                self.intent_runtime.satisfied_goals.push(goal.clone());
                self.intent_runtime.unresolved_goals.retain(|id| id != goal);
            }
        }
        for criterion in &self.definition.intent().completion_criteria {
            let status = if criterion.kind == CriterionKind::MethodUsed
                && result.problem_evaluation.method_compliance == "VIOLATION"
            {
                CriterionStatus::Violated
            } else if result.status == ReasoningStatus::Complete {
                CriterionStatus::Satisfied
            } else if result.status == ReasoningStatus::Unknown
                || result.status == ReasoningStatus::Ambiguous
                || result.status == ReasoningStatus::Unsupported
            {
                CriterionStatus::Unknown
            } else {
                CriterionStatus::Unresolved
            };
            self.intent_runtime
                .criterion_status
                .insert(criterion.id.clone(), status);
        }
        self.intent_runtime
            .evaluation_history
            .push(format!("{:?}", result.status));
        self.history.push(ReasoningTransition {
            id: transition_id,
            state_version: self.semantic_state.version,
            status: result.status.clone(),
            ruo_refs: result.trace.iter().map(|t| t.ruo_ref.clone()).collect(),
        });
        result
    }

    fn result(
        &self,
        answer: Option<Vec<i64>>,
        status: ReasoningStatus,
        math: &str,
        method: &str,
        constraint: &str,
    ) -> MathLangResult {
        let completion = if status == ReasoningStatus::Complete {
            "COMPLETE"
        } else {
            "UNRESOLVED"
        };
        MathLangResult {
            answer,
            status,
            problem_evaluation: ProblemEvaluation {
                mathematical_status: math.into(),
                problem_relevance: if self.trace.is_empty() {
                    "UNKNOWN"
                } else {
                    "RELEVANT"
                }
                .into(),
                intent_alignment: if method == "VIOLATION" {
                    "VIOLATION"
                } else if self.trace.is_empty() {
                    "UNKNOWN"
                } else {
                    "ALIGNED"
                }
                .into(),
                goal_completion: completion.into(),
                method_compliance: method.into(),
                constraint_status: constraint.into(),
                primary_errors: if method == "VIOLATION" {
                    vec!["METHOD_VIOLATION".into()]
                } else {
                    vec![]
                },
                downstream_errors: vec![],
                evidence_refs: self
                    .semantic_state
                    .objects
                    .iter()
                    .filter_map(|o| match o {
                        Object::Evidence(e)
                            if e.source == "ReasonScript/DSN"
                                || e.source == "MathLang_DSN native" =>
                        {
                            Some(e.common.occurrence_id.clone())
                        }
                        _ => None,
                    })
                    .collect(),
            },
            trace: self.trace.clone(),
            evidence: self.evidence.clone(),
            knowledge_decisions: self.knowledge_decisions.clone(),
        }
    }
}

fn polynomial_value(poly: &Polynomial) -> Value {
    Value::Vector(poly.coefficients.into_iter().map(Value::Integer).collect())
}

fn root_set(roots: &[i64]) -> Value {
    Value::Set(roots.iter().copied().map(Value::Integer).collect())
}

pub fn solve_problem(text: &str) -> Result<(MathLangResult, SemanticState, ReasoningPlan), String> {
    solve_problem_with_memory(text, MemorySpace::new())
}

pub fn solve_problem_with_memory(
    text: &str,
    memory: MemorySpace,
) -> Result<(MathLangResult, SemanticState, ReasoningPlan), String> {
    let parsed = parse_problem(text)?;
    let mut context = MathProblemContext::with_memory(parsed.definition, parsed.state, memory)?;
    let result = context.execute(None)?;
    Ok((result, context.semantic_state, context.plan))
}
