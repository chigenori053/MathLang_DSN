use crate::intent::{Constraint, ExpectedOutput, Given, Method, OperationIntent};
use crate::{Object, Provenance, SemanticState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const INTENT_SCHEMA: &str = "mathlang/problem-intent/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentStatus {
    Known,
    Ambiguous,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GoalKind {
    Solve,
    Calculate,
    Simplify,
    Expand,
    Factor,
    Determine,
    Verify,
    Prove,
    Compare,
    FindMaximum,
    FindMinimum,
    Differentiate,
    Integrate,
    Graph,
    Explain,
    Derive,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentQualifier {
    AllSolutions,
    ExactValue,
    ApproximateValue,
    WithReasoning,
    WithProof,
    WithGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationSpec {
    pub kind: Option<OperationIntent>,
    pub qualifiers: Vec<IntentQualifier>,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Cardinality {
    One,
    ZeroOrOne,
    OneOrMore,
    ZeroOrMore,
    ExactSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquivalencePolicy {
    SemanticEquivalence,
    AlgebraicEquivalence,
    SetEquivalence,
    LogicalEquivalence,
    StructuralEquivalence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputSpec {
    pub id: String,
    pub kind: ExpectedOutput,
    pub cardinality: Cardinality,
    pub domain_ref: Option<String>,
    pub equivalence_policy: EquivalencePolicy,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub kind: GoalKind,
    pub target_refs: Vec<String>,
    pub required_relation: Option<String>,
    pub expected_output_ref: String,
    pub parent_goal: Option<String>,
    pub dependencies: Vec<String>,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CriterionKind {
    TargetDerived,
    AllSolutionsFound,
    NoExtraneousSolutions,
    ConstraintsSatisfied,
    OutputTypeMatched,
    RequiredRelationEstablished,
    MethodUsed,
    DomainResolved,
    ProofComplete,
    GraphPropertyEstablished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Quantifier {
    All,
    Any,
    Exists,
    ExactlyOne,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionCriterion {
    pub id: String,
    pub kind: CriterionKind,
    pub target_refs: Vec<String>,
    pub quantifier: Quantifier,
    pub required: bool,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodRequirement {
    pub id: String,
    pub required: Method,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemIntent {
    pub schema_id: String,
    pub problem_id: String,
    pub givens: Vec<Given>,
    pub constraints: Vec<Constraint>,
    pub primary_goal: Option<Goal>,
    pub subgoals: Vec<Goal>,
    pub operation_intent: OperationSpec,
    pub expected_output: Option<OutputSpec>,
    pub method_constraints: Vec<MethodRequirement>,
    pub completion_criteria: Vec<CompletionCriterion>,
    pub intent_status: IntentStatus,
    pub source_occurrence_id: String,
    pub provenance: Provenance,
}

impl ProblemIntent {
    pub fn validate(&self, state: &SemanticState) -> Result<(), String> {
        if self.schema_id != INTENT_SCHEMA {
            return Err("unsupported intent schema".into());
        }
        if self.problem_id.is_empty() {
            return Err("missing problem ID".into());
        }
        source(state, &self.source_occurrence_id, &self.provenance)?;
        source(
            state,
            &self.operation_intent.source_occurrence_id,
            &self.operation_intent.provenance,
        )?;
        for given in &self.givens {
            source(state, &given.source_occurrence_id, &given.provenance)?;
        }
        for constraint in &self.constraints {
            source(
                state,
                &constraint.source_occurrence_id,
                &constraint.provenance,
            )?;
        }
        let mut method_ids = BTreeSet::new();
        for method in &self.method_constraints {
            if method.id.is_empty() || !method_ids.insert(&method.id) {
                return Err("duplicate or missing method ID".into());
            }
            source(state, &method.source_occurrence_id, &method.provenance)?;
        }
        let mut qualifiers = self.operation_intent.qualifiers.clone();
        qualifiers.sort();
        qualifiers.dedup();
        if qualifiers != self.operation_intent.qualifiers {
            return Err("qualifiers must be canonical".into());
        }
        if self.intent_status != IntentStatus::Known {
            return Ok(());
        }
        if self.givens.is_empty() {
            return Err("missing given".into());
        }
        let primary = self.primary_goal.as_ref().ok_or("missing primary goal")?;
        if self.operation_intent.kind.is_none() {
            return Err("known intent has no operation".into());
        }
        let output = self
            .expected_output
            .as_ref()
            .ok_or("missing expected output")?;
        if output.id.is_empty() {
            return Err("missing output ID".into());
        }
        if self.operation_intent.kind == Some(OperationIntent::Solve)
            && self
                .operation_intent
                .qualifiers
                .contains(&IntentQualifier::AllSolutions)
            && (primary.kind != GoalKind::Solve
                || output.kind != ExpectedOutput::SolutionSet
                || output.cardinality != Cardinality::ExactSet
                || output.equivalence_policy != EquivalencePolicy::SetEquivalence)
        {
            return Err("all-solutions goal/output mismatch".into());
        }
        source(state, &output.source_occurrence_id, &output.provenance)?;
        if let Some(domain) = &output.domain_ref {
            if state.get(domain).is_none() {
                return Err("invalid output domain reference".into());
            }
        }
        let mut goals = BTreeMap::new();
        for goal in std::iter::once(primary).chain(&self.subgoals) {
            if goal.id.is_empty() || goals.insert(goal.id.as_str(), goal).is_some() {
                return Err("duplicate or missing goal ID".into());
            }
            source(state, &goal.source_occurrence_id, &goal.provenance)?;
            if goal.target_refs.is_empty() {
                return Err("goal has no target".into());
            }
            for target in &goal.target_refs {
                if !matches!(state.get(target), Some(Object::Entity(_))) {
                    return Err("invalid goal target reference".into());
                }
            }
            if goal.expected_output_ref != output.id {
                return Err("invalid output reference".into());
            }
        }
        if primary.parent_goal.is_some() {
            return Err("primary goal has parent".into());
        }
        for goal in &self.subgoals {
            let parent = goal
                .parent_goal
                .as_deref()
                .ok_or("subgoal missing parent")?;
            if parent == goal.id || !goals.contains_key(parent) {
                return Err("invalid parent goal".into());
            }
        }
        for goal in &self.subgoals {
            let mut seen = BTreeSet::new();
            let mut current = goal;
            while let Some(parent) = &current.parent_goal {
                if !seen.insert(current.id.as_str()) {
                    return Err("goal parent cycle".into());
                }
                current = goals.get(parent.as_str()).ok_or("invalid parent goal")?;
            }
        }
        fn visit<'a>(
            id: &'a str,
            goals: &BTreeMap<&'a str, &'a Goal>,
            path: &mut BTreeSet<&'a str>,
            done: &mut BTreeSet<&'a str>,
        ) -> Result<(), String> {
            if done.contains(id) {
                return Ok(());
            }
            if !path.insert(id) {
                return Err("goal dependency cycle".into());
            }
            let goal = goals.get(id).ok_or("missing goal dependency")?;
            for dependency in &goal.dependencies {
                if dependency == id {
                    return Err("self goal dependency".into());
                }
                visit(dependency, goals, path, done)?;
            }
            path.remove(id);
            done.insert(id);
            Ok(())
        }
        let mut done = BTreeSet::new();
        for id in goals.keys() {
            visit(id, &goals, &mut BTreeSet::new(), &mut done)?;
        }
        let mut criterion_ids = BTreeSet::new();
        let mut basis = BTreeSet::new();
        for criterion in &self.completion_criteria {
            if criterion.id.is_empty() || !criterion_ids.insert(&criterion.id) {
                return Err("duplicate or missing criterion ID".into());
            }
            source(
                state,
                &criterion.source_occurrence_id,
                &criterion.provenance,
            )?;
            if criterion.target_refs.is_empty() {
                return Err("criterion has no target".into());
            }
            for target in &criterion.target_refs {
                if state.get(target).is_none() {
                    return Err("invalid criterion target".into());
                }
            }
            if criterion.required {
                basis.insert(&criterion.kind);
            }
        }
        if self.operation_intent.kind == Some(OperationIntent::Solve)
            && self
                .operation_intent
                .qualifiers
                .contains(&IntentQualifier::AllSolutions)
            && ![
                CriterionKind::AllSolutionsFound,
                CriterionKind::NoExtraneousSolutions,
                CriterionKind::OutputTypeMatched,
            ]
            .iter()
            .all(|kind| basis.contains(kind))
        {
            return Err("missing all-solutions completion basis".into());
        }
        Ok(())
    }

    /// Source-independent structural intent comparison. IDs are stable semantic IDs.
    pub fn semantic_signature(&self) -> serde_json::Value {
        serde_json::json!({
            "givens": self.givens.iter().map(|g| &g.equation).collect::<Vec<_>>(),
            "constraints": self.constraints.iter().map(|c| &c.kind).collect::<Vec<_>>(),
            "goals": std::iter::once(self.primary_goal.as_ref()).flatten()
                .chain(&self.subgoals).map(|g| (&g.kind, &g.target_refs,
                    &g.required_relation, &g.dependencies)).collect::<Vec<_>>(),
            "operation": &self.operation_intent.kind,
            "qualifiers": &self.operation_intent.qualifiers,
            "output": self.expected_output.as_ref().map(|o| (&o.kind, &o.cardinality,
                &o.domain_ref, &o.equivalence_policy)),
            "criteria": self.completion_criteria.iter().map(|c| (&c.kind, &c.target_refs,
                &c.quantifier, c.required)).collect::<Vec<_>>(),
            "status": &self.intent_status,
        })
    }
}

fn source(state: &SemanticState, occurrence: &str, provenance: &Provenance) -> Result<(), String> {
    if provenance.source_type.is_empty() || provenance.source_id.is_empty() {
        return Err("missing provenance".into());
    }
    let object = state
        .get_occurrence(occurrence)
        .ok_or("dangling occurrence reference")?;
    if object.common().provenance.source_id != provenance.source_id
        || object.common().provenance.source_type != provenance.source_type
    {
        return Err("provenance and occurrence disagree".into());
    }
    Ok(())
}

/// The authoritative definition cannot be mutated through reasoning APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemDefinition {
    intent: ProblemIntent,
}

impl ProblemDefinition {
    pub fn new(intent: ProblemIntent, state: &SemanticState) -> Result<Self, String> {
        state.validate().map_err(|error| error.to_string())?;
        intent.validate(state)?;
        Ok(Self { intent })
    }
    pub fn intent(&self) -> &ProblemIntent {
        &self.intent
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceKind {
    MathematicalRelation,
    ConstraintApplication,
    GoalRelation,
    CompletionCriterion,
    MethodTrace,
    DerivationTrace,
    Contradiction,
    InsufficientInformation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub kind: EvidenceKind,
    pub semantic_refs: Vec<String>,
    pub occurrence_refs: Vec<String>,
    pub state_refs: Vec<u64>,
    pub goal_refs: Vec<String>,
    pub criterion_refs: Vec<String>,
    pub provenance: Provenance,
}

impl Evidence {
    pub fn validate(&self, state: &SemanticState, intent: &ProblemIntent) -> Result<(), String> {
        if self.id.is_empty() || self.provenance.source_id.is_empty() {
            return Err("missing evidence identity or provenance".into());
        }
        for id in &self.semantic_refs {
            if state.get(id).is_none() {
                return Err("dangling semantic evidence".into());
            }
        }
        for id in &self.occurrence_refs {
            if state.get_occurrence(id).is_none() {
                return Err("dangling occurrence evidence".into());
            }
        }
        if !self.occurrence_refs.is_empty()
            && !self.occurrence_refs.iter().any(|id| {
                state.get_occurrence(id).is_some_and(|object| {
                    object.common().provenance.source_id == self.provenance.source_id
                })
            })
        {
            return Err("evidence provenance does not match occurrence".into());
        }
        if self
            .state_refs
            .iter()
            .any(|version| *version > state.version)
        {
            return Err("future state evidence".into());
        }
        let goals: BTreeSet<_> = intent
            .primary_goal
            .iter()
            .chain(&intent.subgoals)
            .map(|g| g.id.as_str())
            .collect();
        for id in &self.goal_refs {
            if !goals.contains(id.as_str()) {
                return Err("dangling goal evidence".into());
            }
        }
        let criteria: BTreeSet<_> = intent
            .completion_criteria
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        for id in &self.criterion_refs {
            if !criteria.contains(id.as_str()) {
                return Err("dangling criterion evidence".into());
            }
        }
        Ok(())
    }
}
