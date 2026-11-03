//! Two-stage verification path: the DSN path decides computability first, and only cases that
//! the DSN path computes are passed to Lean. Stage 1 never proves anything.
use crate::{
    dsn, lean,
    proof::{FormalStatus, ProofObligation},
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    IrDecode,
    Validate,
    DsnPlan,
    LeanTranslate,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResult {
    pub step: Step,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage1 {
    pub id: String,
    pub dsn_computable: bool,
    pub blocked_at: Option<Step>,
    pub steps: Vec<StepResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<lean::Recipe>,
    pub refutation: bool,
    /// The obligation when it decoded; needed for Stage 2 and Evidence matching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation: Option<ProofObligation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lean_module_hash: Option<String>,
    #[serde(skip)]
    pub translation: Option<lean::Translation>,
}

fn ok(step: Step, detail: Option<Value>) -> StepResult {
    StepResult {
        step,
        ok: true,
        error: None,
        detail,
    }
}
fn fail(step: Step, error: String) -> StepResult {
    StepResult {
        step,
        ok: false,
        error: Some(error),
        detail: None,
    }
}

/// Stage 1 for one request value: `{obligation, recipe?, refutation?}` or a bare obligation.
/// Steps: MathIR decoding, obligation validation and type check, DSN plan with the supplied
/// Knowledge, Lean translation. A case is DSN-computable only when every step succeeds.
pub fn stage1(id: &str, raw: &Value, space: &KnowledgeSpace, max_nodes: usize) -> Stage1 {
    let mut s = Stage1 {
        id: id.into(),
        dsn_computable: false,
        blocked_at: None,
        steps: vec![],
        recipe: None,
        refutation: false,
        obligation: None,
        lean_module_hash: None,
        translation: None,
    };
    let (o_value, recipe, refutation) = if raw.get("obligation").is_some() {
        (
            raw["obligation"].clone(),
            raw.get("recipe")
                .cloned()
                .unwrap_or(Value::String("Auto".into())),
            raw["refutation"].as_bool().unwrap_or(false),
        )
    } else {
        (raw.clone(), Value::String("Auto".into()), false)
    };
    s.refutation = refutation;
    let decoded = serde_json::from_value::<ProofObligation>(o_value)
        .map_err(|e| e.to_string())
        .and_then(|o| {
            serde_json::from_value::<lean::Recipe>(recipe)
                .map(|r| (o, r))
                .map_err(|e| e.to_string())
        });
    let (o, recipe) = match decoded {
        Ok(x) => x,
        Err(e) => {
            s.steps.push(fail(Step::IrDecode, e));
            s.blocked_at = Some(Step::IrDecode);
            return s;
        }
    };
    s.steps.push(ok(Step::IrDecode, None));
    s.recipe = Some(recipe);
    s.obligation = Some(o.clone());
    if let Err(e) = o.validate() {
        s.steps.push(fail(Step::Validate, e));
        s.blocked_at = Some(Step::Validate);
        return s;
    }
    s.steps.push(ok(
        Step::Validate,
        Some(serde_json::json!(o.context.check(&o.goal))),
    ));
    match dsn::plan(&o, space, max_nodes) {
        Ok(p) => {
            let mut candidates: Vec<String> = p
                .nodes
                .iter()
                .flat_map(|n| n.source_knowledge.iter().map(|d| d.id.clone()))
                .collect();
            candidates.sort();
            candidates.dedup();
            s.steps.push(ok(
                Step::DsnPlan,
                Some(
                    serde_json::json!({"nodes":p.nodes.len(),"root_rule":p.nodes[0].rule,
                    "knowledge_candidates":candidates,"knowledge_revision":p.knowledge_revision,
                    "formal_status":p.nodes[0].formal_status}),
                ),
            ));
        }
        Err(e) => {
            s.steps.push(fail(Step::DsnPlan, e));
            s.blocked_at = Some(Step::DsnPlan);
            return s;
        }
    }
    match lean::translate(&o, recipe, refutation) {
        Ok(t) => {
            s.steps.push(ok(
                Step::LeanTranslate,
                Some(
                    serde_json::json!({"lean_module_hash":t.record.lean_module_hash,
                    "expected_statement":t.record.expected_statement}),
                ),
            ));
            s.lean_module_hash = Some(t.record.lean_module_hash.clone());
            s.translation = Some(t);
            s.dsn_computable = true;
        }
        Err(e) => {
            s.steps.push(fail(Step::LeanTranslate, e));
            s.blocked_at = Some(Step::LeanTranslate);
        }
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Isolation {
    /// macOS sandbox-exec through the existing runner; produces sealed VerificationEvidence.
    Sealed,
    /// Linux namespaces; produces an unsealed build check, never Evidence.
    LinuxUnsealed,
}
impl Isolation {
    pub fn detect() -> Option<Self> {
        let has = |p: &str| {
            std::env::var_os("PATH")
                .is_some_and(|path| std::env::split_paths(&path).any(|d| d.join(p).is_file()))
        };
        if has("sandbox-exec") {
            Some(Self::Sealed)
        } else if cfg!(target_os = "linux") && has("unshare") {
            Some(Self::LinuxUnsealed)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage2 {
    pub id: String,
    pub lean_checked: bool,
    pub isolation: Option<Isolation>,
    /// Sealed path: the Evidence result. Unsealed path: always UNKNOWN (no formal verdict).
    pub formal_status: FormalStatus,
    /// Unsealed path only: kernel outcome of the build check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_check: Option<FormalStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    pub note: String,
}
