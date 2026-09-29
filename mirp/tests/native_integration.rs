use mathlang_mirp::native::{MathProblemContext, ReasoningStatus, RuntimeGoal, Strategy};
use mathlang_mirp::{Object, Value};

fn roots(i: i64) -> (i64, i64) {
    let first = i % 9 - 4;
    let mut second = (i * 7) % 11 - 5;
    if second == first {
        second += 1;
    }
    (first.min(second), first.max(second))
}
fn problem(i: i64, language: &str, suffix: &str) -> String {
    let (r1, r2) = roots(i);
    let scale = 1 + i % 2;
    let equation = format!(
        "{scale}*x^2 + {}*x + {} = 0",
        -scale * (r1 + r2),
        scale * r1 * r2
    );
    if language == "ja" {
        format!("{equation}{suffix} を解け")
    } else {
        format!("Solve {equation}{suffix}.")
    }
}

#[test]
fn required_e2e() {
    let mut ordinary = MathProblemContext::parse("Solve x² - 5x + 6 = 0.").unwrap();
    assert_eq!(ordinary.execute(None).unwrap().answer, Some(vec![2, 3]));
    let mut factor =
        MathProblemContext::parse("Solve x² - 5x + 6 = 0 using factorization.").unwrap();
    let result = factor.execute(None).unwrap();
    assert_eq!(result.status, ReasoningStatus::Complete);
    assert!(result
        .trace
        .iter()
        .any(|t| t.rus_ref == "QuadraticFactorizationRUS"));
    assert_eq!(factor.intent_runtime().method_evidence.len(), 1);
    assert_eq!(
        factor.intent_runtime().method_evidence[0].transition_refs,
        vec![factor.history()[0].id.clone()]
    );
    factor.semantic_state().validate().unwrap();
    assert_eq!(factor.semantic_state().mirp_version, "mirp/0.1-si2");
    let mut violation =
        MathProblemContext::parse("Solve x² - 5x + 6 = 0 using factorization.").unwrap();
    let result = violation.execute(Some(Strategy::QuadraticFormula)).unwrap();
    assert_eq!(result.problem_evaluation.mathematical_status, "VALID");
    assert_eq!(result.problem_evaluation.method_compliance, "VIOLATION");
    assert_ne!(result.status, ReasoningStatus::Complete);
    let mut constrained = MathProblemContext::parse("Solve x² - 5x + 6 = 0 where x > 2.").unwrap();
    assert_eq!(constrained.execute(None).unwrap().answer, Some(vec![3]));
    let mut ambiguous = MathProblemContext::parse("Solve or factor x² - 5x + 6 = 0.").unwrap();
    let result = ambiguous.execute(None).unwrap();
    assert_eq!(result.status, ReasoningStatus::Ambiguous);
    assert!(result.trace.is_empty());
}

#[test]
fn mandatory_520_case_matrix() {
    let mut checks = 0;
    for i in 0..40 {
        let (r1, r2) = roots(i);
        let mut context =
            MathProblemContext::parse(&problem(i, "en", " using factorization")).unwrap();
        // A: pipeline construction
        assert!(!context.definition().intent().problem_id.is_empty());
        checks += 1;
        // B: goal bridge, covering four mapped goals
        let (bridge_input, mapped) = match i % 4 {
            0 => (problem(i, "en", ""), RuntimeGoal::Solve),
            1 => (
                format!("Factor x^2 + {}*x + {}.", -(r1 + r2), r1 * r2),
                RuntimeGoal::Factorization,
            ),
            2 => (
                format!("Expand (x - {r1})*(x - {r2})."),
                RuntimeGoal::Expansion,
            ),
            _ => (
                format!(
                    "Verify that x = {r1} is a solution of x^2 + {}*x + {} = 0.",
                    -(r1 + r2),
                    r1 * r2
                ),
                RuntimeGoal::Verification,
            ),
        };
        assert_eq!(
            MathProblemContext::parse(&bridge_input)
                .unwrap()
                .plan()
                .runtime_goal,
            mapped
        );
        checks += 1;
        // C: planner
        assert_eq!(context.plan().candidate_strategies.len(), 2);
        checks += 1;
        let result = context.execute(None).unwrap();
        // D: RU selection
        assert!(result.trace.iter().any(|t| t.knowledge_ref.as_deref()
            == Some("JH_QUADRATIC_INTEGER_FACTOR")
            && t.ru_ref.starts_with("ru:")));
        checks += 1;
        // E: RUS construction
        assert!(result
            .trace
            .iter()
            .any(|t| t.rus_ref == "QuadraticFactorizationRUS"));
        checks += 1;
        // F: RUO execution
        assert_eq!(result.answer, Some(vec![r1, r2]));
        checks += 1;
        // G: SemanticState transition
        let solution = context
            .semantic_state()
            .resolve("solution_set", "pini", &context.plan().problem_id)
            .unwrap();
        assert!(matches!(
            context
                .semantic_state()
                .value_of(&solution.common.id)
                .unwrap()
                .unwrap()
                .0,
            Value::Set(_)
        ));
        checks += 1;
        // H: evidence bridge
        assert!(result.evidence.iter().all(|e| !e.ru_refs[0].is_empty()
            && !e.rus_refs[0].is_empty()
            && context.semantic_state().get(&e.semantic_refs[0]).is_some()
            && context
                .semantic_state()
                .get_occurrence(&e.occurrence_refs[0])
                .is_some()
            && e.goal_refs == vec![context.plan().goal_ref.clone()]
            && e.criterion_refs.iter().all(|id| context
                .definition()
                .intent()
                .completion_criteria
                .iter()
                .any(|c| &c.id == id))
            && e.ruo_refs
                .iter()
                .all(|id| result.trace.iter().any(|t| &t.ruo_ref == id))
            && e.state_refs[0].parse::<u64>().unwrap() <= context.semantic_state().version));
        checks += 1;
        // I: method compliance in both directions
        let (required, selected) = if i % 2 == 0 {
            (" using factorization", Strategy::QuadraticFormula)
        } else {
            (" using quadratic formula", Strategy::Factorization)
        };
        let mut wrong_method = MathProblemContext::parse(&problem(i, "en", required)).unwrap();
        let wrong_result = wrong_method.execute(Some(selected)).unwrap();
        assert_eq!(
            wrong_result.problem_evaluation.method_compliance,
            "VIOLATION"
        );
        checks += 1;
        // J: partial answer
        assert_eq!(
            context
                .evaluate_submission(
                    &context
                        .definition()
                        .intent()
                        .primary_goal
                        .as_ref()
                        .unwrap()
                        .target_refs[0],
                    &[r1]
                )
                .unwrap()
                .status,
            ReasoningStatus::Partial
        );
        checks += 1;
        // K: wrong target
        assert_eq!(
            context
                .evaluate_submission("wrong-target", &[r1, r2])
                .unwrap()
                .problem_evaluation
                .problem_relevance,
            "WRONG_TARGET"
        );
        checks += 1;
        // L: constraint filtering on native result
        let mut constrained =
            MathProblemContext::parse(&problem(i, "en", &format!(" where x > {r1}"))).unwrap();
        assert_eq!(constrained.execute(None).unwrap().answer, Some(vec![r2]));
        checks += 1;
        assert!(context
            .semantic_state()
            .objects
            .iter()
            .any(|o| matches!(o, Object::Relation(_))));
    }
    for i in 0..20 {
        let mut en = MathProblemContext::parse(&problem(i, "en", "")).unwrap();
        assert_eq!(en.execute(None).unwrap().status, ReasoningStatus::Complete);
        checks += 1;
        let mut ja = MathProblemContext::parse(&problem(i, "ja", "")).unwrap();
        assert_eq!(ja.execute(None).unwrap().status, ReasoningStatus::Complete);
        checks += 1;
    }
    assert_eq!(checks, 520);
}

#[test]
fn nonmonic_metamorphic() {
    let mut monic = MathProblemContext::parse("Solve x^2 - 5*x + 6 = 0.").unwrap();
    let mut scaled = MathProblemContext::parse("Solve 2*x^2 - 10*x + 12 = 0.").unwrap();
    assert_eq!(
        monic.execute(None).unwrap().answer,
        scaled.execute(None).unwrap().answer
    );
}

#[test]
fn status_separation_and_strategy_fallback() {
    let mut no_real = MathProblemContext::parse("Solve x^2 + 1 = 0.").unwrap();
    let result = no_real.execute(None).unwrap();
    assert_eq!(result.status, ReasoningStatus::Complete);
    assert_eq!(result.answer, Some(vec![]));
    assert!(result.trace.iter().any(|t| t.knowledge_ref.as_deref()
        == Some("JH_QUADRATIC_FORMULA")
        && t.ru_ref.starts_with("ru:")));

    let mut unsupported = MathProblemContext::parse("Factor x^2 - 5*x + 6.").unwrap();
    assert_eq!(
        unsupported.execute(None).unwrap().status,
        ReasoningStatus::Unsupported
    );
    let mut unknown = MathProblemContext::parse("Explain why x^2 = 1.").unwrap();
    assert_eq!(
        unknown.execute(None).unwrap().status,
        ReasoningStatus::Unknown
    );
}
