//! Only a closed proof recipe is emitted. Arbitrary Lean commands are never accepted.
use crate::{ir::*, proof::*};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, process::Command};
pub const TRANSLATION_VERSION: &str = "mathir-lean/0.1";
pub const MATHLIB_REVISION: &str = "56a005855ea17e0b0958f73ec801ae140b0a1679";
pub const LEAN_VERSION: &str = "4.35.0";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationRecord {
    pub source_ir_hash: String,
    pub source_model_revision: u64,
    pub translation_version: String,
    pub lean_module_hash: String,
    pub mathlib_revision: String,
    pub assumptions: Vec<Assumption>,
    pub semantic_mapping: BTreeMap<String, String>,
    pub validation_status: String,
    pub expected_statement: String,
    pub refutation: bool,
}
/// A kernel-checked Knowledge theorem used as the whole proof (`exact <constant>`).
/// `pack_manifest_sha256` pins the Knowledge pack; the runner compiles the module and its
/// imports from that pack's sources (each checked against the manifest) inside the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeProof {
    pub knowledge_id: String,
    pub constant: String,
    pub module: String,
    pub pack_manifest_sha256: String,
}
fn lean_name(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').all(|part| {
            let mut c = part.chars();
            c.next()
                .is_some_and(|f| f.is_ascii_alphabetic() || f == '_')
                && c.all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '\'')
        })
}
impl KnowledgeProof {
    pub fn validate(&self) -> Result<(), String> {
        if !lean_name(&self.constant)
            || !lean_name(&self.module)
            || self.module.starts_with("Mathlib")
            || ["Foundation", "Independent", "Generated"].contains(&self.module.as_str())
            || self.pack_manifest_sha256.len() != 64
            || !self
                .pack_manifest_sha256
                .chars()
                .all(|c| c.is_ascii_hexdigit())
            || self.knowledge_id.is_empty()
        {
            return Err(format!(
                "invalid Knowledge proof reference: {}",
                self.knowledge_id
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recipe {
    Auto,
    Reflexivity,
    UnitaryNorm,
    ArbitraryNormCounterexample,
    Collision,
    PhaseObservation,
    PhaseCollision,
    Incomplete,
    Knowledge(KnowledgeProof),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub source: String,
    pub record: TranslationRecord,
    pub recipe: Recipe,
}
fn dim(d: &Dimension) -> String {
    match d {
        Dimension::Fixed(n) => n.to_string(),
        Dimension::Symbol(s) => format!("v_{s}"),
    }
}
pub fn translate_type(t: &MathType) -> Result<String, String> {
    use MathType::*;
    Ok(match t {
        Bool => "Bool".into(),
        Nat => "ℕ".into(),
        Int => "ℤ".into(),
        Rational => "ℚ".into(),
        Real => "ℝ".into(),
        Complex => "ℂ".into(),
        Proposition => "Prop".into(),
        Universe => "Type".into(),
        Abstract(s) if identifier(s) => format!("v_{s}"),
        Finite(n) => format!("Fin {n}"),
        Vector(t, n) => format!("(Fin {} → {})", dim(n), translate_type(t)?),
        Matrix(t, n, m) => format!(
            "Matrix (Fin {}) (Fin {}) {}",
            dim(n),
            dim(m),
            translate_type(t)?
        ),
        Tensor(t, ds) => {
            let mut s = translate_type(t)?;
            for d in ds.iter().rev() {
                s = format!("(Fin {} → {s})", dim(d));
            }
            s
        }
        HilbertSpace(n) => format!("EuclideanSpace ℂ (Fin {})", dim(n)),
        LinearOperator(a, b) => format!("({} →L[ℂ] {})", translate_type(a)?, translate_type(b)?),
        Function(a, b) => format!("({} → {})", translate_type(a)?, translate_type(b)?),
        DensityOperator(n) => format!(
            "{{ m : Matrix (Fin {0}) (Fin {0}) ℂ // m.PosSemidef ∧ m.trace = 1 }}",
            dim(n)
        ),
        _ => return Err("unsupported type/undefined predicate signature".into()),
    })
}
fn emit(e: &MathExpr, c: &Context) -> Result<String, String> {
    use MathExpr::*;
    let bin = |a: &MathExpr, op: &str, b: &MathExpr| {
        Ok::<_, String>(format!("({} {op} {})", emit(a, c)?, emit(b, c)?))
    };
    Ok(match e {
        Variable(n) if identifier(n) => format!("v_{n}"),
        Constant(k) => {
            use crate::ir::Constant as C;
            match k {
                C::Bool(b) => b.to_string(),
                C::Natural(n) => format!("({n} : ℕ)"),
                C::Integer(n) => format!("({n} : ℤ)"),
                C::Rational(n, d) => format!("(({n} : ℚ) / ({d} : ℚ))"),
                C::Zero(t) => format!("(0 : {})", translate_type(t)?),
                C::One(t) | C::Identity(t) => format!("(1 : {})", translate_type(t)?),
                C::IdentityFunction(t) => {
                    format!("(id : {} → {})", translate_type(t)?, translate_type(t)?)
                }
            }
        }
        Add(a, b) => bin(a, "+", b)?,
        Subtract(a, b) => bin(a, "-", b)?,
        Multiply(a, b) => match (c.infer(a)?, c.infer(b)?) {
            (MathType::Complex, MathType::HilbertSpace(_)) => bin(a, "•", b)?,
            _ => bin(a, "*", b)?,
        },
        Divide(a, b) => bin(a, "/", b)?,
        Power(a, b) => bin(a, "^", b)?,
        Conjugate(a) => format!("(star {})", emit(a, c)?),
        Norm(a) => format!("‖{}‖", emit(a, c)?),
        InnerProduct(a, b) => format!("(inner ℂ {} {})", emit(a, c)?, emit(b, c)?),
        Adjoint(a) => match c.infer(a)? {
            MathType::Matrix(_, _, _) => format!("({}).conjTranspose", emit(a, c)?),
            _ => format!("({}).adjoint", emit(a, c)?),
        },
        MatrixMultiply(a, b) => match c.infer(a)? {
            MathType::LinearOperator(_, _) => format!("(({}).comp {})", emit(a, c)?, emit(b, c)?),
            _ => bin(a, "*", b)?,
        },
        Trace(a) => {
            if matches!(c.infer(a)?, MathType::DensityOperator(_)) {
                format!("({}).val.trace", emit(a, c)?)
            } else {
                format!("({}).trace", emit(a, c)?)
            }
        }
        Equality(a, b) => bin(a, "=", b)?,
        Inequality(a, b) => bin(a, "≤", b)?,
        LogicalAnd(a, b) => bin(a, "∧", b)?,
        LogicalOr(a, b) => bin(a, "∨", b)?,
        LogicalNot(a) => format!("(¬ {})", emit(a, c)?),
        Implies(a, b) => {
            let mut inner = c.clone();
            inner.assumptions.push(*a.clone());
            format!("({} → {})", emit(a, c)?, emit(b, &inner)?)
        }
        ForAll(n, t, b) | Exists(n, t, b) => {
            let mut ctx = c.clone();
            ctx.variables.insert(n.clone(), t.clone());
            format!(
                "({} (v_{} : {}), {})",
                if matches!(e, ForAll(_, _, _)) {
                    "∀"
                } else {
                    "∃"
                },
                n,
                translate_type(t)?,
                emit(b, &ctx)?
            )
        }
        PositiveSemidefinite(a) => format!("({}).PosSemidef", emit(a, c)?),
        Hermitian(a) => format!("({}).IsHermitian", emit(a, c)?),
        Normalized(a) => format!("(‖{}‖ = 1)", emit(a, c)?),
        Unitary(a) => {
            let x = emit(a, c)?;
            if matches!(c.infer(a)?, MathType::LinearOperator(_, _)) {
                format!("(({x}).adjoint.comp {x} = 1)")
            } else {
                format!("(({x}).conjTranspose * {x} = 1)")
            }
        }
        LinearMap(a) => {
            let x = emit(a, c)?;
            format!("(IsLinearMap ℂ {x})")
        }
        Update(state, key, value) => format!(
            "(Function.update {} {} {})",
            emit(state, c)?,
            emit(key, c)?,
            emit(value, c)?
        ),
        StateTransition(f, x) | Apply(f, x) => format!("({} {})", emit(f, c)?, emit(x, c)?),
        TensorProduct(a, b) => {
            let dims = |t: MathType| match t {
                MathType::Vector(_, n) => vec![n],
                MathType::Tensor(_, ds) => ds,
                _ => unreachable!("type checked tensor"),
            };
            let left = dims(c.infer(a)?);
            let right = dims(c.infer(b)?);
            let binders = left
                .iter()
                .chain(right.iter())
                .enumerate()
                .map(|(i, d)| format!("(tensor_i_{i} : Fin {})", dim(d)))
                .collect::<Vec<_>>()
                .join(" ");
            let apply = |value: String, start: usize, len: usize| {
                format!(
                    "({value} {})",
                    (start..start + len)
                        .map(|i| format!("tensor_i_{i}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };
            format!(
                "(fun {binders} => {} * {})",
                apply(emit(a, c)?, 0, left.len()),
                apply(emit(b, c)?, left.len(), right.len())
            )
        }
        TensorIndex(a, indices) => format!(
            "({} {})",
            emit(a, c)?,
            indices
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ),

        _ => return Err("invalid expression identifier".into()),
    })
}
pub fn statement(o: &ProofObligation, refutation: bool) -> Result<String, String> {
    o.validate()?;
    let mut goal = emit(&o.goal, &o.context)?;
    for a in o.context.assumptions.iter().rev() {
        goal = format!("({} → {goal})", emit(a, &o.context)?);
    }
    // Context variables are sorted topologically: abstract types and dimensions precede values.
    let mut vars: Vec<_> = o.context.variables.iter().collect();
    vars.sort_by_key(|(_, t)| {
        if matches!(t, MathType::Universe) {
            0
        } else if matches!(t, MathType::Nat) {
            1
        } else {
            2
        }
    });
    for (n, t) in vars.into_iter().rev() {
        goal = format!("(∀ (v_{n} : {}), {goal})", translate_type(t)?);
    }
    if refutation {
        goal = format!("(¬ {goal})");
    }
    Ok(goal)
}
pub fn translate(
    o: &ProofObligation,
    recipe: Recipe,
    refutation: bool,
) -> Result<Translation, String> {
    let goal = statement(o, refutation)?;
    let mut extra_import = String::new();
    let knowledge_proof;
    let proof=match &recipe {
        Recipe::Reflexivity=>"by\n  intros\n  rfl",
        Recipe::Auto=>"by\n  simp_all",
        Recipe::UnitaryNorm=>"by\n  exact Foundation.unitary_norm",
        Recipe::ArbitraryNormCounterexample=>"by\n  exact Foundation.arbitrary_norm_false",
        Recipe::Collision=>"by\n  intro v_A v_B v_E v_D v_x v_y hne hc\n  exact Foundation.collision v_E v_D v_x v_y hne hc",
        Recipe::PhaseCollision=>"by\n  exact Foundation.phase_collision",
        Recipe::PhaseObservation=>"by\n  exact Foundation.phase_observation",
        Recipe::Incomplete=>"by\n  skip",
        Recipe::Knowledge(k)=>{
            k.validate()?;
            if refutation {
                return Err("a Knowledge proof establishes the stated goal; refutation is unsupported".into());
            }
            extra_import = format!("public import {}\n", k.module);
            knowledge_proof = format!("by\n  exact {}", k.constant);
            knowledge_proof.as_str()
        }
    };
    let contract = match o.id.as_str() {
        "MV-01" => Some("mv01"),
        "MV-02" if refutation => Some("mv02"),
        "MV-03" => Some("mv03"),
        "MV-04" => Some("mv04"),
        "MV-05" => Some("mv05"),
        "MV-06" => Some("mv06"),
        _ => None,
    };
    let independent = contract.map_or(String::new(), |name| {
        format!("example : Independent.{name} := Generated.target\n")
    });
    let source=format!("module\npublic import Foundation\npublic import Independent\n{extra_import}set_option autoImplicit false\nset_option warningAsError true\nset_option maxRecDepth 1024\nset_option maxHeartbeats 2000000\nnoncomputable section\nopen scoped ComplexOrder\nnamespace Generated\npublic theorem target : {goal} := {proof}\nset_option pp.proofs true in\n#print Generated.target\n#print axioms Generated.target\n{independent}end Generated\n");
    let record = TranslationRecord {
        source_ir_hash: o.source_hash(),
        source_model_revision: o.model_revision,
        translation_version: TRANSLATION_VERSION.into(),
        lean_module_hash: crate::hash_bytes(source.as_bytes()),
        mathlib_revision: MATHLIB_REVISION.into(),
        assumptions: o.assumptions.clone(),
        semantic_mapping: BTreeMap::from([
            (
                "HilbertSpace".into(),
                "EuclideanSpace ℂ (Fin n), Euclidean norm".into(),
            ),
            (
                "LinearOperator".into(),
                "ContinuousLinearMap ℂ; finite dimensional".into(),
            ),
            (
                "Unitary".into(),
                "adjoint.comp self = 1 (isometry condition)".into(),
            ),
            (
                "Inequality".into(),
                "LE.le, ordered scalar types only".into(),
            ),
            (
                "assumptions/quantifiers".into(),
                "preserved structurally, explicit binders".into(),
            ),
        ]),
        validation_status: if contract.is_some() {
            "STRUCTURAL_AND_INDEPENDENT_KERNEL_CONTRACT".into()
        } else {
            "STRUCTURAL_CHECKED; INDEPENDENT_CASE_AUDIT_REQUIRED".into()
        },
        expected_statement: goal,
        refutation,
    };
    Ok(Translation {
        source,
        record,
        recipe,
    })
}
/// Rebuild from typed input; caller-supplied hashes, statements and revision flags are not trusted.
pub fn audit_translation(o: &ProofObligation, t: &Translation) -> Result<(), String> {
    let expected = translate(o, t.recipe.clone(), t.record.refutation)?;
    if expected.source != t.source || expected.record != t.record {
        return Err("translation/revision/assumption mismatch".into());
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelRun {
    pub status: FormalStatus,
    pub diagnostics: Vec<String>,
    pub exit_code: Option<i32>,
    pub commands: Vec<Vec<String>>,
    pub checked_modules: Vec<String>,
    pub elapsed_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub peak_rss_bytes: u64,
    pub sandboxed: bool,
    pub toolchain_verified: bool,
    pub axioms: Vec<String>,
    pub lean_version: String,
    pub mathlib_revision: String,
    pub source_hash: String,
    pub foundation_hash: String,
    pub independent_hash: String,
}
impl KernelRun {
    pub fn infrastructure_error(message: &str) -> Self {
        Self {
            status: FormalStatus::Error,
            diagnostics: vec![message.into()],
            exit_code: None,
            commands: vec![],
            checked_modules: vec![],
            elapsed_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
            timed_out: false,
            peak_rss_bytes: 0,
            sandboxed: false,
            toolchain_verified: false,
            axioms: vec![],
            lean_version: String::new(),
            mathlib_revision: String::new(),
            source_hash: String::new(),
            foundation_hash: String::new(),
            independent_hash: String::new(),
        }
    }
}
/// Runner arguments for a Knowledge recipe; empty for closed recipes.
fn knowledge_args(t: &Translation, knowledge_root: Option<&Path>) -> Result<Vec<String>, String> {
    let Recipe::Knowledge(k) = &t.recipe else {
        return Ok(vec![]);
    };
    k.validate()?;
    let root = knowledge_root.ok_or("Knowledge recipe requires the Knowledge pack directory")?;
    Ok(vec![
        "--knowledge-root".into(),
        root.to_string_lossy().into_owned(),
        "--knowledge-module".into(),
        k.module.clone(),
        "--knowledge-manifest-sha256".into(),
        k.pack_manifest_sha256.clone(),
    ])
}
pub fn run(
    o: &ProofObligation,
    t: &Translation,
    project: &Path,
    out: &Path,
) -> Result<KernelRun, String> {
    run_with_knowledge(o, t, project, out, None)
}
/// Same as `run`; `knowledge_root` is the Knowledge pack directory (holding
/// `package_manifest.json` and `formal/`) required by a Knowledge recipe.
pub fn run_with_knowledge(
    o: &ProofObligation,
    t: &Translation,
    project: &Path,
    out: &Path,
    knowledge_root: Option<&Path>,
) -> Result<KernelRun, String> {
    audit_translation(o, t)?;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let source = out.join("Generated.lean");
    std::fs::write(&source, &t.source).map_err(|e| e.to_string())?;
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/lean_runner.py");
    let output = Command::new("python3")
        .arg(script)
        .arg("--project")
        .arg(project)
        .arg("--source")
        .arg(&source)
        .args(knowledge_args(t, knowledge_root)?)
        .output();
    let mut r = match output {
        Err(e) => KernelRun::infrastructure_error(&format!("runner launch: {e}")),
        Ok(output) => match serde_json::from_slice::<KernelRun>(&output.stdout) {
            Ok(r) if output.status.success() => r,
            parsed => {
                let mut r = KernelRun::infrastructure_error(&format!(
                    "runner exit/protocol failure: {:?}",
                    parsed.err()
                ));
                r.stdout = String::from_utf8_lossy(&output.stdout).into();
                r.stderr = String::from_utf8_lossy(&output.stderr).into();
                r
            }
        },
    };
    r.status = assess_kernel(t, &r);
    std::fs::write(
        out.join("kernel.json"),
        serde_json::to_vec_pretty(&r).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(r)
}

/// Parse the target-specific audit independently of the runner's parsed axiom list.
pub fn assess_kernel(t: &Translation, r: &KernelRun) -> FormalStatus {
    if !r.sandboxed
        || !r.toolchain_verified
        || r.source_hash != t.record.lean_module_hash
        || r.mathlib_revision != MATHLIB_REVISION
        || !r.lean_version.contains("version 4.35.0,")
    {
        return FormalStatus::Error;
    }
    if r.timed_out {
        return FormalStatus::Unknown;
    }
    if r.exit_code != Some(0) {
        if !r.checked_modules.iter().any(|s| s == "Foundation")
            || !r.checked_modules.iter().any(|s| s == "Independent")
        {
            return FormalStatus::Error;
        }
        return if r.stdout.contains("lean.unknownIdentifier")
            || r.stdout.contains("unexpected token")
        {
            FormalStatus::Error
        } else {
            FormalStatus::Unknown
        };
    }
    let Some(start) = r.stdout.find("'Generated.target' ") else {
        return FormalStatus::Error;
    };
    let text = &r.stdout[start + "'Generated.target' ".len()..];
    let axioms = if let Some(text) = text.strip_prefix("depends on axioms: [") {
        let Some((list, _)) = text.split_once(']') else {
            return FormalStatus::Error;
        };
        list.split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    } else if text.starts_with("does not depend on any axioms") {
        vec![]
    } else {
        return FormalStatus::Error;
    };
    if axioms != r.axioms
        || axioms
            .iter()
            .any(|a| !["propext", "Classical.choice", "Quot.sound"].contains(&a.as_str()))
    {
        return FormalStatus::Error;
    }
    if t.record.refutation {
        FormalStatus::Refuted
    } else {
        FormalStatus::Proved
    }
}

/// Result of a Lean build check without the sealed sandbox. Never Evidence: no evidence_id,
/// no registry entry, and `formal_status` of the obligation stays UNKNOWN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildCheck {
    /// Kernel outcome under the same judgement as the sealed path (`assess_kernel`), with the
    /// isolation requirement lifted: PROVED/REFUTED mean "the kernel accepted the generated
    /// module with only standard axioms", not sealed formal truth.
    pub kernel_outcome: FormalStatus,
    pub isolation: String,
    pub isolation_diagnostics: Vec<String>,
    pub sealed: bool,
    pub run: KernelRun,
}

/// Run the Lean kernel on a translated module with the explicit Linux unsealed isolation mode.
pub fn build_check(
    t: &Translation,
    project: &Path,
    out: &Path,
    knowledge_root: Option<&Path>,
) -> Result<BuildCheck, String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let source = out.join("Generated.lean");
    std::fs::write(&source, &t.source).map_err(|e| e.to_string())?;
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/lean_runner.py");
    let output = Command::new("python3")
        .arg(script)
        .arg("--project")
        .arg(project)
        .arg("--source")
        .arg(&source)
        .arg("--isolation")
        .arg("linux-unsealed")
        .args(knowledge_args(t, knowledge_root)?)
        .output()
        .map_err(|e| format!("runner launch: {e}"))?;
    let raw: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("runner protocol failure: {e}"))?;
    let run: KernelRun = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
    let isolation = raw["isolation"].as_str().unwrap_or_default().to_string();
    let isolation_diagnostics: Vec<String> =
        serde_json::from_value(raw["isolation_diagnostics"].clone()).unwrap_or_default();
    let kernel_outcome = if run.sandboxed || isolation != "linux-unsealed" {
        // A sealed-mode result must go through VerificationEvidence, never through here.
        FormalStatus::Error
    } else if !isolation_diagnostics.is_empty() {
        // Something outside staging stayed writable: do not report a kernel outcome.
        FormalStatus::Error
    } else {
        let mut probe = run.clone();
        probe.sandboxed = true;
        assess_kernel(t, &probe)
    };
    let check = BuildCheck {
        kernel_outcome,
        isolation,
        isolation_diagnostics,
        sealed: false,
        run,
    };
    std::fs::write(
        out.join("build_check.json"),
        serde_json::to_vec_pretty(&check).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(check)
}
