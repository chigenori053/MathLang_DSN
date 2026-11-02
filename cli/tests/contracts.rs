use mathlang_verification::{evidence::*, fixtures, ir::*, proof::*};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};
fn temp() -> PathBuf {
    let p = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(mathlang_cli::run_id());
    fs::create_dir(&p).unwrap();
    p
}
fn run(args: &[&str], code: i32) -> Value {
    let r = Command::new(env!("CARGO_BIN_EXE_mathlang"))
        .args(args)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(
        r.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&r.stdout)
    );
    let v: Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v["schema_version"], "mathlang.cli/0.1");
    v
}
fn write(p: &std::path::Path, v: &impl serde::Serialize) {
    fs::write(p, serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
#[test]
fn arguments_and_output_contract() {
    for flag in ["--help", "--version"] {
        assert!(Command::new(env!("CARGO_BIN_EXE_mathlang"))
            .arg(flag)
            .status()
            .unwrap()
            .success());
    }
    let r = run(&["bogus"], 2);
    assert_eq!(r["execution_status"], "ERROR");
    assert_eq!(r["error"]["category"], "INPUT");
    let p = temp();
    let path = p.join("result.json");
    let o = fixtures::usm().remove(0).0;
    let model = fixtures::model(&[o]);
    let input = p.join("model.json");
    write(&input, &model);
    let v = run(
        &[
            "model",
            "check",
            input.to_str().unwrap(),
            "--output",
            path.to_str().unwrap(),
        ],
        0,
    );
    assert_eq!(
        v,
        serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap()
    );
    run(
        &[
            "model",
            "check",
            input.to_str().unwrap(),
            "--output",
            path.to_str().unwrap(),
        ],
        8,
    );
    let result = Command::new(env!("CARGO_BIN_EXE_mathlang"))
        .args(["model", "check", input.to_str().unwrap(), "--quiet"])
        .output()
        .unwrap();
    assert!(result.stdout.is_empty());
}
#[test]
fn models_normalization_and_comparison() {
    let p = temp();
    let path = p.join("model.json");
    let other = p.join("other.json");
    let o = fixtures::usm().remove(0).0;
    let mut m = fixtures::model(&[o]);
    m.definitions
        .insert("q".into(), MathExpr::Constant(Constant::Rational(2, 4)));
    write(&path, &m);
    let before = fs::read(&path).unwrap();
    let v = run(&["model", "normalize", path.to_str().unwrap()], 0);
    assert_eq!(
        v["result"]["definitions"]["q"],
        json!({"Constant":{"Rational":[1,2]}})
    );
    assert_eq!(before, fs::read(&path).unwrap());
    run(&["model", "inspect", path.to_str().unwrap()], 0);
    let v = run(
        &["compare", path.to_str().unwrap(), path.to_str().unwrap()],
        0,
    );
    assert_eq!(v["result"]["comparability"], "CONDITIONALLY_COMPARABLE");
    assert!(v["result"]["ranking"].is_null());
    m.proof_obligations[0].goal = MathExpr::eq(
        MathExpr::Constant(Constant::Natural(1)),
        MathExpr::Constant(Constant::Natural(2)),
    );
    write(&other, &m);
    assert_eq!(
        run(
            &["compare", path.to_str().unwrap(), other.to_str().unwrap()],
            0
        )["result"]["comparability"],
        "NOT_COMPARABLE"
    );
    m.proof_obligations[0].goal = MathExpr::var("unbound");
    write(&other, &m);
    run(&["model", "check", other.to_str().unwrap()], 3);
    let mut v = json!(m);
    v["surprise"] = json!(true);
    write(&other, &v);
    run(&["model", "check", other.to_str().unwrap()], 2);
    fs::write(&other, "not json").unwrap();
    run(&["model", "check", other.to_str().unwrap()], 2);
}
#[test]
fn plan_and_candidates_never_prove() {
    let p = temp();
    let path = p.join("obligation.json");
    let mut o = fixtures::usm().remove(0).0;
    o.context = Context::default();
    o.assumptions.clear();
    o.goal = MathExpr::forall(
        "x",
        MathType::Bool,
        MathExpr::eq(MathExpr::var("x"), MathExpr::Constant(Constant::Bool(true))),
    );
    write(&path, &o);
    let v = run(&["proof", "plan", path.to_str().unwrap()], 0);
    assert_eq!(v["formal_status"], "UNKNOWN");
    assert!(!v["result"]["nodes"].as_array().unwrap().is_empty());
    let v = run(&["proof", "counterexample", path.to_str().unwrap()], 0);
    assert_eq!(v["formal_status"], "UNKNOWN");
    assert_eq!(v["result"]["candidates"].as_array().unwrap().len(), 1);
    o.goal = MathExpr::forall(
        "x",
        MathType::Finite(1001),
        MathExpr::eq(MathExpr::var("x"), MathExpr::var("x")),
    );
    write(&path, &o);
    run(&["proof", "counterexample", path.to_str().unwrap()], 6);
    o.goal = MathExpr::forall(
        "x",
        MathType::Nat,
        MathExpr::eq(MathExpr::var("x"), MathExpr::var("x")),
    );
    write(&path, &o);
    run(&["proof", "counterexample", path.to_str().unwrap()], 4);
}
#[test]
fn evidence_integrity_and_stale_revision() {
    let p = temp();
    let o = fixtures::usm().remove(0).0;
    let input = p.join("proof_obligations.json");
    write(&input, &vec![o.clone()]);
    let proof = VerificationEvidence::unresolved(&o, FormalStatus::Unknown, "unproved").unwrap();
    let dir = proof.persist(&p.join("registry"), &o, None, None).unwrap();
    let v = run(&["evidence", "validate", dir.to_str().unwrap()], 0);
    assert_eq!(v["formal_status"], "UNKNOWN");
    assert!(v["result"]["revision_current"].is_null());
    let mut r = ModelRegistry::default();
    let mut m = fixtures::model(&[o]);
    r.register(m.clone()).unwrap();
    m.revision = 2;
    for o in &mut m.proof_obligations {
        o.model_revision = 2;
    }
    r.register(m).unwrap();
    let reg = p.join("registry.json");
    write(&reg, &r);
    run(
        &[
            "evidence",
            "validate",
            dir.to_str().unwrap(),
            "--registry",
            reg.to_str().unwrap(),
        ],
        7,
    );
    run(
        &[
            "knowledge",
            "impact",
            reg.to_str().unwrap(),
            "--revision",
            "1",
        ],
        0,
    );
    let mut knowledge = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let mut knowledge_registry = ModelRegistry::default();
    knowledge_registry.import_knowledge(&knowledge);
    let id = knowledge.units().next().unwrap().knowledge_id.clone();
    let mut target = fixtures::usm().remove(0).0;
    target.dependencies = vec![knowledge_registry.snapshot(&id).unwrap()];
    let mut dependent = fixtures::model(&[target.clone()]);
    dependent.dependencies = target.dependencies.clone();
    knowledge_registry.register(dependent).unwrap();
    let evidence =
        VerificationEvidence::unresolved(&target, FormalStatus::Unknown, "pending").unwrap();
    let kroot = p.join("knowledge-target");
    fs::create_dir(&kroot).unwrap();
    write(&kroot.join("proof_obligations.json"), &target);
    let kdir = evidence
        .persist(&kroot.join("registry"), &target, None, None)
        .unwrap();
    let mut units: Vec<_> = knowledge.units().cloned().collect();
    units
        .iter_mut()
        .find(|u| u.knowledge_id == id)
        .unwrap()
        .priority += 1;
    knowledge = mathlang_mirp::knowledge::KnowledgeSpace::empty();
    knowledge.register_batch(units).unwrap();
    knowledge_registry.import_knowledge(&knowledge);
    let kr = p.join("knowledge-registry.json");
    write(&kr, &knowledge_registry);
    run(
        &[
            "evidence",
            "validate",
            kdir.to_str().unwrap(),
            "--registry",
            kr.to_str().unwrap(),
        ],
        7,
    );
    let impact = run(
        &[
            "knowledge",
            "impact",
            kr.to_str().unwrap(),
            "--revision",
            "1",
        ],
        0,
    );
    assert!(!impact["result"]["models"][0]["stale_targets"]
        .as_array()
        .unwrap()
        .is_empty());
    let path = dir.join("evidence.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    v["proof_result"] = json!("PROVED");
    write(&path, &v);
    run(&["evidence", "validate", dir.to_str().unwrap()], 7);
    run(
        &["evidence", "show", p.join("missing").to_str().unwrap()],
        7,
    );
}
#[test]
fn config_and_paths() {
    let p = temp();
    let config = p.join("config.toml");
    fs::write(&config, "[verification]\nmax_rss_mib = 4096\n").unwrap();
    run(&["doctor", "--config", config.to_str().unwrap()], 4);
    fs::write(&config, "[cli]\noutput_format = 'nonsense'\n").unwrap();
    run(&["doctor", "--config", config.to_str().unwrap()], 2);
    fs::write(&config, "[runtime]\nmode = 'cloud'\n").unwrap();
    run(&["doctor", "--config", config.to_str().unwrap()], 4);
    run(&["knowledge", "update", "anything.json"], 4);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&p, p.join("link")).unwrap();
        assert!(mathlang_cli::safe_path(&p.join("link/out")).is_err());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let directory = p.join("readonly");
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o500)).unwrap();
        let denied = directory.join("result.json");
        run(
            &[
                "knowledge",
                "update",
                "unused.json",
                "--output",
                denied.to_str().unwrap(),
            ],
            8,
        );
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
    }
    assert!(mathlang_cli::safe_path(&p.join("../outside")).is_err());
}
