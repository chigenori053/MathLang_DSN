use mathlang_verification::{counterexample::*, evidence::*, ir::*, lean::*, proof::*, *};
use std::{collections::BTreeMap, path::PathBuf};
fn bx(e: MathExpr) -> Box<MathExpr> {
    Box::new(e)
}
fn var(n: &str) -> MathExpr {
    MathExpr::var(n)
}
fn nat(n: u64) -> MathExpr {
    MathExpr::Constant(Constant::Natural(n))
}
fn real_one() -> MathExpr {
    MathExpr::Constant(Constant::One(MathType::Real))
}
fn fixed(n: u32) -> Dimension {
    Dimension::Fixed(n)
}
fn vector(n: u32) -> MathType {
    MathType::Vector(Box::new(MathType::Complex), fixed(n))
}
fn matrix(n: u32, m: u32) -> MathType {
    MathType::Matrix(Box::new(MathType::Complex), fixed(n), fixed(m))
}
fn context(vars: &[(&str, MathType)]) -> Context {
    Context {
        variables: vars
            .iter()
            .map(|(n, t)| (n.to_string(), t.clone()))
            .collect(),
        assumptions: vec![],
    }
}
fn simple(id: &str, goal: MathExpr, ctx: Context) -> ProofObligation {
    ProofObligation {
        id: id.into(),
        model_id: "test".into(),
        model_revision: 1,
        goal,
        context: ctx,
        assumptions: vec![],
        required_lemmas: vec![],
        dependencies: vec![],
        kind: ProofKind::Construction,
        verification_method: "Lean".into(),
        proof_status: FormalStatus::Unknown,
        evidence_refs: vec![],
    }
}
#[test]
fn ir_01_complex() {
    let c = context(&[("z", MathType::Complex)]);
    assert_eq!(
        c.infer(&MathExpr::Conjugate(bx(var("z")))).unwrap(),
        MathType::Complex
    );
    assert!(Context::default()
        .infer(&MathExpr::Add(
            bx(real_one()),
            bx(MathExpr::Constant(Constant::One(MathType::Complex)))
        ))
        .is_err());
}
#[test]
fn ir_02_vector() {
    let c = context(&[("x", vector(2)), ("y", vector(3))]);
    assert!(c.infer(&MathExpr::Add(bx(var("x")), bx(var("y")))).is_err());
    assert_eq!(
        c.infer(&MathExpr::Add(bx(var("x")), bx(var("x")))).unwrap(),
        vector(2)
    );
}
#[test]
fn ir_03_matrix() {
    let c = context(&[("a", matrix(2, 3)), ("b", matrix(3, 4))]);
    assert_eq!(
        c.infer(&MathExpr::MatrixMultiply(bx(var("a")), bx(var("b"))))
            .unwrap(),
        matrix(2, 4)
    );
    assert!(c
        .infer(&MathExpr::MatrixMultiply(bx(var("b")), bx(var("a"))))
        .is_err());
}
#[test]
fn ir_04_inner_product() {
    let t = MathType::HilbertSpace(fixed(3));
    let c = context(&[("x", t.clone()), ("y", t)]);
    assert_eq!(
        c.infer(&MathExpr::InnerProduct(bx(var("x")), bx(var("y"))))
            .unwrap(),
        MathType::Complex
    );
}
#[test]
fn ir_05_adjoint() {
    let c = context(&[("a", matrix(2, 3))]);
    assert_eq!(
        c.infer(&MathExpr::Adjoint(bx(var("a")))).unwrap(),
        matrix(3, 2)
    );
}
#[test]
fn ir_06_tensor() {
    let c = context(&[("x", vector(2)), ("y", vector(3))]);
    let p = MathExpr::TensorProduct(bx(var("x")), bx(var("y")));
    assert_eq!(
        c.infer(&p).unwrap(),
        MathType::Tensor(Box::new(MathType::Complex), vec![fixed(2), fixed(3)])
    );
    assert_eq!(
        c.infer(&MathExpr::TensorIndex(bx(p.clone()), vec![1, 2]))
            .unwrap(),
        MathType::Complex
    );
    assert!(c.infer(&MathExpr::TensorIndex(bx(p), vec![2, 0])).is_err());
}
#[test]
fn ir_07_propositions() {
    for (o, _, _) in fixtures::usm() {
        assert!(o.context.check(&o.goal).valid, "{}", o.id);
    }
    let q = MathExpr::forall("x", MathType::Nat, MathExpr::eq(var("x"), var("x")));
    assert!(Context::default().check(&q).valid);
    assert!(!Context::default().check(&var("undefined")).valid);
}
#[test]
fn ir_08_nonzero_premises() {
    let mut c = context(&[("x", MathType::Real)]);
    let d = MathExpr::Divide(bx(real_one()), bx(var("x")));
    assert!(c.infer(&d).is_err());
    let a = MathExpr::LogicalNot(bx(MathExpr::eq(
        var("x"),
        MathExpr::Constant(Constant::Zero(MathType::Real)),
    )));
    c.assumptions.push(a.clone());
    assert_eq!(c.infer(&d).unwrap(), MathType::Real);
    assert!(
        context(&[("x", MathType::Real)])
            .check(&MathExpr::implies(a, MathExpr::eq(d.clone(), d)))
            .valid
    );
}
#[test]
fn ir_09_reject_mismatches() {
    let h2 = MathType::HilbertSpace(fixed(2));
    let h3 = MathType::HilbertSpace(fixed(3));
    let c = context(&[
        (
            "a",
            MathType::LinearOperator(Box::new(h2.clone()), Box::new(h2)),
        ),
        (
            "b",
            MathType::LinearOperator(Box::new(h3.clone()), Box::new(h3)),
        ),
    ]);
    assert!(c
        .infer(&MathExpr::MatrixMultiply(bx(var("a")), bx(var("b"))))
        .is_err());
    assert!(Context::default()
        .infer(&MathExpr::apply(var("unknown_function"), nat(1)))
        .is_err());
    assert!(
        !Context::default()
            .check(&MathExpr::forall(
                "x",
                MathType::HilbertSpace(Dimension::Symbol("n".into())),
                MathExpr::eq(var("x"), var("x"))
            ))
            .valid
    );
}
#[test]
fn ir_10_normalization() {
    let a = MathExpr::Constant(Constant::Rational(2, 4));
    let b = MathExpr::Constant(Constant::Rational(1, 2));
    assert_eq!(normalized(&a).unwrap(), normalized(&b).unwrap());
    for _ in 0..3 {
        assert_eq!(normalized(&a).unwrap(), normalized(&a).unwrap())
    }
    let z = MathExpr::Constant(Constant::Rational(0, 4));
    assert!(normalized(&z).unwrap().contains("[0,1]"));
    assert!(normalized(&MathExpr::Constant(Constant::Rational(1, 0))).is_err());
}
#[test]
fn lv_01_03_preserves_types_quantifiers_premises() {
    let (o, _, _) = fixtures::usm().remove(0);
    let t = translate(&o, Recipe::UnitaryNorm, false).unwrap();
    assert!(t.source.contains("∀ (v_n : ℕ)"));
    assert!(t.source.contains("EuclideanSpace ℂ"));
    assert!(t.source.contains("adjoint.comp v_U = 1"));
    assert!(t.source.contains("→ (∀ (v_x"));
    assert!(t.source.contains("Independent.mv01"));
    assert_eq!(translate_type(&MathType::Complex).unwrap(), "ℂ");
}
#[test]
fn lv_08_revision_and_mutation_rejected() {
    let (mut o, r, n) = fixtures::usm().remove(0);
    let t = translate(&o, r, n).unwrap();
    o.model_revision += 1;
    assert!(audit_translation(&o, &t).is_err());
    o.model_revision -= 1;
    let mut changed = t.clone();
    changed.source.push_str("\naxiom poison : False\n");
    assert!(audit_translation(&o, &changed).is_err());
    let mut changed = t.clone();
    changed.record.assumptions.push(Assumption {
        proposition: MathExpr::eq(nat(1), nat(2)),
        origin: AssumptionOrigin::Unproved,
    });
    assert!(audit_translation(&o, &changed).is_err());
}
#[test]
fn lv_07_axiom_and_result_audit() {
    let (o, r, n) = fixtures::usm().remove(0);
    let t = translate(&o, r, n).unwrap();
    let mut k = KernelRun::infrastructure_error("test");
    k.sandboxed = true;
    k.checked_modules = vec![
        "Foundation".into(),
        "Independent".into(),
        "Generated".into(),
    ];
    k.toolchain_verified = true;
    k.source_hash = t.record.lean_module_hash.clone();
    k.mathlib_revision = MATHLIB_REVISION.into();
    k.lean_version = "Lean (version 4.35.0, test)".into();
    k.exit_code = Some(0);
    k.stdout =
        "'Generated.target' depends on axioms: [propext, Classical.choice, Quot.sound]".into();
    k.axioms = vec![
        "propext".into(),
        "Classical.choice".into(),
        "Quot.sound".into(),
    ];
    assert_eq!(assess_kernel(&t, &k), FormalStatus::Proved);
    for ax in ["sorryAx", "poison", "Lean.ofReduceBool"] {
        k.stdout = format!("'Generated.target' depends on axioms: [{ax}]");
        k.axioms = vec![ax.into()];
        assert_eq!(assess_kernel(&t, &k), FormalStatus::Error)
    }
    k.exit_code = Some(1);
    assert_eq!(assess_kernel(&t, &k), FormalStatus::Unknown);
    k.timed_out = true;
    assert_eq!(assess_kernel(&t, &k), FormalStatus::Unknown);
}
#[test]
fn lv_09_full_contract_and_coordinate_tensor_mapping() {
    let (o, r, n) = fixtures::usm().remove(3);
    let t = translate(&o, r, n).unwrap();
    assert!(t.source.contains("Independent.mv04"));
    assert!(t.source.contains("∀ (v_A : Type)"));
    let c = context(&[("x", vector(2)), ("y", vector(2))]);
    let p = MathExpr::TensorProduct(bx(var("x")), bx(var("y")));
    let o = simple("unsupported", MathExpr::eq(p.clone(), p), c);
    let t = translate(&o, Recipe::Reflexivity, false).unwrap();
    assert!(t
        .source
        .contains("fun (tensor_i_0 : Fin 2) (tensor_i_1 : Fin 2)"));
}
#[test]
fn dp_01_03_06_07_10_structural_planning() {
    let k = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let (o, _, _) = fixtures::usm().remove(0);
    let p = dsn::plan(&o, &k, 1000).unwrap();
    assert_eq!(p.nodes[0].rule, "forall-introduction");
    assert!(p.nodes.iter().any(|n| n.rule == "implication-introduction"));
    assert!(p.nodes.last().unwrap().assumptions.len() == 1);
    assert!(p
        .nodes
        .iter()
        .all(|n| n.formal_status == FormalStatus::Unknown));
    assert!(p
        .nodes
        .iter()
        .any(|n| n.reasons.contains(&UnknownReason::MissingPremise)));
    for _ in 0..3 {
        assert_eq!(p, dsn::plan(&o, &k, 1000).unwrap())
    }
    assert!(dsn::plan(&o, &k, 1).is_err());
}
#[test]
fn dp_02_conjunction_decomposition() {
    let p = MathExpr::eq(nat(1), nat(1));
    let o = simple("split", MathExpr::and(p.clone(), p), Context::default());
    let plan = dsn::plan(&o, &mathlang_mirp::knowledge::KnowledgeSpace::empty(), 8).unwrap();
    assert_eq!(plan.nodes[0].children.len(), 2);
}
#[test]
fn dp_04_08_dependency_cycles_and_missing() {
    let good = BTreeMap::from([("a".into(), vec!["b".into()]), ("b".into(), vec![])]);
    assert!(validate_graph(&good).is_ok());
    let mut bad = good.clone();
    bad.get_mut("b").unwrap().push("a".into());
    assert!(validate_graph(&bad).is_err());
    bad.remove("b");
    assert!(validate_graph(&bad).is_err());
}
#[test]
fn dp_05_knowledge_candidates_are_not_proofs() {
    let mut k = mathlang_mirp::knowledge::KnowledgeSpace::empty();
    let mut u = mathlang_mirp::knowledge::KnowledgeSpace::legacy()
        .units()
        .find(|u| u.dependencies.is_empty())
        .unwrap()
        .clone();
    let goal = MathExpr::eq(nat(1), nat(1));
    u.conclusion = normalized(&goal).unwrap();
    let id = u.knowledge_id.clone();
    k.register_batch(vec![u]).unwrap();
    let o = simple("lookup", goal, Context::default());
    let plan = dsn::plan(&o, &k, 8).unwrap();
    assert_eq!(plan.nodes[0].source_knowledge[0].id, id);
    assert_eq!(plan.nodes[0].formal_status, FormalStatus::Unknown);
}
#[test]
fn cv_01_finite_search() {
    let goal = MathExpr::forall(
        "x",
        MathType::Bool,
        MathExpr::eq(var("x"), MathExpr::Constant(Constant::Bool(true))),
    );
    let o = simple("finite", goal, Context::default());
    let cs = search_finite(&o, 8).unwrap();
    assert_eq!(cs.len(), 1);
    assert!(cs[0].assumptions_satisfied && cs[0].conclusion_false);
    assert_eq!(cs[0].kernel_evidence, None);
    assert!(search_finite(&o, 1).is_err());
}
#[test]
fn cv_02_exact_algebra_and_overflow() {
    let ctx = context(&[("x", MathType::Rational)]);
    let two = MathExpr::Constant(Constant::Rational(2, 1));
    let g = MathExpr::eq(MathExpr::Multiply(bx(two), bx(var("x"))), var("x"));
    let o = simple("algebraic", g, ctx);
    let w = BTreeMap::from([("x".into(), ExactValue::Number(Rational::new(1, 1).unwrap()))]);
    let c = check_candidate(&o, w, CounterexampleKind::Algebraic).unwrap();
    assert!(c.conclusion_false);
    let max = MathExpr::Constant(Constant::Integer(i64::MAX));
    let square = MathExpr::Multiply(bx(max.clone()), bx(max.clone()));
    assert!(evaluate(&MathExpr::Multiply(bx(square), bx(max)), &BTreeMap::new()).is_err());
}
#[test]
fn cv_03_constraints_and_quantified_domain() {
    let ctx = context(&[("x", MathType::Nat)]);
    let a = MathExpr::LogicalNot(bx(MathExpr::eq(var("x"), nat(0))));
    let goal = MathExpr::implies(a, MathExpr::eq(var("x"), nat(99)));
    let o = simple("constraint", goal, ctx);
    let w = BTreeMap::from([("x".into(), ExactValue::Number(Rational::new(0, 1).unwrap()))]);
    let c = check_candidate(&o, w, CounterexampleKind::Algebraic).unwrap();
    assert_eq!(c.kind, CounterexampleKind::ConstraintViolation);
    assert!(!c.assumptions_satisfied);
    let outside = BTreeMap::from([(
        "x".into(),
        ExactValue::Number(Rational::new(-1, 1).unwrap()),
    )]);
    assert!(check_candidate(&o, outside, CounterexampleKind::Algebraic).is_err());
}
#[test]
fn cv_04_proof_failure_is_not_refutation() {
    let o = simple("unknown", MathExpr::eq(nat(1), nat(2)), Context::default());
    let e = VerificationEvidence::unresolved(&o, FormalStatus::Unknown, "search failure").unwrap();
    assert_eq!(e.result(), FormalStatus::Unknown);
    assert!(VerificationEvidence::unresolved(&o, FormalStatus::Refuted, "no proof").is_err());
}
fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("mldsn-{}-{}-{}", name, std::process::id(), now()))
}
#[test]
fn cv_05_07_model_and_transitive_revisions() {
    let mut o = simple("revision", MathExpr::eq(nat(1), nat(1)), Context::default());
    o.model_id = "base".into();
    let mut m = fixtures::model(&[o.clone()]);
    m.model_id = "base".into();
    let mut r = ModelRegistry::default();
    r.register(m.clone()).unwrap();
    let dep = r.snapshot("base").unwrap();
    let mut child = o.clone();
    child.model_id = "child".into();
    child.dependencies.push(dep.clone());
    let mut cm = m.clone();
    cm.model_id = "child".into();
    cm.dependencies = vec![dep];
    cm.proof_obligations = vec![child.clone()];
    r.register(cm).unwrap();
    let e = VerificationEvidence::unresolved(&child, FormalStatus::Unknown, "pending").unwrap();
    assert!(r.is_current(&e));
    m.revision = 2;
    m.proof_obligations[0].model_revision = 2;
    r.register(m).unwrap();
    assert!(!r.is_current(&e));
    assert_eq!(r.models()["base"].len(), 2);
}
#[test]
fn cv_05_knowledge_revision_invalidates_dependents() {
    let mut k = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let mut r = ModelRegistry::default();
    r.import_knowledge(&k);
    let id = k.units().next().unwrap().knowledge_id.clone();
    let mut o = simple(
        "knowledge",
        MathExpr::eq(nat(1), nat(1)),
        Context::default(),
    );
    o.dependencies = vec![r.snapshot(&id).unwrap()];
    let mut m = fixtures::model(&[o.clone()]);
    m.model_id = o.model_id.clone();
    m.dependencies = o.dependencies.clone();
    r.register(m).unwrap();
    let e = VerificationEvidence::unresolved(&o, FormalStatus::Unknown, "pending").unwrap();
    assert!(r.is_current(&e));
    let mut units: Vec<_> = k.units().cloned().collect();
    units
        .iter_mut()
        .find(|u| u.knowledge_id == id)
        .unwrap()
        .priority += 1;
    k = mathlang_mirp::knowledge::KnowledgeSpace::empty();
    k.register_batch(units).unwrap();
    r.import_knowledge(&k);
    assert!(!r.is_current(&e));
}
#[test]
fn cv_06_08_09_10_evidence_immutable_and_complete() {
    let o = simple("evidence", MathExpr::eq(nat(1), nat(1)), Context::default());
    let e =
        VerificationEvidence::unresolved(&o, FormalStatus::Unknown, "missing evidence").unwrap();
    let root = temp("evidence");
    let dir = e.persist(&root, &o, None, None).unwrap();
    let loaded = VerificationEvidence::load(&dir, &o).unwrap();
    assert_eq!(loaded.evidence_id(), e.evidence_id());
    assert!(e.persist(&root, &o, None, None).is_err());
    let mut changed = o.clone();
    changed.model_revision = 2;
    assert!(VerificationEvidence::load(&dir, &changed).is_err());
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("evidence.json")).unwrap()).unwrap();
    raw["proof_result"] = serde_json::json!("PROVED");
    std::fs::write(dir.join("evidence.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(VerificationEvidence::load(&dir, &o).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn missing_obligation_never_passes() {
    let o = simple("needed", MathExpr::eq(nat(1), nat(1)), Context::default());
    let mut m = fixtures::model(&[o]);
    m.model_id = "test".into();
    let mut r = ModelRegistry::default();
    r.register(m).unwrap();
    assert_eq!(r.evaluate("test", &[]), Gate::NotVerified);
}
#[test]
fn adversarial_identifiers_and_undefined_types() {
    assert!(!identifier("x\naxiom bad : False"));
    let c = context(&[("x", MathType::Predicate)]);
    assert!(!c.check(&MathExpr::eq(var("x"), var("x"))).valid);
    let o = simple(
        "inject",
        MathExpr::forall("x); #eval 1", MathType::Nat, MathExpr::eq(nat(1), nat(1))),
        Context::default(),
    );
    assert!(translate(&o, Recipe::Auto, false).is_err());
}
#[test]
fn dp_definition_expansion_keeps_original_goal() {
    let mut o = simple(
        "definition",
        MathExpr::eq(var("one"), nat(1)),
        context(&[("one", MathType::Nat)]),
    );
    let mut m = fixtures::model(&[o.clone()]);
    m.definitions.insert("one".into(), nat(1));
    let plans =
        dsn::plan_model(&m, &mathlang_mirp::knowledge::KnowledgeSpace::empty(), 10).unwrap();
    assert_eq!(plans[0].nodes[0].goal, o.goal);
    assert_eq!(plans[0].nodes[0].rule, "definition-expansion");
    assert!(plans[0]
        .nodes
        .iter()
        .all(|n| n.formal_status == FormalStatus::Unknown));
    o.goal = MathExpr::forall("x", MathType::Nat, MathExpr::eq(var("x"), var("x")));
    m.proof_obligations = vec![o];
    m.definitions.clear();
    assert!(dsn::plan_model(&m, &mathlang_mirp::knowledge::KnowledgeSpace::empty(), 10).is_ok());
}
#[test]
fn dp_required_lemma_edges_are_tracked() {
    let k = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let mut o = simple("lemma", MathExpr::eq(nat(1), nat(1)), Context::default());
    o.required_lemmas = vec![k.units().next().unwrap().knowledge_id.clone()];
    let p = dsn::plan(&o, &k, 100).unwrap();
    assert!(!p.lemma_graph.is_empty());
    assert!(p.nodes[0]
        .children
        .iter()
        .any(|e| e.starts_with("knowledge:")));
    assert!(validate_graph(&p.graph()).is_ok());
}
#[test]
fn false_or_external_assumptions_are_not_model_pass() {
    let mut o = simple(
        "conditional",
        MathExpr::eq(nat(1), nat(1)),
        Context::default(),
    );
    let premise = MathExpr::eq(nat(1), nat(2));
    o.context.assumptions = vec![premise.clone()];
    o.assumptions = vec![Assumption {
        proposition: premise,
        origin: AssumptionOrigin::False,
    }];
    assert!(o.validate().is_ok());
    let e = VerificationEvidence::unresolved(&o, FormalStatus::Unknown, "false premise").unwrap();
    let mut m = fixtures::model(&[o]);
    m.model_id = "test".into();
    let mut r = ModelRegistry::default();
    r.register(m).unwrap();
    assert_eq!(r.evaluate("test", &[e]), Gate::NotVerified);
}
#[test]
fn finite_search_honors_budget_before_allocating() {
    let o = simple(
        "budget",
        MathExpr::forall(
            "x",
            MathType::Finite(u32::MAX),
            MathExpr::eq(var("x"), var("x")),
        ),
        Context::default(),
    );
    assert!(search_finite(&o, 10).is_err());
}
#[test]
fn dp_09_rejects_evidence_for_another_plan() {
    let o = simple("original", MathExpr::eq(nat(1), nat(1)), Context::default());
    let other = simple("other", MathExpr::eq(nat(2), nat(2)), Context::default());
    let mut plan = dsn::plan(&o, &mathlang_mirp::knowledge::KnowledgeSpace::empty(), 10).unwrap();
    let e = VerificationEvidence::unresolved(&other, FormalStatus::Unknown, "unproved").unwrap();
    assert!(plan.apply_evidence(&other, &e).is_err());
    assert!(plan.nodes[0].evidence_refs.is_empty());
}
#[test]
fn infrastructure_launch_failure_is_saved_as_error() {
    let root = temp("launch-failure");
    std::fs::create_dir(&root).unwrap();
    let o = simple(
        "no-python",
        MathExpr::eq(nat(1), nat(1)),
        Context::default(),
    );
    let input = root.join("request.json");
    std::fs::write(
        &input,
        serde_json::to_vec(&serde_json::json!({"obligation":o,"recipe":"Auto","refutation":false}))
            .unwrap(),
    )
    .unwrap();
    let output = root.join("output");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_mathlang-verification"))
        .arg("verify-obligation")
        .arg(&input)
        .arg(&output)
        .env("PATH", root.join("no-tools"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let e: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("proof_results.json")).unwrap()).unwrap();
    assert_eq!(e["proof_result"], "ERROR");
    assert!(e["toolchain"]["diagnostics"][0]
        .as_str()
        .unwrap()
        .contains("runner launch"));
    let saved = output
        .join("registry")
        .join(e["evidence_id"].as_str().unwrap());
    VerificationEvidence::load(&saved, &o).unwrap();
    assert!(output.join("lean_logs/kernel.json").is_file());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn dp_11_descriptive_conclusions_are_not_formal_retrieval_keys() {
    let legacy = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    assert!(!legacy.is_empty());
    assert!(legacy
        .units()
        .all(|u| dsn::knowledge_retrieval_key(u).is_none()));
}
#[test]
fn dp_12_structural_alpha_equivalent_retrieval() {
    let mut u = mathlang_mirp::knowledge::KnowledgeSpace::legacy()
        .units()
        .find(|u| u.dependencies.is_empty())
        .unwrap()
        .clone();
    // Stored with a different binder name and pretty-printed JSON.
    let stored = MathExpr::forall("x", MathType::Real, MathExpr::eq(var("x"), var("x")));
    u.conclusion = serde_json::to_string_pretty(&stored).unwrap();
    let id = u.knowledge_id.clone();
    let mut k = mathlang_mirp::knowledge::KnowledgeSpace::empty();
    k.register_batch(vec![u]).unwrap();
    let goal = MathExpr::forall("y", MathType::Real, MathExpr::eq(var("y"), var("y")));
    let plan = dsn::plan(&simple("alpha", goal, Context::default()), &k, 8).unwrap();
    assert_eq!(plan.nodes[0].source_knowledge[0].id, id);
    assert_eq!(plan.nodes[0].formal_status, FormalStatus::Unknown);
    // Free variables are not renamed: a different free symbol is a different statement.
    let free_a = MathExpr::forall("x", MathType::Real, MathExpr::eq(var("x"), var("a")));
    let free_b = MathExpr::forall("x", MathType::Real, MathExpr::eq(var("x"), var("b")));
    assert_ne!(
        dsn::retrieval_key(&free_a).unwrap(),
        dsn::retrieval_key(&free_b).unwrap()
    );
    // Binder order matters.
    let xy = MathExpr::forall(
        "x",
        MathType::Real,
        MathExpr::forall("y", MathType::Nat, MathExpr::eq(var("x"), var("x"))),
    );
    let yx = MathExpr::forall(
        "x",
        MathType::Real,
        MathExpr::forall("y", MathType::Nat, MathExpr::eq(var("y"), var("y"))),
    );
    assert_ne!(
        dsn::retrieval_key(&xy).unwrap(),
        dsn::retrieval_key(&yx).unwrap()
    );
}
#[test]
fn dp_13_usm_knowledge_pack_is_retrievable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mirp/knowledge/usm_math_v0_1_1");
    let units: Vec<mathlang_mirp::knowledge::KnowledgeUnit> = serde_json::from_slice(
        &std::fs::read(root.join("catalogue/all_with_legacy.json")).unwrap(),
    )
    .unwrap();
    let mut k = mathlang_mirp::knowledge::KnowledgeSpace::empty();
    k.register_batch(units).unwrap();
    let views: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(root.join("catalogue/ir_views.json")).unwrap())
            .unwrap();
    assert_eq!(views.len(), 18);
    let legacy = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    for view in views {
        let o: ProofObligation = serde_json::from_value(view["obligation"].clone()).unwrap();
        let id = view["knowledge_id"].as_str().unwrap();
        let plan = dsn::plan(&o, &k, 1000).unwrap();
        assert!(
            plan.nodes[0].source_knowledge.iter().any(|d| d.id == id),
            "{id} not retrieved"
        );
        assert!(plan
            .nodes
            .iter()
            .all(|n| n.formal_status == FormalStatus::Unknown));
        let plan = dsn::plan(&o, &legacy, 1000).unwrap();
        assert!(plan.nodes.iter().all(|n| n.source_knowledge.is_empty()));
    }
}
#[test]
fn pl_01_stage1_classifies_dsn_computability() {
    use mathlang_verification::pipeline::{stage1, Step};
    let k = mathlang_mirp::knowledge::KnowledgeSpace::legacy();
    let (o, recipe, refutation) = fixtures::usm().remove(0);
    let raw = serde_json::json!({"obligation":o,"recipe":recipe,"refutation":refutation});
    let s = stage1("ok", &raw, &k, 1000);
    assert!(s.dsn_computable && s.blocked_at.is_none());
    assert_eq!(s.steps.len(), 4);
    assert!(s.translation.is_some());
    assert_eq!(
        s.lean_module_hash.as_deref(),
        Some(
            translate(&o, recipe, refutation)
                .unwrap()
                .record
                .lean_module_hash
                .as_str()
        )
    );
    // Bare obligation defaults to Auto, non-refutation.
    let s = stage1("bare", &serde_json::json!(o), &k, 1000);
    assert!(s.dsn_computable && !s.refutation);
    // Unsupported operator: blocked at IR decoding, no translation.
    let mut bad = serde_json::json!(o);
    bad["goal"] = serde_json::json!({"Sqrt":{"Variable":"x"}});
    let s = stage1("sqrt", &bad, &k, 1000);
    assert_eq!(s.blocked_at, Some(Step::IrDecode));
    assert!(!s.dsn_computable && s.translation.is_none());
    // Type error: blocked at validation.
    let p = MathExpr::Add(bx(var("x")), bx(var("y")));
    let typed = simple(
        "mismatch",
        MathExpr::eq(p.clone(), p),
        context(&[("x", vector(2)), ("y", vector(3))]),
    );
    let s = stage1("mismatch", &serde_json::json!(typed), &k, 1000);
    assert_eq!(s.blocked_at, Some(Step::Validate));
    // Planning budget: blocked at the DSN plan.
    let s = stage1("budget", &raw, &k, 1);
    assert_eq!(s.blocked_at, Some(Step::DsnPlan));
}
