//! Independent-process canonical digest for 100 native end-to-end cases.
use mathlang_mirp::native::MathProblemContext;
use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    for i in 0..100_i64 {
        let first = i % 17 - 8;
        let mut second = (i * 11) % 19 - 9;
        if first == second {
            second += 1;
        }
        let scale = 1 + i % 3;
        let input = format!(
            "Solve {scale}*x^2 + {}*x + {} = 0.",
            -scale * (first + second),
            scale * first * second
        );
        let mut context = MathProblemContext::parse(&input)?;
        let intent = context.definition().intent().clone();
        let plan = context.plan().clone();
        let result = context.execute(None)?;
        let payload = serde_json::to_vec(&serde_json::json!({
            "input": input, "intent": intent, "plan": plan,
            "selected_ru": result.trace.iter().map(|t| &t.ru_ref).collect::<Vec<_>>(),
            "rus": result.trace.iter().map(|t| &t.rus_ref).collect::<Vec<_>>(),
            "ruo_trace": result.trace, "semantic_state": context.semantic_state(),
            "problem_evaluation": result.problem_evaluation,
        }))?;
        hasher.update((payload.len() as u64).to_be_bytes());
        hasher.update(payload);
    }
    println!("{:x}", hasher.finalize());
    Ok(())
}
