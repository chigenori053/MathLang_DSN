use mathlang_mirp::intent::{
    EducationalError, ExpectedOutput, GoalCompletion, MathematicalValidity, Method,
    MethodCompliance, Polynomial,
};
use mathlang_mirp::pif::*;

fn source(n: i64, method: bool) -> String {
    format!(
        "Solve x^2 - {}*x + {} = 0{}.",
        2 * n + 1,
        n * (n + 1),
        if method { " using factorization" } else { "" }
    )
}

fn setup(n: i64, method: bool) -> (ReasoningContext, String, String) {
    let context =
        ReasoningContext::from_parsed(parse_problem(&source(n, method)).unwrap()).unwrap();
    let intent = context.definition().intent();
    let target = intent.primary_goal.as_ref().unwrap().target_refs[0].clone();
    let occurrence = intent.source_occurrence_id.clone();
    (context, target, occurrence)
}

fn step(
    target: &str,
    source: &str,
    execution: Execution,
    declared_method: Option<Method>,
) -> ReasoningStep {
    ReasoningStep {
        target_ref: target.into(),
        source_occurrence_ids: vec![source.into()],
        declared_method,
        execution,
    }
}

#[test]
fn pif_2_definition_is_immutable_and_runtime_criteria_change() {
    for n in 1..=20 {
        let (mut context, target, source) = setup(n, true);
        let definition = context.definition().clone();
        let before = context.runtime().clone();
        let factor = context
            .execute(step(
                &target,
                &source,
                Execution::Factor { roots: [n, n + 1] },
                None,
            ))
            .unwrap();
        assert_eq!(factor.validity.local_validity, MathematicalValidity::Valid);
        assert_eq!(factor.method_compliance, MethodCompliance::Satisfied);
        assert_eq!(factor.goal_completion, GoalCompletion::Incomplete);
        assert_eq!(context.definition(), &definition);
        assert_ne!(context.runtime(), &before);
        let partial = context
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ))
            .unwrap();
        assert_eq!(partial.goal_completion, GoalCompletion::Incomplete);
        assert!(partial
            .errors
            .iter()
            .any(|e| e.kind == EducationalError::MissingSolution));
        let criteria = context
            .runtime()
            .criterion_status
            .values()
            .cloned()
            .collect::<Vec<_>>();
        assert!(criteria.contains(&CriterionStatus::Unresolved));
        let complete = context
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ))
            .unwrap();
        assert_eq!(complete.goal_completion, GoalCompletion::Complete);
        assert_eq!(context.definition(), &definition);
        assert_eq!(
            context.problem_evaluation().goal_completion,
            GoalCompletion::Complete
        );
    }
}

#[test]
fn pif_2_local_global_validity_and_error_origin() {
    for n in 1..=20 {
        let (mut context, target, source) = setup(n, false);
        let initial = context.definition().intent().givens[0].equation.clone();
        let bad = Polynomial {
            coefficients: [
                initial.coefficients[0] + 1,
                initial.coefficients[1],
                initial.coefficients[2],
            ],
        };
        let doubled = Polynomial {
            coefficients: bad.coefficients.map(|x| 2 * x),
        };
        let tripled = Polynomial {
            coefficients: bad.coefficients.map(|x| 3 * x),
        };
        let first = context
            .execute(step(
                &target,
                &source,
                Execution::Transform {
                    before: initial,
                    after: bad.clone(),
                },
                None,
            ))
            .unwrap();
        assert_eq!(first.validity.local_validity, MathematicalValidity::Invalid);
        assert_eq!(
            first.validity.global_derivational_validity,
            GlobalDerivationalValidity::InvalidDerivation
        );
        let second = context
            .execute(step(
                &target,
                &source,
                Execution::Transform {
                    before: bad,
                    after: doubled.clone(),
                },
                None,
            ))
            .unwrap();
        let third = context
            .execute(step(
                &target,
                &source,
                Execution::Transform {
                    before: doubled,
                    after: tripled,
                },
                None,
            ))
            .unwrap();
        for later in [&second, &third] {
            assert_eq!(later.validity.local_validity, MathematicalValidity::Valid);
            assert_eq!(
                later.validity.global_derivational_validity,
                GlobalDerivationalValidity::InvalidDerivation
            );
            assert_eq!(later.errors[0].origin_transition, first.id);
            assert!(!later.errors[0].primary);
        }
        let problem = context.problem_evaluation();
        assert_eq!(problem.primary_errors.len(), 1);
        assert_eq!(problem.downstream_errors.len(), 2);
    }
}

#[test]
fn pif_4_executed_method_wins_over_declaration() {
    for n in 1..=20 {
        let (mut factor_context, target, source) = setup(n, true);
        let unknown = factor_context
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                Some(Method::Factorization),
            ))
            .unwrap();
        assert_eq!(unknown.validity.local_validity, MathematicalValidity::Valid);
        assert_eq!(unknown.method_compliance, MethodCompliance::Unknown);
        assert_eq!(unknown.goal_completion, GoalCompletion::Unknown);
        let executed = factor_context
            .execute(step(
                &target,
                &source,
                Execution::QuadraticFormula,
                Some(Method::Factorization),
            ))
            .unwrap();
        assert_eq!(
            executed.validity.local_validity,
            MathematicalValidity::Valid
        );
        assert_eq!(executed.method_compliance, MethodCompliance::Violation);
        assert_eq!(executed.trace.as_ref().unwrap().ru_id, "QUADRATIC_FORMULA");
        let downstream = factor_context
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                Some(Method::Factorization),
            ))
            .unwrap();
        let method_error = downstream
            .errors
            .iter()
            .find(|e| e.kind == EducationalError::MethodViolation)
            .unwrap();
        assert_eq!(method_error.origin_transition, executed.id);
        assert!(!method_error.primary);
        let (mut correct, target, source) = setup(n, true);
        let factor = correct
            .execute(step(
                &target,
                &source,
                Execution::Factor { roots: [n, n + 1] },
                Some(Method::QuadraticFormula),
            ))
            .unwrap();
        assert_eq!(factor.method_compliance, MethodCompliance::Satisfied);
        assert_eq!(factor.trace.as_ref().unwrap().ru_id, "FACTOR_PRODUCT");
        let answer = correct
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ))
            .unwrap();
        assert_eq!(answer.goal_completion, GoalCompletion::Complete);
        assert_eq!(correct.runtime().method_evidence[0].ruo_refs.len(), 1);
        let parsed = parse_problem(&format!(
            "Solve x^2 - {}*x + {} = 0.",
            2 * n + 1,
            n * (n + 1)
        ))
        .unwrap();
        let mut definition = parsed.definition.intent().clone();
        let problem_source = definition.source_occurrence_id.clone();
        let opposite_source = problem_source.clone();
        let provenance = definition.provenance.clone();
        definition.method_constraints.push(MethodRequirement {
            id: format!("require-formula-{n}"),
            required: Method::QuadraticFormula,
            source_occurrence_id: problem_source.clone(),
            provenance: provenance.clone(),
        });
        definition.completion_criteria.push(CompletionCriterion {
            id: format!("method-criterion-{n}"),
            kind: CriterionKind::MethodUsed,
            target_refs: vec![target.clone()],
            quantifier: Quantifier::All,
            required: true,
            source_occurrence_id: problem_source,
            provenance,
        });
        let frozen = ProblemDefinition::new(definition, &parsed.state).unwrap();
        let mut opposite = ReasoningContext::new(frozen, parsed.state).unwrap();
        let mismatch = opposite
            .execute(step(
                &target,
                &opposite_source,
                Execution::Factor { roots: [n, n + 1] },
                Some(Method::QuadraticFormula),
            ))
            .unwrap();
        assert_eq!(
            mismatch.validity.local_validity,
            MathematicalValidity::Valid
        );
        assert_eq!(mismatch.method_compliance, MethodCompliance::Violation);
    }
}

#[test]
fn pif_2_subgoal_satisfaction_does_not_complete_primary_goal() {
    for n in 1..=20 {
        let parsed = parse_problem(&source(n, false)).unwrap();
        let mut intent = parsed.definition.intent().clone();
        let primary = intent.primary_goal.as_ref().unwrap().clone();
        let target = primary.target_refs[0].clone();
        let source = intent.source_occurrence_id.clone();
        let mut subgoal = primary.clone();
        subgoal.id = format!("{}:factor", primary.id);
        subgoal.kind = GoalKind::Factor;
        subgoal.parent_goal = Some(primary.id.clone());
        subgoal.dependencies.clear();
        intent.primary_goal.as_mut().unwrap().dependencies = vec![subgoal.id.clone()];
        intent.subgoals.push(subgoal.clone());
        let definition = ProblemDefinition::new(intent, &parsed.state).unwrap();
        let mut context = ReasoningContext::new(definition, parsed.state).unwrap();
        let result = context
            .execute(step(
                &target,
                &source,
                Execution::Factor { roots: [n, n + 1] },
                None,
            ))
            .unwrap();
        assert_eq!(result.goal_completion, GoalCompletion::Incomplete);
        assert_eq!(
            context.runtime().goal_status[&subgoal.id],
            GoalStatus::Satisfied
        );
        assert_eq!(
            context.runtime().goal_status[&primary.id],
            GoalStatus::Unresolved
        );
        let final_step = context
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ))
            .unwrap();
        assert_eq!(final_step.goal_completion, GoalCompletion::Complete);
    }
}

#[test]
fn pif_2_each_required_criterion_controls_completion() {
    for n in 1..=20 {
        let parsed = parse_problem(&source(n, true)).unwrap();
        let mut intent = parsed.definition.intent().clone();
        let target = intent.primary_goal.as_ref().unwrap().target_refs[0].clone();
        let source = intent.source_occurrence_id.clone();
        for kind in [CriterionKind::TargetDerived, CriterionKind::DomainResolved] {
            intent.completion_criteria.push(CompletionCriterion {
                id: format!("extra-{kind:?}-{n}"),
                kind,
                target_refs: vec![target.clone()],
                quantifier: Quantifier::All,
                required: true,
                source_occurrence_id: source.clone(),
                provenance: intent.provenance.clone(),
            });
        }
        let definition = ProblemDefinition::new(intent, &parsed.state).unwrap();
        let mut ctx = ReasoningContext::new(definition, parsed.state).unwrap();
        ctx.execute(step(
            &target,
            &source,
            Execution::Answer {
                values: vec![n],
                output: ExpectedOutput::SolutionSet,
            },
            None,
        ))
        .unwrap();
        let status = |context: &ReasoningContext, kind: CriterionKind| {
            let id = &context
                .definition()
                .intent()
                .completion_criteria
                .iter()
                .find(|c| c.kind == kind)
                .unwrap()
                .id;
            context.runtime().criterion_status[id].clone()
        };
        assert_eq!(
            status(&ctx, CriterionKind::AllSolutionsFound),
            CriterionStatus::Unresolved
        );
        assert_eq!(
            status(&ctx, CriterionKind::NoExtraneousSolutions),
            CriterionStatus::Satisfied
        );
        assert_eq!(
            status(&ctx, CriterionKind::ConstraintsSatisfied),
            CriterionStatus::Satisfied
        );
        assert_eq!(
            status(&ctx, CriterionKind::OutputTypeMatched),
            CriterionStatus::Satisfied
        );
        assert_eq!(
            status(&ctx, CriterionKind::MethodUsed),
            CriterionStatus::Unknown
        );
        assert_eq!(
            status(&ctx, CriterionKind::TargetDerived),
            CriterionStatus::Satisfied
        );
        assert_eq!(
            status(&ctx, CriterionKind::DomainResolved),
            CriterionStatus::Satisfied
        );
        let wrong = ctx
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::Solution,
                },
                None,
            ))
            .unwrap();
        assert_eq!(wrong.goal_completion, GoalCompletion::Unknown);
        assert_eq!(
            status(&ctx, CriterionKind::OutputTypeMatched),
            CriterionStatus::Violated
        );
        ctx.execute(step(
            &target,
            &source,
            Execution::Factor { roots: [n, n + 1] },
            None,
        ))
        .unwrap();
        assert_eq!(
            ctx.problem_evaluation().goal_completion,
            GoalCompletion::Incomplete
        );
        let final_step = ctx
            .execute(step(
                &target,
                &source,
                Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                },
                None,
            ))
            .unwrap();
        assert_eq!(final_step.goal_completion, GoalCompletion::Complete);
    }
}
