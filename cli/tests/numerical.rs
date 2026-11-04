use serde_json::{json, Value};
use std::{fs, process::Command};
#[test]
fn numerical_cli_replays_and_keeps_exact_compute_contract() {
    let tmp = std::env::temp_dir().join(format!("mathlang-numerical-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("request.json");
    let request = json!({"operation":"PURE_DENSITY_2D","precision":"binary64",
        "state":[{"real":1.0,"imaginary":2.0},{"real":-3.0,"imaginary":4.0}],
        "thresholds":{"absolute_tolerance":1e-12,"relative_tolerance":1e-12}});
    fs::write(&path, request.to_string()).unwrap();
    let call = |cmd: &str| {
        Command::new(env!("CARGO_BIN_EXE_mathlang"))
            .arg(cmd)
            .arg(&path)
            .args(["--format", "json"])
            .output()
            .unwrap()
    };
    let first = call("numerical");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    let second = call("numerical");
    assert!(second.status.success());
    let a: Value = serde_json::from_slice(&first.stdout).unwrap();
    let b: Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(a["result"], b["result"]);
    assert_ne!(a["run_id"], b["run_id"]);
    assert_eq!(a["result"]["verification"]["formal_status"], "NOT_CHECKED");
    assert!(a["formal_status"].is_null());
    assert!(!call("compute").status.success());
    let mut bad = request.clone();
    bad["thresholds"]["absolute_tolerance"] = json!(0.1);
    fs::write(&path, bad.to_string()).unwrap();
    assert_eq!(call("numerical").status.code(), Some(4));
    fs::write(&path,"{\"operation\":\"PURE_DENSITY_2D\",\"precision\":\"binary64\",\"state\":[{\"real\":NaN,\"imaginary\":0},{\"real\":0,\"imaginary\":1}],\"thresholds\":{\"absolute_tolerance\":1e-12,\"relative_tolerance\":1e-12}}").unwrap();
    assert_eq!(call("numerical").status.code(), Some(2));
    fs::remove_dir_all(tmp).unwrap();
}
