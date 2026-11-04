use mathlang_verification::numerical::*;
use serde_json::json;

fn request() -> Request {
    serde_json::from_value(json!({"operation":"PURE_DENSITY_2D","precision":"binary64",
        "state":[{"real":1.0,"imaginary":2.0},{"real":-3.0,"imaginary":4.0}],
        "thresholds":{"absolute_tolerance":1e-12,"relative_tolerance":1e-12}}))
    .unwrap()
}
#[test]
fn floating_density_extreme_scales_and_complex_states() {
    for scale in [1e-300, 1e-150, 0.1, 1.0, 1e150, 1e300] {
        let mut r = request();
        for z in &mut r.state {
            z.real *= scale;
            z.imaginary *= scale;
        }
        let o = calculate(&r).unwrap();
        verify(&r, &o).unwrap();
        assert!(close(o.matrix[0][0].real, 1.0 / 6.0, r.thresholds));
        assert!(close(o.matrix[0][1].imaginary, -1.0 / 3.0, r.thresholds));
    }
    let mut r = request();
    r.state = [Complex64 {
        real: f64::MAX,
        imaginary: f64::MAX,
    }; 2];
    verify(&r, &calculate(&r).unwrap()).unwrap();
    r.state = [Complex64 {
        real: f64::from_bits(1),
        imaginary: 0.0,
    }; 2];
    verify(&r, &calculate(&r).unwrap()).unwrap();
}
#[test]
fn rejects_zero_nonfinite_inputs_and_invalid_tolerances() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut r = request();
        r.state[0].real = value;
        assert!(r.validate().is_err());
        assert!(calculate(&r).is_err());
        let mut r = request();
        r.thresholds.absolute_tolerance = value;
        assert!(r.validate().is_err());
    }
    let mut r = request();
    r.state = [Complex64 {
        real: 0.0,
        imaginary: 0.0,
    }; 2];
    assert!(r.validate().is_err());
    for value in [-1.0, 1e-7] {
        let mut r = request();
        r.thresholds.relative_tolerance = value;
        assert!(r.validate().is_err());
    }
}
#[test]
fn rejects_altered_matrix_and_inconsistent_summary() {
    let r = request();
    let o = calculate(&r).unwrap();
    let mut bad = o.clone();
    bad.matrix[0][0].real += 1e-4;
    assert!(verify(&r, &bad).is_err());
    let mut bad = o.clone();
    bad.trace = 0.5;
    assert!(verify(&r, &bad).is_err());
    let mut bad = o.clone();
    bad.purity = 0.5;
    assert!(verify(&r, &bad).is_err());
    let mut bad = o.clone();
    bad.residuals.psd = 0.5;
    assert!(verify(&r, &bad).is_err());
    let mut bad = o.clone();
    bad.residuals.solver_converged = false;
    assert!(verify(&r, &bad).is_err());
    let mut bad = o.clone();
    bad.matrix[1][0].imaginary = f64::NAN;
    assert!(verify(&r, &bad).is_err());
}
#[test]
fn tolerance_boundary_and_formal_separation() {
    let r = request();
    let t = r.thresholds;
    assert!(close(0.0, t.absolute_tolerance / 2.0, t));
    assert!(!close(0.0, 1e-5, t));
    assert!(!close(f64::NAN, 0.0, t));
    assert!(!close(0.0, f64::INFINITY, t));
    assert!(!close(
        0.0,
        1.0,
        Tolerance {
            absolute_tolerance: f64::INFINITY,
            relative_tolerance: 0.0
        }
    ));
    assert!(!close(
        1.0,
        1.0,
        Tolerance {
            absolute_tolerance: -1.0,
            relative_tolerance: 0.0
        }
    ));
    let v = execute(&r).unwrap();
    assert_eq!(v["verification"]["formal_status"], "NOT_CHECKED");
    assert_eq!(v["verification"]["status"], "NUMERICALLY_VERIFIED");
}
#[test]
fn rejects_wrong_shape_precision_and_extra_fields() {
    let v = serde_json::to_value(request()).unwrap();
    for key in ["extra", "state", "precision", "operation"] {
        let mut bad = v.clone();
        bad[key] = match key {
            "state" => json!([]),
            "precision" => json!("binary32"),
            "operation" => json!("UNKNOWN"),
            _ => json!(true),
        };
        assert!(serde_json::from_value::<Request>(bad).is_err());
    }
}
