use mathlang_mirp::intent::{ExpectedOutput, OperationIntent};
use mathlang_mirp::pif::*;

fn pair(n: i64) -> (String, String, OperationIntent) {
    let expression = format!("x^2 - {}*x + {}", 2 * n + 1, n * (n + 1));
    let equation = format!("{expression} = 0");
    match n % 4 {
        0 => (
            format!("Solve {equation}."),
            format!("{equation} を解け。"),
            OperationIntent::Solve,
        ),
        1 => (
            format!("Factor {expression}."),
            format!("{expression} を因数分解せよ。"),
            OperationIntent::Factor,
        ),
        2 => (
            format!("Expand (x - {n})*(x - {}).", n + 1),
            format!("(x - {n})*(x - {}) を展開せよ。", n + 1),
            OperationIntent::Expand,
        ),
        _ => (
            format!("Verify that x = {n} is a solution of {equation}."),
            format!("x = {n} が {equation} の解であることを確認せよ。"),
            OperationIntent::Verify,
        ),
    }
}

#[test]
fn pif_3_forty_language_cases_and_twenty_equivalent_pairs() {
    let mut english = 0;
    let mut japanese = 0;
    for n in 1..=20 {
        let (en, ja, operation) = pair(n);
        let left = parse_problem(&en).unwrap();
        let right = parse_problem(&ja).unwrap();
        let a = left.definition.intent();
        let b = right.definition.intent();
        assert_eq!(a.intent_status, IntentStatus::Known);
        assert_eq!(b.intent_status, IntentStatus::Known);
        assert_eq!(a.operation_intent.kind, Some(operation.clone()));
        assert_eq!(a.semantic_signature(), b.semantic_signature());
        assert_eq!(a.problem_id, b.problem_id);
        assert_ne!(a.source_occurrence_id, b.source_occurrence_id);
        if operation == OperationIntent::Solve {
            assert!(a
                .operation_intent
                .qualifiers
                .contains(&IntentQualifier::AllSolutions));
            assert_eq!(
                a.expected_output.as_ref().unwrap().kind,
                ExpectedOutput::SolutionSet
            );
            assert_eq!(
                a.expected_output.as_ref().unwrap().cardinality,
                Cardinality::ExactSet
            );
            assert_eq!(
                a.expected_output.as_ref().unwrap().equivalence_policy,
                EquivalencePolicy::SetEquivalence
            );
        }
        a.validate(&left.state).unwrap();
        b.validate(&right.state).unwrap();
        english += 1;
        japanese += 1;
    }
    assert_eq!((english, japanese), (20, 20));
}

#[test]
fn pif_3_ambiguity_unknown_and_extraction_provenance() {
    let ambiguous = parse_problem("Solve or factor x^2 - 5*x + 6 = 0.").unwrap();
    assert_eq!(
        ambiguous.definition.intent().intent_status,
        IntentStatus::Ambiguous
    );
    assert!(ambiguous.definition.intent().primary_goal.is_none());
    let unknown = parse_problem("Consider the graph.").unwrap();
    assert_eq!(
        unknown.definition.intent().intent_status,
        IntentStatus::Unknown
    );
    assert!(parse_problem("Solve x^3 - 8 = 0.").is_err());
    let parsed = parse_problem("Solve x^2 - 5*x + 6 = 0 using factorization where x > 0.").unwrap();
    let intent = parsed.definition.intent();
    assert_eq!(intent.constraints.len(), 1);
    assert_eq!(intent.method_constraints.len(), 1);
    assert_eq!(
        intent.method_constraints[0].source_occurrence_id,
        intent.source_occurrence_id
    );
    assert_eq!(
        intent
            .expected_output
            .as_ref()
            .unwrap()
            .source_occurrence_id,
        intent.source_occurrence_id
    );
    assert!(intent
        .completion_criteria
        .iter()
        .all(|c| c.source_occurrence_id == intent.source_occurrence_id));
}
