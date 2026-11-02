use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dimension {
    Fixed(u32),
    Symbol(String),
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MathType {
    Bool,
    Nat,
    Int,
    Rational,
    Real,
    Complex,
    Proposition,
    Predicate,
    Universe,
    Abstract(String),
    Finite(u32),
    Vector(Box<MathType>, Dimension),
    Matrix(Box<MathType>, Dimension, Dimension),
    Tensor(Box<MathType>, Vec<Dimension>),
    HilbertSpace(Dimension),
    LinearOperator(Box<MathType>, Box<MathType>),
    DensityOperator(Dimension),
    Function(Box<MathType>, Box<MathType>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constant {
    Bool(bool),
    Natural(u64),
    Integer(i64),
    Rational(i64, u64),
    Zero(MathType),
    One(MathType),
    Identity(MathType),
    IdentityFunction(MathType),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathExpr {
    Constant(Constant),
    Variable(String),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Power(Box<Self>, Box<Self>),
    Conjugate(Box<Self>),
    Norm(Box<Self>),
    InnerProduct(Box<Self>, Box<Self>),
    MatrixMultiply(Box<Self>, Box<Self>),
    Adjoint(Box<Self>),
    TensorProduct(Box<Self>, Box<Self>),
    Trace(Box<Self>),
    Equality(Box<Self>, Box<Self>),
    Inequality(Box<Self>, Box<Self>),
    LogicalAnd(Box<Self>, Box<Self>),
    LogicalOr(Box<Self>, Box<Self>),
    LogicalNot(Box<Self>),
    Implies(Box<Self>, Box<Self>),
    ForAll(String, MathType, Box<Self>),
    Exists(String, MathType, Box<Self>),
    PositiveSemidefinite(Box<Self>),
    Normalized(Box<Self>),
    Unitary(Box<Self>),
    Hermitian(Box<Self>),
    LinearMap(Box<Self>),
    StateTransition(Box<Self>, Box<Self>),
    Apply(Box<Self>, Box<Self>),
    TensorIndex(Box<Self>, Vec<u32>),
    Update(Box<Self>, Box<Self>, Box<Self>),
}
impl MathExpr {
    pub fn var(name: &str) -> Self {
        Self::Variable(name.into())
    }
    pub fn eq(a: Self, b: Self) -> Self {
        Self::Equality(Box::new(a), Box::new(b))
    }
    pub fn and(a: Self, b: Self) -> Self {
        Self::LogicalAnd(Box::new(a), Box::new(b))
    }
    pub fn implies(a: Self, b: Self) -> Self {
        Self::Implies(Box::new(a), Box::new(b))
    }
    pub fn forall(name: &str, ty: MathType, body: Self) -> Self {
        Self::ForAll(name.into(), ty, Box::new(body))
    }
    pub fn apply(f: Self, x: Self) -> Self {
        Self::Apply(Box::new(f), Box::new(x))
    }
}
pub fn identifier(s: &str) -> bool {
    // Prefixing emitted identifiers also avoids Lean keywords and command injection.
    !s.is_empty()
        && s.len() <= 80
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
impl MathType {
    pub fn scalar(&self) -> bool {
        matches!(
            self,
            Self::Nat | Self::Int | Self::Rational | Self::Real | Self::Complex
        )
    }
    pub fn validate(&self, ctx: &BTreeMap<String, MathType>) -> Result<(), String> {
        let dim = |d: &Dimension| match d {
            Dimension::Fixed(n) if *n > 0 => Ok(()),
            Dimension::Symbol(s) if ctx.get(s) == Some(&Self::Nat) => Ok(()),
            _ => Err("invalid or unbound dimension".into()),
        };
        match self {
            Self::Vector(t, n) => {
                if !t.scalar() {
                    return Err("vector field".into());
                }
                t.validate(ctx)?;
                dim(n)
            }
            Self::Matrix(t, n, m) => {
                if !t.scalar() {
                    return Err("matrix field".into());
                }
                t.validate(ctx)?;
                dim(n)?;
                dim(m)
            }
            Self::Tensor(t, ds) => {
                if !t.scalar() || ds.is_empty() {
                    return Err("tensor field/indices".into());
                }
                for d in ds {
                    dim(d)?
                }
                Ok(())
            }
            Self::HilbertSpace(n) | Self::DensityOperator(n) => dim(n),
            Self::LinearOperator(a, b) => {
                if !matches!((&**a, &**b), (Self::HilbertSpace(_), Self::HilbertSpace(_))) {
                    return Err("operator requires complex Hilbert spaces".into());
                }
                a.validate(ctx)?;
                b.validate(ctx)
            }
            Self::Function(a, b) => {
                a.validate(ctx)?;
                b.validate(ctx)
            }
            Self::Abstract(s) if ctx.get(s) == Some(&Self::Universe) => Ok(()),
            Self::Abstract(_) => Err("undefined abstract type".into()),
            Self::Finite(0) | Self::Predicate => {
                Err("empty finite type or undefined predicate signature".into())
            }
            _ => Ok(()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Context {
    pub variables: BTreeMap<String, MathType>,
    pub assumptions: Vec<MathExpr>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvidence {
    pub source_ir_hash: String,
    pub valid: bool,
    pub diagnostics: Vec<String>,
}
impl Context {
    pub fn check(&self, goal: &MathExpr) -> ValidationEvidence {
        let result = (|| {
            for (name, t) in &self.variables {
                if !identifier(name) {
                    return Err("invalid variable".into());
                }
                t.validate(&self.variables)?;
            }
            for a in &self.assumptions {
                if self.infer(a)? != MathType::Proposition {
                    return Err("nonpropositional assumption".into());
                }
            }
            if self.infer(goal)? != MathType::Proposition {
                return Err("goal is not a proposition".into());
            }
            Ok(())
        })();
        ValidationEvidence {
            source_ir_hash: crate::hash(&(self, goal)),
            valid: result.is_ok(),
            diagnostics: result.err().into_iter().collect(),
        }
    }
    pub fn infer(&self, e: &MathExpr) -> Result<MathType, String> {
        use MathExpr::*;
        use MathType as T;
        let pair = |a: &MathExpr, b: &MathExpr| Ok::<_, String>((self.infer(a)?, self.infer(b)?));
        let same = |a: &MathExpr, b: &MathExpr| {
            let (x, y) = pair(a, b)?;
            if x == y {
                Ok(x)
            } else {
                Err(format!("type mismatch: {x:?} versus {y:?}"))
            }
        };
        let prop = |a: &MathExpr| {
            if self.infer(a)? == T::Proposition {
                Ok(T::Proposition)
            } else {
                Err("expected proposition".into())
            }
        };
        match e {
            Variable(n) => self
                .variables
                .get(n)
                .cloned()
                .ok_or_else(|| format!("undefined variable/function: {n}")),
            Constant(c) => {
                use crate::ir::Constant as C;
                let t = match c {
                    C::Bool(_) => T::Bool,
                    C::Natural(_) => T::Nat,
                    C::Integer(_) => T::Int,
                    C::Rational(_, 0) => return Err("zero rational denominator".into()),
                    C::Rational(_, _) => T::Rational,
                    C::Zero(t) | C::One(t)
                        if t.scalar()
                            || matches!(
                                t,
                                T::Matrix(_, _, _)
                                    | T::HilbertSpace(_)
                                    | T::Vector(_, _)
                                    | T::LinearOperator(_, _)
                            ) =>
                    {
                        if matches!(c, C::One(_)) && !t.scalar() {
                            return Err("one only for scalars; use Identity for operators".into());
                        }
                        t.clone()
                    }
                    C::IdentityFunction(t) => T::Function(Box::new(t.clone()), Box::new(t.clone())),
                    C::Identity(t)
                        if matches!(t,T::Matrix(_,n,m) if n==m)
                            || matches!(t,T::LinearOperator(a,b) if a==b) =>
                    {
                        t.clone()
                    }
                    _ => return Err("undefined constant".into()),
                };
                t.validate(&self.variables)?;
                Ok(t)
            }
            Add(a, b) | Subtract(a, b) => {
                let t = same(a, b)?;
                if t.scalar()
                    || matches!(
                        t,
                        T::Vector(_, _)
                            | T::Matrix(_, _, _)
                            | T::Tensor(_, _)
                            | T::HilbertSpace(_)
                            | T::LinearOperator(_, _)
                    )
                {
                    Ok(t)
                } else {
                    Err("undefined addition".into())
                }
            }
            Multiply(a, b) => {
                let (x, y) = pair(a, b)?;
                if x == y && x.scalar() {
                    Ok(x)
                } else if x == T::Complex && matches!(y, T::HilbertSpace(_)) {
                    Ok(y)
                } else {
                    Err("undefined multiplication or coercion".into())
                }
            }
            Divide(a, b) => {
                let t = same(a, b)?;
                if !matches!(t, T::Rational | T::Real | T::Complex) {
                    return Err("division requires a field".into());
                }
                let zero = Constant(crate::ir::Constant::Zero(t.clone()));
                let needed = LogicalNot(Box::new(Equality(b.clone(), Box::new(zero))));
                let literal = matches!(&**b,Constant(crate::ir::Constant::Rational(n,d)) if *n!=0 && *d!=0)
                    || matches!(&**b, Constant(crate::ir::Constant::One(_)));
                if !literal && !self.assumptions.contains(&needed) {
                    return Err("missing nonzero divisor premise".into());
                }
                Ok(t)
            }
            Power(a, b) => {
                let (x, y) = pair(a, b)?;
                if x.scalar() && y == T::Nat {
                    Ok(x)
                } else {
                    Err("power type".into())
                }
            }
            Conjugate(a) => {
                if self.infer(a)? == T::Complex {
                    Ok(T::Complex)
                } else {
                    Err("conjugate type".into())
                }
            }
            Norm(a) => {
                if matches!(self.infer(a)?, T::Real | T::Complex | T::HilbertSpace(_)) {
                    Ok(T::Real)
                } else {
                    Err("undefined norm: use HilbertSpace for Euclidean norm".into())
                }
            }
            InnerProduct(a, b) => {
                let t = same(a, b)?;
                if matches!(t, T::HilbertSpace(_)) {
                    Ok(T::Complex)
                } else {
                    Err("inner product requires HilbertSpace".into())
                }
            }
            Adjoint(a) => match self.infer(a)? {
                T::Matrix(t, n, m) if matches!(*t, T::Real | T::Complex) => Ok(T::Matrix(t, m, n)),
                T::LinearOperator(a, b) => Ok(T::LinearOperator(b, a)),
                _ => Err("adjoint type".into()),
            },
            MatrixMultiply(a, b) => match pair(a, b)? {
                (T::Matrix(t, n, k), T::Matrix(u, l, m)) if t == u && k == l => {
                    Ok(T::Matrix(t, n, m))
                }
                (T::LinearOperator(a, b), T::LinearOperator(c, d)) if a == d => {
                    Ok(T::LinearOperator(c, b))
                }
                _ => Err("matrix product/operator domain mismatch".into()),
            },
            TensorProduct(a, b) => match pair(a, b)? {
                (T::Vector(t, n), T::Vector(u, m)) if t == u => Ok(T::Tensor(t, vec![n, m])),
                (T::Tensor(t, mut ns), T::Tensor(u, ms)) if t == u => {
                    ns.extend(ms);
                    Ok(T::Tensor(t, ns))
                }
                _ => Err("unsupported tensor product spaces".into()),
            },
            TensorIndex(a, indices) => match match self.infer(a)? {
                T::Vector(t, n) => T::Tensor(t, vec![n]),
                T::Matrix(t, n, m) => T::Tensor(t, vec![n, m]),
                t => t,
            } {
                T::Tensor(t, ds)
                    if ds.len() == indices.len()
                        && ds
                            .iter()
                            .zip(indices)
                            .all(|(d, i)| matches!(d,Dimension::Fixed(n) if i<n)) =>
                {
                    Ok(*t)
                }
                _ => Err("invalid tensor indices".into()),
            },
            Trace(a) => match self.infer(a)? {
                T::Matrix(t, n, m) if n == m => Ok(*t),
                T::DensityOperator(_) => Ok(T::Complex),
                _ => Err("trace requires square matrix".into()),
            },
            Equality(a, b) => {
                same(a, b)?;
                Ok(T::Proposition)
            }
            Inequality(a, b) => {
                let t = same(a, b)?;
                if matches!(t, T::Nat | T::Int | T::Rational | T::Real) {
                    Ok(T::Proposition)
                } else {
                    Err("unordered type".into())
                }
            }
            LogicalAnd(a, b) | LogicalOr(a, b) => {
                prop(a)?;
                prop(b)
            }
            Implies(a, b) => {
                prop(a)?;
                let mut ctx = self.clone();
                ctx.assumptions.push(*a.clone());
                if ctx.infer(b)? == T::Proposition {
                    Ok(T::Proposition)
                } else {
                    Err("implication conclusion".into())
                }
            }
            LogicalNot(a) => prop(a),
            ForAll(n, t, body) | Exists(n, t, body) => {
                if !identifier(n) || self.variables.contains_key(n) {
                    return Err("invalid/shadowed binder".into());
                }
                t.validate(&self.variables)?;
                let mut c = self.clone();
                c.variables.insert(n.clone(), t.clone());
                if c.infer(body)? == T::Proposition {
                    Ok(T::Proposition)
                } else {
                    Err("quantifier body".into())
                }
            }
            PositiveSemidefinite(a) | Hermitian(a) => {
                if matches!(self.infer(a)?,T::Matrix(t,n,m) if n==m && *t==T::Complex) {
                    Ok(T::Proposition)
                } else {
                    Err("matrix predicate type".into())
                }
            }
            Normalized(a) => {
                self.infer(&Norm(a.clone()))?;
                Ok(T::Proposition)
            }
            Unitary(a) => {
                if matches!(self.infer(a)?,T::Matrix(t,n,m) if n==m && *t==T::Complex)
                    || matches!(self.infer(a)?,T::LinearOperator(x,y) if x==y)
                {
                    Ok(T::Proposition)
                } else {
                    Err("unitary type".into())
                }
            }
            LinearMap(a) => {
                if matches!(self.infer(a)?, T::LinearOperator(_, _)) {
                    Ok(T::Proposition)
                } else {
                    Err("linearity requires typed linear operator".into())
                }
            }
            Update(state, key, value) => match self.infer(state)? {
                T::Function(a, b) if self.infer(key)? == *a && self.infer(value)? == *b => {
                    Ok(T::Function(a, b))
                }
                _ => Err("update domain/value mismatch".into()),
            },
            Apply(f, x) | StateTransition(f, x) => match pair(f, x)? {
                (T::Function(a, b) | T::LinearOperator(a, b), t) if *a == t => Ok(*b),
                _ => Err("application domain mismatch".into()),
            },
        }
    }
}
/// Canonical JSON retains ordered premises and syntax; it never rewrites a theorem.
pub fn normalized(e: &MathExpr) -> Result<String, String> {
    fn norm(e: &mut serde_json::Value) -> Result<(), String> {
        match e {
            serde_json::Value::Object(m) => {
                if let Some(serde_json::Value::Array(ns)) = m.get_mut("Rational") {
                    let n = ns[0].as_i64().ok_or("rational numerator")?;
                    let d = ns[1].as_u64().ok_or("rational denominator")?;
                    if d == 0 {
                        return Err("zero denominator".into());
                    }
                    let mut a = n.unsigned_abs();
                    let mut b = d;
                    while b != 0 {
                        let r = a % b;
                        a = b;
                        b = r;
                    }
                    ns[0] = serde_json::json!((n as i128 / a as i128) as i64);
                    ns[1] = serde_json::json!(d / a);
                }
                for v in m.values_mut() {
                    norm(v)?;
                }
            }
            serde_json::Value::Array(xs) => {
                for x in xs {
                    norm(x)?
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut v = serde_json::to_value(e).map_err(|e| e.to_string())?;
    norm(&mut v)?;
    serde_json::to_string(&v).map_err(|e| e.to_string())
}
