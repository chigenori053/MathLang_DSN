use mathlang_mirp::*;
use std::collections::BTreeMap;

#[test]
fn core_delta_identity_conflict_and_unknown() {
    let mut state = SemanticState::default();
    let nl = Provenance::input("natural_language", "x is five");
    let math = Provenance::input("mathematics", "x + 2");
    let x = state
        .bind("x", "lesson", "global", "VARIABLE", nl.clone())
        .unwrap();
    assert_eq!(
        x,
        state
            .bind("x", "lesson", "global", "VARIABLE", math)
            .unwrap()
    );
    let local = state
        .bind("x", "lesson", "function_1", "VARIABLE", nl.clone())
        .unwrap();
    assert_ne!(x, local);
    state.add_alias(&x, "the variable").unwrap();
    assert_eq!(
        state
            .resolve("the variable", "lesson", "global")
            .unwrap()
            .common
            .id,
        x
    );
    let unknown = state
        .assert_value(&local, Value::Unknown, nl.clone(), Status::Unknown)
        .unwrap();
    assert!(matches!(
        state.value_of(&local).unwrap(),
        Some((Value::Unknown, _))
    ));
    assert!(state.get(&unknown).is_some());
    let first = state
        .assert_value(&x, Value::Integer(3), nl.clone(), Status::Known)
        .unwrap();
    let second = state
        .assert_value(
            &x,
            Value::Integer(4),
            Provenance::input("code", "x = 4"),
            Status::Known,
        )
        .unwrap();
    assert_eq!(state.status, Status::Conflict);
    let mut expected = [
        state.get_occurrence(&first).unwrap().id().to_owned(),
        state.get_occurrence(&second).unwrap().id().to_owned(),
    ];
    expected.sort();
    let mut conflict = state.conflicts().remove(0);
    conflict.sort();
    assert_eq!(conflict, expected);
    assert_eq!(
        state.value_of(&x).unwrap_err().category,
        "MirpConflictError"
    );
    assert!(state.get(&first).is_some() && state.get(&second).is_some());
    let before = state.clone();
    assert_eq!(
        state
            .apply(SemanticDelta {
                removed: vec!["absent".into()],
                ..Default::default()
            })
            .unwrap_err()
            .category,
        "MirpReferenceError"
    );
    assert_eq!(state, before);
    state
        .apply(SemanticDelta {
            removed: vec![second],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(state.status, Status::Known);
}

#[test]
fn relation_modality_and_validation() {
    let mut state = SemanticState::default();
    let p = Provenance::input("natural_language", "A may not be B");
    let a = state.bind("A", "n", "global", "OBJECT", p.clone()).unwrap();
    let b = state.bind("B", "n", "global", "OBJECT", p.clone()).unwrap();
    let mut relation = Object::Relation(Relation {
        common: Common::unidentified(Status::Known, p.clone()),
        relation_type: "SEMANTIC.IS_A".into(),
        source: a,
        target: b,
        arguments: vec![],
        polarity: Polarity::Negative,
        modality: Modality::Possible,
    });
    relation.normalize_identity("relation");
    state
        .apply(SemanticDelta {
            added: vec![relation],
            ..Default::default()
        })
        .unwrap();
    assert!(state.canonical().unwrap().contains("\"POSSIBLE\""));
    let mut invalid = Object::Relation(Relation {
        common: Common::unidentified(Status::Known, p),
        relation_type: "BAD.IS_A".into(),
        source: "missing".into(),
        target: "missing".into(),
        arguments: vec![],
        polarity: Polarity::Positive,
        modality: Modality::Asserted,
    });
    invalid.normalize_identity("invalid");
    assert_eq!(
        state
            .apply(SemanticDelta {
                added: vec![invalid],
                ..Default::default()
            })
            .unwrap_err()
            .category,
        "MirpRelationError"
    );
}

#[test]
fn canonical_serialization_determinism_and_persistence() {
    let mut state = SemanticState::default();
    let p = Provenance::input("mathematics", "x=3");
    let x = state
        .bind("x", "n", "global", "VARIABLE", p.clone())
        .unwrap();
    let value = state
        .assert_value(&x, Value::Integer(3), p, Status::Known)
        .unwrap();
    let bytes = state.canonical().unwrap();
    assert_eq!(bytes, state.canonical().unwrap());
    assert_eq!(
        bytes,
        SemanticState::from_json(&bytes)
            .unwrap()
            .canonical()
            .unwrap()
    );
    let path = std::env::temp_dir().join(format!("{}.mirp", stable_id("test", &[&x])));
    state.save(&path).unwrap();
    let restored = SemanticState::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(bytes, restored.canonical().unwrap());
    assert_eq!(
        restored.value_of(&x).unwrap(),
        Some((&Value::Integer(3), value.as_str()))
    );
    for _ in 0..3 {
        assert_eq!(bytes, state.canonical().unwrap());
    }
}

#[test]
fn dependency_cycle_and_missing_reference_are_rejected() {
    let mut state = SemanticState::default();
    let p = Provenance::input("test", "input");
    let a = state
        .bind("a", "n", "global", "VARIABLE", p.clone())
        .unwrap();
    let b = state.bind("b", "n", "global", "VARIABLE", p).unwrap();
    let before = state.clone();
    let a_occurrence = state.get(&a).unwrap().occurrence_id().to_owned();
    let b_occurrence = state.get(&b).unwrap().occurrence_id().to_owned();
    let mut dependencies = BTreeMap::new();
    dependencies.insert(a_occurrence.clone(), vec![b_occurrence.clone()]);
    dependencies.insert(b_occurrence, vec![a_occurrence]);
    assert_eq!(
        state
            .apply(SemanticDelta {
                dependencies,
                ..Default::default()
            })
            .unwrap_err()
            .category,
        "MirpValidationError"
    );
    assert_eq!(state, before);
}

#[test]
fn set_value_has_deterministic_order() {
    let p = Provenance::input("mathematics", "S={1,2}");
    let mut first = SemanticState::default();
    let id = first.bind("S", "n", "global", "SET", p.clone()).unwrap();
    first
        .assert_value(
            &id,
            Value::Set(vec![Value::Integer(2), Value::Integer(1)]),
            p.clone(),
            Status::Known,
        )
        .unwrap();
    let mut second = SemanticState::default();
    let id = second.bind("S", "n", "global", "SET", p.clone()).unwrap();
    second
        .assert_value(
            &id,
            Value::Set(vec![Value::Integer(1), Value::Integer(2)]),
            p,
            Status::Known,
        )
        .unwrap();
    assert_eq!(first.canonical().unwrap(), second.canonical().unwrap());
}

#[test]
fn call_result_representation_requires_a_real_call_dependency() {
    let mut state = SemanticState::default();
    let input = Provenance::input("code", "f(x)");
    let target = state
        .bind("f", "lesson", "global", "FUNCTION", input.clone())
        .unwrap();
    let argument = state
        .bind("x", "lesson", "global", "VARIABLE", input.clone())
        .unwrap();
    let mut call = Object::Call(Call {
        common: Common::unidentified(Status::Known, input.clone()),
        target,
        operation: "apply".into(),
        arguments: vec![argument],
        call_id: String::new(),
    });
    call.normalize_identity("call");
    let call_id = call.id().to_owned();
    let call_occurrence_id = call.occurrence_id().to_owned();
    if let Object::Call(inner) = &mut call {
        inner.call_id = call_id.clone();
    }
    let mut second_call = call.clone();
    second_call.normalize_identity("another invocation");
    let second_call_occurrence = second_call.occurrence_id().to_owned();
    assert_eq!(second_call.id(), call.id());
    assert_ne!(second_call_occurrence, call_occurrence_id);
    state
        .apply(SemanticDelta {
            added: vec![call, second_call],
            ..Default::default()
        })
        .unwrap();
    let mut result = Object::CallResult(CallResult {
        common: Common::unidentified(
            Status::Derived,
            input.derived(vec![call_occurrence_id.clone()], "call result"),
        ),
        call_id: call_id.clone(),
        call_occurrence_id: call_occurrence_id.clone(),
        value: Value::Integer(7),
    });
    result.normalize_identity("result");
    let result_id = result.occurrence_id().to_owned();
    let mut second_result = result.clone();
    if let Object::CallResult(inner) = &mut second_result {
        inner.call_occurrence_id = second_call_occurrence.clone();
        inner.common.provenance.parent_occurrence_ids = vec![second_call_occurrence.clone()];
    }
    second_result.normalize_identity("result");
    let second_result_occurrence = second_result.occurrence_id().to_owned();
    assert_eq!(second_result.id(), result.id());
    assert_ne!(second_result_occurrence, result_id);
    let before = state.clone();
    assert_eq!(
        state
            .apply(SemanticDelta {
                added: vec![result.clone()],
                ..Default::default()
            })
            .unwrap_err()
            .category,
        "MirpValidationError"
    );
    assert_eq!(state, before);
    state
        .apply(SemanticDelta {
            added: vec![result, second_result],
            dependencies: BTreeMap::from([
                (result_id.clone(), vec![call_occurrence_id]),
                (
                    second_result_occurrence.clone(),
                    vec![second_call_occurrence],
                ),
            ]),
            ..Default::default()
        })
        .unwrap();
    assert!(matches!(state.get(&result_id), Some(Object::CallResult(_))));
    assert!(matches!(
        state.get(&second_result_occurrence),
        Some(Object::CallResult(_))
    ));
    assert!(state.canonical().unwrap().contains("CALL_RESULT"));
}

#[test]
fn loaded_state_rejects_inconsistent_status_and_provenance() {
    let mut state = SemanticState::default();
    let provenance = Provenance::input("test", "x");
    state
        .bind("x", "n", "global", "VARIABLE", provenance)
        .unwrap();
    let mut tampered = state.clone();
    tampered.status = Status::Unknown;
    assert_eq!(
        tampered.validate().unwrap_err().category,
        "MirpValidationError"
    );
    let mut tampered = state;
    tampered.objects[0].common_mut().provenance.source_span = Some([2, 1]);
    assert_eq!(
        tampered.validate().unwrap_err().category,
        "MirpValidationError"
    );
}
