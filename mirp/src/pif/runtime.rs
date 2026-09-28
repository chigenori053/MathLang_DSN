//! Executed, evidence-bearing intent runtime. Declared methods are never proof.

use super::parser::ParseOutcome;
use super::schema::*;
use crate::intent::{
    integer_quadratic_roots, EducationalError as ErrorKind, ExpectedOutput, GoalCompletion,
    IntentAlignment, MathematicalValidity, Method, MethodCompliance, OperationIntent, Polynomial,
    ProblemRelevance,
};
use crate::{stable_id, Provenance, SemanticDelta, SemanticState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GoalStatus {
    Unresolved,
    Partial,
    Satisfied,
    Conflict,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CriterionStatus {
    Unresolved,
    Satisfied,
    Violated,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GlobalDerivationalValidity {
    ValidDerivation,
    InvalidDerivation,
    PartiallyValidDerivation,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityContext {
    pub local_validity: MathematicalValidity,
    pub global_derivational_validity: GlobalDerivationalValidity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuTrace {
    pub ru_id: String,
    pub rus_id: String,
    pub ruo_id: String,
    pub transition_id: String,
    pub input_coefficients: [i64; 3],
    pub output_values: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodEvidence {
    pub strategy: Method,
    pub transition_refs: Vec<String>,
    pub ru_refs: Vec<String>,
    pub rus_refs: Vec<String>,
    pub ruo_refs: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EducationalErrorRecord {
    pub id: String,
    pub kind: ErrorKind,
    pub origin_transition: String,
    pub affected_states: Vec<u64>,
    pub primary: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Execution {
    Transform {
        before: Polynomial,
        after: Polynomial,
    },
    Factor {
        roots: [i64; 2],
    },
    QuadraticFormula,
    Answer {
        values: Vec<i64>,
        output: ExpectedOutput,
    },
    ArithmeticEquality {
        left: i64,
        right: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub target_ref: String,
    pub source_occurrence_ids: Vec<String>,
    pub declared_method: Option<Method>,
    pub execution: Execution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionEvaluation {
    pub id: String,
    pub validity: ValidityContext,
    pub problem_relevance: ProblemRelevance,
    pub intent_alignment: IntentAlignment,
    pub method_compliance: MethodCompliance,
    pub goal_completion: GoalCompletion,
    pub evidence: Vec<Evidence>,
    pub errors: Vec<EducationalErrorRecord>,
    pub trace: Option<RuTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentRuntimeState {
    pub goal_status: BTreeMap<String, GoalStatus>,
    pub criterion_status: BTreeMap<String, CriterionStatus>,
    pub active_goals: Vec<String>,
    pub satisfied_goals: Vec<String>,
    pub unresolved_goals: Vec<String>,
    pub method_evidence: Vec<MethodEvidence>,
    pub evaluation_history: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemEvaluation {
    pub problem_id: String,
    pub final_state: u64,
    pub mathematical_status: GlobalDerivationalValidity,
    pub goal_completion: GoalCompletion,
    pub constraint_status: CriterionStatus,
    pub method_compliance: MethodCompliance,
    pub satisfied_criteria: Vec<String>,
    pub unresolved_criteria: Vec<String>,
    pub violated_criteria: Vec<String>,
    pub primary_errors: Vec<EducationalErrorRecord>,
    pub downstream_errors: Vec<EducationalErrorRecord>,
    pub evidence_refs: Vec<String>,
}

pub struct ReasoningContext {
    definition: ProblemDefinition,
    semantic_state: SemanticState,
    runtime: IntentRuntimeState,
    history: Vec<TransitionEvaluation>,
    working_equation: Option<Polynomial>,
    invalid_origin: Option<String>,
    final_answer: Option<(Vec<i64>, ExpectedOutput)>,
}

impl ReasoningContext {
    pub fn new(
        definition: ProblemDefinition,
        semantic_state: SemanticState,
    ) -> Result<Self, String> {
        semantic_state
            .validate()
            .map_err(|error| error.to_string())?;
        definition.intent().validate(&semantic_state)?;
        let intent = definition.intent();
        let goal_status: BTreeMap<_, _> = intent
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
        let unresolved_goals: Vec<String> = goal_status.keys().cloned().collect();
        let working_equation = intent.givens.first().map(|g| g.equation.clone());
        Ok(Self {
            definition,
            semantic_state,
            runtime: IntentRuntimeState {
                goal_status,
                criterion_status,
                active_goals: unresolved_goals.clone(),
                satisfied_goals: vec![],
                unresolved_goals,
                method_evidence: vec![],
                evaluation_history: vec![],
            },
            history: vec![],
            working_equation,
            invalid_origin: None,
            final_answer: None,
        })
    }

    pub fn from_parsed(parsed: ParseOutcome) -> Result<Self, String> {
        Self::new(parsed.definition, parsed.state)
    }
    pub fn definition(&self) -> &ProblemDefinition {
        &self.definition
    }
    pub fn runtime(&self) -> &IntentRuntimeState {
        &self.runtime
    }
    pub fn history(&self) -> &[TransitionEvaluation] {
        &self.history
    }
    pub fn semantic_state(&self) -> &SemanticState {
        &self.semantic_state
    }

    pub fn execute(&mut self, step: ReasoningStep) -> Result<TransitionEvaluation, String> {
        if self.semantic_state.get(&step.target_ref).is_none() {
            return Err("dangling target semantic reference".into());
        }
        if step.source_occurrence_ids.is_empty()
            || step
                .source_occurrence_ids
                .iter()
                .any(|id| self.semantic_state.get_occurrence(id).is_none())
        {
            return Err("dangling source occurrence reference".into());
        }
        let intent = self.definition.intent().clone();
        let index = self.history.len().to_string();
        let id = stable_id(
            "transition",
            &[
                &intent.problem_id,
                &index,
                &serde_json::to_string(&step).expect("step serializes"),
            ],
        );
        // Candidate reasoning transitions advance the frozen MIRP state version.
        self.semantic_state
            .apply(SemanticDelta::default())
            .expect("validated MIRP state accepts an empty transition delta");
        let target_matches = intent
            .primary_goal
            .as_ref()
            .is_some_and(|g| g.target_refs.contains(&step.target_ref));
        let all_sources_exist = !step.source_occurrence_ids.is_empty()
            && step
                .source_occurrence_ids
                .iter()
                .all(|s| self.semantic_state.get_occurrence(s).is_some());
        let cites_problem = step
            .source_occurrence_ids
            .contains(&intent.source_occurrence_id);
        let relevance = if !target_matches || (all_sources_exist && !cites_problem) {
            ProblemRelevance::Irrelevant
        } else if all_sources_exist && cites_problem {
            ProblemRelevance::Relevant
        } else {
            ProblemRelevance::Unknown
        };
        let before = self.working_equation.clone();
        let (local, trace) = match &step.execution {
            Execution::Transform {
                before: claimed,
                after,
            } => {
                let validity = if Some(claimed) != before.as_ref()
                    || !claimed.supported()
                    || !after.supported()
                {
                    MathematicalValidity::Unknown
                } else if equivalent(claimed, after) {
                    MathematicalValidity::Valid
                } else {
                    MathematicalValidity::Invalid
                };
                self.working_equation = Some(after.clone());
                (validity, None)
            }
            Execution::Factor { roots } => match &before {
                Some(poly) if poly.supported() => {
                    let [left, right] = roots.map(i128::from);
                    let factored = [left * right, -(left + right), 1];
                    if equivalent_coefficients(poly, factored) {
                        (
                            MathematicalValidity::Valid,
                            Some(executed_trace(&id, "FACTOR_PRODUCT", poly, roots.to_vec())),
                        )
                    } else {
                        (MathematicalValidity::Invalid, None)
                    }
                }
                _ => (MathematicalValidity::Unknown, None),
            },
            Execution::QuadraticFormula => {
                match before.as_ref().and_then(integer_quadratic_roots) {
                    Some(roots) => (
                        MathematicalValidity::Valid,
                        before
                            .as_ref()
                            .map(|p| executed_trace(&id, "QUADRATIC_FORMULA", p, roots)),
                    ),
                    None => (MathematicalValidity::Unknown, None),
                }
            }
            Execution::Answer { values, output } => {
                self.final_answer = Some((values.clone(), output.clone()));
                let validity = match &before {
                    Some(poly) if poly.supported() => {
                        if values.is_empty() && integer_quadratic_roots(poly).is_none() {
                            MathematicalValidity::Unknown
                        } else if values.iter().all(|x| poly.at(*x) == 0) {
                            MathematicalValidity::Valid
                        } else {
                            MathematicalValidity::Invalid
                        }
                    }
                    _ => MathematicalValidity::Unknown,
                };
                (validity, None)
            }
            Execution::ArithmeticEquality { left, right } => (
                if left == right {
                    MathematicalValidity::Valid
                } else {
                    MathematicalValidity::Invalid
                },
                None,
            ),
        };
        if local == MathematicalValidity::Invalid && self.invalid_origin.is_none() {
            self.invalid_origin = Some(id.clone());
        }
        let global = if self.invalid_origin.is_some() {
            GlobalDerivationalValidity::InvalidDerivation
        } else if local == MathematicalValidity::Unknown
            || self
                .history
                .iter()
                .any(|h| h.validity.local_validity == MathematicalValidity::Unknown)
        {
            if local == MathematicalValidity::Valid
                || self
                    .history
                    .iter()
                    .any(|h| h.validity.local_validity == MathematicalValidity::Valid)
            {
                GlobalDerivationalValidity::PartiallyValidDerivation
            } else {
                GlobalDerivationalValidity::Unknown
            }
        } else {
            GlobalDerivationalValidity::ValidDerivation
        };
        let provenance = step
            .source_occurrence_ids
            .iter()
            .find_map(|s| {
                self.semantic_state
                    .get_occurrence(s)
                    .map(|o| o.common().provenance.clone())
            })
            .unwrap_or_else(|| intent.provenance.clone());
        let mut evidence = vec![self.evidence(
            &id,
            "derivation",
            if local == MathematicalValidity::Unknown {
                EvidenceKind::InsufficientInformation
            } else {
                EvidenceKind::DerivationTrace
            },
            &step,
            &provenance,
            vec![],
        )];
        if let Some(trace) = &trace {
            let strategy = if trace.ru_id == "FACTOR_PRODUCT" {
                Method::Factorization
            } else {
                Method::QuadraticFormula
            };
            self.runtime.method_evidence.push(MethodEvidence {
                strategy,
                transition_refs: vec![id.clone()],
                ru_refs: vec![trace.ru_id.clone()],
                rus_refs: vec![trace.rus_id.clone()],
                ruo_refs: vec![trace.ruo_id.clone()],
                provenance: provenance.clone(),
            });
            evidence.push(
                self.evidence(
                    &id,
                    "method",
                    EvidenceKind::MethodTrace,
                    &step,
                    &provenance,
                    intent
                        .completion_criteria
                        .iter()
                        .filter(|c| c.kind == CriterionKind::MethodUsed)
                        .map(|c| c.id.clone())
                        .collect(),
                ),
            );
        }
        let method = method_compliance(&intent, &self.runtime.method_evidence);
        for goal in &intent.subgoals {
            if goal.kind == GoalKind::Factor
                && self
                    .runtime
                    .method_evidence
                    .iter()
                    .any(|m| m.strategy == Method::Factorization)
            {
                self.runtime
                    .goal_status
                    .insert(goal.id.clone(), GoalStatus::Satisfied);
            }
        }
        self.refresh_goal_lists();
        self.evaluate_criteria(&intent, &method, &id, &step, &provenance, &mut evidence);
        let completion = self.completion(&intent, &global, &relevance);
        let alignment = if relevance == ProblemRelevance::Irrelevant {
            IntentAlignment::Misaligned
        } else if relevance == ProblemRelevance::Unknown || local == MathematicalValidity::Unknown {
            IntentAlignment::Unknown
        } else if local == MathematicalValidity::Invalid
            || method == MethodCompliance::Violation
            || (matches!(step.execution, Execution::Answer { .. })
                && completion != GoalCompletion::Complete)
        {
            IntentAlignment::Partial
        } else {
            IntentAlignment::Aligned
        };
        let mut errors = vec![];
        if let Some(origin) = &self.invalid_origin {
            errors.push(EducationalErrorRecord {
                id: stable_id("educational-error", &[origin, &id]),
                kind: if matches!(
                    step.execution,
                    Execution::Transform { .. } | Execution::Factor { .. }
                ) {
                    ErrorKind::AlgebraicTransformationError
                } else {
                    ErrorKind::CalculationError
                },
                origin_transition: origin.clone(),
                affected_states: vec![self.semantic_state.version],
                primary: origin == &id,
                evidence_refs: vec![evidence[0].id.clone()],
            });
        }
        if method == MethodCompliance::Violation {
            let method_origin = self
                .runtime
                .method_evidence
                .iter()
                .find(|observed| {
                    !intent
                        .method_constraints
                        .iter()
                        .any(|required| required.required == observed.strategy)
                })
                .and_then(|observed| observed.transition_refs.first())
                .cloned()
                .unwrap_or_else(|| id.clone());
            errors.push(EducationalErrorRecord {
                id: stable_id("educational-error", &[&method_origin, &id, "method"]),
                kind: ErrorKind::MethodViolation,
                origin_transition: method_origin.clone(),
                affected_states: vec![self.semantic_state.version],
                primary: method_origin == id,
                evidence_refs: vec![stable_id("intent-evidence", &[&method_origin, "method"])],
            });
        }
        if let Execution::Answer { values, .. } = &step.execution {
            if let Some(roots) = intent
                .givens
                .first()
                .and_then(|g| integer_quadratic_roots(&g.equation))
            {
                let required: BTreeSet<_> = roots
                    .iter()
                    .copied()
                    .filter(|x| intent.constraints.iter().all(|c| c.kind.accepts(*x)))
                    .collect();
                if local == MathematicalValidity::Valid
                    && required.iter().any(|x| !values.contains(x))
                {
                    errors.push(error_record(
                        &id,
                        ErrorKind::MissingSolution,
                        &evidence,
                        self.semantic_state.version,
                    ));
                }
                if values
                    .iter()
                    .any(|x| roots.contains(x) && !required.contains(x))
                {
                    errors.push(error_record(
                        &id,
                        ErrorKind::ConstraintError,
                        &evidence,
                        self.semantic_state.version,
                    ));
                }
            }
        }
        if relevance == ProblemRelevance::Irrelevant {
            errors.push(error_record(
                &id,
                if !target_matches {
                    ErrorKind::WrongTarget
                } else {
                    ErrorKind::IrrelevantReasoning
                },
                &evidence,
                self.semantic_state.version,
            ));
        }
        let result = TransitionEvaluation {
            id: id.clone(),
            validity: ValidityContext {
                local_validity: local,
                global_derivational_validity: global,
            },
            problem_relevance: relevance,
            intent_alignment: alignment,
            method_compliance: method,
            goal_completion: completion,
            evidence,
            errors,
            trace,
        };
        self.runtime.evaluation_history.push(id);
        self.history.push(result.clone());
        Ok(result)
    }

    fn evidence(
        &self,
        transition: &str,
        suffix: &str,
        kind: EvidenceKind,
        step: &ReasoningStep,
        provenance: &Provenance,
        criterion_refs: Vec<String>,
    ) -> Evidence {
        Evidence {
            id: stable_id("intent-evidence", &[transition, suffix]),
            kind,
            semantic_refs: if self.semantic_state.get(&step.target_ref).is_some() {
                vec![step.target_ref.clone()]
            } else {
                vec![]
            },
            occurrence_refs: step
                .source_occurrence_ids
                .iter()
                .filter(|s| self.semantic_state.get_occurrence(s).is_some())
                .cloned()
                .collect(),
            state_refs: vec![self.semantic_state.version],
            goal_refs: self
                .definition
                .intent()
                .primary_goal
                .iter()
                .map(|g| g.id.clone())
                .collect(),
            criterion_refs,
            provenance: provenance.clone(),
        }
    }

    fn evaluate_criteria(
        &mut self,
        intent: &ProblemIntent,
        method: &MethodCompliance,
        transition: &str,
        step: &ReasoningStep,
        provenance: &Provenance,
        evidence: &mut Vec<Evidence>,
    ) {
        let Some((values, output)) = &self.final_answer else {
            return;
        };
        let roots = intent
            .givens
            .first()
            .and_then(|g| integer_quadratic_roots(&g.equation));
        let required = roots.as_ref().map(|roots| {
            roots
                .iter()
                .copied()
                .filter(|x| intent.constraints.iter().all(|c| c.kind.accepts(*x)))
                .collect::<BTreeSet<_>>()
        });
        let submitted: BTreeSet<_> = values.iter().copied().collect();
        for criterion in &intent.completion_criteria {
            let status = match criterion.kind {
                CriterionKind::AllSolutionsFound => {
                    required
                        .as_ref()
                        .map_or(CriterionStatus::Unknown, |needed| {
                            if submitted == *needed {
                                CriterionStatus::Satisfied
                            } else {
                                CriterionStatus::Unresolved
                            }
                        })
                }
                CriterionKind::NoExtraneousSolutions => {
                    required
                        .as_ref()
                        .map_or(CriterionStatus::Unknown, |needed| {
                            if submitted.is_subset(needed) {
                                CriterionStatus::Satisfied
                            } else {
                                CriterionStatus::Violated
                            }
                        })
                }
                CriterionKind::ConstraintsSatisfied => {
                    if values
                        .iter()
                        .all(|x| intent.constraints.iter().all(|c| c.kind.accepts(*x)))
                    {
                        CriterionStatus::Satisfied
                    } else {
                        CriterionStatus::Violated
                    }
                }
                CriterionKind::OutputTypeMatched => {
                    if intent
                        .expected_output
                        .as_ref()
                        .is_some_and(|o| &o.kind == output)
                    {
                        CriterionStatus::Satisfied
                    } else {
                        CriterionStatus::Violated
                    }
                }
                CriterionKind::MethodUsed => match method {
                    MethodCompliance::Satisfied => CriterionStatus::Satisfied,
                    MethodCompliance::Violation => CriterionStatus::Violated,
                    _ => CriterionStatus::Unknown,
                },
                CriterionKind::TargetDerived => {
                    if !values.is_empty() {
                        CriterionStatus::Satisfied
                    } else {
                        CriterionStatus::Unresolved
                    }
                }
                CriterionKind::DomainResolved => {
                    if roots.is_some() {
                        CriterionStatus::Satisfied
                    } else {
                        CriterionStatus::Unknown
                    }
                }
                _ => CriterionStatus::Unknown,
            };
            self.runtime
                .criterion_status
                .insert(criterion.id.clone(), status);
            evidence.push(self.evidence(
                transition,
                &criterion.id,
                if criterion.kind == CriterionKind::ConstraintsSatisfied {
                    EvidenceKind::ConstraintApplication
                } else {
                    EvidenceKind::CompletionCriterion
                },
                step,
                provenance,
                vec![criterion.id.clone()],
            ));
        }
        for (i, constraint) in intent.constraints.iter().enumerate() {
            if roots
                .as_ref()
                .is_some_and(|roots| roots.iter().any(|x| !constraint.kind.accepts(*x)))
            {
                let mut item = self.evidence(
                    transition,
                    &format!("constraint:{i}"),
                    EvidenceKind::ConstraintApplication,
                    step,
                    &constraint.provenance,
                    intent
                        .completion_criteria
                        .iter()
                        .filter(|c| c.kind == CriterionKind::ConstraintsSatisfied)
                        .map(|c| c.id.clone())
                        .collect(),
                );
                item.occurrence_refs = vec![constraint.source_occurrence_id.clone()];
                evidence.push(item);
            }
        }
        if let Some(primary) = &intent.primary_goal {
            let criteria_met = intent
                .completion_criteria
                .iter()
                .filter(|c| c.required)
                .all(|c| {
                    self.runtime.criterion_status.get(&c.id) == Some(&CriterionStatus::Satisfied)
                });
            let dependencies_met = primary
                .dependencies
                .iter()
                .all(|id| self.runtime.goal_status.get(id) == Some(&GoalStatus::Satisfied));
            let status = if criteria_met && dependencies_met && self.invalid_origin.is_none() {
                GoalStatus::Satisfied
            } else {
                GoalStatus::Partial
            };
            self.runtime.goal_status.insert(primary.id.clone(), status);
        }
        self.refresh_goal_lists();
    }

    fn refresh_goal_lists(&mut self) {
        self.runtime.satisfied_goals = self
            .runtime
            .goal_status
            .iter()
            .filter(|(_, s)| **s == GoalStatus::Satisfied)
            .map(|(id, _)| id.clone())
            .collect();
        self.runtime.unresolved_goals = self
            .runtime
            .goal_status
            .iter()
            .filter(|(_, s)| **s != GoalStatus::Satisfied)
            .map(|(id, _)| id.clone())
            .collect();
        self.runtime.active_goals = self.runtime.unresolved_goals.clone();
    }

    fn completion(
        &self,
        intent: &ProblemIntent,
        global: &GlobalDerivationalValidity,
        relevance: &ProblemRelevance,
    ) -> GoalCompletion {
        if intent.intent_status != IntentStatus::Known
            || intent.operation_intent.kind != Some(OperationIntent::Solve)
        {
            return GoalCompletion::Unknown;
        }
        if *relevance == ProblemRelevance::Unknown {
            return GoalCompletion::Unknown;
        }
        if self.final_answer.is_none() {
            return GoalCompletion::Incomplete;
        }
        if intent
            .completion_criteria
            .iter()
            .filter(|c| c.required)
            .any(|c| self.runtime.criterion_status.get(&c.id) == Some(&CriterionStatus::Unknown))
        {
            return GoalCompletion::Unknown;
        }
        if *global == GlobalDerivationalValidity::ValidDerivation
            && *relevance == ProblemRelevance::Relevant
            && intent.primary_goal.as_ref().is_some_and(|g| {
                self.runtime.goal_status.get(&g.id) == Some(&GoalStatus::Satisfied)
            })
        {
            GoalCompletion::Complete
        } else {
            GoalCompletion::Incomplete
        }
    }

    pub fn problem_evaluation(&self) -> ProblemEvaluation {
        let intent = self.definition.intent();
        let mut satisfied = vec![];
        let mut unresolved = vec![];
        let mut violated = vec![];
        for criterion in &intent.completion_criteria {
            match self.runtime.criterion_status.get(&criterion.id) {
                Some(CriterionStatus::Satisfied) => satisfied.push(criterion.id.clone()),
                Some(CriterionStatus::Violated) => violated.push(criterion.id.clone()),
                _ => unresolved.push(criterion.id.clone()),
            }
        }
        let errors: Vec<_> = self.history.iter().flat_map(|t| t.errors.clone()).collect();
        let constraint_criterion = intent
            .completion_criteria
            .iter()
            .find(|c| c.kind == CriterionKind::ConstraintsSatisfied);
        ProblemEvaluation {
            problem_id: intent.problem_id.clone(),
            final_state: self.semantic_state.version,
            mathematical_status: self
                .history
                .last()
                .map(|t| t.validity.global_derivational_validity.clone())
                .unwrap_or(GlobalDerivationalValidity::Unknown),
            goal_completion: self
                .history
                .last()
                .map(|t| t.goal_completion.clone())
                .unwrap_or(GoalCompletion::Unknown),
            constraint_status: constraint_criterion
                .and_then(|c| self.runtime.criterion_status.get(&c.id))
                .cloned()
                .unwrap_or(CriterionStatus::Satisfied),
            method_compliance: method_compliance(intent, &self.runtime.method_evidence),
            satisfied_criteria: satisfied,
            unresolved_criteria: unresolved,
            violated_criteria: violated,
            primary_errors: errors.iter().filter(|e| e.primary).cloned().collect(),
            downstream_errors: errors.into_iter().filter(|e| !e.primary).collect(),
            evidence_refs: self
                .history
                .iter()
                .flat_map(|t| t.evidence.iter().map(|e| e.id.clone()))
                .collect(),
        }
    }
}

fn equivalent(a: &Polynomial, b: &Polynomial) -> bool {
    equivalent_coefficients(a, b.coefficients.map(i128::from))
}

fn equivalent_coefficients(a: &Polynomial, b: [i128; 3]) -> bool {
    let a = a.coefficients.map(i128::from);
    a.iter().position(|n| *n != 0).is_some_and(|i| {
        b[i] != 0 && (0..3).all(|j| a[j].checked_mul(b[i]) == b[j].checked_mul(a[i]))
    })
}

fn executed_trace(id: &str, operation: &str, input: &Polynomial, values: Vec<i64>) -> RuTrace {
    RuTrace {
        ru_id: operation.into(),
        rus_id: "PIF_QUADRATIC_RUS".into(),
        ruo_id: stable_id("ruo", &[id, operation, &format!("{:?}", values)]),
        transition_id: id.into(),
        input_coefficients: input.coefficients,
        output_values: values,
    }
}

fn error_record(
    id: &str,
    kind: ErrorKind,
    evidence: &[Evidence],
    state: u64,
) -> EducationalErrorRecord {
    EducationalErrorRecord {
        id: stable_id("educational-error", &[id, &format!("{:?}", kind)]),
        kind,
        origin_transition: id.into(),
        affected_states: vec![state],
        primary: true,
        evidence_refs: evidence.iter().map(|e| e.id.clone()).collect(),
    }
}

fn method_compliance(intent: &ProblemIntent, evidence: &[MethodEvidence]) -> MethodCompliance {
    if intent.method_constraints.is_empty() {
        return MethodCompliance::NotRequired;
    }
    if evidence.is_empty() {
        return MethodCompliance::Unknown;
    }
    if intent.method_constraints.iter().all(|required| {
        evidence
            .iter()
            .any(|observed| observed.strategy == required.required && !observed.ruo_refs.is_empty())
    }) {
        MethodCompliance::Satisfied
    } else {
        MethodCompliance::Violation
    }
}
