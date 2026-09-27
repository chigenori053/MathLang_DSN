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
    assert_eq!(state.conflicts(), vec![[first.clone(), second.clone()]]);
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
    let relation = Relation {
        common: Common::new(
            stable_id("relation", &[&a, &b, "IS_A"]),
            Status::Known,
            p.clone(),
        ),
        relation_type: "SEMANTIC.IS_A".into(),
        source: a,
        target: b,
        arguments: vec![],
        polarity: Polarity::Negative,
        modality: Modality::Possible,
    };
    state
        .apply(SemanticDelta {
            added: vec![Object::Relation(relation)],
            ..Default::default()
        })
        .unwrap();
    assert!(state.canonical().unwrap().contains("\"POSSIBLE\""));
    let invalid = Relation {
        common: Common::new("bad".into(), Status::Known, p),
        relation_type: "BAD.IS_A".into(),
        source: "missing".into(),
        target: "missing".into(),
        arguments: vec![],
        polarity: Polarity::Positive,
        modality: Modality::Asserted,
    };
    assert_eq!(
        state
            .apply(SemanticDelta {
                added: vec![Object::Relation(invalid)],
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
    let mut dependencies = BTreeMap::new();
    dependencies.insert(a.clone(), vec![b.clone()]);
    dependencies.insert(b, vec![a]);
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
