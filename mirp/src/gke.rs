//! Bounded, data-driven predicate inference over MIRP state.
use crate::knowledge::{KnowledgeSpace, KnowledgeUnit};
use crate::session::run_reason_source;
use crate::{
    stable_id, Common, Evidence, Object, PredicateFact, Provenance, SemanticDelta, SemanticState,
    Status,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleValueType {
    Number,
    Symbol,
    Entity,
    GeometryObject,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuleVariable {
    pub name: String,
    pub value_type: RuleValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleTerm {
    Variable(String),
    Integer(i64),
    Rational(i64, i64),
    Symbol(String),
    EntityRef(String),
}

impl RuleTerm {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Variable(name) | Self::Symbol(name) | Self::EntityRef(name) => !name.is_empty(),
            Self::Rational(n, d) => *d > 1 && gcd(n.unsigned_abs(), *d as u64) == 1,
            Self::Integer(_) => true,
        }
    }
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulePredicate {
    pub predicate: String,
    pub arguments: Vec<RuleTerm>,
    #[serde(default)]
    pub negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenericRule {
    pub variables: Vec<RuleVariable>,
    pub premises: Vec<RulePredicate>,
    pub constraints: Vec<RulePredicate>,
    pub conclusions: Vec<RulePredicate>,
}
impl GenericRule {
    pub fn validate(&self) -> Result<(), String> {
        if self.conclusions.is_empty() {
            return Err("generic rule needs a conclusion".into());
        }
        let mut vars = BTreeMap::new();
        for variable in &self.variables {
            if variable.name.is_empty()
                || vars.insert(&variable.name, &variable.value_type).is_some()
            {
                return Err("invalid or duplicate rule variable".into());
            }
        }
        for predicate in self
            .premises
            .iter()
            .chain(&self.constraints)
            .chain(&self.conclusions)
        {
            if predicate.predicate.is_empty() || predicate.arguments.iter().any(|t| !t.valid()) {
                return Err("invalid rule predicate".into());
            }
            for term in &predicate.arguments {
                if let RuleTerm::Variable(name) = term {
                    if !vars.contains_key(name) {
                        return Err(format!("undeclared variable: {name}"));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InferenceStatus {
    Derived,
    Unknown,
    NoMatch,
    Conflict,
    Unsupported,
    ResourceLimit,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceLimits {
    pub max_steps: usize,
    pub max_candidates_per_step: usize,
    pub max_bindings_per_rule: usize,
    pub max_dependency_depth: usize,
}
impl Default for InferenceLimits {
    fn default() -> Self {
        Self {
            max_steps: 64,
            max_candidates_per_step: 128,
            max_bindings_per_rule: 256,
            max_dependency_depth: 32,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTrace {
    pub knowledge_id: String,
    pub knowledge_occurrence_id: String,
    pub activation_id: String,
    pub ru_ref: String,
    pub rus_ref: String,
    pub ruo_ref: String,
    pub bindings: BTreeMap<String, RuleTerm>,
    pub premise_semantic_refs: Vec<String>,
    pub premise_occurrence_refs: Vec<String>,
    pub conclusion_semantic_refs: Vec<String>,
    pub conclusion_occurrence_refs: Vec<String>,
    pub evidence_ref: String,
    pub source_state_ref: String,
    pub result_state_ref: String,
    pub status: InferenceStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericKnowledgeRUS {
    pub rus_ref: String,
    pub goal_ref: String,
    pub candidate_knowledge: Vec<String>,
    pub selected_knowledge: String,
    pub candidate_ru: Vec<String>,
    pub selected_ru: String,
    pub state_ref: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub total_knowledge: usize,
    pub examined_knowledge: usize,
    pub candidate_knowledge: usize,
    pub binding_attempts: usize,
    pub executed_knowledge: usize,
    pub inference_steps: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceReport {
    pub status: InferenceStatus,
    pub traces: Vec<GenericTrace>,
    pub rus: Vec<GenericKnowledgeRUS>,
    pub metrics: InferenceMetrics,
    pub diagnostics: Vec<String>,
}

type Bindings = BTreeMap<String, RuleTerm>;
#[derive(Clone)]
struct Match {
    bindings: Bindings,
    occurrences: Vec<String>,
    semantics: Vec<String>,
}
fn type_ok(value: &RuleTerm, kind: &RuleValueType, state: &SemanticState) -> bool {
    if *kind == RuleValueType::GeometryObject {
        return matches!(value, RuleTerm::EntityRef(id)
            if matches!(state.get(id), Some(Object::Entity(entity)) if entity.entity_type == "GEOMETRY_OBJECT"));
    }
    matches!(
        (value, kind),
        (
            RuleTerm::Integer(_) | RuleTerm::Rational(_, _),
            RuleValueType::Number
        ) | (RuleTerm::Symbol(_), RuleValueType::Symbol)
            | (RuleTerm::EntityRef(_), RuleValueType::Entity)
    )
}
fn unify(
    pattern: &RulePredicate,
    fact: &PredicateFact,
    rule: &GenericRule,
    base: &Bindings,
    state: &SemanticState,
) -> Option<Bindings> {
    if pattern.predicate != fact.predicate
        || pattern.negated != fact.negated
        || pattern.arguments.len() != fact.arguments.len()
    {
        return None;
    }
    let mut next = base.clone();
    for (a, b) in pattern.arguments.iter().zip(&fact.arguments) {
        match a {
            RuleTerm::Variable(name) => {
                let kind = &rule.variables.iter().find(|v| v.name == *name)?.value_type;
                if !type_ok(b, kind, state) || next.get(name).is_some_and(|old| old != b) {
                    return None;
                }
                next.insert(name.clone(), b.clone());
            }
            _ if a != b => return None,
            _ => {}
        }
    }
    Some(next)
}
fn matches(
    rule: &GenericRule,
    facts: &[&PredicateFact],
    state: &SemanticState,
    limit: usize,
    metrics: &mut InferenceMetrics,
) -> Result<Vec<Match>, InferenceStatus> {
    let mut partial = vec![Match {
        bindings: Bindings::new(),
        occurrences: vec![],
        semantics: vec![],
    }];
    for premise in &rule.premises {
        let mut next = vec![];
        for m in &partial {
            for fact in facts {
                if fact.predicate != premise.predicate {
                    continue;
                }
                metrics.binding_attempts += 1;
                if let Some(bindings) = unify(premise, fact, rule, &m.bindings, state) {
                    let mut candidate = m.clone();
                    candidate.bindings = bindings;
                    candidate
                        .occurrences
                        .push(fact.common.occurrence_id.clone());
                    candidate.semantics.push(fact.common.id.clone());
                    next.push(candidate);
                    if next.len() > limit {
                        return Err(InferenceStatus::ResourceLimit);
                    }
                }
            }
        }
        partial = next;
        if partial.is_empty() {
            break;
        }
    }
    Ok(partial)
}
fn instantiate(predicate: &RulePredicate, bindings: &Bindings) -> Option<RulePredicate> {
    let arguments = predicate
        .arguments
        .iter()
        .map(|term| match term {
            RuleTerm::Variable(name) => bindings.get(name).cloned(),
            other => Some(other.clone()),
        })
        .collect::<Option<Vec<_>>>()?;
    Some(RulePredicate {
        predicate: predicate.predicate.clone(),
        arguments,
        negated: predicate.negated,
    })
}
fn constraints(rule: &GenericRule, bindings: &Bindings) -> InferenceStatus {
    for constraint in &rule.constraints {
        let Some(bound) = instantiate(constraint, bindings) else {
            return InferenceStatus::Unknown;
        };
        let result = match (bound.predicate.as_str(), bound.arguments.as_slice()) {
            ("DISTINCT", [a, b]) => Some(a != b),
            (kind @ ("NONZERO" | "POSITIVE" | "INTEGER"), [RuleTerm::Integer(n)]) => {
                numeric_constraint(kind, *n, 1)
            }
            (kind @ ("NONZERO" | "POSITIVE" | "INTEGER"), [RuleTerm::Rational(n, d)]) => {
                numeric_constraint(kind, *n, *d)
            }
            _ => None,
        };
        match result {
            None => return InferenceStatus::Unsupported,
            Some(value) if value == bound.negated => return InferenceStatus::NoMatch,
            _ => {}
        }
    }
    InferenceStatus::Derived
}
fn numeric_constraint(kind: &str, numerator: i64, denominator: i64) -> Option<bool> {
    let source = format!("GenericKnowledge::Check(\"{kind}\", {numerator}, {denominator})");
    match run_reason_source(&source, &[("generic_knowledge", "GenericKnowledge")])
        .ok()?
        .as_str()?
    {
        "SATISFIED" => Some(true),
        "VIOLATED" => Some(false),
        _ => None,
    }
}
fn dependency_depth(
    id: &str,
    space: &KnowledgeSpace,
    seen: &mut BTreeSet<String>,
    remaining: usize,
) -> usize {
    if remaining == 0 || !seen.insert(id.into()) {
        return usize::MAX;
    }
    let depth = space
        .get(id)
        .map(|u| {
            u.dependencies
                .iter()
                .map(|d| dependency_depth(d, space, seen, remaining - 1))
                .max()
                .unwrap_or(0)
                .saturating_add(1)
        })
        .unwrap_or(usize::MAX);
    seen.remove(id);
    depth
}

pub fn infer(
    space: &KnowledgeSpace,
    state: &mut SemanticState,
    goal: &str,
    limits: InferenceLimits,
) -> InferenceReport {
    infer_target(space, state, goal, None, limits)
}

pub fn infer_goal(
    space: &KnowledgeSpace,
    state: &mut SemanticState,
    goal: &RulePredicate,
    limits: InferenceLimits,
) -> InferenceReport {
    infer_target(space, state, &goal.predicate, Some(goal), limits)
}

fn infer_target(
    space: &KnowledgeSpace,
    state: &mut SemanticState,
    goal: &str,
    target: Option<&RulePredicate>,
    limits: InferenceLimits,
) -> InferenceReport {
    let mut report = InferenceReport {
        status: InferenceStatus::Unknown,
        traces: vec![],
        rus: vec![],
        metrics: InferenceMetrics {
            total_knowledge: space.len(),
            ..Default::default()
        },
        diagnostics: vec![],
    };
    let goal_ref = stable_id(
        "goal",
        &[&target.map_or_else(|| goal.to_owned(), |t| serde_json::to_string(t).unwrap())],
    );
    if goal.is_empty()
        || target.is_some_and(|t| {
            t.arguments.iter().any(|term| {
                !term.valid()
                    || matches!(term, RuleTerm::Variable(_))
                    || matches!(term, RuleTerm::EntityRef(id) if state.get(id).is_none())
            })
        })
    {
        report.status = InferenceStatus::Unsupported;
        return report;
    }
    if let Err(error) = state.validate() {
        report.status = InferenceStatus::Unsupported;
        report.diagnostics.push(error.to_string());
        return report;
    }
    if state.status == Status::Conflict {
        report.status = InferenceStatus::Conflict;
        return report;
    }
    if limits.max_steps == 0
        || limits.max_candidates_per_step == 0
        || limits.max_bindings_per_rule == 0
    {
        report.status = InferenceStatus::ResourceLimit;
        return report;
    }
    for _ in 0..limits.max_steps {
        let facts: Vec<&PredicateFact> = state
            .objects
            .iter()
            .filter_map(|o| {
                if let Object::Predicate(f) = o {
                    if !matches!(
                        f.common.status,
                        Status::Known | Status::Derived | Status::Assumed
                    ) {
                        return None;
                    }
                    Some(f)
                } else {
                    None
                }
            })
            .collect();
        if facts.iter().any(|f| {
            f.predicate == goal
                && f.negated == target.is_some_and(|t| t.negated)
                && target.is_none_or(|t| f.arguments == t.arguments && f.negated == t.negated)
        }) {
            report.status = InferenceStatus::Derived;
            return report;
        }
        let mut proposed: Vec<(&KnowledgeUnit, Match, RulePredicate)> = vec![];
        let mut step_candidates = 0;
        for unit in space.query_generic(facts.iter().map(|fact| fact.predicate.as_str())) {
            report.metrics.examined_knowledge += 1;
            let Some(rule) = &unit.generic_rule else {
                continue;
            };
            if !unit.applicable_goals.is_empty() && !unit.applicable_goals.iter().any(|g| g == goal)
            {
                continue;
            }
            report.metrics.candidate_knowledge += 1;
            step_candidates += 1;
            if step_candidates > limits.max_candidates_per_step {
                report.status = InferenceStatus::ResourceLimit;
                return report;
            }
            if dependency_depth(
                &unit.knowledge_id,
                space,
                &mut BTreeSet::new(),
                limits.max_dependency_depth,
            ) > limits.max_dependency_depth
            {
                report.status = InferenceStatus::ResourceLimit;
                return report;
            }
            let found = match matches(
                rule,
                &facts,
                state,
                limits.max_bindings_per_rule,
                &mut report.metrics,
            ) {
                Ok(found) => found,
                Err(status) => {
                    report.status = status;
                    return report;
                }
            };
            for m in found {
                match constraints(rule, &m.bindings) {
                    InferenceStatus::Derived => {}
                    InferenceStatus::NoMatch => continue,
                    other => {
                        report
                            .diagnostics
                            .push(format!("{}: {other:?}", unit.knowledge_id));
                        continue;
                    }
                }
                for conclusion in &rule.conclusions {
                    let Some(bound) = instantiate(conclusion, &m.bindings) else {
                        report.diagnostics.push(format!(
                            "{}: UNBOUND_CONCLUSION_VARIABLE",
                            unit.knowledge_id
                        ));
                        continue;
                    };
                    if !facts.iter().any(|f| {
                        f.predicate == bound.predicate
                            && f.arguments == bound.arguments
                            && f.negated == bound.negated
                    }) {
                        proposed.push((unit, m.clone(), bound));
                    }
                }
            }
        }
        if proposed.is_empty() {
            report.status = if report.diagnostics.iter().any(|d| d.contains("Unsupported")) {
                InferenceStatus::Unsupported
            } else if report.metrics.candidate_knowledge == 0 || !report.diagnostics.is_empty() {
                InferenceStatus::Unknown
            } else {
                InferenceStatus::NoMatch
            };
            return report;
        }
        proposed.sort_by(|a, b| {
            b.0.priority
                .cmp(&a.0.priority)
                .then(
                    b.0.generic_rule
                        .as_ref()
                        .unwrap()
                        .premises
                        .len()
                        .cmp(&a.0.generic_rule.as_ref().unwrap().premises.len()),
                )
                .then(a.0.knowledge_id.cmp(&b.0.knowledge_id))
                .then(a.1.bindings.cmp(&b.1.bindings))
        });
        let (unit, matched, conclusion) = &proposed[0];
        let rank = (
            unit.priority,
            unit.generic_rule.as_ref().unwrap().premises.len(),
        );
        if proposed.iter().skip(1).any(|(other, _, opposite)| {
            (
                other.priority,
                other.generic_rule.as_ref().unwrap().premises.len(),
            ) == rank
                && opposite.predicate == conclusion.predicate
                && opposite.arguments == conclusion.arguments
                && opposite.negated != conclusion.negated
        }) {
            report.status = InferenceStatus::Conflict;
            report.diagnostics.push("KNOWLEDGE_CONFLICT".into());
            return report;
        }
        let source_state_ref = stable_id("state", &[&state.canonical().unwrap()]);
        let knowledge_occurrence_id = unit.occurrence_id();
        let binding_json = serde_json::to_string(&matched.bindings).unwrap();
        let conclusion_json = serde_json::to_string(conclusion).unwrap();
        let ru_ref = stable_id(
            "ru",
            &[
                &source_state_ref,
                &knowledge_occurrence_id,
                &binding_json,
                &conclusion_json,
            ],
        );
        let rus_ref = stable_id("rus", &[&goal_ref, &source_state_ref, &unit.knowledge_id]);
        let candidate_knowledge: Vec<String> = proposed
            .iter()
            .map(|(unit, _, _)| unit.knowledge_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let candidate_ru: Vec<String> = proposed
            .iter()
            .map(|(candidate, bindings, conclusion)| {
                stable_id(
                    "ru",
                    &[
                        &source_state_ref,
                        &candidate.occurrence_id(),
                        &serde_json::to_string(&bindings.bindings).unwrap(),
                        &serde_json::to_string(conclusion).unwrap(),
                    ],
                )
            })
            .collect();
        report.rus.push(GenericKnowledgeRUS {
            rus_ref: rus_ref.clone(),
            goal_ref: goal_ref.clone(),
            candidate_knowledge,
            selected_knowledge: unit.knowledge_id.clone(),
            candidate_ru,
            selected_ru: ru_ref.clone(),
            state_ref: source_state_ref.clone(),
        });
        let activation_id = stable_id("activation", &[&ru_ref, &rus_ref]);
        let mut provenance = Provenance::input("GKE", &unit.knowledge_id)
            .derived(matched.occurrences.clone(), &ru_ref);
        provenance.source_id = knowledge_occurrence_id.clone();
        provenance
            .transformation
            .extend([rus_ref.clone(), activation_id.clone()]);
        let mut fact = Object::Predicate(PredicateFact {
            common: Common::unidentified(Status::Derived, provenance),
            predicate: conclusion.predicate.clone(),
            arguments: conclusion.arguments.clone(),
            negated: conclusion.negated,
        });
        fact.normalize_identity(&ru_ref);
        let conclusion_semantic_ref = fact.id().to_owned();
        let conclusion_occurrence_ref = fact.occurrence_id().to_owned();
        let ruo_ref = stable_id("ruo", &[&activation_id, &conclusion_occurrence_ref]);
        fact.common_mut().metadata.insert(
            "gke".into(),
            serde_json::json!({
                "knowledge_id": unit.knowledge_id,
                "knowledge_occurrence_id": knowledge_occurrence_id,
                "ru_ref": ru_ref,
                "rus_ref": rus_ref,
                "ruo_ref": ruo_ref,
                "premise_occurrence_refs": matched.occurrences,
            }),
        );
        let mut evidence_provenance = Provenance::input("GKE", &unit.knowledge_id)
            .derived(vec![conclusion_occurrence_ref.clone()], &ruo_ref);
        evidence_provenance.source_id = knowledge_occurrence_id.clone();
        let mut evidence = Object::Evidence(Evidence {
            common: Common::unidentified(Status::Derived, evidence_provenance),
            evidence_type: "GKE_DERIVATION".into(),
            source: unit.knowledge_id.clone(),
            supports: vec![conclusion_semantic_ref.clone()],
            contradicts: vec![],
        });
        evidence.normalize_identity(&ruo_ref);
        evidence.common_mut().metadata.insert(
            "gke".into(),
            serde_json::json!({
                "knowledge_id": unit.knowledge_id,
                "knowledge_occurrence_id": knowledge_occurrence_id,
                "ru_ref": ru_ref,
                "rus_ref": rus_ref,
                "ruo_ref": ruo_ref,
                "premise_occurrence_refs": matched.occurrences,
            }),
        );
        let evidence_ref = evidence.id().to_owned();
        let evidence_occurrence_ref = evidence.occurrence_id().to_owned();
        let delta = SemanticDelta {
            added: vec![fact, evidence],
            dependencies: BTreeMap::from([
                (
                    conclusion_occurrence_ref.clone(),
                    matched.occurrences.clone(),
                ),
                (
                    evidence_occurrence_ref,
                    vec![conclusion_occurrence_ref.clone()],
                ),
            ]),
            ..Default::default()
        };
        if let Err(error) = state.apply(delta) {
            report.status = if error.category == "MirpConflictError" {
                InferenceStatus::Conflict
            } else {
                InferenceStatus::Unsupported
            };
            report.diagnostics.push(error.to_string());
            return report;
        }
        let result_state_ref = stable_id("state", &[&state.canonical().unwrap()]);
        report.traces.push(GenericTrace {
            knowledge_id: unit.knowledge_id.clone(),
            knowledge_occurrence_id,
            activation_id,
            ru_ref,
            rus_ref,
            ruo_ref,
            bindings: matched.bindings.clone(),
            premise_semantic_refs: matched.semantics.clone(),
            premise_occurrence_refs: matched.occurrences.clone(),
            conclusion_semantic_refs: vec![conclusion_semantic_ref],
            conclusion_occurrence_refs: vec![conclusion_occurrence_ref],
            evidence_ref,
            source_state_ref,
            result_state_ref,
            status: InferenceStatus::Derived,
        });
        report.metrics.executed_knowledge += 1;
        report.metrics.inference_steps += 1;
        if state.status == Status::Conflict {
            report.traces.last_mut().unwrap().status = InferenceStatus::Conflict;
            report.status = InferenceStatus::Conflict;
            return report;
        }
        if conclusion.predicate == goal
            && conclusion.negated == target.is_some_and(|t| t.negated)
            && target.is_none_or(|t| {
                conclusion.arguments == t.arguments && conclusion.negated == t.negated
            })
        {
            report.status = InferenceStatus::Derived;
            return report;
        }
    }
    report.status = InferenceStatus::ResourceLimit;
    report
}
