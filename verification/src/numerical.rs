//! Verification-only binary64 arithmetic. It does not implement DSN inference.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const VERSION: &str = "mathlang.numerical/0.1.0";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Complex64 {
    pub real: f64,
    pub imaginary: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tolerance {
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub operation: Operation,
    pub precision: Precision,
    pub state: [Complex64; 2],
    pub thresholds: Tolerance,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    #[serde(rename = "PURE_DENSITY_2D")]
    PureDensity2d,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Precision {
    #[serde(rename = "binary64")]
    Binary64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Residuals {
    pub trace: f64,
    pub hermitian: f64,
    #[serde(rename = "PSD")]
    pub psd: f64,
    pub minimum_eigenvalue: f64,
    pub eigensolver: String,
    pub solver_converged: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub kind: String,
    pub precision: String,
    pub matrix: [[Complex64; 2]; 2],
    pub trace: f64,
    pub purity: f64,
    pub residuals: Residuals,
}

impl Request {
    pub fn validate(&self) -> Result<(), String> {
        let t = self.thresholds;
        if [t.absolute_tolerance, t.relative_tolerance]
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0 || *x > 1e-8)
        {
            return Err(
                "CAPABILITY_INPUT_INVALID: tolerances must be finite and in 0..=1e-8".into(),
            );
        }
        let mut nonzero = false;
        for z in self.state {
            for x in [z.real, z.imaginary] {
                if !x.is_finite() {
                    return Err("CAPABILITY_INPUT_INVALID: non-finite input".into());
                }
                nonzero |= x != 0.0;
            }
        }
        if !nonzero {
            return Err("CAPABILITY_INPUT_INVALID: zero vector cannot be normalized".into());
        }
        Ok(())
    }
}

pub fn close(a: f64, b: f64, t: Tolerance) -> bool {
    [t.absolute_tolerance, t.relative_tolerance]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.0 && *x <= 1e-8)
        && a.is_finite()
        && b.is_finite()
        && (a - b).abs() <= t.absolute_tolerance + t.relative_tolerance * a.abs().max(b.abs())
}

pub fn residuals(m: &[[Complex64; 2]; 2]) -> Residuals {
    let mut h = 0.0_f64;
    for i in 0..2 {
        for j in 0..2 {
            h = h.hypot(m[i][j].real - m[j][i].real);
            h = h.hypot(m[i][j].imaginary + m[j][i].imaginary);
        }
    }
    // Eigenvalues of the Hermitian part; Hermiticity is checked separately.
    let off_re = (m[0][1].real + m[1][0].real) / 2.0;
    let off_im = (m[0][1].imaginary - m[1][0].imaginary) / 2.0;
    let gap = (m[0][0].real - m[1][1].real).hypot(2.0 * off_re.hypot(off_im));
    let tr = m[0][0].real + m[1][1].real;
    let minimum = (tr - gap) / 2.0;
    Residuals {
        trace: (tr - 1.0).abs(),
        hermitian: h,
        psd: (-minimum).max(0.0),
        minimum_eigenvalue: minimum,
        eigensolver: "analytic_hermitian_2x2".into(),
        solver_converged: minimum.is_finite(),
    }
}

pub fn calculate(r: &Request) -> Result<Outcome, String> {
    r.validate()?;
    // Scale first: mixed maximum/subnormal magnitudes must not overflow the norm.
    let scale = r
        .state
        .iter()
        .flat_map(|z| [z.real.abs(), z.imaginary.abs()])
        .fold(0.0_f64, f64::max);
    let mut v = r.state;
    let mut norm = 0.0_f64;
    for z in &mut v {
        z.real /= scale;
        z.imaginary /= scale;
        norm = norm.hypot(z.real).hypot(z.imaginary);
    }
    for z in &mut v {
        z.real /= norm;
        z.imaginary /= norm;
    }
    let zero = Complex64 {
        real: 0.0,
        imaginary: 0.0,
    };
    let mut matrix = [[zero; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            matrix[i][j] = Complex64 {
                real: v[i].real * v[j].real + v[i].imaginary * v[j].imaginary,
                imaginary: v[i].imaginary * v[j].real - v[i].real * v[j].imaginary,
            };
        }
    }
    let trace = matrix[0][0].real + matrix[1][1].real;
    let purity = matrix
        .iter()
        .flatten()
        .map(|z| z.real * z.real + z.imaginary * z.imaginary)
        .sum();
    Ok(Outcome {
        kind: "DENSITY_MATRIX".into(),
        precision: "binary64".into(),
        matrix,
        trace,
        purity,
        residuals: residuals(&matrix),
    })
}

pub fn verify(r: &Request, o: &Outcome) -> Result<(), String> {
    r.validate()?;
    let bad = || "CAPABILITY_OUTPUT_INVALID: independent binary64 check failed".to_string();
    if o.kind != "DENSITY_MATRIX" || o.precision != "binary64" {
        return Err(bad());
    }
    // Different computation from normalize+outer: scale, form v v†, divide by sum |v|².
    let s = r
        .state
        .iter()
        .flat_map(|z| [z.real.abs(), z.imaginary.abs()])
        .fold(0.0_f64, f64::max);
    let v: Vec<_> = r
        .state
        .iter()
        .map(|z| Complex64 {
            real: z.real / s,
            imaginary: z.imaginary / s,
        })
        .collect();
    let n: f64 = v
        .iter()
        .map(|z| z.real * z.real + z.imaginary * z.imaginary)
        .sum();
    for i in 0..2 {
        for j in 0..2 {
            let expected = Complex64 {
                real: (v[i].real * v[j].real + v[i].imaginary * v[j].imaginary) / n,
                imaginary: (v[i].imaginary * v[j].real - v[i].real * v[j].imaginary) / n,
            };
            if !close(o.matrix[i][j].real, expected.real, r.thresholds)
                || !close(o.matrix[i][j].imaginary, expected.imaginary, r.thresholds)
            {
                return Err(bad());
            }
        }
    }
    let rr = residuals(&o.matrix);
    let purity: f64 = o
        .matrix
        .iter()
        .flatten()
        .map(|z| z.real * z.real + z.imaginary * z.imaginary)
        .sum();
    if o.trace != o.matrix[0][0].real + o.matrix[1][1].real
        || o.purity != purity
        || rr != o.residuals
        || !close(o.trace, 1.0, r.thresholds)
        || !close(o.purity, 1.0, r.thresholds)
        || !rr.solver_converged
        || ![rr.trace, rr.hermitian, rr.psd]
            .iter()
            .all(|x| close(*x, 0.0, r.thresholds))
    {
        return Err(bad());
    }
    Ok(())
}

pub fn execute(r: &Request) -> Result<Value, String> {
    let outcome = calculate(r)?;
    verify(r, &outcome)?;
    Ok(
        json!({"schema_version":VERSION,"request":r,"outcome":outcome,
        "verification":{"status":"NUMERICALLY_VERIFIED","formal_status":"NOT_CHECKED",
            "method":"INDEPENDENT_SCALED_OUTER_OVER_NORM_SQUARED","thresholds":r.thresholds},
        "execution_metadata":{"arithmetic":"IEEE754_BINARY64","normalization":"scaled_iterated_hypot",
            "random_seed":null,"seed_policy":"NOT_APPLICABLE_DETERMINISTIC","dimension":2,
            "limitations":["Not a mathematical proof","No generic eigensolver or semantic inference","Tiny components may underflow; only tolerance-bounded density values are validated"]}}),
    )
}
