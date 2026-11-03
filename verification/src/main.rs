use mathlang_verification::{evidence::*, lean::*, proof::*, *};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn save<T: serde::Serialize>(dir: &Path, name: &str, v: &T) -> Result<(), String> {
    write_new(
        &dir.join(name),
        &serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
}
fn source_snapshot(root: &Path) -> Result<BTreeMap<String, String>, String> {
    fn walk(root: &Path, p: &Path, out: &mut BTreeMap<String, String>) -> Result<(), String> {
        if p.is_dir() {
            for f in std::fs::read_dir(p).map_err(|e| e.to_string())? {
                let f = f.map_err(|e| e.to_string())?;
                walk(root, &f.path(), out)?;
            }
        } else {
            out.insert(
                p.strip_prefix(root).unwrap().to_string_lossy().into(),
                hash_bytes(&std::fs::read(p).map_err(|e| e.to_string())?),
            );
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    for name in [
        "src",
        "tests",
        "scripts",
        "fixtures",
        "Cargo.toml",
        "Cargo.lock",
        "toolchain.json",
        "lean/Foundation.lean",
        "lean/Independent.lean",
        "lean/lean-toolchain",
        "lean/lakefile.toml",
        "lean/lake-manifest.json",
    ] {
        walk(root, &root.join(name), &mut out)?;
    }
    Ok(out)
}
fn execute(out: PathBuf) -> Result<(), String> {
    std::fs::create_dir(&out).map_err(|e| e.to_string())?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, compiled) in [
        ("lib.rs", include_bytes!("lib.rs").as_slice()),
        ("ir.rs", include_bytes!("ir.rs").as_slice()),
        ("proof.rs", include_bytes!("proof.rs").as_slice()),
        ("dsn.rs", include_bytes!("dsn.rs").as_slice()),
        ("lean.rs", include_bytes!("lean.rs").as_slice()),
        (
            "counterexample.rs",
            include_bytes!("counterexample.rs").as_slice(),
        ),
        ("evidence.rs", include_bytes!("evidence.rs").as_slice()),
        ("fixtures.rs", include_bytes!("fixtures.rs").as_slice()),
        ("main.rs", include_bytes!("main.rs").as_slice()),
    ] {
        if std::fs::read(root.join("src").join(name)).map_err(|e| e.to_string())? != compiled {
            return Err(format!(
                "stale verification binary: {name}; rebuild before running"
            ));
        }
    }
    let project = root.join("lean");
    let knowledge = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let mut fixtures = fixtures::usm();
    fixtures.extend(fixtures::auxiliary());
    let obligations: Vec<_> = fixtures.iter().map(|(o, _, _)| o.clone()).collect();
    let mut registry = ModelRegistry::default();
    registry.import_knowledge(&knowledge);
    registry.register(fixtures::model(&obligations))?;
    save(&out, "source_manifest.json", &source_snapshot(root)?)?;
    save(&out, "model_registry.json", &registry)?;
    save(
        &out,
        "model_hashes.json",
        &vec![registry.snapshot("USM-MVF").ok_or("missing model hash")?],
    )?;
    save(&out, "proof_obligations.json", &obligations)?;
    let mut all_evidence = vec![];
    let mut plans = vec![];
    let mut repeats = vec![];
    for repeat in 0..3 {
        let mut summary = BTreeMap::new();
        for (o, recipe, negation) in fixtures.iter() {
            let mut plan = dsn::plan(o, &knowledge, 1000)?;
            let work = out
                .join("lean_logs")
                .join(format!("repeat-{repeat}-{}", o.id));
            let t = translate(o, recipe.clone(), *negation)?;
            let e = VerificationEvidence::verify(o, &t, &project, &work)?;
            let path = e.persist(
                &out.join("registry"),
                o,
                Some(&t.source),
                Some(include_bytes!("../lean/Foundation.lean")),
            )?;
            VerificationEvidence::load(&path, o)?;
            plan.apply_evidence(o, &e)?;
            summary.insert(o.id.clone(),json!({"status":e.result(),"source_hash":o.source_hash(),"translation_hash":t.record.lean_module_hash,"plan_hash":hash(&dsn::plan(o,&knowledge,1000)?)}));
            println!("repeat={repeat} {} {:?}", o.id, e.result());
            all_evidence.push(e);
            plans.push(plan);
        }
        repeats.push(summary);
    }
    let deterministic = repeats.windows(2).all(|w| w[0] == w[1]);
    save(
        &out,
        "repeatability.json",
        &json!({"repetitions":3,"identical":deterministic,"runs":repeats}),
    )?;
    save(
        &out,
        "commands.json",
        &json!({"entry":std::env::args().collect::<Vec<_>>(),"kernel_commands":"proof_results.json -> toolchain.commands","rust_tests":"recorded separately in regression baseline and Rust test logs"}),
    )?;
    save(
        &out,
        "validation_results.json",
        &obligations
            .iter()
            .map(|o| o.context.check(&o.goal))
            .collect::<Vec<_>>(),
    )?;
    save(&out, "proof_results.json", &all_evidence)?;
    save(&out, "dependency_graph.json", &plans)?;
    let mut finite = obligations[0].clone();
    finite.id = "CV-FINITE-CANDIDATE".into();
    finite.kind = ProofKind::Construction;
    finite.goal = ir::MathExpr::forall(
        "x",
        ir::MathType::Bool,
        ir::MathExpr::eq(
            ir::MathExpr::var("x"),
            ir::MathExpr::Constant(ir::Constant::Bool(true)),
        ),
    );
    let candidates = counterexample::search_finite(&finite, 16)?;
    save(
        &out,
        "exact_candidates.json",
        &json!({"obligation":finite,"candidates":candidates,"formal_status":"UNKNOWN","note":"Exact finite evaluation is retained as a candidate until a kernel negation is checked."}),
    )?;
    let invalid_context = ir::Context {
        variables: BTreeMap::from([
            (
                "x".into(),
                ir::MathType::Vector(Box::new(ir::MathType::Complex), ir::Dimension::Fixed(2)),
            ),
            (
                "y".into(),
                ir::MathType::Vector(Box::new(ir::MathType::Complex), ir::Dimension::Fixed(3)),
            ),
        ]),
        assumptions: vec![],
    };
    let bad = ir::MathExpr::Add(
        Box::new(ir::MathExpr::var("x")),
        Box::new(ir::MathExpr::var("y")),
    );
    let bad_goal = ir::MathExpr::eq(bad.clone(), bad);
    save(
        &out,
        "rejected_typecheck.json",
        &json!({"context":invalid_context,"goal":bad_goal,"evidence":invalid_context.check(&bad_goal)}),
    )?;
    save(
        &out,
        "counterexamples.json",
        &json!({"MV-02":{"operator":"2 • identity","dimension":1,"state":"EuclideanSpace.single 0 1","exact_negation":"Foundation.arbitrary_norm_false","evidence":"proof_results.json"},"MV-04":{"condition":"E x = E y and x ≠ y","claim":"not both deterministic decodings correct","evidence":"proof_results.json"},"CV-UPDATE":"same-key updates 1 then 2 differ from 2 then 1","CV-PHASE":"unit modulus phase preserves norm; observational model only","MV-06":"missing positive evidence excludes known in conjunction model"}),
    )?;
    let mv_pass = (0..3).all(|i| {
        fixtures.iter().all(|(o, _, neg)| {
            repeats[i][&o.id]["status"]
                == json!(if *neg {
                    FormalStatus::Refuted
                } else {
                    FormalStatus::Proved
                })
        })
    });
    save(
        &out,
        "phase_results.json",
        &json!({"P0":"SEE_BASELINE_REPORT","P1":"SEE_RUST_TESTS","P2":if mv_pass{"PARTIAL"}else{"NOT_VERIFIED"},"P3":"SEE_RUST_TESTS","P4":"SEE_RUST_TESTS","P5":if mv_pass && deterministic{"PASS"}else{"NOT_VERIFIED"},"overall":"PARTIAL","note":"This per-run gate file excludes separate negative and semantic suites; final gate adjudication requires their integration and independent source/artifact audit; model-level PASS is distinct from expected-refutation test PASS."}),
    )?;
    save(
        &out,
        "environment.json",
        &json!({"lean":LEAN_VERSION,"mathlib_revision":MATHLIB_REVISION,"translation_version":TRANSLATION_VERSION,"knowledge_revision":hash(&knowledge),"max_rss_bytes":2147483648u64,"seconds_per_case":300,"parallelism":1,"planner_max_nodes":1000,"unix_timestamp":now(),"rustc":std::process::Command::new("rustc").arg("--version").output().map(|x|String::from_utf8_lossy(&x.stdout).trim().to_string()).unwrap_or_default(),"binary_sha256":hash_bytes(&std::fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?)}),
    )?;
    if source_snapshot(root)?
        != serde_json::from_slice::<BTreeMap<String, String>>(
            &std::fs::read(out.join("source_manifest.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    {
        return Err("source changed during verification; run is not canonical".into());
    }
    let manifest = artifact_manifest(&out)?;
    save(&out, "artifact_manifest.json", &manifest)?;
    if !deterministic {
        return Err("nondeterministic verification results".into());
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    obligation: ProofObligation,
    recipe: Recipe,
    refutation: bool,
}
fn verify_request(input: &Path, out: &Path) -> Result<(), String> {
    let req: Request = serde_json::from_slice(&std::fs::read(input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir(out).map_err(|e| e.to_string())?;
    let t = translate(&req.obligation, req.recipe, req.refutation)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let e = VerificationEvidence::verify(
        &req.obligation,
        &t,
        &root.join("lean"),
        &out.join("lean_logs"),
    )?;
    e.persist(
        &out.join("registry"),
        &req.obligation,
        Some(&t.source),
        Some(include_bytes!("../lean/Foundation.lean")),
    )?;
    save(out, "proof_results.json", &e)?;
    save(out, "proof_obligations.json", &req.obligation)?;
    println!("{} {:?}", req.obligation.id, e.result());
    Ok(())
}

fn check_translations(out: &Path) -> Result<(), String> {
    std::fs::create_dir(out).map_err(|e| e.to_string())?;
    let cases = fixtures::translation_checks();
    let mut source=String::from("module\npublic import Foundation\nset_option warningAsError true\nset_option autoImplicit false\nnoncomputable section\nopen scoped ComplexOrder\nnamespace Generated\n");
    for (i, (o, expected)) in cases.iter().enumerate() {
        let goal = statement(o, false)?;
        source.push_str(&format!("public theorem case_{i} : {goal} := by\n  intros\n  rfl\nexample : {expected} := case_{i}\n#print axioms Generated.case_{i}\n"));
    }
    source.push_str("public theorem target : True := True.intro\n#print axioms Generated.target\nend Generated\n");
    let file = out.join("Generated.lean");
    write_new(&file, source.as_bytes())?;
    save(out, "cases.json", &cases)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let run = std::process::Command::new("python3")
        .arg(root.join("scripts/lean_runner.py"))
        .arg("--project")
        .arg(root.join("lean"))
        .arg("--source")
        .arg(file)
        .output()
        .map_err(|e| e.to_string())?;
    write_new(&out.join("runner.stderr.log"), &run.stderr)?;
    write_new(&out.join("kernel.json"), &run.stdout)?;
    let r: KernelRun = serde_json::from_slice(&run.stdout).map_err(|e| e.to_string())?;
    if r.status != FormalStatus::Proved
        || !r.sandboxed
        || !r.toolchain_verified
        || r.exit_code != Some(0)
    {
        return Err(format!(
            "independent semantic checks failed: {} {}",
            r.stdout, r.stderr
        ));
    }
    println!("{} independent translation checks PASS", cases.len());
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let result = match args.as_slice() {
        [_, command, out] if command == "verify-usm" => execute(PathBuf::from(out)),
        [_, command, out] if command == "check-translations" => check_translations(Path::new(out)),
        [_, command, input, out] if command == "verify-obligation" => {
            verify_request(Path::new(input), Path::new(out))
        }
        _ => {
            eprintln!("usage: mathlang-verification verify-usm <new-directory> | verify-obligation <request.json> <new-directory>");
            std::process::exit(2)
        }
    };
    if let Err(e) = result {
        eprintln!("verification error: {e}");
        std::process::exit(1)
    }
}
