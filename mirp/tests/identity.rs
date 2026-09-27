use mathlang_mirp::{
    memory::MemorySpace,
    session::{Domain, Session},
    *,
};
use std::collections::BTreeSet;

fn value_occurrences(state: &SemanticState, subject: &str) -> Vec<(String, String, Provenance)> {
    state
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Attribute(attribute)
                if attribute.subject == subject && attribute.key == "VALUE" =>
            {
                Some((
                    attribute.common.id.clone(),
                    attribute.common.occurrence_id.clone(),
                    attribute.common.provenance.clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn same_fact_across_domains_has_one_meaning_and_two_occurrences() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    session.apply_input(Domain::Mathematics, "x = 5").unwrap();
    let x = &session
        .state
        .resolve("x", "lesson", "global")
        .unwrap()
        .common
        .id;
    let facts = value_occurrences(&session.state, x);
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].0, facts[1].0);
    assert_ne!(facts[0].1, facts[1].1);
    assert_ne!(facts[0].2.source_type, facts[1].2.source_type);
    assert_eq!(session.state.status, Status::Known);
    assert_eq!(
        session.state.value_of(x).unwrap().unwrap().0,
        &Value::Integer(5)
    );
}

#[test]
fn commutative_expressions_share_meaning_but_not_occurrence() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    session
        .apply_input(Domain::Mathematics, "a = x + 2")
        .unwrap();
    session
        .apply_input(Domain::Mathematics, "b = 2 + x")
        .unwrap();
    let adds: Vec<_> = session
        .state
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Expression(expression) if expression.operator == "ADD" => Some(expression),
            _ => None,
        })
        .collect();
    assert_eq!(adds.len(), 2);
    assert_eq!(adds[0].common.id, adds[1].common.id);
    assert_ne!(adds[0].common.occurrence_id, adds[1].common.occurrence_id);
}

#[test]
fn symmetric_and_directed_relations_have_the_right_semantic_ids() {
    let mut state = SemanticState::default();
    let p = Provenance::input("test", "relations");
    let x = state
        .bind("x", "lesson", "global", "VARIABLE", p.clone())
        .unwrap();
    let three = state
        .bind("3", "literal", "global", "NUMBER", p.clone())
        .unwrap();
    let relation = |relation_type: &str, source: String, target: String, index: &str| {
        let mut object = Object::Relation(Relation {
            common: Common::new(String::new(), Status::Known, p.clone()),
            relation_type: relation_type.into(),
            source,
            target,
            arguments: vec![],
            polarity: Polarity::Positive,
            modality: Modality::Asserted,
        });
        object.normalize_identity(index);
        object
    };
    let equal_a = relation("COMPARISON.EQUAL", x.clone(), three.clone(), "eq1");
    let equal_b = relation("COMPARISON.EQUAL", three.clone(), x.clone(), "eq2");
    assert_eq!(equal_a.id(), equal_b.id());
    assert_ne!(equal_a.occurrence_id(), equal_b.occurrence_id());
    let greater_a = relation("COMPARISON.GREATER_THAN", x.clone(), three.clone(), "gt1");
    let greater_b = relation("COMPARISON.GREATER_THAN", three, x, "gt2");
    assert_ne!(greater_a.id(), greater_b.id());
    state
        .apply(SemanticDelta {
            added: vec![equal_a, equal_b, greater_a, greater_b],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        state
            .objects
            .iter()
            .filter(|object| matches!(object, Object::Relation(_)))
            .count(),
        4
    );
}

#[test]
fn repeated_assertions_are_distinct_occurrences_and_conflicts_are_semantic() {
    let mut session = Session::new("lesson", "global");
    session.apply_input(Domain::Mathematics, "x = 3").unwrap();
    session.apply_input(Domain::Mathematics, "x = 3").unwrap();
    let x = session
        .state
        .resolve("x", "lesson", "global")
        .unwrap()
        .common
        .id
        .clone();
    let facts = value_occurrences(&session.state, &x);
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].0, facts[1].0);
    assert_ne!(facts[0].1, facts[1].1);
    assert_eq!(session.state.status, Status::Known);
    session.apply_input(Domain::Mathematics, "x = 4").unwrap();
    let facts = value_occurrences(&session.state, &x);
    assert_eq!(
        facts
            .iter()
            .map(|fact| fact.0.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        2
    );
    assert_eq!(session.state.status, Status::Conflict);
}

#[test]
fn an_occurrence_cannot_be_rewritten_to_a_different_meaning() {
    let mut state = SemanticState::default();
    let provenance = Provenance::input("test", "x=3");
    let x = state
        .bind("x", "n", "global", "VARIABLE", provenance.clone())
        .unwrap();
    let occurrence = state
        .assert_value(&x, Value::Integer(3), provenance, Status::Known)
        .unwrap();
    let mut changed = state.get_occurrence(&occurrence).unwrap().clone();
    if let Object::Attribute(attribute) = &mut changed {
        attribute.value = Value::Integer(4);
    }
    let before = state.clone();
    assert!(state
        .apply(SemanticDelta {
            updated: vec![changed],
            ..Default::default()
        })
        .is_err());
    assert_eq!(state, before);
}

#[test]
fn unknown_does_not_mask_a_later_known_conflict() {
    let mut state = SemanticState::default();
    let provenance = Provenance::input("test", "unknown then observations");
    let x = state
        .bind("x", "n", "global", "VARIABLE", provenance.clone())
        .unwrap();
    state
        .assert_value(&x, Value::Unknown, provenance.clone(), Status::Unknown)
        .unwrap();
    state
        .assert_value(&x, Value::Integer(5), provenance.clone(), Status::Known)
        .unwrap();
    assert_eq!(state.status, Status::Known);
    assert_eq!(state.value_of(&x).unwrap().unwrap().0, &Value::Integer(5));
    state
        .assert_value(&x, Value::Integer(6), provenance, Status::Known)
        .unwrap();
    assert_eq!(state.status, Status::Conflict);
    assert_eq!(
        state.value_of(&x).unwrap_err().category,
        "MirpConflictError"
    );
}

#[test]
fn persistence_and_memory_preserve_original_occurrences() {
    let mut session = Session::new("lesson", "global");
    session
        .apply_input(Domain::NaturalLanguage, "x is five.")
        .unwrap();
    let math = session
        .apply_input(Domain::Mathematics, "y = x + 2")
        .unwrap();
    let original = session.state.clone();
    let bytes = original.canonical().unwrap();
    let path = std::env::temp_dir().join(format!(
        "{}.mirp",
        stable_id("si-test", &[&std::process::id().to_string()])
    ));
    original.save(&path).unwrap();
    let loaded = SemanticState::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.canonical().unwrap(), bytes);
    assert_eq!(loaded.dependencies, original.dependencies);
    let mut legacy: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    legacy["mirp_version"] = serde_json::json!("mirp/1.0");
    assert_eq!(
        SemanticState::from_json(&legacy.to_string())
            .unwrap_err()
            .category,
        "MirpUnsupportedError"
    );
    for object in &original.objects {
        let restored = loaded.get_occurrence(object.occurrence_id()).unwrap();
        assert_eq!(restored.id(), object.id());
        assert_eq!(restored.common().provenance, object.common().provenance);
    }
    let mut memory = MemorySpace::new();
    memory.store("m", &original, &math[0].ruos).unwrap();
    let retrieved = memory.retrieve("m").unwrap();
    for object in &original.objects {
        let restored = retrieved.get_occurrence(object.occurrence_id()).unwrap();
        assert_eq!(restored, object);
    }
    let evidence = retrieved
        .objects
        .iter()
        .find_map(|object| match object {
            Object::Evidence(evidence) if evidence.evidence_type == "MEMORY_RETRIEVAL" => {
                Some(evidence)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(evidence.common.provenance.memory_id.as_deref(), Some("m"));
    assert_eq!(
        evidence.common.metadata["retrieved_occurrence_ids"]
            .as_array()
            .unwrap()
            .len(),
        original.objects.len()
    );
}
