use mathlang_mirp::intent::{
    EducationalError, ExpectedOutput, GoalCompletion, IntentAlignment, MathematicalValidity,
    Method, ProblemRelevance,
};
use mathlang_mirp::pif::*;

fn equation(n: i64) -> String {
    format!("x^2 - {}*x + {} = 0", 2 * n + 1, n * (n + 1))
}

fn context(text: &str) -> (ReasoningContext, String, String) {
    let ctx = ReasoningContext::from_parsed(parse_problem(text).unwrap()).unwrap();
    let target = ctx
        .definition()
        .intent()
        .primary_goal
        .as_ref()
        .unwrap()
        .target_refs[0]
        .clone();
    let source = ctx.definition().intent().source_occurrence_id.clone();
    (ctx, target, source)
}

fn step(target: &str, source: &str, execution: Execution) -> ReasoningStep {
    ReasoningStep {
        target_ref: target.into(),
        source_occurrence_ids: vec![source.into()],
        declared_method: None,
        execution,
    }
}

fn answer(n: i64) -> Execution {
    Execution::Answer {
        values: vec![n, n + 1],
        output: ExpectedOutput::SolutionSet,
    }
}

#[test]
fn pif_5_remaining_matrix_groups_and_end_to_end() {
    let mut counts = [0usize; 8]; // G, L, M, N, O, P, Q, R
    for n in 1..=20 {
        let text = format!("Solve {}.", equation(n));
        // G: two executed, mathematically distinct strategy traces both complete.
        let (mut factor, target, source) = context(&text);
        let factor_step = factor
            .execute(step(
                &target,
                &source,
                Execution::Factor { roots: [n, n + 1] },
            ))
            .unwrap();
        let factor_trace = factor_step.trace.as_ref().unwrap().clone();
        assert_eq!(factor_step.intent_alignment, IntentAlignment::Aligned);
        assert_eq!(
            factor
                .execute(step(&target, &source, answer(n)))
                .unwrap()
                .goal_completion,
            GoalCompletion::Complete
        );
        let (mut formula, _, _) = context(&text);
        let formula_step = formula
            .execute(step(&target, &source, Execution::QuadraticFormula))
            .unwrap();
        let formula_trace = formula_step.trace.as_ref().unwrap().clone();
        assert_eq!(
            formula
                .execute(step(&target, &source, answer(n)))
                .unwrap()
                .goal_completion,
            GoalCompletion::Complete
        );
        assert_ne!(factor_trace.ru_id, formula_trace.ru_id);
        assert_ne!(factor_trace.ruo_id, formula_trace.ruo_id);
        counts[0] += 1;

        // L: domain restriction eliminates a genuine root with sourced evidence.
        let restriction = match n % 4 {
            0 => format!("x > {n}"),
            1 => format!("x >= {}", n + 1),
            2 => format!("x != {n}"),
            _ => format!("x in [{},{}]", n + 1, n + 3),
        };
        let restricted = format!("Solve {} where {restriction}.", equation(n));
        let (mut constrained, target, source) = context(&restricted);
        let valid = constrained
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
            ))
            .unwrap();
        assert_eq!(valid.goal_completion, GoalCompletion::Complete);
        assert!(valid
            .evidence
            .iter()
            .any(|e| e.kind == EvidenceKind::ConstraintApplication));
        assert!(!valid
            .errors
            .iter()
            .any(|e| e.kind == EducationalError::CalculationError));
        let rejected = constrained
            .execute(step(&target, &source, answer(n)))
            .unwrap();
        assert_eq!(
            rejected.validity.local_validity,
            MathematicalValidity::Valid
        );
        assert_eq!(rejected.goal_completion, GoalCompletion::Incomplete);
        assert!(rejected
            .errors
            .iter()
            .any(|e| e.kind == EducationalError::ConstraintError));
        counts[1] += 1;

        // M: one correct root is partial, not mathematically invalid.
        let (mut partial, target, source) = context(&text);
        let result = partial
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n],
                    output: ExpectedOutput::SolutionSet,
                },
            ))
            .unwrap();
        assert_eq!(result.validity.local_validity, MathematicalValidity::Valid);
        assert_eq!(result.goal_completion, GoalCompletion::Incomplete);
        assert_eq!(result.intent_alignment, IntentAlignment::Partial);
        assert!(result
            .errors
            .iter()
            .any(|e| e.kind == EducationalError::MissingSolution));
        counts[2] += 1;

        // N: true mathematics about the wrong semantic target is irrelevant.
        let (mut wrong, _, source) = context(&text);
        let other = wrong
            .semantic_state()
            .resolve("problem", "pif", "global")
            .unwrap()
            .common
            .id
            .clone();
        let result = wrong
            .execute(step(
                &other,
                &source,
                Execution::ArithmeticEquality {
                    left: 2 * n,
                    right: n + n,
                },
            ))
            .unwrap();
        assert_eq!(result.validity.local_validity, MathematicalValidity::Valid);
        assert_eq!(result.problem_relevance, ProblemRelevance::Irrelevant);
        assert_eq!(result.intent_alignment, IntentAlignment::Misaligned);
        assert_eq!(result.goal_completion, GoalCompletion::Incomplete);
        counts[3] += 1;

        // O: ambiguous instruction and unsupported mathematics stay unresolved.
        let ambiguous = format!("Solve or factor {}.", equation(n));
        let parsed = parse_problem(&ambiguous).unwrap();
        assert_eq!(
            parsed.definition.intent().intent_status,
            IntentStatus::Ambiguous
        );
        let mut uncertain = ReasoningContext::from_parsed(parsed).unwrap();
        let source = uncertain.definition().intent().source_occurrence_id.clone();
        let target = uncertain
            .semantic_state()
            .resolve("x", "pif", "global")
            .unwrap()
            .common
            .id
            .clone();
        let result = uncertain
            .execute(step(&target, &source, Execution::QuadraticFormula))
            .unwrap();
        assert_eq!(result.goal_completion, GoalCompletion::Unknown);
        assert_eq!(
            result.validity.local_validity,
            MathematicalValidity::Unknown
        );
        match n % 4 {
            0 => assert_eq!(
                parse_problem("Consider the graph.")
                    .unwrap()
                    .definition
                    .intent()
                    .intent_status,
                IntentStatus::Unknown
            ),
            1 => assert!(parse_problem(&format!("Solve x^3 - {n} = 0.")).is_err()),
            2 => {
                let irrational = format!("Solve x^2 - {} = 0.", n * n + 1);
                let (mut ctx, target, source) = context(&irrational);
                let attempt = ctx
                    .execute(step(&target, &source, Execution::QuadraticFormula))
                    .unwrap();
                assert_eq!(
                    attempt.validity.local_validity,
                    MathematicalValidity::Unknown
                );
                let empty = ctx
                    .execute(step(
                        &target,
                        &source,
                        Execution::Answer {
                            values: vec![],
                            output: ExpectedOutput::SolutionSet,
                        },
                    ))
                    .unwrap();
                assert_eq!(empty.validity.local_validity, MathematicalValidity::Unknown);
                assert_eq!(empty.goal_completion, GoalCompletion::Unknown);
            }
            _ => assert!(parse_problem("Solve x^2 + 1000001 = 0.").is_err()),
        }
        counts[4] += 1;

        // P: every decisive evaluation reference resolves; tampering is rejected.
        let (mut evidenced, target, source) = context(&text);
        evidenced
            .execute(step(&target, &source, Execution::QuadraticFormula))
            .unwrap();
        let result = evidenced
            .execute(step(&target, &source, answer(n)))
            .unwrap();
        for item in &result.evidence {
            item.validate(evidenced.semantic_state(), evidenced.definition().intent())
                .unwrap();
        }
        let mut dangling = result.evidence[0].clone();
        dangling.occurrence_refs = vec!["missing".into()];
        assert!(dangling
            .validate(evidenced.semantic_state(), evidenced.definition().intent())
            .is_err());
        let before_rejection = evidenced.semantic_state().version;
        assert!(evidenced
            .execute(step(&target, "missing", answer(n)))
            .is_err());
        assert_eq!(evidenced.semantic_state().version, before_rejection);
        counts[5] += 1;

        // Q: no reference trajectory or fixture answer is used to select a path.
        assert_eq!(
            factor.problem_evaluation().goal_completion,
            formula.problem_evaluation().goal_completion
        );
        assert_eq!(
            factor.definition().intent().semantic_signature(),
            formula.definition().intent().semantic_signature()
        );
        assert_ne!(
            factor.runtime().method_evidence[0].strategy,
            formula.runtime().method_evidence[0].strategy
        );
        counts[6] += 1;

        // R: 20 English + 20 Japanese fully parsed and executed problems.
        for text in [
            format!("Find all solutions of {}.", equation(n)),
            format!("{} のすべての解を求めよ。", equation(n)),
        ] {
            let (mut end_to_end, target, source) = context(&text);
            let executed = end_to_end
                .execute(step(&target, &source, Execution::QuadraticFormula))
                .unwrap();
            assert_eq!(
                executed.validity.local_validity,
                MathematicalValidity::Valid
            );
            assert_eq!(executed.trace.as_ref().unwrap().rus_id, "PIF_QUADRATIC_RUS");
            let completed = end_to_end
                .execute(step(&target, &source, answer(n)))
                .unwrap();
            assert_eq!(completed.goal_completion, GoalCompletion::Complete);
            assert_eq!(
                end_to_end.problem_evaluation().goal_completion,
                GoalCompletion::Complete
            );
            assert_eq!(end_to_end.runtime().evaluation_history.len(), 2);
            for evidence in &completed.evidence {
                evidence
                    .validate(
                        end_to_end.semantic_state(),
                        end_to_end.definition().intent(),
                    )
                    .unwrap();
            }
            counts[7] += 1;
        }
    }
    assert_eq!(counts, [20, 20, 20, 20, 20, 20, 20, 40]);
}

#[test]
fn pif_5_method_constraint_without_execution_is_unknown() {
    let (mut context, target, source) = context("Solve x^2 - 5*x + 6 = 0 using factorization.");
    let result = context
        .execute(ReasoningStep {
            target_ref: target,
            source_occurrence_ids: vec![source],
            declared_method: Some(Method::Factorization),
            execution: Execution::Answer {
                values: vec![2, 3],
                output: ExpectedOutput::SolutionSet,
            },
        })
        .unwrap();
    assert_eq!(
        result.method_compliance,
        mathlang_mirp::intent::MethodCompliance::Unknown
    );
    assert_eq!(result.goal_completion, GoalCompletion::Unknown);
}

#[test]
fn pif_5_frozen_state_and_anti_cheating_source_gate() {
    assert_eq!(mathlang_mirp::VERSION, "mirp/0.1-si2");
    assert_eq!(mathlang_mirp::SemanticState::default().canonical().unwrap(),
        "{\"mirp_version\":\"mirp/0.1-si2\",\"version\":0,\"status\":\"UNKNOWN\",\"objects\":[],\"goals\":[],\"dependencies\":{}}");
    for source in [
        include_str!("../src/pif/schema.rs"),
        include_str!("../src/pif/parser.rs"),
        include_str!("../src/pif/runtime.rs"),
    ] {
        for forbidden in [
            "problem_id ==",
            "fixture_name ==",
            "expected_answer ==",
            "test_group ==",
            "reference_solution",
            "reference_ru_sequence",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden fixture branch: {forbidden}"
            );
        }
    }
}
