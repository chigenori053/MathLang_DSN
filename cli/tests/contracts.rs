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
fn exact_calculations_use_knowledge_and_preserve_error_contracts() {
    let directory = temp();
    let input = directory.join("request.json");
    for (request, expected) in [
        (
            json!({"operation":"SIMPLIFY_SQRT","radicand":72}),
            json!({"kind":"RADICAL","coefficient":6,"radicand":2}),
        ),
        (
            json!({"operation":"DESCRIBE_DATA","values":[3,1,2]}),
            json!({"kind":"SUMMARY","count":3,"mean":{"numerator":2,"denominator":1},"median":{"numerator":2,"denominator":1},"lower_quartile":{"numerator":1,"denominator":1},"upper_quartile":{"numerator":3,"denominator":1},"range":2}),
        ),
        (
            json!({"operation":"DATA_VARIANCE","values":[1,2,3]}),
            json!({"kind":"VARIANCE","count":3,"mean":{"numerator":2,"denominator":1},"population_variance":{"numerator":2,"denominator":3},"sample_variance":{"numerator":1,"denominator":1}}),
        ),
    ] {
        write(&input, &request);
        let response = run(&["compute", input.to_str().unwrap()], 0);
        assert_eq!(response["result"]["answer"], expected);
        assert_eq!(response["result"]["verification"]["status"], "VERIFIED");
        assert_eq!(
            response["result"]["verification"]["formal_status"],
            "NOT_CHECKED"
        );
        assert!(response["result"]["explanation"].as_array().unwrap().len() >= 3);
        assert!(response["formal_status"].is_null());
    }
    for request in [
        json!({"operation":"SIMPLIFY_SQRT","radicand":-1}),
        json!({"operation":"DESCRIBE_DATA","values":[]}),
    ] {
        write(&input, &request);
        let response = run(&["compute", input.to_str().unwrap()], 4);
        assert!(response["result"].get("answer").is_none());
    }
    write(
        &input,
        &json!({"operation":"SIMPLIFY_SQRT","radicand":72,"source":"injected"}),
    );
    run(&["compute", input.to_str().unwrap()], 2);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rational_statistics_and_radical_operations_have_cli_evidence() {
    let fixtures = std::path::Path::new(mathlang_cli::ROOT).join("mirp/fixtures/calculation");
    for (file, id, kind) in [
        ("covariance.json", "EXACT_DATA_COVARIANCE", "COVARIANCE"),
        ("weights.json", "EXACT_WEIGHT_NORMALIZATION", "WEIGHTS"),
        (
            "expectation.json",
            "EXACT_DISCRETE_EXPECTATION",
            "EXPECTATION",
        ),
        (
            "exact_expression.json",
            "EXACT_RADICAL_NORMAL_FORM",
            "EXACT_EXPRESSION",
        ),
        (
            "exact_equal.json",
            "EXACT_RADICAL_EQUALITY",
            "EXACT_COMPARISON",
        ),
        (
            "exact_compare.json",
            "EXACT_RADICAL_COMPARISON",
            "EXACT_COMPARISON",
        ),
    ] {
        let path = fixtures.join(file);
        let response = run(&["compute", path.to_str().unwrap()], 0);
        let result = &response["result"];
        assert_eq!(result["trace"]["knowledge_id"], id);
        assert_eq!(
            result["knowledge_decision"]["activation"]["knowledge_id"],
            id
        );
        assert_eq!(result["answer"]["kind"], kind);
        assert_eq!(result["verification"]["status"], "VERIFIED");
        assert_eq!(result["verification"]["formal_status"], "NOT_CHECKED");
        assert!(result["explanation"][0].as_str().unwrap().contains(id));
        assert!(response["formal_status"].is_null());
    }
    let path = fixtures.join("comparison_limit.json");
    let limited = run(&["compute", path.to_str().unwrap()], 6);
    assert!(limited["result"].get("answer").is_none());
    assert!(limited["result"]["diagnostic"]
        .as_str()
        .unwrap()
        .starts_with("RESOURCE_LIMIT"));
    let directory = temp();
    let path = directory.join("invalid.json");
    for request in [
        json!({"operation":"DATA_COVARIANCE","x":[],"y":[]}),
        json!({"operation":"NORMALIZE_WEIGHTS","weights":[{"numerator":-1,"denominator":1}]}),
        json!({"operation":"DISCRETE_EXPECTATION","values":[{"numerator":1,"denominator":1}],"weights":[{"numerator":2,"denominator":1}]}),
        json!({"operation":"NORMALIZE_EXACT","expression":{"terms":[{"coefficient":{"numerator":1,"denominator":1},"radicand":-1}]}}),
    ] {
        write(&path, &request);
        run(&["compute", path.to_str().unwrap()], 4);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn exact_plane_geometry_has_cli_results_degeneracy_and_evidence() {
    let fixtures = std::path::Path::new(mathlang_cli::ROOT).join("mirp/fixtures/calculation");
    for (file, id, kind) in [
        (
            "plane_distance.json",
            "GEOMETRY_PLANE_DISTANCE",
            "PLANE_DISTANCE",
        ),
        (
            "plane_dot.json",
            "GEOMETRY_PLANE_DOT_PRODUCT",
            "PLANE_DOT_PRODUCT",
        ),
        (
            "plane_orthogonal.json",
            "GEOMETRY_PLANE_ORTHOGONAL",
            "PLANE_ORTHOGONALITY",
        ),
        (
            "triangle_area.json",
            "GEOMETRY_TRIANGLE_AREA",
            "TRIANGLE_AREA",
        ),
        (
            "line_line.json",
            "GEOMETRY_LINE_LINE_INTERSECTIONS",
            "GEOMETRY_INTERSECTIONS",
        ),
        (
            "line_circle.json",
            "GEOMETRY_LINE_CIRCLE_INTERSECTIONS",
            "GEOMETRY_INTERSECTIONS",
        ),
        (
            "circle_circle.json",
            "GEOMETRY_CIRCLE_CIRCLE_INTERSECTIONS",
            "GEOMETRY_INTERSECTIONS",
        ),
    ] {
        let path = fixtures.join(file);
        let response = run(&["compute", path.to_str().unwrap()], 0);
        let result = &response["result"];
        assert_eq!(result["trace"]["knowledge_id"], id);
        assert_eq!(
            result["knowledge_decision"]["activation"]["knowledge_id"],
            id
        );
        assert_eq!(result["answer"]["kind"], kind);
        assert_eq!(result["verification"]["status"], "VERIFIED");
        assert_eq!(result["verification"]["formal_status"], "NOT_CHECKED");
        assert!(result["explanation"][0].as_str().unwrap().contains(id));
        assert!(response["formal_status"].is_null());
    }
    let path = fixtures.join("coincident_circles.json");
    let infinite = run(&["compute", path.to_str().unwrap()], 0);
    assert_eq!(
        infinite["result"]["answer"]["intersection"]["infinite"],
        true
    );
    assert_eq!(
        infinite["result"]["answer"]["intersection"]["relation"],
        "COINCIDENT"
    );
    assert!(infinite["result"]["explanation"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("交点は無限個")));
    let path = fixtures.join("point_circles.json");
    let point = run(&["compute", path.to_str().unwrap()], 0);
    assert_eq!(point["result"]["answer"]["intersection"]["infinite"], false);
    assert_eq!(
        point["result"]["answer"]["intersection"]["points"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let directory = temp();
    let path = directory.join("invalid.json");
    for request in [
        serde_json::json!({"operation":"PLANE_DISTANCE","a":[],"b":[]}),
        serde_json::json!({"operation":"LINE_LINE_INTERSECTIONS","first":{"a":{"numerator":0,"denominator":1},"b":{"numerator":0,"denominator":1},"c":{"numerator":0,"denominator":1}},"second":{"a":{"numerator":1,"denominator":1},"b":{"numerator":0,"denominator":1},"c":{"numerator":0,"denominator":1}}}),
    ] {
        write(&path, &request);
        run(&["compute", path.to_str().unwrap()], 4);
    }
    fs::remove_dir_all(directory).unwrap();
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

#[test]
fn classical_physics_cli_preserves_units_context_and_conditional_evidence() {
    let fixtures = std::path::Path::new(mathlang_cli::ROOT).join("mirp/fixtures/calculation");
    for op in [
        "constant_acceleration",
        "newton_force",
        "newton_acceleration",
        "constant_force_work",
        "kinetic_energy",
        "work_energy",
    ] {
        let file = fixtures.join(format!("physics_{op}.json"));
        let response = run(&["compute", file.to_str().unwrap()], 0);
        assert_eq!(
            response["result"]["trace"]["knowledge_id"],
            format!("PHYSICS_{}", op.to_uppercase())
        );
        assert_eq!(response["result"]["answer"]["kind"], "PHYSICS");
        assert_eq!(
            response["result"]["answer"]["result"]["context"]["frame"]["id"],
            "lab"
        );
        assert_eq!(
            response["result"]["answer"]["result"]["applicability"],
            "CONDITIONAL_ON_DECLARED_ASSUMPTIONS"
        );
        assert_eq!(response["result"]["verification"]["status"], "VERIFIED");
        assert_eq!(
            response["result"]["verification"]["formal_status"],
            "NOT_CHECKED"
        );
    }
    let dir = temp();
    let path = dir.join("request.json");
    let base: Value =
        serde_json::from_slice(&fs::read(fixtures.join("physics_work_energy.json")).unwrap())
            .unwrap();
    let mut v = base.clone();
    v["net_work"]["value"]["numerator"] = json!(71);
    write(&path, &v);
    let response = run(&["compute", path.to_str().unwrap()], 0);
    assert_eq!(
        response["result"]["answer"]["result"]["outcome"]["consistent"],
        false
    );
    assert_eq!(
        response["result"]["answer"]["result"]["outcome"]["residual"]["value"]["numerator"],
        "-1"
    );
    for v in [
        {
            let mut v = base.clone();
            v["net_work"]["unit"] = json!("N");
            v
        },
        {
            let mut v = base.clone();
            v["final_velocity"]["frame_id"] = json!("other");
            v
        },
        {
            let mut v = base.clone();
            v["context"]["assumptions"] = json!([]);
            v
        },
        {
            let mut v = base.clone();
            v["context"]["frame"]["kind"] = json!("ROTATING");
            v
        },
        {
            let mut v = base.clone();
            v["context"]["model"] = json!("RELATIVISTIC_PARTICLE");
            v
        },
    ] {
        write(&path, &v);
        run(&["compute", path.to_str().unwrap()], 4);
    }
    let mut v = base;
    v["context"]["injected"] = json!(true);
    write(&path, &v);
    run(&["compute", path.to_str().unwrap()], 2);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn finite_quantum_cli_preserves_exact_complex_states_and_error_contracts() {
    let fixtures = std::path::Path::new(mathlang_cli::ROOT).join("mirp/fixtures/calculation");
    for op in [
        "normalize",
        "inner_product",
        "born",
        "expectation",
        "density_matrix",
        "unitary",
    ] {
        let file = fixtures.join(format!("quantum_{op}.json"));
        let response = run(&["compute", file.to_str().unwrap()], 0);
        assert_eq!(
            response["result"]["trace"]["knowledge_id"],
            format!("QUANTUM_{}", op.to_uppercase())
        );
        assert_eq!(response["result"]["answer"]["kind"], "QUANTUM");
        assert_eq!(
            response["result"]["answer"]["result"]["basis"]["id"],
            "qubit_z"
        );
        assert_eq!(response["result"]["verification"]["status"], "VERIFIED");
        assert_eq!(
            response["result"]["verification"]["method"],
            "EXACT_COMPLEX_IDENTITY_CHECK"
        );
        assert_eq!(
            response["result"]["verification"]["formal_status"],
            "NOT_CHECKED"
        );
    }
    let dir = temp();
    let path = dir.join("request.json");
    let base: Value =
        serde_json::from_slice(&fs::read(fixtures.join("quantum_born.json")).unwrap()).unwrap();
    let normalize = run(
        &[
            "compute",
            fixtures.join("quantum_normalize.json").to_str().unwrap(),
        ],
        0,
    );
    let mut v = base.clone();
    v["state"] = normalize["result"]["answer"]["result"]["outcome"]["state"].clone();
    v["state"]["kind"] = json!("PURE");
    write(&path, &v);
    let response = run(&["compute", path.to_str().unwrap()], 0);
    assert_eq!(
        response["result"]["answer"]["result"]["outcome"]["probabilities"],
        json!([{"numerator":"1","denominator":"2"},{"numerator":"1","denominator":"2"}])
    );
    for v in [
        {
            let mut v = base.clone();
            v["state"]["squared_scale"] = json!({"numerator":1,"denominator":1});
            v
        },
        {
            let mut v = base.clone();
            v["state"]["basis_id"] = json!("other");
            v
        },
        {
            let mut v = base.clone();
            v["projectors"].as_array_mut().unwrap().pop();
            v
        },
        {
            let mut v = base.clone();
            v["state"]["amplitudes"] = json!([]);
            v
        },
        {
            let mut v = base.clone();
            v["state"]["amplitudes"][0]["real"]["denominator"] = json!(0);
            v
        },
    ] {
        write(&path, &v);
        run(&["compute", path.to_str().unwrap()], 4);
    }
    let mut v = base;
    v["state"]["unexamined"] = json!(true);
    write(&path, &v);
    run(&["compute", path.to_str().unwrap()], 2);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qrm_quantum_extension_runs_through_cli_including_requests_larger_than_argv_limit() {
    let fixtures = std::path::Path::new(mathlang_cli::ROOT).join("mirp/fixtures/calculation");
    for op in ["mix", "effect_score", "charpoly"] {
        let file = fixtures.join(format!("quantum_{op}.json"));
        let response = run(&["compute", file.to_str().unwrap()], 0);
        assert_eq!(
            response["result"]["trace"]["knowledge_id"],
            format!("QUANTUM_{}", op.to_uppercase())
        );
        assert_eq!(response["result"]["verification"]["status"], "VERIFIED");
    }
    // 32-dimensional maximally mixed state with zero entries written as 0/10^40:
    // the JSON request exceeds Linux's 128 KiB single-argument limit.
    let n = 32;
    let big = format!("1{}", "0".repeat(40));
    let entries: Vec<Vec<Value>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    let (num, den) = if i == j {
                        ("1".to_string(), "32".to_string())
                    } else {
                        ("0".to_string(), big.clone())
                    };
                    json!({"real":{"numerator":num,"denominator":den},
                           "imaginary":{"numerator":"0","denominator":big}})
                })
                .collect()
        })
        .collect();
    let request = json!({"operation":"QUANTUM_DENSITY_MATRIX",
        "basis":{"id":"b","labels":(0..n).map(|i| format!("e{i}")).collect::<Vec<_>>()},
        "state":{"kind":"MIXED","basis_id":"b","entries":entries}});
    let body = serde_json::to_vec(&request).unwrap();
    assert!(body.len() > 131_072, "request must exceed argv limit");
    let directory = temp();
    let input = directory.join("large.json");
    fs::write(&input, body).unwrap();
    let response = run(&["compute", input.to_str().unwrap()], 0);
    assert_eq!(
        response["result"]["answer"]["result"]["outcome"]["purity"],
        json!({"numerator":"1","denominator":"32"})
    );
}
