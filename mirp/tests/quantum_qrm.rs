//! USM-QRM-001 extension: QUANTUM_MIX, QUANTUM_EFFECT_SCORE, QUANTUM_CHARPOLY and 1..=32 dimensions.
use mathlang_mirp::calculation::{compute, verify_answer, CalculationRequest as R};
use serde_json::{json, Value};

fn q(n: i64, d: i64) -> Value {
    json!({"numerator":n.to_string(),"denominator":d.to_string()})
}
fn c(re: i64, im: i64, d: i64) -> Value {
    json!({"real":q(re,d),"imaginary":q(im,d)})
}
fn z() -> Value {
    c(0, 0, 1)
}
fn fixture(op: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/fixtures/calculation/quantum_{op}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
fn run(v: Value) -> Value {
    let r: R = serde_json::from_value(v).unwrap();
    let result = compute(&r).unwrap();
    verify_answer(&r, &result.answer).unwrap();
    serde_json::to_value(result.answer).unwrap()["result"]["outcome"].clone()
}
fn reject(v: Value) {
    let r: R = serde_json::from_value(v).unwrap();
    let e = compute(&r).unwrap_err();
    assert!(e.starts_with("CAPABILITY_INPUT_INVALID"), "{e}");
}
fn tamper(v: Value, pointer: &str, value: Value) {
    let r: R = serde_json::from_value(v).unwrap();
    let result = compute(&r).unwrap();
    let mut answer = serde_json::to_value(&result.answer).unwrap();
    *answer
        .pointer_mut(&format!("/result/outcome{pointer}"))
        .unwrap() = value;
    let altered = serde_json::from_value(answer).unwrap();
    assert!(
        verify_answer(&r, &altered).is_err(),
        "tamper {pointer} undetected"
    );
}
fn diag_basis(n: usize) -> Value {
    json!({"id":"b","labels":(0..n).map(|i| format!("e{i}")).collect::<Vec<_>>()})
}
fn maximally_mixed(n: usize) -> Value {
    let entries: Vec<Vec<Value>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { c(1, 0, n as i64) } else { z() })
                .collect()
        })
        .collect();
    json!({"kind":"MIXED","basis_id":"b","entries":entries})
}

#[test]
fn mix_updates_density_and_bounds_effect_scores() {
    let out = run(fixture("mix"));
    assert_eq!(out["kind"], "MIX");
    assert_eq!(out["trace"], q(1, 1));
    let ch = &out["effect_changes"][0];
    assert_eq!(ch["before"], q(1, 2));
    assert_eq!(ch["after"], q(11, 20));
    assert_eq!(ch["new_state_score"], q(1, 1));
    assert_eq!(ch["absolute_change"], q(1, 20));
    assert_eq!(ch["bound_holds"], true);
    // alpha = 0 and alpha = 1 boundaries
    let mut v = fixture("mix");
    v["alpha"] = q(0, 1);
    let o = run(v);
    assert_eq!(
        o["effect_changes"][0]["after"],
        o["effect_changes"][0]["before"]
    );
    assert_eq!(o["state"]["entries"][0][0]["real"], q(1, 4));
    let mut v = fixture("mix");
    v["alpha"] = q(1, 1);
    assert_eq!(run(v)["effect_changes"][0]["after"], q(1, 1));
}

#[test]
fn mix_rejects_out_of_range_alpha_and_invalid_effects_and_states() {
    for a in [q(-1, 10), q(11, 10)] {
        let mut v = fixture("mix");
        v["alpha"] = a;
        reject(v);
    }
    let mut v = fixture("mix");
    v["effects"] = json!([{"basis_id":"qubit_z","entries":[[c(2,0,1),z()],[z(),z()]]}]); // E > I
    reject(v);
    let mut v = fixture("mix");
    v["effects"] = json!([{"basis_id":"qubit_z","entries":[[c(-1,0,2),z()],[z(),z()]]}]); // E not PSD
    reject(v);
    let mut v = fixture("mix");
    v["new_state"] = json!({"kind":"MIXED","basis_id":"qubit_z","entries":[[c(1,0,2),c(1,0,1)],[c(1,0,1),c(1,0,2)]]});
    reject(v);
}

#[test]
fn mix_verification_detects_altered_results() {
    tamper(fixture("mix"), "/effect_changes/0/after", q(3, 5));
    tamper(
        fixture("mix"),
        "/effect_changes/0/bound_holds",
        json!(false),
    );
    tamper(fixture("mix"), "/alpha", q(1, 5));
    tamper(fixture("mix"), "/state/entries/0/0", c(1, 0, 3));
    tamper(fixture("mix"), "/psd_certificate/diagonal/0", q(1, 1));
}

#[test]
fn effect_score_is_certified_and_in_unit_interval() {
    let out = run(fixture("effect_score"));
    assert_eq!(out["value"], q(1, 2));
    let mut v = fixture("effect_score");
    v["state"] = json!({"kind":"PURE","basis_id":"qubit_z","amplitudes":[c(1,0,1),c(0,1,1)],"squared_scale":q(1,2)});
    assert_eq!(run(v)["value"], q(1, 1));
    let mut v = fixture("effect_score");
    v["effect"] = json!({"basis_id":"qubit_z","entries":[[c(3,0,2),z()],[z(),z()]]});
    reject(v);
    tamper(fixture("effect_score"), "/value", q(1, 3));
    tamper(
        fixture("effect_score"),
        "/complement_certificate/diagonal/0",
        q(2, 1),
    );
}

#[test]
fn charpoly_is_exact_and_unitarily_invariant() {
    let out = run(fixture("charpoly"));
    assert_eq!(out["coefficients"], json!([q(3, 16), q(-1, 1), q(1, 1)]));
    assert_eq!(out["determinant"], q(3, 16));
    // Hadamard-conjugated diag(1/4,3/4) = [[1/2,-1/4],[-1/4,1/2]] has the same charpoly
    let mut v = fixture("charpoly");
    v["state"]["entries"] = json!([[c(1, 0, 2), c(-1, 0, 4)], [c(-1, 0, 4), c(1, 0, 2)]]);
    assert_eq!(run(v)["coefficients"], out["coefficients"]);
    // complex off-diagonal 3x3 case
    let mut v = fixture("charpoly");
    v["basis"] = json!({"id":"qubit_z","labels":["0","1","2"]});
    v["state"]["entries"] = json!([
        [c(1, 0, 3), c(0, 1, 6), z()],
        [c(0, -1, 6), c(1, 0, 3), z()],
        [z(), z(), c(1, 0, 3)]
    ]);
    let o = run(v);
    assert_eq!(o["trace"], q(1, 1));
    tamper(fixture("charpoly"), "/coefficients/0", q(1, 4));
    tamper(fixture("charpoly"), "/determinant", q(1, 4));
}

#[test]
fn dimensions_up_to_thirty_two_are_supported_and_thirty_three_rejected() {
    for n in [9usize, 16, 32] {
        let v = json!({"operation":"QUANTUM_DENSITY_MATRIX","basis":diag_basis(n),"state":maximally_mixed(n)});
        let out = run(v);
        assert_eq!(out["purity"], q(1, n as i64));
        let v = json!({"operation":"QUANTUM_CHARPOLY","basis":diag_basis(n),"state":maximally_mixed(n)});
        assert_eq!(run(v)["coefficients"][n], q(1, 1));
    }
    let v = json!({"operation":"QUANTUM_DENSITY_MATRIX","basis":diag_basis(33),"state":maximally_mixed(33)});
    reject(v);
}
