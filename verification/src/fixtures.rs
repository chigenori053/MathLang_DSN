//! New IR instances of USM claims; no historical USM status is imported.
use crate::{ir::*, lean::Recipe, proof::*};
use std::collections::BTreeMap;
fn b(e: MathExpr) -> Box<MathExpr> {
    Box::new(e)
}
fn v(n: &str) -> MathExpr {
    MathExpr::var(n)
}
fn all(n: &str, t: MathType, e: MathExpr) -> MathExpr {
    MathExpr::forall(n, t, e)
}
fn eq(a: MathExpr, c: MathExpr) -> MathExpr {
    MathExpr::eq(a, c)
}
fn app(f: MathExpr, x: MathExpr) -> MathExpr {
    MathExpr::apply(f, x)
}
fn not(a: MathExpr) -> MathExpr {
    MathExpr::LogicalNot(b(a))
}
fn imp(a: MathExpr, c: MathExpr) -> MathExpr {
    MathExpr::implies(a, c)
}
fn func(a: MathType, c: MathType) -> MathType {
    MathType::Function(Box::new(a), Box::new(c))
}
fn quantify(vars: Vec<(&str, MathType)>, mut body: MathExpr) -> MathExpr {
    for (n, t) in vars.into_iter().rev() {
        body = all(n, t, body);
    }
    body
}
fn obligation(id: &str, goal: MathExpr, kind: ProofKind) -> ProofObligation {
    ProofObligation {
        id: id.into(),
        model_id: "USM-MVF".into(),
        model_revision: 1,
        goal,
        context: Context::default(),
        assumptions: vec![],
        required_lemmas: vec![],
        dependencies: vec![],
        kind,
        verification_method: "Lean4+Mathlib".into(),
        proof_status: FormalStatus::Unknown,
        evidence_refs: vec![],
    }
}
pub fn usm() -> Vec<(ProofObligation, Recipe, bool)> {
    let h = MathType::HilbertSpace(Dimension::Symbol("n".into()));
    let op = MathType::LinearOperator(Box::new(h.clone()), Box::new(h.clone()));
    let preserved = eq(
        MathExpr::Norm(b(app(v("U"), v("x")))),
        MathExpr::Norm(b(v("x"))),
    );
    let mv01 = all(
        "n",
        MathType::Nat,
        all(
            "U",
            op.clone(),
            imp(MathExpr::Unitary(b(v("U"))), all("x", h.clone(), preserved)),
        ),
    );
    let mv02 = all(
        "n",
        MathType::Nat,
        all(
            "A",
            op,
            all(
                "x",
                h,
                eq(
                    MathExpr::Norm(b(app(v("A"), v("x")))),
                    MathExpr::Norm(b(v("x"))),
                ),
            ),
        ),
    );
    let id = MathExpr::Constant(Constant::IdentityFunction(MathType::Bool));
    let mv03 = all(
        "x",
        MathType::Bool,
        eq(app(id.clone(), app(id, v("x"))), v("x")),
    );
    let a = MathType::Abstract("A".into());
    let c = MathType::Abstract("B".into());
    let mv04_goal = imp(
        not(eq(v("x"), v("y"))),
        imp(
            eq(app(v("E"), v("x")), app(v("E"), v("y"))),
            not(MathExpr::and(
                eq(app(v("D"), app(v("E"), v("x"))), v("x")),
                eq(app(v("D"), app(v("E"), v("y"))), v("y")),
            )),
        ),
    );
    let mv04 = quantify(
        vec![
            ("A", MathType::Universe),
            ("B", MathType::Universe),
            ("E", func(a.clone(), c.clone())),
            ("D", func(c, a.clone())),
            ("x", a.clone()),
            ("y", a),
        ],
        mv04_goal,
    );
    let updated = MathExpr::Update(b(v("S")), b(v("i")), b(v("delta")));
    let mv05 = quantify(
        vec![
            ("S", func(MathType::Nat, MathType::Nat)),
            ("i", MathType::Nat),
            ("j", MathType::Nat),
            ("delta", MathType::Nat),
        ],
        imp(
            not(eq(v("j"), v("i"))),
            eq(app(updated, v("j")), app(v("S"), v("j"))),
        ),
    );
    let mv06 = quantify(
        vec![
            ("positive", MathType::Proposition),
            ("valid", MathType::Proposition),
        ],
        imp(
            not(v("positive")),
            not(MathExpr::and(v("positive"), v("valid"))),
        ),
    );
    vec![
        (
            obligation("MV-01", mv01, ProofKind::Conditional),
            Recipe::UnitaryNorm,
            false,
        ),
        (
            obligation("MV-02", mv02, ProofKind::Construction),
            Recipe::ArbitraryNormCounterexample,
            true,
        ),
        (
            obligation("MV-03", mv03, ProofKind::Construction),
            Recipe::Auto,
            false,
        ),
        (
            obligation("MV-04", mv04, ProofKind::Impossibility),
            Recipe::Collision,
            false,
        ),
        (
            obligation("MV-05", mv05, ProofKind::Conditional),
            Recipe::Auto,
            false,
        ),
        (
            obligation("MV-06", mv06, ProofKind::Conditional),
            Recipe::Auto,
            false,
        ),
    ]
}
pub fn auxiliary() -> Vec<(ProofObligation, Recipe, bool)> {
    let up = |s: MathExpr, k: MathExpr| {
        MathExpr::Update(b(s), b(MathExpr::Constant(Constant::Natural(0))), b(k))
    };
    let left = up(
        up(v("S"), MathExpr::Constant(Constant::Natural(1))),
        MathExpr::Constant(Constant::Natural(2)),
    );
    let right = up(
        up(v("S"), MathExpr::Constant(Constant::Natural(2))),
        MathExpr::Constant(Constant::Natural(1)),
    );
    let noncommute = all(
        "S",
        func(MathType::Nat, MathType::Nat),
        not(eq(
            app(left, MathExpr::Constant(Constant::Natural(0))),
            app(right, MathExpr::Constant(Constant::Natural(0))),
        )),
    );
    let phase = all(
        "z",
        MathType::Complex,
        all(
            "x",
            MathType::Complex,
            imp(
                MathExpr::Normalized(b(v("z"))),
                eq(
                    MathExpr::Norm(b(MathExpr::Multiply(b(v("z")), b(v("x"))))),
                    MathExpr::Norm(b(v("x"))),
                ),
            ),
        ),
    );
    let indistinguishable = all(
        "x",
        MathType::Complex,
        all(
            "y",
            MathType::Complex,
            imp(
                eq(MathExpr::Norm(b(v("x"))), MathExpr::Norm(b(v("y")))),
                eq(v("x"), v("y")),
            ),
        ),
    );
    vec![
        (
            obligation(
                "CV-PHASE-COLLISION",
                indistinguishable,
                ProofKind::Construction,
            ),
            Recipe::PhaseCollision,
            true,
        ),
        (
            obligation("CV-UPDATE", noncommute, ProofKind::Construction),
            Recipe::Auto,
            false,
        ),
        (
            obligation("CV-PHASE", phase, ProofKind::Conditional),
            Recipe::PhaseObservation,
            false,
        ),
    ]
}
pub fn model(os: &[ProofObligation]) -> MathematicalModel {
    MathematicalModel {
        model_id: "USM-MVF".into(),
        revision: 1,
        definitions: BTreeMap::new(),
        assumptions: vec![],
        operations: vec![
            "norm".into(),
            "adjoint".into(),
            "application".into(),
            "update".into(),
        ],
        invariants: os.iter().map(|o| o.goal.clone()).collect(),
        proof_obligations: os.to_vec(),
        dependencies: vec![],
    }
}

/// Handwritten Lean statements independently constrain representative node semantics.
pub fn translation_checks() -> Vec<(ProofObligation, String)> {
    let mut cases = vec![];
    let mut add = |id: &str, vars: Vec<(&str, MathType)>, expr: MathExpr, expected: &str| {
        let mut o = obligation(id, eq(expr.clone(), expr), ProofKind::Construction);
        o.context.variables = vars.into_iter().map(|(n, t)| (n.into(), t)).collect();
        cases.push((o, expected.into()));
    };
    let h = MathType::HilbertSpace(Dimension::Fixed(2));
    let op = MathType::LinearOperator(Box::new(h.clone()), Box::new(h.clone()));
    let mat = MathType::Matrix(
        Box::new(MathType::Complex),
        Dimension::Fixed(2),
        Dimension::Fixed(2),
    );
    let vector = |n| MathType::Vector(Box::new(MathType::Complex), Dimension::Fixed(n));
    add(
        "MAP-INNER",
        vec![("x", h.clone()), ("y", h.clone())],
        MathExpr::InnerProduct(b(v("x")), b(v("y"))),
        "∀ (v_x v_y : EuclideanSpace ℂ (Fin 2)), inner ℂ v_x v_y = inner ℂ v_x v_y",
    );
    add("MAP-ADJOINT-TRACE",vec![("x",mat.clone()),("y",mat.clone())],MathExpr::Trace(b(MathExpr::MatrixMultiply(b(MathExpr::Adjoint(b(v("x")))),b(v("y"))))),
        "∀ (v_x v_y : Matrix (Fin 2) (Fin 2) ℂ), (v_x.conjTranspose * v_y).trace = (v_x.conjTranspose * v_y).trace");
    add(
        "MAP-TENSOR",
        vec![("x", vector(2)), ("y", vector(3))],
        MathExpr::TensorIndex(b(MathExpr::TensorProduct(b(v("x")), b(v("y")))), vec![1, 2]),
        "∀ (v_x : Fin 2 → ℂ) (v_y : Fin 3 → ℂ), v_x 1 * v_y 2 = v_x 1 * v_y 2",
    );
    let predicate = MathExpr::and(
        MathExpr::and(
            MathExpr::PositiveSemidefinite(b(v("x"))),
            MathExpr::Hermitian(b(v("x"))),
        ),
        MathExpr::Unitary(b(v("x"))),
    );
    add("MAP-MATRIX-PREDICATES",vec![("x",mat.clone())],predicate,
        "∀ (v_x : Matrix (Fin 2) (Fin 2) ℂ), ((v_x.PosSemidef ∧ v_x.IsHermitian) ∧ v_x.conjTranspose * v_x = 1) = ((v_x.PosSemidef ∧ v_x.IsHermitian) ∧ v_x.conjTranspose * v_x = 1)");
    add("MAP-DENSITY",vec![("x",MathType::DensityOperator(Dimension::Fixed(2)))],MathExpr::Trace(b(v("x"))),
        "∀ (v_x : {m : Matrix (Fin 2) (Fin 2) ℂ // m.PosSemidef ∧ m.trace = 1}), v_x.val.trace = v_x.val.trace");
    add("MAP-LINEAR",vec![("f",op)],MathExpr::LinearMap(b(v("f"))),
        "∀ (v_f : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2)), IsLinearMap ℂ v_f = IsLinearMap ℂ v_f");
    add(
        "MAP-NORMALIZED",
        vec![("x", h)],
        MathExpr::Normalized(b(v("x"))),
        "∀ (v_x : EuclideanSpace ℂ (Fin 2)), (‖v_x‖ = 1) = (‖v_x‖ = 1)",
    );
    add(
        "MAP-VECTOR",
        vec![("x", vector(2)), ("y", vector(2))],
        MathExpr::Add(b(v("x")), b(v("y"))),
        "∀ (v_x v_y : Fin 2 → ℂ), v_x + v_y = v_x + v_y",
    );
    add(
        "MAP-TRANSITION",
        vec![
            ("f", func(MathType::Real, MathType::Real)),
            ("x", MathType::Real),
        ],
        MathExpr::StateTransition(b(v("f")), b(v("x"))),
        "∀ (v_f : ℝ → ℝ) (v_x : ℝ), v_f v_x = v_f v_x",
    );
    let logic = MathExpr::implies(v("p"), MathExpr::LogicalOr(b(v("q")), b(not(v("q")))));
    add(
        "MAP-LOGIC",
        vec![("p", MathType::Proposition), ("q", MathType::Proposition)],
        logic,
        "∀ (v_p v_q : Prop), (v_p → v_q ∨ ¬ v_q) = (v_p → v_q ∨ ¬ v_q)",
    );
    let expr = MathExpr::Power(
        b(MathExpr::Conjugate(b(MathExpr::Divide(
            b(MathExpr::Multiply(
                b(MathExpr::Subtract(
                    b(MathExpr::Add(b(v("x")), b(v("y")))),
                    b(v("x")),
                )),
                b(v("y")),
            )),
            b(v("y")),
        )))),
        b(v("n")),
    );
    let mut o = obligation("MAP-SCALAR", eq(expr.clone(), expr), ProofKind::Conditional);
    o.context.variables = BTreeMap::from([
        ("n".into(), MathType::Nat),
        ("x".into(), MathType::Complex),
        ("y".into(), MathType::Complex),
    ]);
    let premise = not(eq(
        v("y"),
        MathExpr::Constant(Constant::Zero(MathType::Complex)),
    ));
    o.context.assumptions.push(premise.clone());
    o.assumptions.push(Assumption {
        proposition: premise,
        origin: AssumptionOrigin::ExternalSpecification("semantic test input".into()),
    });
    cases.push((o,"∀ (v_n : ℕ) (v_x v_y : ℂ), v_y ≠ 0 → (star (((v_x + v_y - v_x) * v_y) / v_y)) ^ v_n = (star (((v_x + v_y - v_x) * v_y) / v_y)) ^ v_n".into()));
    cases
}
