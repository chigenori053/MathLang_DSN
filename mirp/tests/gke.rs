use mathlang_mirp::gke::{
    infer, infer_goal, GenericRule, InferenceLimits, InferenceStatus, RulePredicate, RuleTerm,
    RuleValueType, RuleVariable,
};
use mathlang_mirp::knowledge::{KnowledgeSpace, KnowledgeUnit};
use mathlang_mirp::{
    Common, Object, PredicateFact, Provenance, SemanticDelta, SemanticState, Status,
};

fn variable(name: &str) -> RuleTerm {
    RuleTerm::Variable(name.into())
}
fn symbol(name: &str) -> RuleTerm {
    RuleTerm::Symbol(name.into())
}
fn pred(name: &str, args: Vec<RuleTerm>) -> RulePredicate {
    RulePredicate {
        predicate: name.into(),
        arguments: args,
        negated: false,
    }
}
fn unit(id: &str, premises: Vec<RulePredicate>, conclusions: Vec<RulePredicate>) -> KnowledgeUnit {
    let mut unit = KnowledgeSpace::legacy()
        .get("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    unit.knowledge_id = id.into();
    unit.category = "LOGIC".into();
    unit.knowledge_type = "THEOREM".into();
    unit.pattern = Default::default();
    unit.premises.clear();
    unit.constraints.clear();
    unit.dependencies.clear();
    unit.applicable_goals.clear();
    unit.applicable_rus.clear();
    unit.capabilities.clear();
    unit.runtime_rule_id = None;
    unit.generic_rule = Some(GenericRule {
        variables: ["a", "b", "c", "t", "a2", "b2", "sum", "ac", "bc"]
            .into_iter()
            .map(|name| RuleVariable {
                name: name.into(),
                value_type: RuleValueType::Symbol,
            })
            .collect(),
        premises,
        constraints: vec![],
        conclusions,
    });
    unit
}
fn add(state: &mut SemanticState, name: &str, args: Vec<RuleTerm>) {
    let mut fact = Object::Predicate(PredicateFact {
        common: Common::unidentified(Status::Known, Provenance::input("test", name)),
        predicate: name.into(),
        arguments: args,
        negated: false,
    });
    fact.normalize_identity(name);
    state
        .apply(SemanticDelta {
            added: vec![fact],
            ..Default::default()
        })
        .unwrap();
}
fn transitive(id: &str, predicate: &str) -> KnowledgeUnit {
    unit(
        id,
        vec![
            pred(predicate, vec![variable("a"), variable("b")]),
            pred(predicate, vec![variable("b"), variable("c")]),
        ],
        vec![pred(predicate, vec![variable("a"), variable("c")])],
    )
}
#[test]
fn hot_knowledge_injection_and_trace() {
    let mut space = KnowledgeSpace::empty();
    let mut state = SemanticState::default();
    add(&mut state, "EQUAL", vec![symbol("x"), symbol("y")]);
    add(&mut state, "EQUAL", vec![symbol("y"), symbol("z")]);
    assert_eq!(
        infer(
            &space,
            &mut state.clone(),
            "EQUAL",
            InferenceLimits::default()
        )
        .status,
        InferenceStatus::Derived
    );
    // A missing goal predicate stays unknown before adding the rule.
    let mut state = SemanticState::default();
    add(&mut state, "LINK", vec![symbol("x"), symbol("y")]);
    add(&mut state, "LINK", vec![symbol("y"), symbol("z")]);
    assert_eq!(
        infer(
            &space,
            &mut state.clone(),
            "REACH",
            InferenceLimits::default()
        )
        .status,
        InferenceStatus::Unknown
    );
    space
        .register_batch(vec![unit(
            "TEST_TRANSITIVE_EQUALITY",
            vec![
                pred("LINK", vec![variable("a"), variable("b")]),
                pred("LINK", vec![variable("b"), variable("c")]),
            ],
            vec![pred("REACH", vec![variable("a"), variable("c")])],
        )])
        .unwrap();
    let report = infer(&space, &mut state, "REACH", InferenceLimits::default());
    assert_eq!(report.status, InferenceStatus::Derived);
    assert_eq!(report.traces[0].knowledge_id, "TEST_TRANSITIVE_EQUALITY");
    let fact = state
        .objects
        .iter()
        .find_map(|o| {
            if let Object::Predicate(f) = o {
                if f.predicate == "REACH" {
                    Some(f)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(fact.arguments, vec![symbol("x"), symbol("z")]);
    assert_eq!(fact.common.provenance.parent_occurrence_ids.len(), 2);
    assert_eq!(
        fact.common.metadata["gke"]["ruo_ref"],
        report.traces[0].ruo_ref
    );
    state.validate().unwrap();
}
#[test]
fn bounded_multistep_and_order_invariance() {
    let first = unit(
        "RULE_FIRST",
        vec![pred("P", vec![variable("a")])],
        vec![pred("Q", vec![variable("a")])],
    );
    let second = unit(
        "RULE_SECOND",
        vec![pred("Q", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    let mut outputs = vec![];
    for order in [
        vec![first.clone(), second.clone()],
        vec![second.clone(), first.clone()],
    ] {
        let mut space = KnowledgeSpace::empty();
        space.register_batch(order).unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol("x")]);
        let result = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(result.status, InferenceStatus::Derived);
        assert_eq!(result.metrics.inference_steps, 2);
        outputs.push((
            serde_json::to_string(&result.traces).unwrap(),
            state.canonical().unwrap(),
        ));
    }
    assert_eq!(outputs[0], outputs[1]);
}
#[test]
fn conflicts_and_limits() {
    let yes = unit(
        "RULE_YES",
        vec![pred("P", vec![variable("a")])],
        vec![pred("Q", vec![variable("a")])],
    );
    let mut no = unit(
        "RULE_NO",
        vec![pred("P", vec![variable("a")])],
        vec![RulePredicate {
            negated: true,
            ..pred("Q", vec![variable("a")])
        }],
    );
    no.priority = yes.priority;
    let mut space = KnowledgeSpace::empty();
    space.register_batch(vec![yes, no]).unwrap();
    let mut state = SemanticState::default();
    add(&mut state, "P", vec![symbol("x")]);
    assert_eq!(
        infer(&space, &mut state, "Q", InferenceLimits::default()).status,
        InferenceStatus::Conflict
    );
    assert_eq!(state.objects.len(), 1);
    let limits = InferenceLimits {
        max_candidates_per_step: 1,
        ..Default::default()
    };
    assert_eq!(
        infer(&space, &mut state, "Q", limits).status,
        InferenceStatus::ResourceLimit
    );
}
#[test]
fn four_domain_patterns_use_same_executor() {
    let mut cases: Vec<(
        &str,
        KnowledgeUnit,
        Vec<(&str, Vec<RuleTerm>)>,
        &str,
        Vec<RuleTerm>,
    )> = vec![];
    let mut logic = transitive("LOGIC_TRANSITIVE", "EQUAL");
    logic.category = "LOGIC".into();
    cases.push((
        "logic",
        logic,
        vec![
            ("EQUAL", vec![symbol("a"), symbol("b")]),
            ("EQUAL", vec![symbol("b"), symbol("c")]),
        ],
        "EQUAL",
        vec![symbol("a"), symbol("c")],
    ));
    let mut arithmetic = transitive("ARITHMETIC_TRANSITIVE", "GREATER_THAN");
    arithmetic.category = "ARITHMETIC".into();
    cases.push((
        "arithmetic",
        arithmetic,
        vec![
            ("GREATER_THAN", vec![symbol("a"), symbol("b")]),
            ("GREATER_THAN", vec![symbol("b"), symbol("c")]),
        ],
        "GREATER_THAN",
        vec![symbol("a"), symbol("c")],
    ));
    let mut algebra = unit(
        "ALGEBRA_ADDITIVE_EQUALITY",
        vec![
            pred("EQUAL", vec![variable("a"), variable("b")]),
            pred("ADD", vec![variable("a"), variable("c"), variable("ac")]),
            pred("ADD", vec![variable("b"), variable("c"), variable("bc")]),
        ],
        vec![pred("EQUAL", vec![variable("ac"), variable("bc")])],
    );
    algebra.category = "ALGEBRA".into();
    cases.push((
        "algebra",
        algebra,
        vec![
            ("EQUAL", vec![symbol("a"), symbol("b")]),
            ("ADD", vec![symbol("a"), symbol("c"), symbol("a_plus_c")]),
            ("ADD", vec![symbol("b"), symbol("c"), symbol("b_plus_c")]),
        ],
        "EQUAL",
        vec![symbol("a_plus_c"), symbol("b_plus_c")],
    ));
    let mut geometry = unit(
        "GEOMETRY_PYTHAGOREAN",
        vec![
            pred("RIGHT_TRIANGLE", vec![variable("t")]),
            pred("LEG", vec![variable("t"), variable("a")]),
            pred("LEG", vec![variable("t"), variable("b")]),
            pred("SQUARE", vec![variable("a"), variable("a2")]),
            pred("SQUARE", vec![variable("b"), variable("b2")]),
            pred("ADD", vec![variable("a2"), variable("b2"), variable("sum")]),
        ],
        vec![pred(
            "HYPOTENUSE_SQUARED",
            vec![variable("t"), variable("sum")],
        )],
    );
    geometry.category = "GEOMETRY".into();
    cases.push((
        "geometry",
        geometry,
        vec![
            ("RIGHT_TRIANGLE", vec![symbol("triangle")]),
            ("LEG", vec![symbol("triangle"), symbol("a")]),
            ("LEG", vec![symbol("triangle"), symbol("b")]),
            ("SQUARE", vec![symbol("a"), symbol("a2")]),
            ("SQUARE", vec![symbol("b"), symbol("b2")]),
            ("ADD", vec![symbol("a2"), symbol("b2"), symbol("sum")]),
        ],
        "HYPOTENUSE_SQUARED",
        vec![symbol("triangle"), symbol("sum")],
    ));
    for (domain, rule, inputs, goal, expected) in cases {
        let mut space = KnowledgeSpace::empty();
        space.register_batch(vec![rule]).unwrap();
        let mut state = SemanticState::default();
        for (name, args) in inputs {
            add(&mut state, name, args);
        }
        let before = state.objects.len();
        let result = infer_goal(
            &space,
            &mut state,
            &pred(goal, expected.clone()),
            InferenceLimits::default(),
        );
        assert_eq!(
            result.status,
            InferenceStatus::Derived,
            "{domain}: {:?}",
            result.diagnostics
        );
        assert_eq!(state.objects.len(), before + 2, "{domain}");
        assert!(state.objects.iter().any(
            |o| matches!(o, Object::Predicate(f) if f.predicate == goal && f.arguments == expected)
        ));
    }
}

#[test]
fn built_binary_accepts_knowledge_added_after_build() {
    use std::process::Command;
    let directory = std::env::temp_dir().join(format!("gke-hot-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let state_file = directory.join("state.mirp");
    let knowledge_file = directory.join("knowledge.json");
    let mut state = SemanticState::default();
    add(&mut state, "LINK", vec![symbol("x"), symbol("y")]);
    add(&mut state, "LINK", vec![symbol("y"), symbol("z")]);
    state.save(&state_file).unwrap();
    let binary = env!("CARGO_BIN_EXE_mathlang-mirp");
    let run = || -> serde_json::Value {
        let output = Command::new(binary)
            .args([
                "gke-infer",
                state_file.to_str().unwrap(),
                knowledge_file.to_str().unwrap(),
                "REACH",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    };
    std::fs::write(&knowledge_file, "[]").unwrap();
    assert_eq!(run()["report"]["status"], "UNKNOWN");
    let rule = unit(
        "TEST_TRANSITIVE_EQUALITY",
        vec![
            pred("LINK", vec![variable("a"), variable("b")]),
            pred("LINK", vec![variable("b"), variable("c")]),
        ],
        vec![pred("REACH", vec![variable("a"), variable("c")])],
    );
    std::fs::write(&knowledge_file, serde_json::to_vec(&vec![rule]).unwrap()).unwrap();
    let result = run();
    assert_eq!(result["report"]["status"], "DERIVED");
    assert_eq!(
        result["report"]["traces"][0]["knowledge_id"],
        "TEST_TRANSITIVE_EQUALITY"
    );
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn exact_constraints_are_evaluated_by_reasonscript() {
    let mut rule = unit(
        "RULE_POSITIVE",
        vec![pred("INPUT", vec![variable("a")])],
        vec![pred("OUTPUT", vec![variable("a")])],
    );
    rule.generic_rule.as_mut().unwrap().variables[0].value_type = RuleValueType::Number;
    rule.generic_rule
        .as_mut()
        .unwrap()
        .constraints
        .push(pred("POSITIVE", vec![variable("a")]));
    let mut space = KnowledgeSpace::empty();
    space.register_batch(vec![rule]).unwrap();
    let mut state = SemanticState::default();
    add(&mut state, "INPUT", vec![RuleTerm::Rational(3, 2)]);
    let report = infer(&space, &mut state, "OUTPUT", InferenceLimits::default());
    assert_eq!(
        report.status,
        InferenceStatus::Derived,
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn formal_1300_scenario_matrix() {
    let mut counts = std::collections::BTreeMap::new();
    for i in 0..50 {
        let mut rule = unit(
            &format!("SCHEMA_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("Q", vec![variable("a")])],
        );
        if i % 2 == 0 {
            assert!(rule.validate().is_ok());
            let copy: KnowledgeUnit =
                serde_json::from_str(&serde_json::to_string(&rule).unwrap()).unwrap();
            assert!(copy.validate().is_ok());
        } else {
            let duplicate = rule.generic_rule.as_ref().unwrap().variables[0].clone();
            rule.generic_rule
                .as_mut()
                .unwrap()
                .variables
                .push(duplicate);
            assert!(rule.validate().is_err());
        }
        *counts.entry("schema").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("MATCH_{i}"),
                vec![
                    pred("P", vec![variable("a"), variable("b")]),
                    pred("Q", vec![variable("b"), variable("c")]),
                ],
                vec![pred("R", vec![variable("a"), variable("c")])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(
            &mut state,
            "P",
            vec![symbol(&format!("x{i}")), symbol(&format!("y{i}"))],
        );
        add(
            &mut state,
            "Q",
            vec![
                symbol(&format!("{}{}", if i % 2 == 0 { "y" } else { "other" }, i)),
                symbol(&format!("z{i}")),
            ],
        );
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(report.status == InferenceStatus::Derived, i % 2 == 0);
        *counts.entry("matching").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("BIND_{i}"),
                vec![
                    pred("P", vec![variable("a"), variable("b")]),
                    pred("Q", vec![variable("a"), variable("c")]),
                ],
                vec![pred("R", vec![variable("b"), variable("c")])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(
            &mut state,
            "P",
            vec![symbol(&format!("x{i}")), symbol("left")],
        );
        add(
            &mut state,
            "Q",
            vec![
                symbol(&format!(
                    "{}{}",
                    if i % 2 == 0 { "x" } else { "different" },
                    i
                )),
                symbol("right"),
            ],
        );
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status
                == InferenceStatus::Derived,
            i % 2 == 0
        );
        *counts.entry("binding").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut rule = unit(
            &format!("CONSTRAINT_{i}"),
            vec![pred("P", vec![variable("a"), variable("b")])],
            vec![pred("R", vec![variable("a")])],
        );
        rule.generic_rule
            .as_mut()
            .unwrap()
            .constraints
            .push(pred("DISTINCT", vec![variable("a"), variable("b")]));
        let mut space = KnowledgeSpace::empty();
        space.register_batch(vec![rule]).unwrap();
        let mut state = SemanticState::default();
        add(
            &mut state,
            "P",
            vec![
                symbol(&format!("x{i}")),
                symbol(&format!("{}{}", if i % 2 == 0 { "y" } else { "x" }, i)),
            ],
        );
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status
                == InferenceStatus::Derived,
            i % 2 == 0
        );
        *counts.entry("constraint").or_insert(0) += 1;
    }
    for i in 0..100 {
        let conclusion = if i % 2 == 0 {
            variable("a")
        } else {
            variable("c")
        };
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("INSTANTIATE_{i}"),
                vec![pred("P", vec![variable("a")])],
                vec![pred("R", vec![conclusion])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(report.status == InferenceStatus::Derived, i % 2 == 0);
        if i % 2 != 0 {
            assert!(report.diagnostics[0].contains("UNBOUND_CONCLUSION_VARIABLE"));
        }
        *counts.entry("instantiation").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("SINGLE_{i}"),
                vec![pred("P", vec![variable("a")])],
                vec![pred("R", vec![variable("a")])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(report.status, InferenceStatus::Derived);
        assert_eq!(report.metrics.inference_steps, 1);
        *counts.entry("single_step").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![
                unit(
                    &format!("MULTI_A_{i}"),
                    vec![pred("P", vec![variable("a")])],
                    vec![pred("Q", vec![variable("a")])],
                ),
                unit(
                    &format!("MULTI_B_{i}"),
                    vec![pred("Q", vec![variable("a")])],
                    vec![pred("R", vec![variable("a")])],
                ),
            ])
            .unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(report.status, InferenceStatus::Derived);
        assert_eq!(report.metrics.inference_steps, 2);
        *counts.entry("multi_step").or_insert(0) += 1;
    }
    for i in 0..50 {
        let a = unit(
            &format!("DEP_A_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("Q", vec![variable("a")])],
        );
        let mut b = unit(
            &format!("DEP_B_{i}"),
            vec![pred("Q", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        b.dependencies = vec![if i % 2 == 0 {
            a.knowledge_id.clone()
        } else {
            format!("MISSING_{i}")
        }];
        let mut space = KnowledgeSpace::empty();
        assert_eq!(space.register_batch(vec![a, b]).is_ok(), i % 2 == 0);
        *counts.entry("dependency").or_insert(0) += 1;
    }
    for i in 0..100 {
        let yes = unit(
            &format!("CONFLICT_Y_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        let mut no = unit(
            &format!("CONFLICT_N_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![RulePredicate {
                negated: true,
                ..pred("R", vec![variable("a")])
            }],
        );
        no.priority = yes.priority - (i % 2) as i32;
        let mut space = KnowledgeSpace::empty();
        space.register_batch(vec![yes, no]).unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status
                == InferenceStatus::Conflict,
            i % 2 == 0
        );
        *counts.entry("conflict").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("TRACE_{i}"),
                vec![pred("P", vec![variable("a")])],
                vec![pred("R", vec![variable("a")])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        let trace = &report.traces[0];
        assert_eq!(trace.premise_occurrence_refs.len(), 1);
        assert_eq!(report.rus[0].selected_ru, trace.ru_ref);
        assert_eq!(
            state
                .get_occurrence(&trace.conclusion_occurrence_refs[0])
                .unwrap()
                .common()
                .metadata["gke"]["ruo_ref"],
            trace.ruo_ref
        );
        *counts.entry("trace").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        space
            .register_batch(vec![unit(
                &format!("DELTA_{i}"),
                vec![pred("P", vec![variable("a")])],
                vec![pred("R", vec![variable("a")])],
            )])
            .unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        let before = state.version;
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status,
            InferenceStatus::Derived
        );
        assert_eq!(state.version, before + 1);
        state.validate().unwrap();
        *counts.entry("semantic_delta").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut space = KnowledgeSpace::empty();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
        assert_eq!(
            infer(&space, &mut state.clone(), "R", InferenceLimits::default()).status,
            InferenceStatus::Unknown
        );
        space
            .register_batch(vec![unit(
                &format!("DYNAMIC_{i}"),
                vec![pred("P", vec![variable("a")])],
                vec![pred("R", vec![variable("a")])],
            )])
            .unwrap();
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status,
            InferenceStatus::Derived
        );
        *counts.entry("dynamic").or_insert(0) += 1;
    }
    for i in 0..100 {
        let a = unit(
            &format!("ORDER_A_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        let b = unit(
            &format!("ORDER_B_{i}"),
            vec![pred("UNUSED", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        let mut results = vec![];
        for order in [vec![a.clone(), b.clone()], vec![b, a]] {
            let mut space = KnowledgeSpace::empty();
            space.register_batch(order).unwrap();
            let mut state = SemanticState::default();
            add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
            let report = infer(&space, &mut state, "R", InferenceLimits::default());
            results.push((
                serde_json::to_string(&report.traces).unwrap(),
                state.canonical().unwrap(),
            ));
        }
        assert_eq!(results[0], results[1]);
        *counts.entry("determinism").or_insert(0) += 1;
    }
    for i in 0..100 {
        let mut rule = unit(
            &format!("NEGATIVE_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        match i % 4 {
            0 => {
                rule.generic_rule.as_mut().unwrap().conclusions[0].arguments[0] =
                    RuleTerm::Rational(1, 0);
                assert!(rule.validate().is_err());
            }
            1 => {
                rule.generic_rule.as_mut().unwrap().conclusions[0].arguments[0] =
                    variable("missing");
                assert!(rule.validate().is_err());
            }
            2 => {
                rule.generic_rule
                    .as_mut()
                    .unwrap()
                    .constraints
                    .push(pred("UNSUPPORTED_GUARD", vec![variable("a")]));
                let mut space = KnowledgeSpace::empty();
                space.register_batch(vec![rule]).unwrap();
                let mut state = SemanticState::default();
                add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
                assert_eq!(
                    infer(&space, &mut state, "R", InferenceLimits::default()).status,
                    InferenceStatus::Unsupported
                );
            }
            _ => {
                let mut space = KnowledgeSpace::empty();
                space.register_batch(vec![rule]).unwrap();
                let mut state = SemanticState::default();
                add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
                assert_eq!(
                    infer(
                        &space,
                        &mut state,
                        "R",
                        InferenceLimits {
                            max_steps: 0,
                            ..Default::default()
                        }
                    )
                    .status,
                    InferenceStatus::ResourceLimit
                );
            }
        }
        *counts.entry("negative").or_insert(0) += 1;
    }
    assert_eq!(counts.values().sum::<i32>(), 1300, "{counts:?}");
}

#[test]
#[ignore = "explicit 100k KnowledgeSpace performance gate"]
fn performance_scales() {
    for size in [100, 1_000, 10_000, 50_000, 100_000] {
        let template = unit(
            "BENCH_TEMPLATE",
            vec![pred("UNUSED", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        let mut units = Vec::with_capacity(size);
        for i in 0..size {
            let mut unit = template.clone();
            unit.knowledge_id = format!("BENCH_{i:06}");
            if i == size - 1 {
                unit.generic_rule.as_mut().unwrap().premises[0].predicate = "P".into();
            }
            units.push(unit);
        }
        let start = std::time::Instant::now();
        let mut space = KnowledgeSpace::empty();
        space.register_batch(units).unwrap();
        let registration_ms = start.elapsed().as_millis();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![symbol("x")]);
        let start = std::time::Instant::now();
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        let inference_ms = start.elapsed().as_millis();
        assert_eq!(report.status, InferenceStatus::Derived);
        assert_eq!(report.metrics.total_knowledge, size);
        assert_eq!(report.metrics.examined_knowledge, 1);
        println!(
            "size={size} registration_ms={registration_ms} inference_ms={inference_ms} metrics={}",
            serde_json::to_string(&report.metrics).unwrap()
        );
    }
}

#[test]
fn three_processes_and_three_registration_orders_are_identical() {
    use std::process::Command;
    let directory = std::env::temp_dir().join(format!("gke-determinism-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let state_file = directory.join("state.mirp");
    let knowledge_file = directory.join("knowledge.json");
    let mut state = SemanticState::default();
    add(&mut state, "P", vec![symbol("x")]);
    state.save(&state_file).unwrap();
    let relevant = unit(
        "DETERMINISM_RELEVANT",
        vec![pred("P", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    let irrelevant_a = unit(
        "DETERMINISM_OTHER_A",
        vec![pred("A", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    let irrelevant_b = unit(
        "DETERMINISM_OTHER_B",
        vec![pred("B", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    let orders = [
        vec![relevant.clone(), irrelevant_a.clone(), irrelevant_b.clone()],
        vec![irrelevant_b.clone(), relevant.clone(), irrelevant_a.clone()],
        vec![irrelevant_a, irrelevant_b, relevant],
    ];
    let mut results = vec![];
    for order in orders {
        std::fs::write(&knowledge_file, serde_json::to_vec(&order).unwrap()).unwrap();
        for _ in 0..3 {
            let output = Command::new(env!("CARGO_BIN_EXE_mathlang-mirp"))
                .args([
                    "gke-infer",
                    state_file.to_str().unwrap(),
                    knowledge_file.to_str().unwrap(),
                    "R",
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["report"]["status"], "DERIVED");
            results.push(value);
        }
    }
    assert!(results.iter().all(|value| value == &results[0]));
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn state_insertion_order_and_symbol_renaming_are_metamorphic() {
    let rule = unit(
        "METAMORPHIC_RULE",
        vec![
            pred("P", vec![variable("a")]),
            pred("Q", vec![variable("b")]),
        ],
        vec![pred("R", vec![variable("a"), variable("b")])],
    );
    let mut space = KnowledgeSpace::empty();
    space.register_batch(vec![rule]).unwrap();
    let mut outputs = vec![];
    for inputs in [
        vec![("P", "x"), ("Q", "y")],
        vec![("Q", "y"), ("P", "x")],
        vec![("P", "a"), ("Q", "b")],
    ] {
        let mut state = SemanticState::default();
        for (predicate, value) in inputs {
            add(&mut state, predicate, vec![symbol(value)]);
        }
        let report = infer(&space, &mut state, "R", InferenceLimits::default());
        assert_eq!(report.status, InferenceStatus::Derived);
        let terms = state
            .objects
            .iter()
            .find_map(|object| match object {
                Object::Predicate(fact) if fact.predicate == "R" => Some(fact.arguments.clone()),
                _ => None,
            })
            .unwrap();
        outputs.push((report.traces, state.canonical().unwrap(), terms));
    }
    assert_eq!(
        serde_json::to_string(&outputs[0].0).unwrap(),
        serde_json::to_string(&outputs[1].0).unwrap()
    );
    assert_eq!(outputs[0].1, outputs[1].1);
    assert_eq!(outputs[0].2, vec![symbol("x"), symbol("y")]);
    assert_eq!(outputs[2].2, vec![symbol("a"), symbol("b")]);
}

#[test]
fn typed_terms_and_exact_rationals() {
    let mut space = KnowledgeSpace::empty();
    let mut rule = unit(
        "TYPED_NUMBER",
        vec![pred("P", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    rule.generic_rule.as_mut().unwrap().variables[0].value_type = RuleValueType::Number;
    space.register_batch(vec![rule]).unwrap();
    let mut state = SemanticState::default();
    add(&mut state, "P", vec![symbol("3/2")]);
    assert_eq!(
        infer(&space, &mut state, "R", InferenceLimits::default()).status,
        InferenceStatus::NoMatch
    );
    add(&mut state, "P", vec![RuleTerm::Rational(3, 2)]);
    assert_eq!(
        infer_goal(
            &space,
            &mut state,
            &pred("R", vec![RuleTerm::Rational(3, 2)]),
            InferenceLimits::default()
        )
        .status,
        InferenceStatus::Derived
    );
    let mut invalid = SemanticState::default();
    let mut malformed = Object::Predicate(PredicateFact {
        common: Common::unidentified(Status::Known, Provenance::input("test", "invalid")),
        predicate: "P".into(),
        arguments: vec![RuleTerm::Rational(2, 4)],
        negated: false,
    });
    malformed.normalize_identity("invalid");
    assert!(invalid
        .apply(SemanticDelta {
            added: vec![malformed],
            ..Default::default()
        })
        .is_err());

    let mut geometry = unit(
        "TYPED_GEOMETRY",
        vec![pred("TRIANGLE", vec![variable("t")])],
        vec![pred("RIGHT_TRIANGLE", vec![variable("t")])],
    );
    geometry
        .generic_rule
        .as_mut()
        .unwrap()
        .variables
        .iter_mut()
        .find(|v| v.name == "t")
        .unwrap()
        .value_type = RuleValueType::GeometryObject;
    let mut space = KnowledgeSpace::empty();
    space.register_batch(vec![geometry]).unwrap();
    let mut state = SemanticState::default();
    let id = state
        .bind(
            "T",
            "gke",
            "test",
            "GEOMETRY_OBJECT",
            Provenance::input("test", "T"),
        )
        .unwrap();
    add(
        &mut state,
        "TRIANGLE",
        vec![RuleTerm::EntityRef(id.clone())],
    );
    assert_eq!(
        infer_goal(
            &space,
            &mut state,
            &pred("RIGHT_TRIANGLE", vec![RuleTerm::EntityRef(id)]),
            InferenceLimits::default()
        )
        .status,
        InferenceStatus::Derived
    );
}

#[test]
fn mandatory_limits_stop_before_unbounded_work() {
    let limits = InferenceLimits::default();
    assert_eq!(
        (
            limits.max_steps,
            limits.max_candidates_per_step,
            limits.max_bindings_per_rule,
            limits.max_dependency_depth
        ),
        (64, 128, 256, 32)
    );

    let mut space = KnowledgeSpace::empty();
    space
        .register_batch(vec![unit(
            "BINDING_LIMIT",
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        )])
        .unwrap();
    let mut state = SemanticState::default();
    for i in 0..257 {
        add(&mut state, "P", vec![symbol(&format!("x{i}"))]);
    }
    assert_eq!(
        infer(&space, &mut state, "R", limits.clone()).status,
        InferenceStatus::ResourceLimit
    );

    let template = unit(
        "CANDIDATE_TEMPLATE",
        vec![pred("P", vec![variable("a")])],
        vec![pred("R", vec![variable("a")])],
    );
    let mut units = vec![];
    for i in 0..129 {
        let mut next = template.clone();
        next.knowledge_id = format!("CANDIDATE_{i}");
        units.push(next);
    }
    let mut space = KnowledgeSpace::empty();
    space.register_batch(units).unwrap();
    let mut state = SemanticState::default();
    add(&mut state, "P", vec![symbol("x")]);
    assert_eq!(
        infer(&space, &mut state, "R", limits.clone()).status,
        InferenceStatus::ResourceLimit
    );
    assert_eq!(state.objects.len(), 1);

    let mut chain = vec![];
    for i in 0..33 {
        let mut next = template.clone();
        next.knowledge_id = format!("CHAIN_{i:02}");
        next.generic_rule.as_mut().unwrap().premises[0].predicate =
            if i == 32 { "P".into() } else { "UNUSED".into() };
        if i > 0 {
            next.dependencies.push(format!("CHAIN_{:02}", i - 1));
        }
        chain.push(next);
    }
    let mut space = KnowledgeSpace::empty();
    space.register_batch(chain).unwrap();
    assert_eq!(
        infer(&space, &mut state, "R", limits.clone()).status,
        InferenceStatus::ResourceLimit
    );

    let mut space = KnowledgeSpace::empty();
    space
        .register_batch(vec![
            unit(
                "STEP_A",
                vec![pred("P", vec![variable("a")])],
                vec![pred("Q", vec![variable("a")])],
            ),
            unit(
                "STEP_B",
                vec![pred("Q", vec![variable("a")])],
                vec![pred("R", vec![variable("a")])],
            ),
        ])
        .unwrap();
    assert_eq!(
        infer(
            &space,
            &mut state,
            "R",
            InferenceLimits {
                max_steps: 1,
                ..limits
            }
        )
        .status,
        InferenceStatus::ResourceLimit
    );
}

#[test]
fn unknown_facts_do_not_satisfy_premises() {
    let mut space = KnowledgeSpace::empty();
    space
        .register_batch(vec![unit(
            "UNKNOWN_PREMISE",
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        )])
        .unwrap();
    let mut state = SemanticState::default();
    let mut fact = Object::Predicate(PredicateFact {
        common: Common::unidentified(Status::Unknown, Provenance::input("test", "unknown")),
        predicate: "P".into(),
        arguments: vec![symbol("x")],
        negated: false,
    });
    fact.normalize_identity("unknown");
    state
        .apply(SemanticDelta {
            added: vec![fact],
            ..Default::default()
        })
        .unwrap();
    let report = infer(&space, &mut state, "R", InferenceLimits::default());
    assert_eq!(report.status, InferenceStatus::Unknown);
    assert_eq!(report.metrics.executed_knowledge, 0);
}

#[test]
fn prior_opposite_fact_transitions_to_mirp_conflict() {
    let mut space = KnowledgeSpace::empty();
    space
        .register_batch(vec![unit(
            "OPPOSITE_RULE",
            vec![pred("P", vec![variable("a")])],
            vec![pred("Q", vec![variable("a")])],
        )])
        .unwrap();
    let mut state = SemanticState::default();
    add(&mut state, "P", vec![symbol("x")]);
    let mut negative = Object::Predicate(PredicateFact {
        common: Common::unidentified(Status::Known, Provenance::input("test", "negative")),
        predicate: "Q".into(),
        arguments: vec![symbol("x")],
        negated: true,
    });
    negative.normalize_identity("negative");
    state
        .apply(SemanticDelta {
            added: vec![negative],
            ..Default::default()
        })
        .unwrap();
    let report = infer_goal(
        &space,
        &mut state,
        &pred("Q", vec![symbol("x")]),
        InferenceLimits::default(),
    );
    assert_eq!(report.status, InferenceStatus::Conflict);
    assert_eq!(report.traces[0].status, InferenceStatus::Conflict);
    assert_eq!(state.status, Status::Conflict);
    state.validate().unwrap();
}

#[test]
fn reasonscript_numeric_guard_truth_table() {
    for (i, guard, value, expected) in [
        (0, "NONZERO", RuleTerm::Integer(0), InferenceStatus::NoMatch),
        (1, "NONZERO", RuleTerm::Integer(5), InferenceStatus::Derived),
        (
            2,
            "POSITIVE",
            RuleTerm::Integer(-1),
            InferenceStatus::NoMatch,
        ),
        (
            3,
            "POSITIVE",
            RuleTerm::Rational(1, 2),
            InferenceStatus::Derived,
        ),
        (
            4,
            "INTEGER",
            RuleTerm::Rational(3, 2),
            InferenceStatus::NoMatch,
        ),
        (5, "INTEGER", RuleTerm::Integer(4), InferenceStatus::Derived),
    ] {
        let mut rule = unit(
            &format!("NUMERIC_GUARD_{i}"),
            vec![pred("P", vec![variable("a")])],
            vec![pred("R", vec![variable("a")])],
        );
        rule.generic_rule.as_mut().unwrap().variables[0].value_type = RuleValueType::Number;
        rule.generic_rule
            .as_mut()
            .unwrap()
            .constraints
            .push(pred(guard, vec![variable("a")]));
        let mut space = KnowledgeSpace::empty();
        space.register_batch(vec![rule]).unwrap();
        let mut state = SemanticState::default();
        add(&mut state, "P", vec![value]);
        assert_eq!(
            infer(&space, &mut state, "R", InferenceLimits::default()).status,
            expected,
            "{guard} {i}"
        );
    }
}
