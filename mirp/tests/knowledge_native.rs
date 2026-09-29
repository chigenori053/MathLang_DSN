use mathlang_mirp::capability::{
    CapabilityExecutor, CapabilityRequest, CapabilityResult, RuntimeCapabilityRegistry,
};
use mathlang_mirp::memory::MemorySpace;
use mathlang_mirp::native::{solve_problem_with_memory, ReasoningStatus};
use mathlang_mirp::Object;
use std::process::Command;

fn added(memory: &mut MemorySpace, id: &str, kind: &str, capability: &str, priority: i32) {
    let mut unit = memory
        .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    unit.knowledge_id = id.into();
    unit.knowledge_type = kind.into();
    unit.capabilities = vec![capability.into()];
    unit.runtime_rule_id = None;
    unit.priority = priority;
    memory.register_knowledge(vec![unit]).unwrap();
}

struct TestExecutor;
impl CapabilityExecutor for TestExecutor {
    fn capability_id(&self) -> &'static str {
        "TEST_CAPABILITY"
    }
    fn supports_goal(&self, goal: &str) -> bool {
        goal == "FACTOR_EXPRESSION"
    }
    fn validate(
        &self,
        _: &mathlang_mirp::knowledge::KnowledgeActivation,
        _: &mathlang_mirp::SemanticState,
    ) -> Result<(), String> {
        Ok(())
    }
    fn execute(
        &self,
        _: &CapabilityRequest,
        _: &mathlang_mirp::SemanticState,
    ) -> Result<CapabilityResult, String> {
        Err("CAPABILITY_EXECUTION_FAILED".into())
    }
}

#[test]
fn capability_registry_has_stable_lookup_and_rejects_duplicates() {
    let mut registry = RuntimeCapabilityRegistry::quadratic();
    assert_eq!(
        registry.ids().collect::<Vec<_>>(),
        ["FACTOR_QUADRATIC_INTEGER", "SOLVE_QUADRATIC"]
    );
    assert!(registry.get("MISSING_CAPABILITY").is_none());
    registry.register(Box::new(TestExecutor)).unwrap();
    assert!(registry
        .get("TEST_CAPABILITY")
        .unwrap()
        .supports_goal("FACTOR_EXPRESSION"));
    assert!(registry
        .register(Box::new(TestExecutor))
        .unwrap_err()
        .starts_with("DUPLICATE_CAPABILITY"));
    assert_eq!(registry.ids().last(), Some("TEST_CAPABILITY"));
}

#[test]
fn new_knowledge_types_share_a_capability_without_numeric_ids() {
    let problem = "Solve x^2 - 5*x + 6 = 0.";
    let (legacy, _, _) = solve_problem_with_memory(problem, MemorySpace::new()).unwrap();
    let knowledge = [
        ("NEW_FACTOR_THEOREM", "THEOREM"),
        ("NEW_FACTOR_RULE", "RULE"),
        ("NEW_FACTOR_PROPERTY", "PROPERTY"),
    ];
    for (id, _) in knowledge {
        let mut memory = MemorySpace::new();
        for (candidate, kind) in knowledge {
            added(
                &mut memory,
                candidate,
                kind,
                "FACTOR_QUADRATIC_INTEGER",
                if candidate == id { 10 } else { 5 },
            );
        }
        let (result, state, _) = solve_problem_with_memory(problem, memory).unwrap();
        assert_eq!(result.status, ReasoningStatus::Complete);
        assert_eq!(result.answer, legacy.answer);
        let activation = result.knowledge_decisions[0].activation.as_ref().unwrap();
        assert_eq!(activation.knowledge_id, id);
        assert_eq!(activation.legacy_runtime_rule_id, None);
        assert_eq!(activation.capability_id, "FACTOR_QUADRATIC_INTEGER");
        let trace = result
            .trace
            .iter()
            .find(|t| t.ruo["knowledge_id"] == id)
            .unwrap();
        assert_eq!(trace.ruo["capability_id"], "FACTOR_QUADRATIC_INTEGER");
        assert_eq!(trace.ruo["ru_ref"], trace.ru_ref);
        assert!(trace.ru_ref.starts_with("ru:"));
        assert_eq!(trace.ruo["source"], "MathLang_DSN/knowledge-runtime");
        assert!(state.objects.iter().any(|object| match object {
            Object::Evidence(e) =>
                e.common
                    .metadata
                    .get("knowledge_runtime")
                    .is_some_and(|value| value["knowledge_id"] == id
                        && value["capability_id"] == "FACTOR_QUADRATIC_INTEGER"
                        && value["ruo_ref"] == trace.ruo_ref),
            _ => false,
        }));
    }
}

#[test]
fn missing_capability_is_distinct_from_missing_knowledge() {
    let problem = "Solve x^2 - 5*x + 6 = 0.";
    let mut memory = MemorySpace::new();
    added(
        &mut memory,
        "NEW_UNSUPPORTED_THEOREM",
        "THEOREM",
        "UNREGISTERED_CAPABILITY",
        10,
    );
    let (result, _, _) = solve_problem_with_memory(problem, memory).unwrap();
    assert_eq!(result.status, ReasoningStatus::Unsupported);
    assert!(result.problem_evaluation.primary_errors[0].starts_with("CAPABILITY_NOT_FOUND"));
    let (empty, _, _) = solve_problem_with_memory(
        problem,
        MemorySpace::with_knowledge_space(Default::default()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        empty.problem_evaluation.primary_errors[0],
        "NO_APPLICABLE_KNOWLEDGE"
    );
}

#[test]
fn explicitly_incompatible_equal_rank_knowledge_reports_conflict() {
    let mut memory = MemorySpace::new();
    let template = memory
        .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    let mut first = template.clone();
    first.knowledge_id = "CONFLICT_A".into();
    first.priority = 10;
    first.runtime_rule_id = None;
    first
        .activation_metadata
        .insert("conflicts_with".into(), serde_json::json!(["CONFLICT_B"]));
    let mut second = template;
    second.knowledge_id = "CONFLICT_B".into();
    second.priority = 10;
    second.runtime_rule_id = None;
    memory.register_knowledge(vec![first, second]).unwrap();
    let (result, _, _) = solve_problem_with_memory("Solve x^2 - 5*x + 6 = 0.", memory).unwrap();
    assert_eq!(result.status, ReasoningStatus::Conflict);
    assert_eq!(
        result.problem_evaluation.primary_errors,
        ["KNOWLEDGE_CONFLICT"]
    );
    assert!(result.knowledge_decisions[0].activation.is_none());
}

#[test]
fn schema_accepts_capability_free_knowledge_and_migrates_old_catalogs() {
    let memory = MemorySpace::new();
    let mut unit = memory
        .get_knowledge("JH_QUADRATIC_INTEGER_FACTOR")
        .unwrap()
        .clone();
    unit.knowledge_id = "NEW_DEFINITION".into();
    unit.knowledge_type = "DEFINITION".into();
    unit.capabilities.clear();
    unit.applicable_rus.clear();
    unit.runtime_rule_id = None;
    assert!(unit.validate().is_ok());
    unit.capabilities = vec!["bad id".into()];
    assert!(unit.validate().is_err());

    for kind in [
        "THEOREM",
        "AXIOM",
        "RULE",
        "DEFINITION",
        "PROPERTY",
        "IDENTITY",
        "LEMMA",
    ] {
        for capabilities in [
            vec![],
            vec!["FACTOR_QUADRATIC_INTEGER".into()],
            vec!["FACTOR_QUADRATIC_INTEGER".into(), "SOLVE_QUADRATIC".into()],
        ] {
            let mut variant = unit.clone();
            variant.knowledge_type = kind.into();
            variant.capabilities = capabilities;
            variant.applicable_rus = if variant.capabilities.is_empty() {
                vec![]
            } else {
                vec!["QuadraticFactorizationRUS".into()]
            };
            assert!(variant.validate().is_ok(), "{kind}");
        }
    }

    let mut serialized = serde_json::to_value(memory).unwrap();
    for entry in serialized["knowledge_space"]["units"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        entry.as_object_mut().unwrap().remove("capabilities");
    }
    let path = std::env::temp_dir().join(format!("old-catalog-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&serialized).unwrap()).unwrap();
    let migrated = MemorySpace::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        migrated.legacy_by_rule(35).unwrap().capabilities,
        ["FACTOR_QUADRATIC_INTEGER"]
    );
    assert_eq!(
        migrated.legacy_by_rule(23).unwrap().capabilities,
        ["SOLVE_QUADRATIC"]
    );
    for original in MemorySpace::new().knowledge_space().units() {
        assert_eq!(
            migrated
                .get_knowledge(&original.knowledge_id)
                .unwrap()
                .capabilities,
            original.capabilities
        );
    }
}

#[test]
fn persisted_knowledge_is_deterministic_across_three_processes() {
    let mut memory = MemorySpace::new();
    added(
        &mut memory,
        "NEW_PERSISTED_THEOREM",
        "THEOREM",
        "FACTOR_QUADRATIC_INTEGER",
        10,
    );
    let path = std::env::temp_dir().join(format!("knowledge-native-{}.json", std::process::id()));
    memory.save(&path).unwrap();
    let loaded = MemorySpace::load(&path).unwrap();
    let problem = "Solve x^2 - 5*x + 6 = 0.";
    let (result, state, plan) = solve_problem_with_memory(problem, loaded.clone()).unwrap();
    assert_eq!(
        result.knowledge_decisions[0]
            .activation
            .as_ref()
            .unwrap()
            .knowledge_id,
        "NEW_PERSISTED_THEOREM"
    );
    let ruos: Vec<_> = result
        .trace
        .iter()
        .filter(|t| t.ruo["source"] == "MathLang_DSN/knowledge-runtime")
        .map(|t| t.ruo.clone())
        .collect();
    let mut stored = loaded;
    stored.store("knowledge-native", &state, &ruos).unwrap();
    let restored = stored.retrieve("knowledge-native").unwrap();
    let subject = state
        .resolve("solution_set", "pini", &plan.problem_id)
        .unwrap()
        .common
        .id
        .clone();
    assert_eq!(
        restored.value_of(&subject).unwrap(),
        state.value_of(&subject).unwrap()
    );
    let outputs: Vec<_> = (0..3)
        .map(|_| {
            let output = Command::new(env!("CARGO_BIN_EXE_mathlang-mirp"))
                .args(["solve-problem", problem, path.to_str().unwrap()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        })
        .collect();
    std::fs::remove_file(path).unwrap();
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
}

#[test]
fn twenty_inputs_match_across_three_processes_and_registration_orders() {
    let knowledge = [
        ("NEW_ORDER_THEOREM", "THEOREM", 10),
        ("NEW_ORDER_RULE", "RULE", 5),
        ("NEW_ORDER_PROPERTY", "PROPERTY", 5),
    ];
    let paths: Vec<_> = (0..3)
        .map(|offset| {
            let mut memory = MemorySpace::new();
            for index in 0..3 {
                let (id, kind, priority) = knowledge[(index + offset) % 3];
                added(&mut memory, id, kind, "FACTOR_QUADRATIC_INTEGER", priority);
            }
            let path = std::env::temp_dir().join(format!(
                "knowledge-order-{}-{offset}.json",
                std::process::id()
            ));
            memory.save(&path).unwrap();
            path
        })
        .collect();
    let mut checks = 0;
    for i in 0..20_i64 {
        let first = i % 7 - 3;
        let second = i % 11 + 4;
        let problem = format!(
            "Solve x^2 + {}*x + {} = 0.",
            -(first + second),
            first * second
        );
        let outputs: Vec<_> = paths
            .iter()
            .map(|path| {
                let output = Command::new(env!("CARGO_BIN_EXE_mathlang-mirp"))
                    .args(["solve-problem", &problem, path.to_str().unwrap()])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                checks += 1;
                output.stdout
            })
            .collect();
        assert_eq!(outputs[0], outputs[1]);
        assert_eq!(outputs[1], outputs[2]);
    }
    for path in paths {
        std::fs::remove_file(path).unwrap();
    }
    assert_eq!(checks, 60);
}
