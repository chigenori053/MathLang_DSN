use mathlang_mirp::{
    memory::MemorySpace,
    session::{Domain, Session},
    SemanticState,
};
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("solve-problem") {
        if args.len() != 3 && args.len() != 4 {
            return Err("usage: mirp solve-problem <problem> [memory.json]".into());
        }
        let memory = if args.len() == 4 {
            MemorySpace::load(&args[3])?
        } else {
            MemorySpace::new()
        };
        let (result, semantic_state, plan) =
            mathlang_mirp::native::solve_problem_with_memory(&args[2], memory)?;
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "result": result, "plan": plan, "semantic_state": semantic_state
            }))?
        );
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("memory-store") {
        if args.len() != 6 {
            return Err(
                "usage: mirp memory-store <state.mirp> <memory.json> <id> <ruos.json>".into(),
            );
        }
        let state = SemanticState::load(&args[2])?;
        let trace: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(&args[5])?)?;
        let mut memory = if Path::new(&args[3]).exists() {
            MemorySpace::load(&args[3])?
        } else {
            MemorySpace::new()
        };
        memory.store(&args[4], &state, &trace)?;
        memory.save(&args[3])?;
        println!("{{\"status\":\"STORED\"}}");
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("memory-restore") {
        if args.len() != 5 {
            return Err("usage: mirp memory-restore <memory.json> <id> <state.mirp>".into());
        }
        let state = MemorySpace::load(&args[2])?.retrieve(&args[3])?;
        state.save(&args[4])?;
        println!("{{\"status\":\"RESTORED\"}}");
        return Ok(());
    }
    if args.len() != 6 {
        return Err(
            "usage: mirp <state.mirp> <namespace> <scope> <nl|ja|math|code> <input>".into(),
        );
    }
    let domain = match args[4].as_str() {
        "nl" => Domain::NaturalLanguage,
        "ja" => Domain::Japanese,
        "math" => Domain::Mathematics,
        "code" => Domain::Code,
        _ => return Err("domain must be nl, ja, math, or code".into()),
    };
    let path = Path::new(&args[1]);
    let state = if path.exists() {
        SemanticState::load(path)?
    } else {
        SemanticState::default()
    };
    let mut session = Session::restore(state, &args[2], &args[3])?;
    let results = session.apply_input(domain, &args[5])?;
    session.state.save(path)?;
    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}
