use mathlang_mirp::knowledge::{
    ActivationClass, Applicability, ApplicabilityEngine, KnowledgeActivation, KnowledgeQuery,
    KnowledgeSpace,
};
use mathlang_mirp::memory::MemorySpace;
use mathlang_mirp::native::{MathProblemContext, ReasoningStatus};
use mathlang_mirp::pif::parse_problem;
use mathlang_mirp::{Object, Provenance, SemanticState, Status, Value};
use std::time::Instant;

fn query(goal: &str, symbol: &str) -> KnowledgeQuery {
    KnowledgeQuery {
        domain: "EQUATION".into(),
        goal: goal.into(),
        symbols: vec![symbol.into()],
        relations: vec![],
        required_properties: vec![],
        constraints: vec!["INTEGER_COEFFICIENTS".into(), "NONZERO_QUADRATIC".into()],
        semantic_state_refs: vec![],
        limit: 8,
    }
}

fn problem(i: i64) -> String {
    let first = i % 11 - 5;
    let mut second = (i * 7) % 13 - 6;
    if first == second {
        second += 1;
    }
    let scale = 1 + i % 2;
    format!(
        "Solve {scale}*x^2 + {}*x + {} = 0.",
        -scale * (first + second),
        scale * first * second
    )
}

#[test]
fn mandatory_640_case_matrix() {
    let mut checks = 0;
    for i in 0..40_i64 {
        let mut memory = MemorySpace::new();
        let mut unit = memory
            .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
            .unwrap()
            .clone();
        unit.knowledge_id = format!("EQUATION_PARAMETERIZED_RULE_{i:02}");
        unit.runtime_rule_id = None;
        unit.priority = i as i32;
        memory.register_knowledge(vec![unit.clone()]).unwrap();
        let mut context = MathProblemContext::parse(&problem(i)).unwrap();
        let state = context.semantic_state().clone();
        let mut q = query("FACTOR_EXPRESSION", "quadratic");
        q.semantic_state_refs.push(
            state
                .resolve("equation", "pini", &context.plan().problem_id)
                .unwrap()
                .common
                .id
                .clone(),
        );
        let retrieval = memory.query_knowledge(&q);
        let candidate = retrieval
            .candidates
            .iter()
            .find(|c| c.knowledge_id == unit.knowledge_id)
            .unwrap();
        // A: schema
        assert!(unit.validate().is_ok());
        checks += 1;
        // B: category lookup
        let mut other_category = q.clone();
        other_category.domain = "GEOMETRY".into();
        assert!(memory
            .query_knowledge(&other_category)
            .candidates
            .is_empty());
        checks += 1;
        // C: goal filter
        let mut other_goal = q.clone();
        other_goal.goal = "PROVE_RELATION".into();
        assert!(memory.query_knowledge(&other_goal).candidates.is_empty());
        checks += 1;
        // D: pattern matching
        assert!(candidate.matched_features.contains(&"quadratic".into()));
        checks += 1;
        // E: dependency resolution
        assert!(unit
            .dependencies
            .iter()
            .all(|id| memory.get_knowledge(id).is_some()));
        checks += 1;
        // F: constraint filter
        let mut no_constraint = q.clone();
        no_constraint.constraints.clear();
        assert!(memory
            .query_knowledge(&no_constraint)
            .rejected
            .iter()
            .any(|c| c.knowledge_id == unit.knowledge_id));
        checks += 1;
        // G: exact retrieval
        assert_eq!(candidate.activation_class, ActivationClass::Exact);
        checks += 1;
        // H: related retrieval
        let related = memory.query_knowledge(&query("FACTOR_EXPRESSION", "linear"));
        assert!(related
            .candidates
            .iter()
            .any(|c| c.knowledge_id == unit.knowledge_id
                && c.activation_class == ActivationClass::Related));
        checks += 1;
        // I: inapplicable rejection
        assert!(memory
            .query_knowledge(&no_constraint)
            .rejected
            .iter()
            .any(|c| c.activation_class == ActivationClass::Inapplicable));
        checks += 1;
        // J: applicability is separate from retrieval
        assert_eq!(
            ApplicabilityEngine::evaluate(&unit, candidate, &q, &state, |id| memory
                .get_knowledge(id)
                .is_some()),
            Applicability::Applicable
        );
        checks += 1;
        // K: RUS activation
        let legacy = memory.get_knowledge("JH_QUADRATIC_INTEGER_FACTOR").unwrap();
        assert_eq!(
            KnowledgeActivation::from_applicable(legacy, &state)
                .unwrap()
                .selected_rus,
            "QuadraticFactorizationRUS"
        );
        checks += 1;
        // L: RU selection linkage
        assert_eq!(
            memory.legacy_by_rule(35).unwrap().knowledge_id,
            legacy.knowledge_id
        );
        checks += 1;
        // M: native MIRP transition
        let result = context.execute(None).unwrap();
        assert_eq!(result.status, ReasoningStatus::Complete);
        assert!(context
            .semantic_state()
            .resolve("solution_set", "pini", &context.plan().problem_id)
            .is_some());
        checks += 1;
        // N: Knowledge provenance is attached to MIRP evidence
        assert!(context
            .semantic_state()
            .objects
            .iter()
            .any(|object| match object {
                Object::Evidence(e) =>
                    e.common
                        .metadata
                        .get("knowledge_activation")
                        .is_some_and(
                            |value| value["knowledge_id"] == "JH_QUADRATIC_INTEGER_FACTOR"
                                && value["selected_ru"] == 35
                                && value["ruo_ref"].is_string()
                                && value["result_state"].is_string()
                        ),
                _ => false,
            }));
        checks += 1;
        // O: migrated numeric bridge resolves the original semantic ID and RUS
        let selected = &result.knowledge_decisions[0].activation.as_ref().unwrap();
        assert_eq!(
            (
                selected.selected_ru,
                selected.knowledge_id.as_str(),
                selected.selected_rus.as_str()
            ),
            (
                35,
                "JH_QUADRATIC_INTEGER_FACTOR",
                "QuadraticFactorizationRUS"
            )
        );
        checks += 1;
        // P: unknown refs and conflicting working state do not activate knowledge
        let mut missing_ref = q.clone();
        missing_ref.semantic_state_refs.push(format!("missing-{i}"));
        assert_eq!(
            ApplicabilityEngine::evaluate(&unit, candidate, &missing_ref, &state, |id| memory
                .get_knowledge(id)
                .is_some()),
            Applicability::Unknown
        );
        let mut conflict = SemanticState::default();
        let p = Provenance::input("matrix", &format!("{i}"));
        let x = conflict
            .bind("x", "matrix", "global", "VARIABLE", p.clone())
            .unwrap();
        conflict
            .assert_value(&x, Value::Integer(1), p.clone(), Status::Known)
            .unwrap();
        conflict
            .assert_value(&x, Value::Integer(2), p, Status::Known)
            .unwrap();
        assert_eq!(
            ApplicabilityEngine::evaluate(&unit, candidate, &q, &conflict, |id| memory
                .get_knowledge(id)
                .is_some()),
            Applicability::Conflict
        );
        checks += 1;
    }
    assert_eq!(checks, 640);
}

#[test]
fn registry_validation_persistence_and_boundaries() {
    let legacy = MemorySpace::new();
    assert_eq!(legacy.knowledge_space().len(), 29);
    let unit = legacy
        .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    let mut memory = legacy.clone();
    assert!(memory.register_knowledge(vec![unit.clone()]).is_err());
    let mut malformed = unit.clone();
    malformed.knowledge_id = "bad id".into();
    assert!(memory.register_knowledge(vec![malformed]).is_err());
    let mut missing = unit.clone();
    missing.knowledge_id = "MISSING_DEPENDENCY_TEST".into();
    missing.runtime_rule_id = None;
    missing.dependencies = vec!["ABSENT".into()];
    assert!(memory.register_knowledge(vec![missing]).is_err());
    let mut left = unit.clone();
    left.knowledge_id = "CYCLE_LEFT".into();
    left.runtime_rule_id = None;
    left.dependencies = vec!["CYCLE_RIGHT".into()];
    let mut right = left.clone();
    right.knowledge_id = "CYCLE_RIGHT".into();
    right.dependencies = vec!["CYCLE_LEFT".into()];
    assert!(memory.register_knowledge(vec![left, right]).is_err());
    let mut unsupported = unit.clone();
    unsupported.knowledge_id = "UNKNOWN_CATEGORY".into();
    unsupported.runtime_rule_id = None;
    unsupported.category = "ASTROLOGY".into();
    assert!(memory.register_knowledge(vec![unsupported]).is_err());
    let empty = KnowledgeQuery {
        domain: "EQUATION".into(),
        goal: String::new(),
        symbols: vec![],
        relations: vec![],
        required_properties: vec![],
        constraints: vec![],
        semantic_state_refs: vec![],
        limit: 0,
    };
    assert!(memory.query_knowledge(&empty).candidates.is_empty());
    let mut added = unit.clone();
    added.knowledge_id = "EQUATION_NEW_REUSABLE_RULE".into();
    added.runtime_rule_id = None;
    memory.register_knowledge(vec![added]).unwrap();
    let path = std::env::temp_dir().join(format!("knowledge-space-{}.json", std::process::id()));
    memory.save(&path).unwrap();
    let loaded = MemorySpace::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.knowledge_space().len(), 30);
    assert!(loaded.get_knowledge("EQUATION_NEW_REUSABLE_RULE").is_some());
    assert_eq!(
        loaded.query_knowledge(&query("FACTOR_EXPRESSION", "quadratic")),
        memory.query_knowledge(&query("FACTOR_EXPRESSION", "quadratic"))
    );
    let source = include_str!("../knowledge/legacy.json");
    assert!(
        !source.contains("expected_answer")
            && !source.contains("fixture")
            && !source.contains("test_index")
    );
}

#[test]
fn native_runtime_requires_retrieved_applicable_knowledge() {
    let parsed = parse_problem("Solve x^2 - 5*x + 6 = 0.").unwrap();
    let memory = MemorySpace::with_knowledge_space(KnowledgeSpace::empty()).unwrap();
    let mut context =
        MathProblemContext::with_memory(parsed.definition, parsed.state, memory).unwrap();
    let result = context.execute(None).unwrap();
    assert_eq!(result.status, ReasoningStatus::Unsupported);
    assert!(result.knowledge_decisions[0]
        .retrieval
        .candidates
        .is_empty());
    assert!(result
        .trace
        .iter()
        .all(|trace| trace.ruo["source"] == "MathLang_DSN/native"));
}

#[test]
fn indexed_retrieval_reduces_candidate_space_at_scale() {
    for total in [100, 1_000, 10_000] {
        let seed = MemorySpace::new();
        let mut space = KnowledgeSpace::empty();
        let template = seed.get_knowledge("JH_QUADRATIC_INTEGER_FACTOR").unwrap();
        let mut units = Vec::with_capacity(total);
        for i in 0..total {
            let mut unit = template.clone();
            unit.knowledge_id = format!("SCALE_RULE_{i:05}");
            unit.runtime_rule_id = None;
            unit.dependencies.clear();
            unit.category = if i % 11 == 0 { "EQUATION" } else { "GEOMETRY" }.into();
            unit.applicable_goals = vec![if i % 5 == 0 {
                "FACTOR_EXPRESSION"
            } else {
                "FIND_LENGTH"
            }
            .into()];
            units.push(unit);
        }
        space.register_batch(units).unwrap();
        let mut q = query("FACTOR_EXPRESSION", "quadratic");
        q.limit = 3;
        let context = MathProblemContext::parse("Solve x^2 - 5*x + 6 = 0.").unwrap();
        let state = context.semantic_state();
        q.semantic_state_refs.push(
            state
                .resolve("equation", "pini", &context.plan().problem_id)
                .unwrap()
                .common
                .id
                .clone(),
        );
        let start = Instant::now();
        let result = space.query(&q);
        let latency = start.elapsed();
        assert_eq!(result.retrieval_trace.total_knowledge, total);
        assert!(result.retrieval_trace.examined <= total / 20);
        assert!(result.candidates.len() <= 3);
        let applicable = result
            .candidates
            .iter()
            .filter(|candidate| {
                let unit = space.get(&candidate.knowledge_id).unwrap();
                ApplicabilityEngine::evaluate(unit, candidate, &q, state, |id| {
                    space.get(id).is_some()
                }) == Applicability::Applicable
            })
            .count();
        assert!(applicable <= 3);
        eprintln!(
            "knowledge retrieval n={total} latency_us={} examined={} evaluated={} applicable={} final={} ratio={:.4}",
            latency.as_micros(),
            result.retrieval_trace.examined,
            result.candidates.len(),
            applicable,
            result.candidates.len(),
            result.candidates.len() as f64 / total as f64
        );
    }
}

#[test]
fn applicability_checks_mirp_type_and_required_relation() {
    use mathlang_mirp::{Common, Modality, Polarity, Relation, SemanticDelta};
    let context = MathProblemContext::parse("Solve x^2 - 5*x + 6 = 0.").unwrap();
    let mut state = context.semantic_state().clone();
    let equation = state
        .resolve("equation", "pini", &context.plan().problem_id)
        .unwrap()
        .common
        .id
        .clone();
    let variable = context
        .definition()
        .intent()
        .primary_goal
        .as_ref()
        .unwrap()
        .target_refs[0]
        .clone();
    let mut q = query("FACTOR_EXPRESSION", "quadratic");
    q.semantic_state_refs = vec![equation.clone()];
    let mut unit = MemorySpace::new()
        .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    unit.pattern.relations = vec!["MATHEMATICAL.EQUIVALENT_TO".into()];
    q.relations = unit.pattern.relations.clone();
    let candidate = mathlang_mirp::knowledge::KnowledgeCandidate {
        knowledge_id: unit.knowledge_id.clone(),
        match_type: "FULL".into(),
        matched_features: vec!["quadratic".into()],
        activation_class: ActivationClass::Exact,
    };
    assert_eq!(
        ApplicabilityEngine::evaluate(&unit, &candidate, &q, &state, |_| true),
        Applicability::PartiallyApplicable
    );
    let mut relation = Object::Relation(Relation {
        common: Common::unidentified(Status::Known, Provenance::input("test", "equivalence")),
        relation_type: "MATHEMATICAL.EQUIVALENT_TO".into(),
        source: equation.clone(),
        target: variable.clone(),
        arguments: vec![],
        polarity: Polarity::Positive,
        modality: Modality::Asserted,
    });
    relation.normalize_identity("test-relation");
    state
        .apply(SemanticDelta {
            added: vec![relation],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        ApplicabilityEngine::evaluate(&unit, &candidate, &q, &state, |_| true),
        Applicability::Applicable
    );
    q.semantic_state_refs = vec![variable];
    assert_eq!(
        ApplicabilityEngine::evaluate(&unit, &candidate, &q, &state, |_| true),
        Applicability::Unknown
    );
}
