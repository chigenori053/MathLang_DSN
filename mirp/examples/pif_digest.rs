use mathlang_mirp::intent::{ExpectedOutput, GoalCompletion, MathematicalValidity};
use mathlang_mirp::pif::{parse_problem, Execution, ReasoningContext, ReasoningStep};
use sha2::{Digest, Sha256};

fn main() {
    let mut cases = vec![];
    for n in 1..=20 {
        let equation = format!("x^2 - {}*x + {} = 0", 2 * n + 1, n * (n + 1));
        for text in [
            format!("Find all solutions of {equation}."),
            format!("{equation} のすべての解を求めよ。"),
        ] {
            let parsed = parse_problem(&text).expect("bounded problem parses");
            let signature = parsed.definition.intent().semantic_signature();
            let full_intent = serde_json::to_value(parsed.definition.intent()).unwrap();
            let target = parsed
                .definition
                .intent()
                .primary_goal
                .as_ref()
                .unwrap()
                .target_refs[0]
                .clone();
            let source = parsed.definition.intent().source_occurrence_id.clone();
            let mut context = ReasoningContext::from_parsed(parsed).unwrap();
            let execute = |execution| ReasoningStep {
                target_ref: target.clone(),
                source_occurrence_ids: vec![source.clone()],
                declared_method: None,
                execution,
            };
            let method = context
                .execute(execute(Execution::QuadraticFormula))
                .unwrap();
            assert_eq!(method.validity.local_validity, MathematicalValidity::Valid);
            let answer = context
                .execute(execute(Execution::Answer {
                    values: vec![n, n + 1],
                    output: ExpectedOutput::SolutionSet,
                }))
                .unwrap();
            assert_eq!(answer.goal_completion, GoalCompletion::Complete);
            cases.push(serde_json::json!({
                "semantic_intent": signature, "full_intent": full_intent,
                "goal_graph": context.definition().intent().primary_goal,
                "criteria": context.definition().intent().completion_criteria,
                "runtime": context.runtime(), "method": method,
                "answer": answer, "problem_evaluation": context.problem_evaluation(),
            }));
        }
    }
    let digest = Sha256::digest(serde_json::to_vec(&cases).unwrap());
    println!(
        "{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
