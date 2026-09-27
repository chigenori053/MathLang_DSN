use mathlang_mirp::intent::*;
use mathlang_mirp::{Provenance, SemanticDelta, SemanticState};

fn context(n: i64) -> (ReasoningContext, String, String, String, Polynomial) {
    let provenance = Provenance::input("problem", &format!("quadratic case {n}"));
    let mut state = SemanticState::default();
    let x = state
        .bind("x", "lesson", "global", "VARIABLE", provenance.clone())
        .unwrap();
    let x_occurrence = state.get(&x).unwrap().occurrence_id().to_owned();
    let other = Provenance::input("problem", &format!("unrelated case {n}"));
    let y = state
        .bind("y", "lesson", "global", "VARIABLE", other)
        .unwrap();
    let y_occurrence = state.get(&y).unwrap().occurrence_id().to_owned();
    let equation = Polynomial {
        coefficients: [n * (n + 1), -(2 * n + 1), 1],
    };
    let intent = ProblemIntent {
        given: vec![Given {
            equation: equation.clone(),
            provenance,
            source_occurrence_id: x_occurrence.clone(),
        }],
        constraints: vec![],
        target: Target {
            semantic_id: x.clone(),
        },
        operation_intent: OperationIntent::Solve,
        expected_output: ExpectedOutput::SolutionSet,
        method_constraints: vec![],
        completion_criteria: vec![
            CompletionCriterion::AllValidSolutions,
            CompletionCriterion::NoExtraneousSolutions,
            CompletionCriterion::OutputType(ExpectedOutput::SolutionSet),
        ],
    };
    (
        ReasoningContext::new(state, intent).unwrap(),
        x_occurrence,
        y_occurrence,
        y,
        equation,
    )
}

fn step(target: &str, source: &str, claim: Claim, method: Option<Method>) -> ReasoningStep {
    ReasoningStep {
        target: Target {
            semantic_id: target.into(),
        },
        source_occurrence_ids: vec![source.into()],
        method,
        claim,
    }
}

fn solutions(n: i64, output: ExpectedOutput) -> Claim {
    Claim::Solutions {
        values: vec![n, n + 1],
        output,
    }
}

fn algebra_path(n: i64, square: bool) -> AlgebraExpression {
    use AlgebraExpression as E;
    let x_minus = |root| E::Subtract(Box::new(E::Variable), Box::new(E::Constant(root)));
    if square {
        // (2x - (2n+1))² - 1 = 4(x-n)(x-(n+1)).
        E::Subtract(
            Box::new(E::Square(Box::new(E::Subtract(
                Box::new(E::Multiply(Box::new(E::Constant(2)), Box::new(E::Variable))),
                Box::new(E::Constant(2 * n + 1)),
            )))),
            Box::new(E::Constant(1)),
        )
    } else {
        E::Multiply(Box::new(x_minus(n)), Box::new(x_minus(n + 1)))
    }
}

#[test]
fn mandatory_200_case_matrix_is_deterministic() {
    for run in 0..3 {
        let mut checked = 0;
        for n in 1..=20 {
            // A: exact, aligned complete solution.
            let (ctx, source, _, _, _) = context(n);
            let target = &ctx.problem_intent.target.semantic_id;
            let a = ctx.evaluate(&step(
                target,
                &source,
                solutions(n, ExpectedOutput::SolutionSet),
                None,
            ));
            assert_eq!(a.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(a.problem_relevance, ProblemRelevance::Relevant);
            assert_eq!(a.intent_alignment, IntentAlignment::Aligned);
            assert_eq!(a.goal_completion, GoalCompletion::Complete);
            checked += 1;

            // B: true arithmetic from a different source is irrelevant.
            let (_, _, unrelated, _, _) = context(n);
            let b = ctx.evaluate(&step(
                target,
                &unrelated,
                Claim::ArithmeticEquality {
                    left: n + n,
                    right: 2 * n,
                },
                None,
            ));
            assert_eq!(b.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(b.problem_relevance, ProblemRelevance::Irrelevant);
            checked += 1;

            // C: the intended algebraic operation is executed incorrectly.
            let (_, _, _, _, equation) = context(n);
            let mut wrong = equation.clone();
            wrong.coefficients[0] += 1;
            let c = ctx.evaluate(&step(
                target,
                &source,
                Claim::Transformation {
                    before: equation.clone(),
                    after: wrong,
                },
                None,
            ));
            assert_eq!(c.mathematical_validity, MathematicalValidity::Invalid);
            assert_eq!(c.problem_relevance, ProblemRelevance::Relevant);
            assert!(c
                .errors
                .contains(&EducationalError::AlgebraicTransformationError));
            checked += 1;

            // D: a valid root is not the whole solution set.
            let d = ctx.evaluate(&step(
                target,
                &source,
                Claim::Solutions {
                    values: vec![n],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ));
            assert_eq!(d.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(d.intent_alignment, IntentAlignment::Partial);
            assert_eq!(d.goal_completion, GoalCompletion::Incomplete);
            assert!(d.errors.contains(&EducationalError::MissingSolution));
            checked += 1;

            // E: independent valid methods lead to identical semantic success.
            let method = match n % 3 {
                0 => Method::Factorization,
                1 => Method::QuadraticFormula,
                _ => Method::CompletingSquare,
            };
            let (mut alternative, _, _, _, _) = context(n);
            if method != Method::QuadraticFormula {
                let intermediate = alternative.accept(step(
                    target,
                    &source,
                    Claim::ExpressionTransformation {
                        before: equation.clone(),
                        after: algebra_path(n, method == Method::CompletingSquare),
                    },
                    Some(method.clone()),
                ));
                assert_eq!(
                    intermediate.mathematical_validity,
                    MathematicalValidity::Valid
                );
                assert_eq!(intermediate.goal_completion, GoalCompletion::Incomplete);
            }
            let e = alternative.evaluate(&step(
                target,
                &source,
                solutions(n, ExpectedOutput::SolutionSet),
                Some(method),
            ));
            assert_eq!(e.goal_completion, GoalCompletion::Complete);
            assert_eq!(e.method_compliance, MethodCompliance::NotRequired);
            checked += 1;

            // F: method violation never changes mathematical validity.
            let (mut method_ctx, _, _, _, _) = context(n);
            method_ctx
                .problem_intent
                .method_constraints
                .push(MethodConstraint {
                    required: Method::Factorization,
                    provenance: method_ctx.problem_intent.given[0].provenance.clone(),
                    source_occurrence_id: source.clone(),
                });
            let used = if n % 2 == 0 {
                Method::Factorization
            } else {
                Method::QuadraticFormula
            };
            let f = method_ctx.evaluate(&step(
                target,
                &source,
                solutions(n, ExpectedOutput::SolutionSet),
                Some(used),
            ));
            assert_eq!(f.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(
                f.method_compliance,
                if n % 2 == 0 {
                    MethodCompliance::Satisfied
                } else {
                    MethodCompliance::Violation
                }
            );
            checked += 1;

            // G: a root eliminated by a sourced domain constraint is retained in the trace.
            let (mut constrained, _, _, _, _) = context(n);
            let constraint = match n % 4 {
                0 => ConstraintKind::GreaterThan(n),
                1 => ConstraintKind::GreaterOrEqual(n + 1),
                2 => ConstraintKind::NotEqual(n),
                _ => ConstraintKind::ClosedInterval(n + 1, n + 3),
            };
            constrained.problem_intent.constraints.push(Constraint {
                kind: constraint.clone(),
                provenance: constrained.problem_intent.given[0].provenance.clone(),
                source_occurrence_id: source.clone(),
            });
            let g = constrained.evaluate(&step(
                target,
                &source,
                solutions(n, ExpectedOutput::SolutionSet),
                None,
            ));
            assert_eq!(g.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(g.goal_completion, GoalCompletion::Incomplete);
            assert_eq!(g.eliminations[0].candidate, n);
            assert_eq!(g.eliminations[0].constraint, constraint);
            assert_eq!(g.eliminations[0].constraint_occurrence_id, source);
            assert!(g.errors.contains(&EducationalError::ConstraintError));
            checked += 1;

            // H: a correct result addressed to a different semantic target.
            let (_, _, _, wrong_target, _) = context(n);
            let h = ctx.evaluate(&step(
                &wrong_target,
                &source,
                solutions(n, ExpectedOutput::SolutionSet),
                None,
            ));
            assert_eq!(h.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(h.problem_relevance, ProblemRelevance::Irrelevant);
            assert!(h.errors.contains(&EducationalError::WrongTarget));
            checked += 1;

            // I: a valid transformed equation is still intermediate.
            let i = ctx.evaluate(&step(
                target,
                &source,
                Claim::Transformation {
                    before: equation.clone(),
                    after: equation,
                },
                Some(Method::Factorization),
            ));
            assert_eq!(i.mathematical_validity, MathematicalValidity::Valid);
            assert_eq!(i.intent_alignment, IntentAlignment::Aligned);
            assert_eq!(i.goal_completion, GoalCompletion::Incomplete);
            checked += 1;

            // J: non-integral roots or oversized coefficients preserve UNKNOWN.
            let (mut unknown_ctx, _, _, _, _) = context(n);
            unknown_ctx.problem_intent.given[0].equation = Polynomial {
                coefficients: if n % 2 == 0 {
                    [1_000_001, 0, 1]
                } else {
                    [-(n * n + 1), 0, 1]
                },
            };
            let j = unknown_ctx.evaluate(&step(
                target,
                &source,
                Claim::Solutions {
                    values: vec![],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ));
            assert_eq!(j.goal_completion, GoalCompletion::Unknown);
            if n % 2 == 0 {
                assert_eq!(j.mathematical_validity, MathematicalValidity::Unknown);
            }
            checked += 1;
        }
        assert_eq!(checked, 200, "run {run}");
    }
}

#[test]
fn provenance_output_constraints_and_history_are_inspectable() {
    let (mut ctx, source, _, _, equation) = context(2);
    let target = ctx.problem_intent.target.semantic_id.clone();
    let transformed = step(
        &target,
        &source,
        Claim::Transformation {
            before: equation.clone(),
            after: equation,
        },
        Some(Method::Factorization),
    );
    assert_eq!(
        ctx.accept(transformed).goal_completion,
        GoalCompletion::Incomplete
    );
    assert_eq!(ctx.reasoning_history.len(), 1);
    ctx.problem_intent
        .method_constraints
        .push(MethodConstraint {
            required: Method::Factorization,
            provenance: ctx.problem_intent.given[0].provenance.clone(),
            source_occurrence_id: source.clone(),
        });
    let answer = ctx.evaluate(&step(
        &target,
        &source,
        solutions(2, ExpectedOutput::SolutionSet),
        None,
    ));
    assert_eq!(answer.method_compliance, MethodCompliance::Satisfied);
    assert_eq!(answer.evidence_occurrence_ids, vec![source.clone()]);
    let wrong_type = ctx.evaluate(&step(
        &target,
        &source,
        solutions(2, ExpectedOutput::Solution),
        None,
    ));
    assert_eq!(
        wrong_type.mathematical_validity,
        MathematicalValidity::Valid
    );
    assert_eq!(wrong_type.goal_completion, GoalCompletion::Incomplete);
    let serialized = serde_json::to_string(&ctx.problem_intent).unwrap();
    assert_eq!(
        serde_json::from_str::<ProblemIntent>(&serialized).unwrap(),
        ctx.problem_intent
    );
    let problem = Problem {
        statement_provenance: ctx.problem_intent.given[0].provenance.clone(),
        statement_occurrence_id: source.clone(),
        intent: ctx.problem_intent.clone(),
    };
    problem.validate(&ctx.semantic_state).unwrap();
    assert_eq!(
        serde_json::to_string(&answer).unwrap(),
        serde_json::to_string(&answer).unwrap()
    );
    let previous_version = ctx.semantic_state.version;
    let accepted = ctx
        .accept_transition(
            SemanticDelta::default(),
            step(
                &target,
                &source,
                solutions(2, ExpectedOutput::SolutionSet),
                None,
            ),
        )
        .unwrap();
    assert_eq!(accepted.goal_completion, GoalCompletion::Complete);
    assert_eq!(ctx.semantic_state.version, previous_version + 1);
    assert_eq!(ctx.reasoning_history.len(), 2);
    let unrelated = ctx.evaluate(&step(
        &target,
        &source,
        Claim::Transformation {
            before: Polynomial {
                coefficients: [42, 1, 0],
            },
            after: Polynomial {
                coefficients: [42, 1, 0],
            },
        },
        None,
    ));
    assert_eq!(unrelated.mathematical_validity, MathematicalValidity::Valid);
    assert_eq!(unrelated.problem_relevance, ProblemRelevance::Unknown);
    let out_of_scope = ctx.evaluate(&step(
        &target,
        &source,
        Claim::Transformation {
            before: Polynomial {
                coefficients: [i64::MIN, 0, 0],
            },
            after: Polynomial {
                coefficients: [i64::MIN, 0, 0],
            },
        },
        None,
    ));
    assert_eq!(
        out_of_scope.mathematical_validity,
        MathematicalValidity::Unknown
    );
    let mut bad = ctx.problem_intent.clone();
    bad.given[0].source_occurrence_id = "missing".into();
    assert!(bad.validate(&ctx.semantic_state).is_err());
}
