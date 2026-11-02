//! Bounded structural proof planning. No rule here establishes formal truth.
use crate::{
    ir::{MathExpr, MathType},
    proof::*,
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateStatus {
    Candidate,
    Verified,
    Rejected,
    Blocked,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanNode {
    pub id: String,
    pub goal: MathExpr,
    pub assumptions: Vec<MathExpr>,
    pub variables: BTreeMap<String, MathType>,
    pub rule: String,
    pub children: Vec<String>,
    pub source_knowledge: Vec<Dependency>,
    pub candidate_status: CandidateStatus,
    pub formal_status: FormalStatus,
    pub reasons: Vec<UnknownReason>,
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub obligation_id: String,
    pub obligation_hash: String,
    pub knowledge_revision: String,
    pub nodes: Vec<PlanNode>,
    pub lemma_graph: BTreeMap<String, Vec<String>>,
}
impl Plan {
    pub fn graph(&self) -> BTreeMap<String, Vec<String>> {
        self.nodes
            .iter()
            .map(|n| (n.id.clone(), n.children.clone()))
            .chain(
                self.lemma_graph
                    .iter()
                    .map(|(id, ds)| (id.clone(), ds.clone())),
            )
            .collect()
    }
    pub fn apply_evidence(
        &mut self,
        obligation: &ProofObligation,
        e: &crate::evidence::VerificationEvidence,
    ) -> Result<(), String> {
        if self.obligation_id != obligation.id || self.obligation_hash != obligation.source_hash() {
            return Err("plan/evidence obligation mismatch".into());
        }
        e.matches(obligation)?;
        let root = self.nodes.first_mut().ok_or("missing root")?;
        root.formal_status = e.result();
        root.evidence_refs.push(e.evidence_id().into());
        root.candidate_status = match e.result() {
            FormalStatus::Proved => CandidateStatus::Verified,
            FormalStatus::Refuted => CandidateStatus::Rejected,
            _ => CandidateStatus::Unknown,
        };
        Ok(())
    }
}
/// Retrieval key for a typed IR statement: normalized IR with bound variables renamed by
/// binder depth, so alpha-equivalent statements share a key. Free variables keep their names.
pub fn retrieval_key(e: &MathExpr) -> Result<String, String> {
    fn rename(v: &mut serde_json::Value, bound: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(name)) = map.get_mut("Variable") {
                    if let Some(depth) = bound.iter().rposition(|b| b == name) {
                        *name = format!("#{depth}");
                    }
                    return;
                }
                for quantifier in ["ForAll", "Exists"] {
                    if let Some(serde_json::Value::Array(parts)) = map.get_mut(quantifier) {
                        if let (Some(name), 3) = (parts[0].as_str().map(String::from), parts.len())
                        {
                            parts[0] = serde_json::Value::String(format!("#{}", bound.len()));
                            bound.push(name);
                            rename(&mut parts[2], bound);
                            bound.pop();
                            return;
                        }
                    }
                }
                for value in map.values_mut() {
                    rename(value, bound);
                }
            }
            serde_json::Value::Array(xs) => {
                for x in xs {
                    rename(x, bound);
                }
            }
            _ => {}
        }
    }
    let mut v: serde_json::Value =
        serde_json::from_str(&crate::ir::normalized(e)?).map_err(|e| e.to_string())?;
    rename(&mut v, &mut vec![]);
    serde_json::to_string(&v).map_err(|e| e.to_string())
}
/// A KnowledgeUnit participates in formal goal retrieval only when its conclusion is a typed
/// MathExpr IR statement. Descriptive conclusions (for example the bundled operational
/// `validated result of ...` RULEs) are not formal statements and are never matched.
pub fn knowledge_retrieval_key(u: &mathlang_mirp::knowledge::KnowledgeUnit) -> Option<String> {
    let e: MathExpr = serde_json::from_str(&u.conclusion).ok()?;
    retrieval_key(&e).ok()
}
pub fn plan(o: &ProofObligation, space: &KnowledgeSpace, max_nodes: usize) -> Result<Plan, String> {
    o.validate()?;
    let index: Vec<(String, &mathlang_mirp::knowledge::KnowledgeUnit)> = space
        .units()
        .filter_map(|u| knowledge_retrieval_key(u).map(|k| (k, u)))
        .collect();
    let revisions: Vec<_> = space
        .units()
        .map(|u| (u.knowledge_id.clone(), u.occurrence_id()))
        .collect();
    let mut p = Plan {
        obligation_id: o.id.clone(),
        obligation_hash: o.source_hash(),
        knowledge_revision: crate::hash(&revisions),
        nodes: vec![],
        lemma_graph: BTreeMap::new(),
    };
    fn expand(
        p: &mut Plan,
        g: &MathExpr,
        a: Vec<MathExpr>,
        v: BTreeMap<String, MathType>,
        index: &[(String, &mathlang_mirp::knowledge::KnowledgeUnit)],
        max: usize,
    ) -> Result<String, String> {
        if p.nodes.len() >= max {
            return Err("proof planning budget exceeded".into());
        }
        let id = format!("{}:{}", p.obligation_id, p.nodes.len());
        let i = p.nodes.len();
        let mut node = PlanNode {
            id: id.clone(),
            goal: g.clone(),
            assumptions: a.clone(),
            variables: v.clone(),
            rule: "lemma-extraction".into(),
            children: vec![],
            source_knowledge: vec![],
            candidate_status: CandidateStatus::Unknown,
            formal_status: FormalStatus::Unknown,
            reasons: vec![],
            evidence_refs: vec![],
        };
        // An existing textual theorem is a retrieval candidate, never a checked lemma.
        let key = retrieval_key(g)?;
        for (_, u) in index.iter().filter(|(k, _)| *k == key) {
            node.source_knowledge.push(Dependency {
                id: u.knowledge_id.clone(),
                revision: 1,
                content_hash: u.occurrence_id(),
            });
            node.candidate_status = CandidateStatus::Candidate;
        }
        p.nodes.push(node);
        match g {
            MathExpr::LogicalAnd(x, y) => {
                p.nodes[i].rule = "conjunction-elimination".into();
                let l = expand(p, x, a.clone(), v.clone(), index, max)?;
                let r = expand(p, y, a, v, index, max)?;
                p.nodes[i].children = vec![l, r];
            }
            MathExpr::Implies(x, y) => {
                p.nodes[i].rule = "implication-introduction".into();
                let mut a = a;
                a.push(*x.clone());
                let c = expand(p, y, a, v, index, max)?;
                p.nodes[i].children.push(c);
            }
            MathExpr::ForAll(n, t, b) => {
                p.nodes[i].rule = "forall-introduction".into();
                let mut v = v;
                v.insert(n.clone(), t.clone());
                let c = expand(p, b, a, v, index, max)?;
                p.nodes[i].children.push(c);
            }
            _ if a.contains(g) => {
                p.nodes[i].rule = "premise-match".into();
                p.nodes[i].candidate_status = CandidateStatus::Candidate;
            }
            _ => {
                p.nodes[i].reasons.push(UnknownReason::UnprovedLemma);
                if p.nodes[i].source_knowledge.is_empty() {
                    p.nodes[i].reasons.push(UnknownReason::MissingPremise);
                    p.nodes[i].candidate_status = CandidateStatus::Blocked;
                }
            }
        }
        Ok(id)
    }
    expand(
        &mut p,
        &o.goal,
        o.context.assumptions.clone(),
        o.context.variables.clone(),
        &index,
        max_nodes,
    )?;
    fn retrieve(
        id: &str,
        space: &KnowledgeSpace,
        graph: &mut BTreeMap<String, Vec<String>>,
        max: usize,
    ) -> Result<(), String> {
        if graph.contains_key(&format!("knowledge:{id}")) {
            return Ok(());
        }
        if graph.len() >= max {
            return Err("lemma graph budget exceeded".into());
        }
        let u = space
            .get(id)
            .ok_or_else(|| format!("missing lemma: {id}"))?;
        graph.insert(
            format!("knowledge:{id}"),
            u.dependencies
                .iter()
                .map(|d| format!("knowledge:{d}"))
                .collect(),
        );
        for dep in &u.dependencies {
            retrieve(dep, space, graph, max)?;
        }
        Ok(())
    }
    let candidates: Vec<_> = p
        .nodes
        .iter()
        .flat_map(|n| n.source_knowledge.iter().map(|d| d.id.clone()))
        .collect();
    for dep in o.required_lemmas.iter().chain(candidates.iter()) {
        if let Some(unit) = space.get(dep) {
            retrieve(
                dep,
                space,
                &mut p.lemma_graph,
                max_nodes.saturating_sub(p.nodes.len()),
            )?;
            p.nodes[0].source_knowledge.push(Dependency {
                id: dep.clone(),
                revision: 1,
                content_hash: unit.occurrence_id(),
            });
            let edge = format!("knowledge:{dep}");
            if !p.nodes[0].children.contains(&edge) {
                p.nodes[0].children.push(edge);
            }
        } else {
            p.nodes[0].reasons.push(UnknownReason::UnprovedLemma);
            p.nodes[0].candidate_status = CandidateStatus::Blocked;
        }
    }
    validate_graph(&p.graph())?;
    Ok(p)
}
/// Closed, typed definitions may be expanded into a candidate subgoal. Formal source remains original.
pub fn plan_model(
    m: &MathematicalModel,
    space: &KnowledgeSpace,
    max_nodes: usize,
) -> Result<Vec<Plan>, String> {
    let mut plans = vec![];
    for o in &m.proof_obligations {
        let mut p = plan(o, space, max_nodes)?;
        if !m.definitions.is_empty() {
            for (name, value) in &m.definitions {
                if o.context.variables.get(name)
                    != Some(&crate::ir::Context::default().infer(value)?)
                {
                    return Err("definition signature mismatch".into());
                }
            }
            fn replace(
                v: &mut serde_json::Value,
                ds: &BTreeMap<String, MathExpr>,
                bound: &mut Vec<String>,
            ) {
                if let serde_json::Value::Object(map) = v {
                    if let Some(serde_json::Value::String(name)) = map.get("Variable") {
                        if !bound.contains(name) {
                            if let Some(e) = ds.get(name) {
                                *v = serde_json::to_value(e).unwrap();
                                return;
                            }
                        }
                    }
                    for quantifier in ["ForAll", "Exists"] {
                        if let Some(serde_json::Value::Array(parts)) = map.get_mut(quantifier) {
                            if let Some(name) = parts[0].as_str() {
                                bound.push(name.into());
                                replace(&mut parts[2], ds, bound);
                                bound.pop();
                                return;
                            }
                        }
                    }
                    for value in map.values_mut() {
                        replace(value, ds, bound)
                    }
                } else if let serde_json::Value::Array(xs) = v {
                    for value in xs {
                        replace(value, ds, bound)
                    }
                }
            }
            let mut value = serde_json::to_value(&o.goal).unwrap();
            replace(&mut value, &m.definitions, &mut vec![]);
            let expanded: MathExpr = serde_json::from_value(value).map_err(|e| e.to_string())?;
            if expanded != o.goal {
                if p.nodes.len() + p.lemma_graph.len() >= max_nodes {
                    return Err("definition expansion budget exceeded".into());
                }
                let mut child = o.clone();
                child.id = format!("{}:definition", o.id);
                child.goal = expanded;
                let extra = plan(
                    &child,
                    space,
                    max_nodes - p.nodes.len() - p.lemma_graph.len(),
                )?;
                p.nodes[0].rule = "definition-expansion".into();
                p.nodes[0].children.push(extra.nodes[0].id.clone());
                p.nodes.extend(extra.nodes);
                p.lemma_graph.extend(extra.lemma_graph);
            }
        }
        validate_graph(&p.graph())?;
        plans.push(p);
    }
    Ok(plans)
}
