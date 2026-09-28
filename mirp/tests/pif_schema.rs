use mathlang_mirp::pif::*;

fn problem(n: i64) -> ParseOutcome {
    parse_problem(&format!(
        "Solve x^2 - {}*x + {} = 0.",
        2 * n + 1,
        n * (n + 1)
    ))
    .unwrap()
}

#[test]
fn pif_1_schema_positive_and_negative_cases() {
    for n in 1..=20 {
        let parsed = problem(n);
        let mut intent = parsed.definition.intent().clone();
        intent.validate(&parsed.state).unwrap();
        assert_eq!(intent.schema_id, INTENT_SCHEMA);
        assert_eq!(intent.intent_status, IntentStatus::Known);
        let id = intent.problem_id.clone();
        let encoded = serde_json::to_string(&intent).unwrap();
        assert_eq!(
            serde_json::from_str::<ProblemIntent>(&encoded).unwrap(),
            intent
        );
        assert_eq!(parsed.state.mirp_version, "mirp/0.1-si2");
        assert!(!id.is_empty());
        match n % 8 {
            0 => intent.schema_id = "mathlang/problem-intent/1.0".into(),
            1 => intent.primary_goal = None,
            2 => intent.primary_goal.as_mut().unwrap().target_refs.clear(),
            3 => intent.primary_goal.as_mut().unwrap().target_refs = vec!["missing".into()],
            4 => intent.source_occurrence_id = "missing".into(),
            5 => intent
                .completion_criteria
                .retain(|c| c.kind != CriterionKind::AllSolutionsFound),
            6 => intent.provenance.source_id = "tampered".into(),
            _ => intent.completion_criteria[1].id = intent.completion_criteria[0].id.clone(),
        }
        assert!(intent.validate(&parsed.state).is_err(), "negative case {n}");
    }
}

#[test]
fn pif_1_goal_graph_accepts_alternatives_and_rejects_bad_links() {
    for n in 1..=20 {
        let parsed = problem(n);
        let mut intent = parsed.definition.intent().clone();
        let primary = intent.primary_goal.as_ref().unwrap().clone();
        let mut subgoal = primary.clone();
        subgoal.id = format!("{}:sub:{n}", primary.id);
        subgoal.kind = GoalKind::Verify;
        subgoal.parent_goal = Some(primary.id.clone());
        subgoal.dependencies.clear();
        intent.subgoals.push(subgoal.clone());
        intent.primary_goal.as_mut().unwrap().dependencies = vec![subgoal.id.clone()];
        intent.validate(&parsed.state).unwrap();
        let mut alternative = intent.clone();
        alternative
            .primary_goal
            .as_mut()
            .unwrap()
            .dependencies
            .clear();
        alternative.validate(&parsed.state).unwrap();
        assert_ne!(
            intent.semantic_signature(),
            alternative.semantic_signature()
        );
        let mut bad = intent.clone();
        match n % 4 {
            0 => bad.subgoals[0].dependencies = vec![subgoal.id],
            1 => bad.subgoals[0].dependencies = vec!["missing".into()],
            2 => bad.subgoals[0].dependencies = vec![primary.id],
            _ => bad.subgoals[0].parent_goal = Some("missing".into()),
        }
        assert!(bad.validate(&parsed.state).is_err(), "graph negative {n}");
    }
}
