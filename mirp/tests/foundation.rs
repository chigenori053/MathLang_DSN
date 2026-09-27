use mathlang_mirp::{
    memory::MemorySpace,
    session::{Domain, Session},
    *,
};
use std::process::Command;

#[test]
fn cross_domain_reasoning_and_restart() {
    let mut session = Session::new("lesson", "global");
    let first = session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    assert_eq!(first[0].result, Some(Value::Integer(5)));
    let x = session
        .state
        .resolve("x", "lesson", "global")
        .unwrap()
        .common
        .id
        .clone();
    let x_fact = first[0].result_id.clone().unwrap();
    let path = std::env::temp_dir().join(format!("{}.mirp", stable_id("restart", &[&x])));
    session.state.save(&path).unwrap();
    let restored = SemanticState::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let mut session = Session::restore(restored, "lesson", "global").unwrap();
    let second = session
        .apply_input(Domain::Mathematics, "y = x + 2")
        .unwrap();
    assert_eq!(second[0].result, Some(Value::Integer(7)));
    assert_eq!(second[0].ruos.last().unwrap()["knowledge_id"], "POLY_ADD");
    let y = session
        .state
        .resolve("y", "lesson", "global")
        .unwrap()
        .common
        .id
        .clone();
    let y_fact = second[0].result_id.clone().unwrap();
    assert!(session.state.dependencies[&y_fact].contains(&x_fact));
    assert_eq!(
        session.state.dependencies[&y_fact],
        session
            .state
            .get_occurrence(&y_fact)
            .unwrap()
            .common()
            .provenance
            .parent_occurrence_ids
    );
    assert!(session
        .state
        .get_occurrence(&y_fact)
        .unwrap()
        .common()
        .provenance
        .parent_occurrence_ids
        .contains(&x_fact));
    let third = session
        .apply_input(Domain::Code, "if y > 6:\n    result = y")
        .unwrap();
    assert_eq!(third[0].result, Some(Value::Integer(7)));
    assert_eq!(third[0].ruos[0]["source_rus"], "ComparisonRUS");
    assert!(session.state.objects.iter().any(|object| matches!(object,
        Object::Condition(condition) if condition.truth == Truth::True)));
    let result = session
        .state
        .resolve("result", "lesson", "global")
        .unwrap()
        .common
        .id
        .clone();
    let result_fact = third[0].result_id.clone().unwrap();
    assert!(session.state.dependencies[&result_fact].contains(&y_fact));
    assert!(session
        .state
        .get_occurrence(&result_fact)
        .unwrap()
        .common()
        .provenance
        .parent_occurrence_ids
        .contains(&y_fact));
    assert_eq!(
        session.state.value_of(&result).unwrap().unwrap().0,
        &Value::Integer(7)
    );
    assert_eq!(
        session.state.value_of(&y).unwrap().unwrap().0,
        &Value::Integer(7)
    );
    let bytes = session.state.canonical().unwrap();
    assert_eq!(
        bytes,
        SemanticState::from_json(&bytes)
            .unwrap()
            .canonical()
            .unwrap()
    );
}

#[test]
fn code_scope_and_unknown_are_preserved() {
    let mut session = Session::new("lesson", "function_a");
    let result = session
        .apply_input(Domain::Code, "if missing > 0:\n    result = 1")
        .unwrap();
    assert_eq!(result[0].status, Status::Unknown);
    assert_eq!(result[0].result, Some(Value::Unknown));
    assert!(session.state.objects.iter().any(|object| matches!(object,
        Object::Condition(condition) if condition.truth == Truth::Unknown)));
    assert!(session
        .state
        .resolve("result", "lesson", "function_a")
        .is_some());
    assert!(session
        .state
        .value_of(
            &session
                .state
                .resolve("result", "lesson", "function_a")
                .unwrap()
                .common
                .id
        )
        .unwrap()
        .is_none());
    assert!(session.apply_input(Domain::Code, "eval('bad')").is_err());
}

#[test]
fn independent_processes_share_persisted_state() {
    let path = std::env::temp_dir().join(format!("{}.mirp", stable_id("process", &["foundation"])));
    let binary = env!("CARGO_BIN_EXE_mathlang-mirp");
    for (domain, input, expected) in [
        ("nl", "x is five.", 5),
        ("math", "y = x + 2", 7),
        ("code", "if y > 6:\n    result = y", 7),
    ] {
        let output = Command::new(binary)
            .args([path.to_str().unwrap(), "lesson", "global", domain, input])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result[0]["result"]["value"], expected);
    }
    let state = SemanticState::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(state.status, Status::Known);
    assert!(state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Evidence(_))));
}

#[test]
fn three_fresh_runs_have_byte_identical_mirp() {
    let mut outputs = vec![];
    for _ in 0..3 {
        let mut session = Session::new("lesson", "global");
        session
            .apply_input(Domain::NaturalLanguage, "x is five.")
            .unwrap();
        session
            .apply_input(Domain::Mathematics, "y = x + 2")
            .unwrap();
        session
            .apply_input(Domain::Code, "if y > 6:\n    result = y")
            .unwrap();
        outputs.push(session.state.canonical().unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
}

#[test]
fn cross_domain_runtime_generalizes_beyond_reference_numbers() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "a is four.")
        .unwrap();
    let arithmetic = session
        .apply_input(Domain::Mathematics, "b = a * 3")
        .unwrap();
    assert_eq!(arithmetic[0].result, Some(Value::Integer(12)));
    assert_eq!(arithmetic[0].ruos[0]["source_rus"], "PolynomialRUS");
    let branch = session
        .apply_input(Domain::Code, "if b <= 12:\n    output = b")
        .unwrap();
    assert_eq!(branch[0].result, Some(Value::Integer(12)));
    assert_eq!(branch[0].ruos[0]["source_rus"], "ComparisonRUS");
    let false_branch = session
        .apply_input(Domain::Code, "if b < 12:\n    skipped = b")
        .unwrap();
    assert_eq!(false_branch[0].result, Some(Value::Boolean(false)));
    assert!(session
        .state
        .resolve("skipped", "lesson", "global")
        .is_some());
    let skipped_id = &session
        .state
        .resolve("skipped", "lesson", "global")
        .unwrap()
        .common
        .id;
    assert!(session.state.value_of(skipped_id).unwrap().is_none());
}

#[test]
fn natural_language_relation_preserves_negation_modality_and_condition() {
    let mut session = Session::new("story", "global");
    for sentence in [
        "A is not B.",
        "A may be B.",
        "A must be B.",
        "If C, A is B.",
    ] {
        session
            .apply_input(Domain::NaturalLanguage, sentence)
            .unwrap();
    }
    let relations: Vec<_> = session
        .state
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Relation(relation) => Some(relation),
            _ => None,
        })
        .collect();
    assert_eq!(relations.len(), 4);
    assert!(relations
        .iter()
        .any(|r| r.polarity == Polarity::Negative && r.modality == Modality::Asserted));
    assert!(relations.iter().any(|r| r.modality == Modality::Possible));
    assert!(relations.iter().any(|r| r.modality == Modality::Necessary));
    assert!(relations
        .iter()
        .any(|r| r.modality == Modality::Conditional && r.arguments.len() == 1));
    assert!(session
        .state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Condition(c) if c.truth == Truth::Unknown)));
}

#[test]
fn copied_nlsrv_adapter_enters_shared_mirp() {
    let mut session = Session::new("story", "global");
    session
        .apply_input(Domain::Japanese, "太郎は学生である。")
        .unwrap();
    session
        .apply_input(Domain::Japanese, "太郎は先生ではない。")
        .unwrap();
    let taro = session
        .state
        .resolve("太郎", "story", "global")
        .unwrap()
        .common
        .id
        .clone();
    let relations: Vec<_> = session
        .state
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Relation(relation) if relation.source == taro => Some(relation),
            _ => None,
        })
        .collect();
    assert_eq!(relations.len(), 2);
    assert!(relations.iter().any(|r| r.polarity == Polarity::Positive));
    assert!(relations.iter().any(|r| r.polarity == Polarity::Negative));
    assert!(relations
        .iter()
        .all(|r| r.common.provenance.source_type == "nlsrv_japanese"));
    assert!(relations.iter().all(|r| {
        r.common.metadata.contains_key("source_sentence")
            && r.common.metadata.contains_key("source_index")
            && r.common.metadata.contains_key("temporal_context")
    }));
}

#[test]
fn copied_nlsrv_quantity_and_reference_binding() {
    let mut session = Session::new("story", "global");
    session
        .apply_input(
            Domain::Japanese,
            "太郎は本を3個持っている。彼は学生である。",
        )
        .unwrap();
    let taro = session
        .state
        .resolve("太郎", "story", "global")
        .unwrap()
        .common
        .id
        .clone();
    let book = session
        .state
        .resolve("本", "story", "global")
        .unwrap()
        .common
        .id
        .clone();
    assert!(session
        .state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Attribute(a)
        if a.subject == book && a.key == "COUNT" && a.value == Value::Integer(3))));
    assert!(session
        .state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Relation(r)
        if r.source == taro && r.target == book && r.relation_type == "SEMANTIC.HAS")));
    assert!(session
        .state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Evidence(e)
        if e.evidence_type == "REFERENCE_BINDING" && e.supports.contains(&taro)
        && e.common.metadata.contains_key("source_sentence"))));
    let before = session.state.canonical().unwrap();
    assert!(session
        .apply_input(Domain::Japanese, "太郎は学生である。\"); result = true; //")
        .is_err());
    assert_eq!(before, session.state.canonical().unwrap());
}

#[test]
fn copied_memory_codec_restores_mirp_after_restart() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    let derived = session
        .apply_input(Domain::Mathematics, "y = x + 2")
        .unwrap();
    let original = session.state.clone();
    let mut memory = MemorySpace::new();
    memory
        .store("lesson-1", &session.state, &derived[0].ruos)
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "{}.memory",
        stable_id("memory-test", &["lesson-1"])
    ));
    memory.save(&path).unwrap();
    let restored = MemorySpace::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let state = restored.retrieve("lesson-1").unwrap();
    let y = state.resolve("y", "lesson", "global").unwrap();
    assert_eq!(
        state.value_of(&y.common.id).unwrap().unwrap().0,
        &Value::Integer(7)
    );
    for object in &original.objects {
        let restored = state.get_occurrence(object.occurrence_id()).unwrap();
        assert_eq!(restored.id(), object.id());
        assert_eq!(restored.common().provenance, object.common().provenance);
    }
    assert!(state.objects.iter().any(|object| matches!(object,
        Object::Evidence(evidence) if evidence.evidence_type == "MEMORY_RETRIEVAL"
            && evidence.source == "MemorySpace" && evidence.supports.contains(&y.common.id)
            && evidence.common.provenance.memory_id.as_deref() == Some("lesson-1"))));
}

#[test]
fn memory_bridge_continues_reasoning_in_new_processes() {
    let base = std::env::temp_dir().join(format!("mirp-memory-process-{}", std::process::id()));
    std::fs::create_dir_all(&base).unwrap();
    let state_path = base.join("state.mirp");
    let memory_path = base.join("memory.json");
    let trace_path = base.join("trace.json");
    let restored_path = base.join("restored.mirp");
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    let derived = session
        .apply_input(Domain::Mathematics, "y = x + 2")
        .unwrap();
    let original = session.state.clone();
    session.state.save(&state_path).unwrap();
    std::fs::write(
        &trace_path,
        serde_json::to_string(&derived[0].ruos).unwrap(),
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_mathlang-mirp");
    for args in [
        vec![
            "memory-store",
            state_path.to_str().unwrap(),
            memory_path.to_str().unwrap(),
            "m-1",
            trace_path.to_str().unwrap(),
        ],
        vec![
            "memory-restore",
            memory_path.to_str().unwrap(),
            "m-1",
            restored_path.to_str().unwrap(),
        ],
        vec![
            restored_path.to_str().unwrap(),
            "lesson",
            "global",
            "code",
            "if y > 6:\n    result = y",
        ],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let resumed = SemanticState::load(&restored_path).unwrap();
    for object in &original.objects {
        let restored = resumed.get_occurrence(object.occurrence_id()).unwrap();
        assert_eq!(restored.id(), object.id());
        assert_eq!(
            restored.common().occurrence_key,
            object.common().occurrence_key
        );
        assert_eq!(restored.common().provenance, object.common().provenance);
    }
    let result = resumed.resolve("result", "lesson", "global").unwrap();
    assert_eq!(
        resumed.value_of(&result.common.id).unwrap().unwrap().0,
        &Value::Integer(7)
    );
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn copied_math_and_code_adapters_use_shared_runtime() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    assert_eq!(
        session.apply_input(Domain::Mathematics, "y:=x").unwrap()[0].result,
        Some(Value::Integer(5))
    );
    let arithmetic = session.apply_input(Domain::Mathematics, "x+2=z").unwrap();
    assert_eq!(arithmetic[0].result, Some(Value::Integer(7)));
    assert_eq!(arithmetic[0].ruos[0]["source_rus"], "PolynomialRUS");
    let branch = session
        .apply_input(Domain::Code, "if(z>6){result=z;}")
        .unwrap();
    assert_eq!(branch[0].result, Some(Value::Integer(7)));
    assert_eq!(branch[0].ruos[0]["source_rus"], "ComparisonRUS");
    assert_eq!(
        session
            .apply_input(Domain::Code, "let w = result;")
            .unwrap()[0]
            .result,
        Some(Value::Integer(7))
    );
    session.apply_input(Domain::Mathematics, "z>6").unwrap();
    assert!(session
        .state
        .objects
        .iter()
        .any(|object| matches!(object, Object::Relation(r)
        if r.relation_type == "COMPARISON.GREATER_THAN")));
}
