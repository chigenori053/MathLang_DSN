use crate::ir::{Context, MathExpr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormalStatus {
    Proved,
    Refuted,
    Unknown,
    OutOfScope,
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gate {
    Pass,
    Partial,
    Fail,
    NotVerified,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofKind {
    Construction,
    Conditional,
    Impossibility,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssumptionOrigin {
    ConstructionEvidence(String),
    ExternalSpecification(String),
    Unproved,
    False,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    pub proposition: MathExpr,
    pub origin: AssumptionOrigin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownReason {
    MissingDefinition,
    MissingPremise,
    UnprovedLemma,
    UnsupportedTranslation,
    BudgetExceeded,
    CounterexampleCandidate,
    RefutationEstablished,
    MissingEvidence,
    StaleRevision,
    /// No Knowledge statement matched this goal and it is not an assumption. This says nothing
    /// about whether the goal needs extra premises (use MissingPremise only when that is known).
    NoMatchingKnowledge,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub revision: u64,
    pub content_hash: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: String,
    pub model_id: String,
    pub model_revision: u64,
    pub goal: MathExpr,
    pub context: Context,
    pub assumptions: Vec<Assumption>,
    pub required_lemmas: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub kind: ProofKind,
    pub verification_method: String,
    pub proof_status: FormalStatus,
    pub evidence_refs: Vec<String>,
}
impl ProofObligation {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.model_id.is_empty() || self.model_revision == 0 {
            return Err("obligation identity/revision".into());
        }
        if self.context.assumptions
            != self
                .assumptions
                .iter()
                .map(|a| a.proposition.clone())
                .collect::<Vec<_>>()
        {
            return Err("assumption audit mismatch".into());
        }
        let v = self.context.check(&self.goal);
        if !v.valid {
            return Err(v.diagnostics.join("; "));
        }
        Ok(())
    }
    pub fn source_hash(&self) -> String {
        crate::hash(&(
            self.model_id.clone(),
            self.model_revision,
            &self.goal,
            &self.context,
            &self.assumptions,
            &self.required_lemmas,
            &self.dependencies,
            &self.kind,
        ))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MathematicalModel {
    pub model_id: String,
    pub revision: u64,
    pub definitions: BTreeMap<String, MathExpr>,
    pub assumptions: Vec<Assumption>,
    pub operations: Vec<String>,
    pub invariants: Vec<MathExpr>,
    pub proof_obligations: Vec<ProofObligation>,
    pub dependencies: Vec<Dependency>,
}
impl MathematicalModel {
    pub fn content_hash(&self) -> String {
        crate::hash(self)
    }
}
pub fn validate_graph(graph: &BTreeMap<String, Vec<String>>) -> Result<(), String> {
    fn visit(
        id: &str,
        g: &BTreeMap<String, Vec<String>>,
        active: &mut Vec<String>,
        done: &mut Vec<String>,
    ) -> Result<(), String> {
        if done.iter().any(|x| x == id) {
            return Ok(());
        }
        if active.iter().any(|x| x == id) {
            return Err(format!("cyclic dependency: {id}"));
        }
        let deps = g
            .get(id)
            .ok_or_else(|| format!("missing dependency: {id}"))?;
        active.push(id.into());
        for d in deps {
            visit(d, g, active, done)?
        }
        active.pop();
        done.push(id.into());
        Ok(())
    }
    let mut done = vec![];
    for id in graph.keys() {
        visit(id, graph, &mut vec![], &mut done)?
    }
    Ok(())
}
