use crate::{
    lean::{self, KernelRun, Translation},
    proof::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEvidence {
    evidence_id: String,
    model_id: String,
    model_revision: u64,
    obligation_id: String,
    source_ir_hash: String,
    lean_source_hash: String,
    proof_result: FormalStatus,
    counterexample: Option<crate::counterexample::Counterexample>,
    assumptions: Vec<Assumption>,
    dependencies: Vec<Dependency>,
    toolchain: KernelRun,
    timestamp: u64,
    artifact_hashes: BTreeMap<String, String>,
    translation: Option<lean::TranslationRecord>,
    diagnostics: Vec<String>,
    recipe: Option<lean::Recipe>,
}
impl VerificationEvidence {
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }
    pub fn result(&self) -> FormalStatus {
        self.proof_result
    }
    pub fn matches(&self, o: &ProofObligation) -> Result<(), String> {
        if self.obligation_id != o.id
            || self.model_id != o.model_id
            || self.model_revision != o.model_revision
            || self.source_ir_hash != o.source_hash()
            || self.dependencies != o.dependencies
            || self.assumptions != o.assumptions
        {
            return Err("evidence target/revision/dependency mismatch".into());
        }
        if let Some(c) = &self.counterexample {
            if crate::counterexample::check_candidate(o, c.witness.clone(), c.kind.clone())? != *c {
                return Err("counterexample target/revision mismatch".into());
            }
        }
        let mut copy = self.clone();
        copy.evidence_id.clear();
        if crate::hash(&copy) != self.evidence_id {
            return Err("evidence SHA-256 mismatch".into());
        }
        if matches!(
            self.proof_result,
            FormalStatus::Proved | FormalStatus::Refuted
        ) {
            let t = self
                .translation
                .as_ref()
                .ok_or("missing translation evidence")?;
            let rebuilt = lean::translate(
                o,
                self.recipe.clone().ok_or("missing proof recipe")?,
                t.refutation,
            )?;
            if rebuilt.record != *t {
                return Err("translation contract failed independent reconstruction".into());
            }
            if !self.toolchain.sandboxed
                || !self.toolchain.toolchain_verified
                || self.toolchain.exit_code != Some(0)
                || self.toolchain.timed_out
                || self.toolchain.status != self.proof_result
                || t.source_ir_hash != self.source_ir_hash
                || t.source_model_revision != self.model_revision
                || t.lean_module_hash != self.lean_source_hash
                || self.toolchain.source_hash != self.lean_source_hash
                || self.toolchain.mathlib_revision != lean::MATHLIB_REVISION
                || self.toolchain.foundation_hash.is_empty()
                || self.toolchain.independent_hash.is_empty()
                || !self.toolchain.stdout.contains("'Generated.target'")
                || self
                    .toolchain
                    .axioms
                    .iter()
                    .any(|a| !["propext", "Classical.choice", "Quot.sound"].contains(&a.as_str()))
                || self.artifact_hashes.get("Generated.lean") != Some(&self.lean_source_hash)
                || self.artifact_hashes.get("stdout.log")
                    != Some(&crate::hash_bytes(self.toolchain.stdout.as_bytes()))
                || self.artifact_hashes.get("stderr.log")
                    != Some(&crate::hash_bytes(self.toolchain.stderr.as_bytes()))
                || self.artifact_hashes.get("Foundation.lean")
                    != Some(&self.toolchain.foundation_hash)
                || self.artifact_hashes.get("Independent.lean")
                    != Some(&self.toolchain.independent_hash)
            {
                return Err("incomplete or unaudited kernel evidence".into());
            }
        }
        Ok(())
    }
    fn seal(&mut self) {
        self.evidence_id.clear();
        self.evidence_id = crate::hash(self);
    }
    pub fn verify(
        o: &ProofObligation,
        t: &Translation,
        project: &Path,
        work: &Path,
    ) -> Result<Self, String> {
        Self::verify_with_knowledge(o, t, project, work, None)
    }
    /// `verify` for translations whose recipe may be a Knowledge proof; `knowledge_root` is the
    /// Knowledge pack directory the runner compiles the referenced module from.
    pub fn verify_with_knowledge(
        o: &ProofObligation,
        t: &Translation,
        project: &Path,
        work: &Path,
        knowledge_root: Option<&Path>,
    ) -> Result<Self, String> {
        let r = lean::run_with_knowledge(o, t, project, work, knowledge_root)?;
        let mut e = Self {
            evidence_id: String::new(),
            model_id: o.model_id.clone(),
            model_revision: o.model_revision,
            obligation_id: o.id.clone(),
            source_ir_hash: o.source_hash(),
            lean_source_hash: t.record.lean_module_hash.clone(),
            proof_result: r.status,
            counterexample: None,
            assumptions: o.assumptions.clone(),
            dependencies: o.dependencies.clone(),
            toolchain: r.clone(),
            timestamp: now(),
            artifact_hashes: BTreeMap::from([
                (
                    "Independent.lean".into(),
                    crate::hash_bytes(include_bytes!("../lean/Independent.lean")),
                ),
                ("Generated.lean".into(), t.record.lean_module_hash.clone()),
                ("stdout.log".into(), crate::hash_bytes(r.stdout.as_bytes())),
                ("stderr.log".into(), crate::hash_bytes(r.stderr.as_bytes())),
                (
                    "Foundation.lean".into(),
                    crate::hash_bytes(include_bytes!("../lean/Foundation.lean")),
                ),
            ]),
            translation: Some(t.record.clone()),
            recipe: Some(t.recipe.clone()),
            diagnostics: r.diagnostics.clone(),
        };
        e.seal();
        e.matches(o)?;
        Ok(e)
    }
    pub fn verify_counterexample(
        o: &ProofObligation,
        c: &crate::counterexample::Counterexample,
        t: &Translation,
        project: &Path,
        work: &Path,
    ) -> Result<Self, String> {
        let checked = crate::counterexample::check_candidate(o, c.witness.clone(), c.kind.clone())?;
        if checked != *c
            || !checked.assumptions_satisfied
            || !checked.conclusion_false
            || !t.record.refutation
            || matches!(
                c.kind,
                crate::counterexample::CounterexampleKind::NumericalCandidate
            )
        {
            return Err("counterexample does not establish an exact in-domain violation".into());
        }
        let mut e = Self::verify(o, t, project, work)?;
        e.counterexample = Some(checked);
        e.seal();
        e.matches(o)?;
        Ok(e)
    }
    pub fn unresolved(
        o: &ProofObligation,
        status: FormalStatus,
        reason: &str,
    ) -> Result<Self, String> {
        o.validate()?;
        if matches!(status, FormalStatus::Proved | FormalStatus::Refuted) {
            return Err("formal status requires kernel evidence".into());
        }
        let mut e = Self {
            evidence_id: String::new(),
            model_id: o.model_id.clone(),
            model_revision: o.model_revision,
            obligation_id: o.id.clone(),
            source_ir_hash: o.source_hash(),
            lean_source_hash: String::new(),
            proof_result: status,
            counterexample: None,
            assumptions: o.assumptions.clone(),
            dependencies: o.dependencies.clone(),
            toolchain: KernelRun::infrastructure_error(reason),
            timestamp: now(),
            artifact_hashes: BTreeMap::new(),
            translation: None,
            recipe: None,
            diagnostics: vec![reason.into()],
        };
        e.seal();
        Ok(e)
    }
    /// Immutable content-addressed directories; retries never overwrite an earlier run.
    pub fn persist(
        &self,
        root: &Path,
        o: &ProofObligation,
        source: Option<&str>,
        foundation: Option<&[u8]>,
    ) -> Result<PathBuf, String> {
        self.matches(o)?;
        let mut artifacts: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        if let Some(s) = source {
            artifacts.insert("Generated.lean".into(), s.as_bytes().to_vec());
        }
        if let Some(s) = foundation {
            artifacts.insert("Foundation.lean".into(), s.to_vec());
        }
        if !self.artifact_hashes.is_empty() {
            artifacts.insert(
                "Independent.lean".into(),
                include_bytes!("../lean/Independent.lean").to_vec(),
            );
        }
        if !self.artifact_hashes.is_empty() {
            artifacts.insert(
                "stdout.log".into(),
                self.toolchain.stdout.as_bytes().to_vec(),
            );
            artifacts.insert(
                "stderr.log".into(),
                self.toolchain.stderr.as_bytes().to_vec(),
            );
        }
        for (name, hash) in &self.artifact_hashes {
            if artifacts.get(name).map(|v| crate::hash_bytes(v)) != Some(hash.clone()) {
                return Err(format!("missing/corrupt evidence artifact: {name}"));
            }
        }
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let dir = root.join(&self.evidence_id);
        fs::create_dir(&dir).map_err(|e| format!("immutable evidence: {e}"))?;
        for (name, data) in artifacts {
            write_new(&dir.join(name), &data)?;
        }
        write_new(
            &dir.join("evidence.json"),
            &serde_json::to_vec_pretty(self).unwrap(),
        )?;
        Ok(dir)
    }
    pub fn load(dir: &Path, o: &ProofObligation) -> Result<Self, String> {
        let e: Self = serde_json::from_slice(
            &fs::read(dir.join("evidence.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        e.matches(o)?;
        if dir.file_name().and_then(|s| s.to_str()) != Some(&e.evidence_id) {
            return Err("evidence directory identity".into());
        }
        for (name, hash) in &e.artifact_hashes {
            if Path::new(name).components().count() != 1 {
                return Err("unsafe artifact path".into());
            }
            let bytes = fs::read(dir.join(name)).map_err(|e| e.to_string())?;
            if crate::hash_bytes(&bytes) != *hash {
                return Err("artifact SHA-256 mismatch".into());
            }
        }
        Ok(e)
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
pub fn write_new(p: &Path, data: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(data).map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    models: BTreeMap<String, BTreeMap<u64, MathematicalModel>>,
    knowledge: BTreeMap<String, Dependency>,
}
impl ModelRegistry {
    pub fn models(&self) -> &BTreeMap<String, BTreeMap<u64, MathematicalModel>> {
        &self.models
    }
    pub fn register(&mut self, m: MathematicalModel) -> Result<(), String> {
        for (name, expr) in &m.definitions {
            if !crate::ir::identifier(name) {
                return Err("invalid definition name".into());
            }
            crate::ir::Context::default().infer(expr)?;
        }
        for invariant in &m.invariants {
            let c = crate::ir::Context::default();
            if !c.check(invariant).valid {
                return Err("invalid model invariant".into());
            }
        }
        if m.model_id.is_empty() || m.revision == 0 || m.proof_obligations.is_empty() {
            return Err("model identity/revision/obligations".into());
        }
        let mut seen = BTreeSet::new();
        for o in &m.proof_obligations {
            o.validate()?;
            if o.model_id != m.model_id || o.model_revision != m.revision || !seen.insert(&o.id) {
                return Err("obligation identity/missing revision".into());
            }
        }
        if self
            .models
            .get(&m.model_id)
            .is_some_and(|rs| rs.keys().last().is_some_and(|r| *r >= m.revision))
        {
            return Err("model revision must increase; history is immutable".into());
        }
        let mut next = self.clone();
        next.models
            .entry(m.model_id.clone())
            .or_default()
            .insert(m.revision, m);
        let graph = next
            .models
            .iter()
            .map(|(id, rs)| {
                (
                    id.clone(),
                    rs.values()
                        .last()
                        .unwrap()
                        .dependencies
                        .iter()
                        .filter(|d| next.models.contains_key(&d.id))
                        .map(|d| d.id.clone())
                        .collect(),
                )
            })
            .collect();
        validate_graph(&graph)?;
        *self = next;
        Ok(())
    }
    pub fn import_knowledge(&mut self, space: &mathlang_mirp::knowledge::KnowledgeSpace) {
        let mut next = BTreeMap::new();
        for u in space.units() {
            let h = u.occurrence_id();
            let revision = self
                .knowledge
                .get(&u.knowledge_id)
                .map_or(1, |old| old.revision + u64::from(old.content_hash != h));
            next.insert(
                u.knowledge_id.clone(),
                Dependency {
                    id: u.knowledge_id.clone(),
                    revision,
                    content_hash: h,
                },
            );
        }
        self.knowledge = next;
    }
    pub fn snapshot(&self, id: &str) -> Option<Dependency> {
        self.models
            .get(id)
            .and_then(|rs| rs.values().last())
            .map(|m| Dependency {
                id: id.into(),
                revision: m.revision,
                content_hash: m.content_hash(),
            })
            .or_else(|| self.knowledge.get(id).cloned())
    }
    pub fn is_current(&self, e: &VerificationEvidence) -> bool {
        fn deps_current(r: &ModelRegistry, ds: &[Dependency], seen: &mut BTreeSet<String>) -> bool {
            ds.iter().all(|d| {
                if r.snapshot(&d.id).as_ref() != Some(d) {
                    return false;
                }
                if !seen.insert(d.id.clone()) {
                    return true;
                }
                r.models
                    .get(&d.id)
                    .and_then(|rs| rs.values().last())
                    .is_none_or(|m| deps_current(r, &m.dependencies, seen))
            })
        }
        let Some(m) = self
            .models
            .get(&e.model_id)
            .and_then(|rs| rs.values().last())
        else {
            return false;
        };
        m.revision == e.model_revision
            && m.proof_obligations.iter().any(|o| e.matches(o).is_ok())
            && deps_current(self, &e.dependencies, &mut BTreeSet::new())
            && deps_current(self, &m.dependencies, &mut BTreeSet::new())
    }
    pub fn evaluate(&self, id: &str, es: &[VerificationEvidence]) -> Gate {
        let Some(m) = self.models.get(id).and_then(|rs| rs.values().last()) else {
            return Gate::NotVerified;
        };
        if m.proof_obligations.iter().all(|o| {
            es.iter().any(|e| {
                e.obligation_id == o.id
                    && e.proof_result == FormalStatus::Proved
                    && self.is_current(e)
                    && e.assumptions.iter().all(|a| match &a.origin {
                        AssumptionOrigin::ConstructionEvidence(id) => es.iter().any(|proof| {
                            proof.evidence_id == *id
                                && proof.proof_result == FormalStatus::Proved
                                && self.is_current(proof)
                                && self
                                    .models
                                    .get(&proof.model_id)
                                    .and_then(|rs| rs.get(&proof.model_revision))
                                    .is_some_and(|m| {
                                        m.proof_obligations.iter().any(|o| {
                                            o.id == proof.obligation_id
                                                && o.goal == a.proposition
                                                && o.assumptions.is_empty()
                                                && o.context.variables.is_empty()
                                        })
                                    })
                        }),
                        _ => false,
                    })
            })
        }) {
            Gate::Pass
        } else {
            Gate::NotVerified
        }
    }
}
pub fn artifact_manifest(root: &Path) -> Result<BTreeMap<String, String>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
                return Err("symlink in evidence tree".into());
            }
            if p.is_dir() {
                walk(root, &p, out)?
            } else if p.file_name().and_then(|s| s.to_str()) != Some("artifact_manifest.json") {
                out.insert(
                    p.strip_prefix(root).unwrap().to_string_lossy().into(),
                    crate::hash_bytes(&fs::read(&p).map_err(|e| e.to_string())?),
                );
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out)?;
    Ok(out)
}
