use crate::*;
use mathlang_mirp::knowledge::{KnowledgeSpace, KnowledgeUnit};
use mathlang_verification::{
    counterexample, dsn,
    evidence::{ModelRegistry, VerificationEvidence},
    fixtures, ir, lean,
    proof::*,
};
use std::{collections::BTreeMap, process::Command as Process};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    obligation: ProofObligation,
    recipe: lean::Recipe,
    refutation: bool,
}
fn request(p: &Path) -> Result<Request> {
    let v: Value = read(p)?;
    if v.get("obligation").is_some() {
        decode(v)
    } else {
        Ok(Request {
            obligation: decode(v)?,
            recipe: lean::Recipe::Auto,
            refutation: false,
        })
    }
}
fn validate(o: &ProofObligation) -> Result<()> {
    o.validate().map_err(|e| Error::new(3, e))
}
fn model(p: &Path) -> Result<MathematicalModel> {
    let m: MathematicalModel = read(p)?;
    ModelRegistry::default()
        .register(m.clone())
        .map_err(|e| Error::new(3, e))?;
    Ok(m)
}
fn normalized_model(m: &MathematicalModel) -> Result<Value> {
    // Normalize only typed expression slots with the existing IR function.
    fn expr(e: &mut ir::MathExpr) -> Result<()> {
        *e = serde_json::from_str(&ir::normalized(e).map_err(|e| Error::new(3, e))?)
            .map_err(|e| Error::new(9, e))?;
        Ok(())
    }
    fn assumptions(a: &mut [Assumption]) -> Result<()> {
        for a in a {
            expr(&mut a.proposition)?;
        }
        Ok(())
    }
    let mut m = m.clone();
    for e in m.definitions.values_mut().chain(m.invariants.iter_mut()) {
        expr(e)?;
    }
    assumptions(&mut m.assumptions)?;
    for o in &mut m.proof_obligations {
        expr(&mut o.goal)?;
        for a in &mut o.context.assumptions {
            expr(a)?;
        }
        assumptions(&mut o.assumptions)?;
    }
    Ok(json!(m))
}
fn set_model(e: &mut Value, m: &MathematicalModel) {
    e["model_id"] = json!(m.model_id);
    e["model_revision"] = json!(m.revision);
}
fn set_obligation(e: &mut Value, o: &ProofObligation) {
    e["model_id"] = json!(o.model_id);
    e["model_revision"] = json!(o.model_revision);
}
fn resource_error(message: String) -> Error {
    let code = if message.contains("budget") {
        6
    } else if message.contains("unsupported") || message.contains("requires Bool") {
        4
    } else {
        3
    };
    Error::new(code, message)
}

pub fn execute(cli: &Cli, config: &Config, e: &mut Value) -> Result<()> {
    match &cli.command {
        Command::Model { action } => {
            let (input, op) = match action {
                ModelAction::Check { input } => (input, "check"),
                ModelAction::Inspect { input } => (input, "inspect"),
                ModelAction::Normalize { input } => (input, "normalize"),
            };
            let m = model(input)?;
            set_model(e, &m);
            e["result"] = match op {
                "check" => {
                    json!({"valid":true,"content_hash":m.content_hash(),"obligations":m.proof_obligations.iter().map(|o|o.context.check(&o.goal)).collect::<Vec<_>>()})
                }
                "normalize" => normalized_model(&m)?,
                _ => json!({"model":m,"content_hash":m.content_hash()}),
            };
        }
        Command::Verify { input } => verify(cli, config, e, input)?,
        Command::Proof { action } => {
            let (input, plan) = match action {
                ProofAction::Plan { input } => (input, true),
                ProofAction::Counterexample { input } => (input, false),
            };
            let r = request(input)?;
            validate(&r.obligation)?;
            set_obligation(e, &r.obligation);
            e["formal_status"] = json!(FormalStatus::Unknown);
            e["result"] = if plan {
                json!(dsn::plan(&r.obligation, &KnowledgeSpace::legacy(), 1000)
                    .map_err(resource_error)?)
            } else {
                json!({"obligation":r.obligation,"candidates":counterexample::search_finite(&r.obligation,1000).map_err(resource_error)?,"note":"Candidates only; REFUTED requires verify with a checked whole-statement negation."})
            };
        }
        Command::Solve { problem } => {
            let binary = std::env::current_exe().map_err(|x| Error::new(5, x))?;
            let mut child = Process::new(binary);
            child.args(["__solve-worker", problem]);
            let output = crate::process::capture(&mut child, std::time::Duration::from_secs(300))?;
            e["result"] = serde_json::from_slice(&output.stdout).map_err(|x| {
                Error::new(
                    5,
                    format!(
                        "solver protocol failure: {x}; {}",
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )
            })?;
            if !output.status.success() {
                return Err(Error::new(
                    output
                        .status
                        .code()
                        .filter(|c| (2..=9).contains(c))
                        .unwrap_or(5),
                    "existing MIRP solver did not complete; inspect result diagnostics",
                ));
            }
        }
        Command::Knowledge { action } => match action {
            KnowledgeAction::Update { .. } => {
                return Err(Error::new(4, "Knowledge update is OUT_OF_SCOPE in v0.1"))
            }
            KnowledgeAction::Inspect { input } => {
                let v: Value = read(input)?;
                let mut space = KnowledgeSpace::empty();
                let units: Vec<KnowledgeUnit> = if v.is_array() {
                    decode(v)?
                } else {
                    vec![decode(v)?]
                };
                space.register_batch(units).map_err(|x| Error::new(3, x))?;
                e["result"] = json!({"knowledge":space.units().map(|u|json!({"unit":u,"occurrence_id":u.occurrence_id(),"revision":1,"revision_scope":"initial import; historical revision requires ModelRegistry"})).collect::<Vec<_>>()});
            }
            KnowledgeAction::Impact { input, revision } => {
                if *revision == 0 {
                    return Err(Error::new(2, "revision must be positive"));
                }
                let registry = registry(input)?;
                let mut models = vec![];
                for (id, revisions) in registry.models() {
                    let Some(old) = revisions.get(revision) else {
                        continue;
                    };
                    let latest = registry.snapshot(id);
                    let mut affected = vec![];
                    // Core is_current includes transitive model and Knowledge dependency invalidation.
                    for (other, rs) in registry.models() {
                        let m = rs
                            .values()
                            .last()
                            .ok_or_else(|| Error::new(7, "empty revision history"))?;
                        let mut stale = vec![];
                        for o in &m.proof_obligations {
                            let candidate = VerificationEvidence::unresolved(
                                o,
                                FormalStatus::Unknown,
                                "dependency inspection",
                            )
                            .map_err(|x| Error::new(7, x))?;
                            if !registry.is_current(&candidate) {
                                stale.push(o.id.clone());
                            }
                        }
                        if !stale.is_empty() {
                            affected.push(json!({"model_id":other,"stale_obligations":stale}));
                        }
                    }
                    models.push(json!({"model_id":id,"selected_revision":old.revision,"latest":latest,"dependencies":old.dependencies,"obligations":old.proof_obligations,"stale_targets":affected}));
                }
                if models.is_empty() {
                    return Err(Error::new(7, "revision not found in registry"));
                }
                e["result"] = json!({"models":models,"note":"Read-only dependency inspection; no revision updates or Evidence writes."});
            }
        },
        Command::Compare { left, right } => {
            let a = model(left)?;
            let b = model(right)?;
            let av = json!(a);
            let bv = json!(b);
            let fields = [
                "model_id",
                "revision",
                "definitions",
                "assumptions",
                "operations",
                "invariants",
                "proof_obligations",
                "dependencies",
            ];
            let differences: Vec<_> = fields
                .into_iter()
                .filter(|k| av[*k] != bv[*k])
                .map(|k| json!({"field":k,"left":av[k],"right":bv[k]}))
                .collect();
            let conditions = a.assumptions == b.assumptions
                && a.dependencies == b.dependencies
                && a.proof_obligations
                    .iter()
                    .map(|o| (&o.goal, &o.context, &o.assumptions, &o.dependencies))
                    .eq(b
                        .proof_obligations
                        .iter()
                        .map(|o| (&o.goal, &o.context, &o.assumptions, &o.dependencies)));
            e["result"] = json!({"comparability":if conditions {"CONDITIONALLY_COMPARABLE"}else{"NOT_COMPARABLE"},"comparison_scope":"IR structure and declared conditions; execution environments unavailable","differences":differences,"formal_evidence":compare_evidence(&a,left,&b,right)?,"environment_comparability":"UNKNOWN: no execution environment supplied","mathematical_equivalence":"UNKNOWN","ranking":null,"note":"Structural comparison only; equality of hashes or IR does not prove mathematical equivalence."});
        }
        Command::Evidence { action } => {
            let (input, obligation, reg) = match action {
                EvidenceAction::Show {
                    input,
                    obligation,
                    registry,
                }
                | EvidenceAction::Validate {
                    input,
                    obligation,
                    registry,
                } => (input, obligation, registry),
            };
            let (proof, o) = load_evidence(input, obligation.as_deref())?;
            let current = reg
                .as_ref()
                .map(|p| registry(p).map(|r| r.is_current(&proof)))
                .transpose()?;
            set_obligation(e, &o);
            e["formal_status"] = json!(proof.result());
            e["evidence"]["reference"] = json!(input);
            e["result"] = json!({"evidence":proof,"integrity":"VALID","revision_current":current,"authenticity":"NOT_ESTABLISHED_BY_HASH"});
            if current == Some(false) {
                return Err(Error::new(
                    7,
                    "Evidence is stale against supplied ModelRegistry",
                ));
            }
        }
        Command::Doctor => e["result"] = doctor(config),
    }
    Ok(())
}
fn registry(p: &Path) -> Result<ModelRegistry> {
    let r: ModelRegistry = read(p)?;
    let mut rebuilt = ModelRegistry::default();
    // Revalidate deserialized histories with the existing register contract.
    for revisions in r.models().values() {
        for m in revisions.values() {
            rebuilt.register(m.clone()).map_err(|x| Error::new(7, x))?;
        }
    }
    Ok(r)
}
fn load_evidence(
    p: &Path,
    explicit: Option<&Path>,
) -> Result<(VerificationEvidence, ProofObligation)> {
    safe_path(p)?;
    let raw: Value = read::<VerificationEvidence>(&p.join("evidence.json"))
        .map(|v| json!(v))
        .map_err(|x| Error::new(7, x.message))?;
    let id = raw["obligation_id"]
        .as_str()
        .ok_or_else(|| Error::new(7, "missing obligation ID"))?;
    let mut o = None;
    let paths: Vec<_> = if let Some(p) = explicit {
        vec![p.to_path_buf()]
    } else {
        p.ancestors()
            .take(4)
            .map(|p| p.join("proof_obligations.json"))
            .filter(|p| p.exists())
            .collect()
    };
    for path in paths {
        let v: Value = read(&path).map_err(|x| Error::new(7, x.message))?;
        let values = if let Some(a) = v.as_array() {
            a.clone()
        } else if v.get("obligation").is_some() {
            vec![v["obligation"].clone()]
        } else {
            vec![v]
        };
        for v in values {
            if v["id"] == id && v["model_revision"] == raw["model_revision"] {
                o = Some(decode::<ProofObligation>(v).map_err(|x| Error::new(7, x.message))?);
            }
        }
    }
    let o =
        o.ok_or_else(|| Error::new(7, "target obligation unavailable; supply --obligation PATH"))?;
    validate(&o).map_err(|x| Error::new(7, x.message))?;
    let artifacts = raw["artifact_hashes"]
        .as_object()
        .ok_or_else(|| Error::new(7, "missing artifact hashes"))?;
    for name in artifacts.keys() {
        safe_path(&p.join(name)).map_err(|x| Error::new(7, x.message))?;
    }
    let proof = VerificationEvidence::load(p, &o).map_err(|x| Error::new(7, x))?;
    Ok((proof, o))
}
fn compare_evidence(
    a: &MathematicalModel,
    ap: &Path,
    b: &MathematicalModel,
    bp: &Path,
) -> Result<Value> {
    fn refs(m: &MathematicalModel, input: &Path) -> Vec<Value> {
        m.proof_obligations.iter().flat_map(|o|o.evidence_refs.iter().map(move |reference| {
            let p=input.parent().unwrap_or(Path::new(".")).join(reference);
            match load_evidence(&p,None) {
                Ok((e,target)) if target.id==o.id && e.matches(o).is_ok()=>json!({"reference":reference,"integrity":"VALID","status":e.result(),"revision_currency":"UNKNOWN_WITHOUT_REGISTRY"}),
                _=>json!({"reference":reference,"integrity":"INVALID_OR_UNAVAILABLE"})
            }
        })).collect()
    }
    Ok(json!({"left":refs(a,ap),"right":refs(b,bp)}))
}
fn verify(cli: &Cli, config: &Config, e: &mut Value, input: &str) -> Result<()> {
    let cases = if input == "usm" {
        let mut c = fixtures::usm();
        c.extend(fixtures::auxiliary());
        c.into_iter()
            .map(|(obligation, recipe, refutation)| Request {
                obligation,
                recipe,
                refutation,
            })
            .collect::<Vec<_>>()
    } else {
        vec![request(Path::new(input))?]
    };
    for r in &cases {
        validate(&r.obligation)?;
    }
    let out = cli.output.clone().unwrap_or_else(|| {
        config
            .evidence
            .directory
            .join(e["run_id"].as_str().unwrap_or("run"))
    });
    new_directory(&out)?;
    e["evidence"]["reference"] = json!(out);
    let obligations: Vec<_> = cases.iter().map(|r| r.obligation.clone()).collect();
    save(&out.join("proof_obligations.json"), &obligations)?;
    save(&out.join("configuration.json"), config)?;
    let binary = std::env::current_exe().map_err(|x| Error::new(5, x))?;
    save(
        &out.join("environment.json"),
        &json!({"cli_version":env!("CARGO_PKG_VERSION"),"binary_sha256":hash_bytes(&fs::read(binary).map_err(|x|Error::new(8,x))?),"rust":probe("rustc",&["--version"],Path::new(ROOT)),"reason":probe("reason",&["--version"],Path::new(ROOT)),"lean":lean::LEAN_VERSION,"mathlib_revision":lean::MATHLIB_REVISION,"configuration":config}),
    )?;
    let sources = source_manifest()?;
    save(&out.join("source_manifest.json"), &sources)?;

    save(
        &out.join("commands.json"),
        &std::env::args().collect::<Vec<_>>(),
    )?;
    let outcome = (|| {
        let mut results = vec![];
        let mut failure_code = None;
        let repeats = if input == "usm" { 3 } else { 1 };
        for repeat in 0..repeats {
            for (i, r) in cases.iter().enumerate() {
                let t = lean::translate(&r.obligation, r.recipe, r.refutation)
                    .map_err(resource_error)?;
                let proof = VerificationEvidence::verify(
                    &r.obligation,
                    &t,
                    &Path::new(ROOT).join("verification/lean"),
                    &out.join(format!("lean_logs/{repeat}-{i}")),
                )
                .map_err(|x| Error::new(5, x))?;
                let path = proof
                    .persist(
                        &out.join("registry"),
                        &r.obligation,
                        Some(&t.source),
                        Some(include_bytes!("../../verification/lean/Foundation.lean")),
                    )
                    .map_err(|x| Error::new(8, x))?;
                VerificationEvidence::load(&path, &r.obligation).map_err(|x| Error::new(7, x))?;
                let v = json!(proof);
                if v["toolchain"]["timed_out"] == true {
                    failure_code = Some(6);
                } else if proof.result() == FormalStatus::Error && failure_code != Some(6) {
                    failure_code = Some(5);
                }
                results.push(proof);
            }
        }
        save(&out.join("proof_results.json"), &results)?;
        e["result"] = json!({"proof_results":results,"repetitions":repeats});
        // A set retains individual statuses; never invent a model-wide formal verdict.
        if cases.len() == 1 {
            set_obligation(e, &cases[0].obligation);
            e["formal_status"] = json!(results[0].result());
        }
        if let Some(code) = failure_code {
            return Err(Error::new(code,"verification interrupted or environment failed; inspect persisted kernel diagnostics"));
        }
        Ok(())
    })();
    if let Err(ref error) = outcome {
        save(&out.join("failure.json"), error)?;
    }
    let manifest =
        mathlang_verification::evidence::artifact_manifest(&out).map_err(|x| Error::new(8, x))?;
    save(&out.join("artifact_manifest.json"), &manifest)?;
    outcome
}

fn probe(program: &str, args: &[&str], cwd: &Path) -> Value {
    // Diagnostics use fixed executables and offline arguments only.
    match crate::process::capture(
        Process::new(program).args(args).current_dir(cwd),
        std::time::Duration::from_secs(30),
    ) {
        Ok(o) => {
            json!({"available":o.status.success(),"exit_code":o.status.code(),"stdout":String::from_utf8_lossy(&o.stdout).trim(),"stderr":String::from_utf8_lossy(&o.stderr).trim()})
        }
        Err(x) => json!({"available":false,"error":x.message}),
    }
}
fn doctor(config: &Config) -> Value {
    let root = Path::new(ROOT);
    let mut checks = BTreeMap::new();
    checks.insert("rust", probe("rustc", &["--version"], root));
    checks.insert("reason", probe("reason", &["--version"], root));
    let lean = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".elan/toolchains/leanprover--lean4---v4.35.0/bin/lean");
    checks.insert(
        "lean",
        probe(
            lean.to_str().unwrap_or("/unavailable/lean"),
            &["--version"],
            root,
        ),
    );
    checks.insert(
        "cargo",
        probe(
            "cargo",
            &[
                "metadata",
                "--locked",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
                "cli/Cargo.toml",
            ],
            root,
        ),
    );
    // Do not retain the large dependency metadata in the response.
    if checks["cargo"]["available"] == true {
        checks.get_mut("cargo").unwrap()["stdout"] =
            json!("locked offline dependency resolution succeeded");
    }
    let sandbox = probe(
        "/usr/bin/sandbox-exec",
        &[
            "-p",
            "(version 1) (allow default) (deny network*)",
            "/usr/bin/true",
        ],
        root,
    );
    checks.insert("sandbox", sandbox);
    checks.insert("python", probe("python3", &["--version"], root));
    let manifest: Value = fs::read(root.join("verification/lean/lake-manifest.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let mut packages = vec![];
    for p in manifest["packages"].as_array().into_iter().flatten() {
        if let Some(name) = p["name"].as_str() {
            let path = root.join("verification/lean/.lake/packages").join(name);
            let rev = probe("git", &["rev-parse", "HEAD"], &path);
            let clean = probe("git", &["diff", "--quiet", "HEAD", "--"], &path);
            packages.push(json!({"package":name,"expected":p["rev"],"actual":rev["stdout"],"available":rev["available"]==true&&rev["stdout"]==p["rev"]&&clean["available"]==true,"cache_present":path.join(".lake/build/lib/lean").is_dir()}));
        }
    }
    let packages_ok = !packages.is_empty()
        && packages
            .iter()
            .all(|p| p["available"] == true && p["cache_present"] == true);
    let files = [
        "verification/scripts/lean_runner.py",
        "verification/lean/Foundation.lean",
        "verification/lean/Independent.lean",
        "verification/lean/lean-toolchain",
        "mathlang_dsn/reason.toml",
    ];
    let resources: Vec<_> = files
        .iter()
        .map(|name| json!({"path":name,"available":fs::File::open(root.join(name)).is_ok()}))
        .collect();
    let tools_ok = checks.values().all(|v| v["available"] == true);
    let pins_ok = checks["rust"]["stdout"]
        .as_str()
        .is_some_and(|s| s.starts_with("rustc 1.93.1 "))
        && checks["reason"]["stdout"]
            .as_str()
            .is_some_and(|s| s.contains("0.5.6.5"))
        && checks["lean"]["stdout"]
            .as_str()
            .is_some_and(|s| s.contains("version 4.35.0,"));
    let ready =
        tools_ok && pins_ok && packages_ok && resources.iter().all(|v| v["available"] == true);
    json!({"readiness":if ready {"READY"}else if checks["rust"]["available"]==true {"PARTIAL"}else{"NOT_READY"},"cli_version":env!("CARGO_PKG_VERSION"),"checks":checks,"packages":packages,"resources":resources,"configuration":config,"required_reason":"0.5.6.5","mathlib_revision":lean::MATHLIB_REVISION,"source_root":root,"note":"Read-only offline checks; runtime compatibility and formal verification require acceptance tests."})
}

fn source_manifest() -> Result<BTreeMap<String, String>> {
    fn walk(root: &Path, p: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        if p.is_dir() {
            for e in fs::read_dir(p).map_err(|x| Error::new(8, x))? {
                walk(root, &e.map_err(|x| Error::new(8, x))?.path(), out)?;
            }
        } else {
            out.insert(
                p.strip_prefix(root)
                    .map_err(|x| Error::new(9, x))?
                    .to_string_lossy()
                    .into_owned(),
                hash_bytes(&fs::read(p).map_err(|x| Error::new(8, x))?),
            );
        }
        Ok(())
    }
    let root = Path::new(ROOT);
    let mut out = BTreeMap::new();
    for name in [
        "cli/src",
        "cli/Cargo.toml",
        "cli/Cargo.lock",
        "mirp/src",
        "mirp/Cargo.toml",
        "mirp/Cargo.lock",
        "verification/src",
        "verification/Cargo.toml",
        "verification/Cargo.lock",
        "verification/scripts",
        "verification/lean/Foundation.lean",
        "verification/lean/Independent.lean",
        "verification/lean/lean-toolchain",
        "verification/lean/lake-manifest.json",
    ] {
        walk(root, &root.join(name), &mut out)?;
    }
    Ok(out)
}
