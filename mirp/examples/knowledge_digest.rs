//! Canonical digest for 100 independent MemorySpace query → RU → MIRP executions.
use mathlang_mirp::native::MathProblemContext;
use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut digest = Sha256::new();
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
        let result = context.execute(None)?;
        let canonical = serde_json::to_vec(&serde_json::json!({
            "input": input,
            "queries": result.knowledge_decisions,
            "selected_ru": result.trace.iter().map(|trace| &trace.ru_ref).collect::<Vec<_>>(),
            "selected_rus": result.trace.iter().map(|trace| &trace.rus_ref).collect::<Vec<_>>(),
            "semantic_result": context.semantic_state(),
            "status": result.status,
            "answer": result.answer,
        }))?;
        digest.update((canonical.len() as u64).to_be_bytes());
        digest.update(canonical);
    }
    println!("{:x}", digest.finalize());
    Ok(())
}
