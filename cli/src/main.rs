use clap::Parser;
use mathlang_cli::*;
use serde_json::json;
use std::io::{self, Write};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "__numerical-worker") {
        let result = args
            .get(2)
            .ok_or("missing numerical request".to_string())
            .and_then(|s| {
                serde_json::from_str::<mathlang_verification::numerical::Request>(s)
                    .map_err(|e| e.to_string())
            })
            .and_then(|r| mathlang_verification::numerical::execute(&r));
        match result {
            Ok(v) => println!("{v}"),
            Err(e) => {
                println!("{}", json!({"diagnostic":e}));
                std::process::exit(9);
            }
        }
        return;
    }
    if args.get(1).is_some_and(|s| s == "__compute-worker") {
        let outcome = args
            .get(2)
            .ok_or("missing calculation request".to_owned())
            .and_then(|s| {
                serde_json::from_str::<mathlang_mirp::calculation::CalculationRequest>(s)
                    .map_err(|e| e.to_string())
            })
            .and_then(|request| mathlang_mirp::calculation::compute(&request));
        let (value, code) = match outcome {
            Ok(result) => (json!(result), 0),
            Err(message) => (
                json!({"diagnostic":message}),
                if message.starts_with("CAPABILITY_EXECUTION_FAILED") {
                    5
                } else if message.starts_with("RESOURCE_LIMIT") {
                    6
                } else if message.starts_with("CAPABILITY_INPUT_INVALID") {
                    4
                } else {
                    9
                },
            ),
        };
        println!("{value}");
        std::process::exit(code);
    }
    if args.get(1).is_some_and(|s| s == "__solve-worker") {
        if args.len() != 3 {
            std::process::exit(2);
        }
        let (value, code) = match mathlang_mirp::native::solve_problem(&args[2]) {
            Ok((result, state, plan)) => {
                let code = if result.status == mathlang_mirp::native::ReasoningStatus::Unsupported {
                    4
                } else {
                    0
                };
                (
                    json!({"result":result,"semantic_state":state,"plan":plan}),
                    code,
                )
            }
            Err(message) => (
                json!({"diagnostic":message}),
                if message.contains("ReasonScript") || message.contains("reason") {
                    5
                } else {
                    4
                },
            ),
        };
        println!("{value}");
        std::process::exit(code);
    }
    let parsed = Cli::try_parse_from(&args);
    let cli = match parsed {
        Ok(c) => c,
        Err(error) => {
            use clap::error::ErrorKind;
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                print!("{error}");
                return;
            }
            let mut e = envelope("arguments", &run_id());
            failure(&mut e, Error::new(2, error.to_string()), "arguments");
            if args.windows(2).any(|a| a == ["--format", "json"])
                || args.iter().any(|a| a == "--format=json")
            {
                println!("{e}");
            } else {
                eprintln!("{error}");
            }
            std::process::exit(2);
        }
    };
    let command = match cli.command {
        Command::Model { .. } => "model",
        Command::Verify { .. } => "verify",
        Command::Proof { .. } => "proof",
        Command::Knowledge { .. } => "knowledge",
        Command::Compare { .. } => "compare",
        Command::Evidence { .. } => "evidence",
        Command::Solve { .. } => "solve",
        Command::Compute { .. } => "compute",
        Command::Numerical { .. } => "numerical",
        Command::Doctor => "doctor",
    };
    let mut e = envelope(command, &run_id());
    let config = Config::load(cli.config.as_deref());
    let format = cli.format.unwrap_or_else(|| {
        config
            .as_ref()
            .map(|c| c.cli.output_format)
            .unwrap_or(Format::Text)
    });
    let outcome = config.and_then(|config| {
        if cli.verbose {
            eprintln!("mathlang {command}: run_id={}", e["run_id"]);
        }
        commands::execute(&cli, &config, &mut e)?;
        Ok(())
    });
    let mut code = match outcome {
        Ok(()) => 0,
        Err(error) => failure(&mut e, error, command),
    };
    if !matches!(cli.command, Command::Verify { .. }) {
        if let Some(p) = &cli.output {
            if let Err(error) = save(p, &e) {
                code = failure(&mut e, error, command);
            }
        }
    }
    let output = match format {
        Format::Json => e.to_string(),
        Format::Text => {
            if cli.quiet && code == 0 {
                String::new()
            } else {
                serde_json::to_string_pretty(&e)
                    .unwrap_or_else(|_| json!({"error":"serialization failure"}).to_string())
            }
        }
    };
    if !output.is_empty() && writeln!(io::stdout().lock(), "{output}").is_err() {
        std::process::exit(8);
    }
    std::process::exit(code);
}
