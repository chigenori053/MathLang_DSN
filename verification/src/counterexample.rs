//! Exact bounded evaluation supplies candidates, never a replacement Lean kernel.
use crate::ir::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    n: i128,
    d: i128,
}
impl Rational {
    pub fn new(n: i128, d: i128) -> Result<Self, String> {
        if d <= 0 {
            return Err("nonpositive exact denominator".into());
        }
        let mut a = n.unsigned_abs();
        let mut b = d as u128;
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        let gcd = a as i128;
        Ok(Self {
            n: n / gcd,
            d: d / gcd,
        })
    }
    fn add(&self, b: &Self, subtract: bool) -> Result<Self, String> {
        let left = self.n.checked_mul(b.d).ok_or("exact overflow")?;
        let right = b.n.checked_mul(self.d).ok_or("exact overflow")?;
        Self::new(
            if subtract {
                left.checked_sub(right)
            } else {
                left.checked_add(right)
            }
            .ok_or("exact overflow")?,
            self.d.checked_mul(b.d).ok_or("exact overflow")?,
        )
    }
    fn mul(&self, b: &Self) -> Result<Self, String> {
        Self::new(
            self.n.checked_mul(b.n).ok_or("exact overflow")?,
            self.d.checked_mul(b.d).ok_or("exact overflow")?,
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExactValue {
    Bool(bool),
    Number(Rational),
    Finite(u32, u32),
}
pub fn evaluate(e: &MathExpr, env: &BTreeMap<String, ExactValue>) -> Result<ExactValue, String> {
    use ExactValue as V;
    use MathExpr::*;
    let num = |a: &MathExpr| match evaluate(a, env)? {
        V::Number(n) => Ok(n),
        _ => Err("expected exact scalar".to_string()),
    };
    let boolean = |a: &MathExpr| match evaluate(a, env)? {
        V::Bool(v) => Ok(v),
        _ => Err("expected boolean".to_string()),
    };
    Ok(match e {
        Variable(n) => env.get(n).cloned().ok_or("unbound candidate variable")?,
        Constant(c) => {
            use crate::ir::Constant as C;
            match c {
                C::Bool(b) => V::Bool(*b),
                C::Natural(n) => V::Number(Rational::new(*n as i128, 1)?),
                C::Integer(n) => V::Number(Rational::new(*n as i128, 1)?),
                C::Rational(n, d) => V::Number(Rational::new(*n as i128, *d as i128)?),
                C::Zero(t) if t.scalar() => V::Number(Rational::new(0, 1)?),
                C::One(t) if t.scalar() => V::Number(Rational::new(1, 1)?),
                _ => return Err("unsupported exact constant".into()),
            }
        }
        Add(a, b) => V::Number(num(a)?.add(&num(b)?, false)?),
        Subtract(a, b) => V::Number(num(a)?.add(&num(b)?, true)?),
        Multiply(a, b) => V::Number(num(a)?.mul(&num(b)?)?),
        Equality(a, b) => V::Bool(evaluate(a, env)? == evaluate(b, env)?),
        Inequality(a, b) => {
            let x = num(a)?;
            let y = num(b)?;
            V::Bool(
                x.n.checked_mul(y.d).ok_or("exact overflow")?
                    <= y.n.checked_mul(x.d).ok_or("exact overflow")?,
            )
        }
        LogicalAnd(a, b) => V::Bool(boolean(a)? && boolean(b)?),
        LogicalOr(a, b) => V::Bool(boolean(a)? || boolean(b)?),
        LogicalNot(a) => V::Bool(!boolean(a)?),
        Implies(a, b) => V::Bool(!boolean(a)? || boolean(b)?),
        _ => return Err("unsupported exact evaluator; retain candidate only".into()),
    })
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterexampleKind {
    Finite,
    Algebraic,
    ConstraintViolation,
    NumericalCandidate,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counterexample {
    pub obligation_id: String,
    pub model_revision: u64,
    pub source_ir_hash: String,
    pub kind: CounterexampleKind,
    pub witness: BTreeMap<String, ExactValue>,
    pub assumptions_satisfied: bool,
    pub conclusion_false: bool,
    pub kernel_evidence: Option<String>,
    pub notes: Vec<String>,
}
pub fn check_candidate(
    o: &crate::proof::ProofObligation,
    witness: BTreeMap<String, ExactValue>,
    kind: CounterexampleKind,
) -> Result<Counterexample, String> {
    o.validate()?;
    let mut ctx = o.context.clone();
    let mut goal = &o.goal;
    while let MathExpr::ForAll(n, t, body) = goal {
        ctx.variables.insert(n.clone(), t.clone());
        goal = body;
    }
    if witness.len() != ctx.variables.len() {
        return Err("incomplete or extraneous witness".into());
    }
    for (n, t) in &ctx.variables {
        let v = witness.get(n).ok_or("missing witness")?;
        let valid = match (t, v) {
            (MathType::Nat, ExactValue::Number(r)) => r.d == 1 && r.n >= 0,
            (MathType::Int, ExactValue::Number(r)) => r.d == 1,
            (MathType::Rational, ExactValue::Number(_)) | (MathType::Bool, ExactValue::Bool(_)) => {
                true
            }
            (MathType::Finite(n), ExactValue::Finite(i, m)) => m == n && i < n,
            _ => false,
        };
        if !valid {
            return Err("witness outside quantified domain or unsupported exact domain".into());
        }
    }
    let holds = |e: &MathExpr| match evaluate(e, &witness)? {
        ExactValue::Bool(b) => Ok::<_, String>(b),
        _ => Err("nonpropositional candidate".into()),
    };
    let mut assumptions = true;
    for a in &ctx.assumptions {
        assumptions &= holds(a)?;
    }
    while let MathExpr::Implies(a, b) = goal {
        assumptions &= holds(a)?;
        goal = b;
    }
    let conclusion_false = !holds(goal)?;
    Ok(Counterexample {
        obligation_id: o.id.clone(),
        model_revision: o.model_revision,
        source_ir_hash: o.source_hash(),
        kind: if assumptions {
            kind
        } else {
            CounterexampleKind::ConstraintViolation
        },
        witness,
        assumptions_satisfied: assumptions,
        conclusion_false,
        kernel_evidence: None,
        notes: vec![
            "Exact Rust evaluation is candidate evidence; REFUTED requires a checked negation."
                .into(),
        ],
    })
}
pub fn search_finite(
    o: &crate::proof::ProofObligation,
    budget: usize,
) -> Result<Vec<Counterexample>, String> {
    if budget == 0 {
        return Err("counterexample search budget exceeded".into());
    }
    let mut variables = o.context.variables.clone();
    let mut body = &o.goal;
    while let MathExpr::ForAll(n, t, b) = body {
        variables.insert(n.clone(), t.clone());
        body = b;
    }
    let mut assignments = vec![BTreeMap::new()];
    for (n, t) in variables {
        let values = match t {
            MathType::Bool => vec![ExactValue::Bool(false), ExactValue::Bool(true)],
            MathType::Finite(m) => {
                if m as usize > budget {
                    return Err("counterexample search budget exceeded".into());
                }
                (0..m).map(|i| ExactValue::Finite(i, m)).collect()
            }
            _ => return Err("finite search requires Bool or Fin n domains".into()),
        };
        let mut next = vec![];
        for a in &assignments {
            for value in &values {
                if next.len() >= budget {
                    return Err("counterexample search budget exceeded".into());
                }
                let mut a = a.clone();
                a.insert(n.clone(), value.clone());
                next.push(a);
            }
        }
        assignments = next;
    }
    let mut found = vec![];
    for w in assignments {
        let c = check_candidate(o, w, CounterexampleKind::Finite)?;
        if c.assumptions_satisfied && c.conclusion_false {
            found.push(c)
        }
    }
    Ok(found)
}
