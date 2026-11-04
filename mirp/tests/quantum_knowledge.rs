//! Quantum-mechanics Knowledge pack (mirp/knowledge/qm_v0_1).
//!
//! Every unit marked EXACT_FINITE_CHECKED names a check below. The checks use exact
//! Gaussian-rational arithmetic; factors of 1/√2 are carried as rational matrices scaled
//! by √2 and only compared after they square back to rationals.
use mathlang_mirp::gke::{infer_goal, InferenceLimits, InferenceStatus, RulePredicate, RuleTerm};
use mathlang_mirp::knowledge::{KnowledgeQuery, KnowledgeSpace, KnowledgeUnit};
use mathlang_mirp::memory::MemorySpace;
use mathlang_mirp::{
    Common, Object, PredicateFact, Provenance, SemanticDelta, SemanticState, Status,
};
use std::collections::BTreeSet;
use std::ops::{Add, Mul, Neg, Sub};
use std::path::PathBuf;

// ------------------------------------------------------------------ exact arithmetic
fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Q(i128, i128);
impl Q {
    fn new(n: i128, d: i128) -> Self {
        assert!(d != 0);
        let g = gcd(n, d).max(1) * d.signum();
        Q(n / g, d / g)
    }
    fn int(n: i128) -> Self {
        Q(n, 1)
    }
}
impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(self.0 * o.1 + o.0 * self.1, self.1 * o.1)
    }
}
impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(self.0 * o.0, self.1 * o.1)
    }
}
impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        Q(-self.0, self.1)
    }
}
impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        self + -o
    }
}

/// Gaussian rational re + i·im.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct C(Q, Q);
const ZERO: C = C(Q(0, 1), Q(0, 1));
const ONE: C = C(Q(1, 1), Q(0, 1));
const I: C = C(Q(0, 1), Q(1, 1));
fn r(n: i128, d: i128) -> C {
    C(Q::new(n, d), Q::int(0))
}
fn c(re: (i128, i128), im: (i128, i128)) -> C {
    C(Q::new(re.0, re.1), Q::new(im.0, im.1))
}
impl C {
    fn conj(self) -> C {
        C(self.0, -self.1)
    }
    fn norm_sq(self) -> Q {
        self.0 * self.0 + self.1 * self.1
    }
}
impl Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C(self.0 + o.0, self.1 + o.1)
    }
}
impl Sub for C {
    type Output = C;
    fn sub(self, o: C) -> C {
        C(self.0 - o.0, self.1 - o.1)
    }
}
impl Mul for C {
    type Output = C;
    fn mul(self, o: C) -> C {
        C(self.0 * o.0 - self.1 * o.1, self.0 * o.1 + self.1 * o.0)
    }
}
impl Neg for C {
    type Output = C;
    fn neg(self) -> C {
        C(-self.0, -self.1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M(Vec<Vec<C>>);
impl M {
    fn of(rows: &[&[C]]) -> M {
        M(rows.iter().map(|r| r.to_vec()).collect())
    }
    fn n(&self) -> usize {
        self.0.len()
    }
    fn id(n: usize) -> M {
        M((0..n)
            .map(|i| (0..n).map(|j| if i == j { ONE } else { ZERO }).collect())
            .collect())
    }
    fn zero(n: usize) -> M {
        M(vec![vec![ZERO; n]; n])
    }
    fn dag(&self) -> M {
        let n = self.n();
        M((0..n)
            .map(|i| (0..n).map(|j| self.0[j][i].conj()).collect())
            .collect())
    }
    fn scale(&self, s: C) -> M {
        M(self
            .0
            .iter()
            .map(|row| row.iter().map(|&x| s * x).collect())
            .collect())
    }
    fn tr(&self) -> C {
        (0..self.n()).fold(ZERO, |acc, i| acc + self.0[i][i])
    }
    fn kron(&self, o: &M) -> M {
        let (a, b) = (self.n(), o.n());
        M((0..a * b)
            .map(|i| {
                (0..a * b)
                    .map(|j| self.0[i / b][j / b] * o.0[i % b][j % b])
                    .collect()
            })
            .collect())
    }
    fn apply(&self, v: &[C]) -> Vec<C> {
        self.0
            .iter()
            .map(|row| row.iter().zip(v).fold(ZERO, |acc, (&a, &b)| acc + a * b))
            .collect()
    }
    fn is_hermitian(&self) -> bool {
        *self == self.dag()
    }
    fn is_unitary(&self) -> bool {
        &self.dag() * self == M::id(self.n()) && self * &self.dag() == M::id(self.n())
    }
    /// tr_B for a 4×4 operator on C²⊗C².
    fn partial_trace_b(&self) -> M {
        M((0..2)
            .map(|i| {
                (0..2)
                    .map(|j| self.0[2 * i][2 * j] + self.0[2 * i + 1][2 * j + 1])
                    .collect()
            })
            .collect())
    }
}
impl Mul for &M {
    type Output = M;
    fn mul(self, o: &M) -> M {
        let n = self.n();
        M((0..n)
            .map(|i| {
                (0..n)
                    .map(|j| (0..n).fold(ZERO, |acc, k| acc + self.0[i][k] * o.0[k][j]))
                    .collect()
            })
            .collect())
    }
}
impl Add for &M {
    type Output = M;
    fn add(self, o: &M) -> M {
        M(self
            .0
            .iter()
            .zip(&o.0)
            .map(|(a, b)| a.iter().zip(b).map(|(&x, &y)| x + y).collect())
            .collect())
    }
}
impl Sub for &M {
    type Output = M;
    fn sub(self, o: &M) -> M {
        self + &o.scale(-ONE)
    }
}
fn comm(a: &M, b: &M) -> M {
    &(a * b) - &(b * a)
}
fn anticomm(a: &M, b: &M) -> M {
    &(a * b) + &(b * a)
}
fn inner(a: &[C], b: &[C]) -> C {
    a.iter()
        .zip(b)
        .fold(ZERO, |acc, (&x, &y)| acc + x.conj() * y)
}
fn outer(a: &[C]) -> M {
    M(a.iter()
        .map(|&x| a.iter().map(|&y| x * y.conj()).collect())
        .collect())
}
fn expect(a: &M, v: &[C]) -> C {
    inner(v, &a.apply(v))
}
fn kron_v(a: &[C], b: &[C]) -> Vec<C> {
    a.iter()
        .flat_map(|&x| b.iter().map(move |&y| x * y))
        .collect()
}

fn sx() -> M {
    M::of(&[&[ZERO, ONE], &[ONE, ZERO]])
}
fn sy() -> M {
    M::of(&[&[ZERO, -I], &[I, ZERO]])
}
fn sz() -> M {
    M::of(&[&[ONE, ZERO], &[ZERO, -ONE]])
}
fn paulis() -> [M; 3] {
    [sx(), sy(), sz()]
}
fn levi(j: usize, k: usize, l: usize) -> i128 {
    match (j, k, l) {
        (0, 1, 2) | (1, 2, 0) | (2, 0, 1) => 1,
        (0, 2, 1) | (2, 1, 0) | (1, 0, 2) => -1,
        _ => 0,
    }
}
/// √2·H as an exact rational matrix.
fn h_scaled() -> M {
    M::of(&[&[ONE, ONE], &[ONE, -ONE]])
}
fn cnot() -> M {
    let p0 = M::of(&[&[ONE, ZERO], &[ZERO, ZERO]]);
    let p1 = M::of(&[&[ZERO, ZERO], &[ZERO, ONE]]);
    &p0.kron(&M::id(2)) + &p1.kron(&sx())
}
fn psi() -> Vec<C> {
    vec![r(3, 5), c((0, 1), (4, 5))]
}
/// √2·Bell vectors: Φ+, Φ−, Ψ+, Ψ−.
fn bell_scaled() -> [Vec<C>; 4] {
    [
        vec![ONE, ZERO, ZERO, ONE],
        vec![ONE, ZERO, ZERO, -ONE],
        vec![ZERO, ONE, ONE, ZERO],
        vec![ZERO, ONE, -ONE, ZERO],
    ]
}

// ------------------------------------------------------------------ named checks
fn pauli_hermitian_unitary() {
    for s in paulis() {
        assert!(s.is_hermitian() && s.is_unitary());
        assert_eq!(&s * &s, M::id(2));
        assert_eq!(s.tr(), ZERO);
        assert_eq!(s.0[0][0] * s.0[1][1] - s.0[0][1] * s.0[1][0], -ONE);
    }
}
fn pauli_product() {
    let p = paulis();
    for j in 0..3 {
        for k in 0..3 {
            let mut expected = if j == k { M::id(2) } else { M::zero(2) };
            for (l, pl) in p.iter().enumerate() {
                expected = &expected + &pl.scale(I * r(levi(j, k, l), 1));
            }
            assert_eq!(&p[j] * &p[k], expected, "σ{j}σ{k}");
        }
    }
}
fn pauli_commutator() {
    let p = paulis();
    for j in 0..3 {
        for k in 0..3 {
            let mut expected = M::zero(2);
            for (l, pl) in p.iter().enumerate() {
                expected = &expected + &pl.scale(I * r(2 * levi(j, k, l), 1));
            }
            assert_eq!(comm(&p[j], &p[k]), expected);
            let delta = if j == k { r(2, 1) } else { ZERO };
            assert_eq!(anticomm(&p[j], &p[k]), M::id(2).scale(delta));
        }
    }
}
fn spin_half_algebra() {
    let s: Vec<M> = paulis().iter().map(|m| m.scale(r(1, 2))).collect();
    for (a, b, c3) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
        assert_eq!(comm(&s[a], &s[b]), s[c3].scale(I));
    }
    let casimir = s.iter().fold(M::zero(2), |acc, m| &acc + &(m * m));
    assert_eq!(casimir, M::id(2).scale(r(3, 4)));
    assert_eq!(s[2].apply(&[ONE, ZERO]), vec![r(1, 2), ZERO]);
    assert_eq!(s[2].apply(&[ZERO, ONE]), vec![ZERO, r(-1, 2)]);
}
fn spin_one_algebra() {
    // Mx = √2 Sx, My = √2 Sy are rational; Sz is rational.
    let mx = M::of(&[&[ZERO, ONE, ZERO], &[ONE, ZERO, ONE], &[ZERO, ONE, ZERO]]);
    let my = M::of(&[&[ZERO, -I, ZERO], &[I, ZERO, -I], &[ZERO, I, ZERO]]);
    let szm = M::of(&[&[ONE, ZERO, ZERO], &[ZERO, ZERO, ZERO], &[ZERO, ZERO, -ONE]]);
    assert!(mx.is_hermitian() && my.is_hermitian() && szm.is_hermitian());
    // [Sx,Sy] = ½[Mx,My] = i Sz
    assert_eq!(comm(&mx, &my).scale(r(1, 2)), szm.scale(I));
    // [Sy,Sz] = (1/√2)[My,Sz] = i Sx = i (1/√2) Mx  ⇔  [My,Sz] = i Mx
    assert_eq!(comm(&my, &szm), mx.scale(I));
    // [Sz,Sx] = i Sy  ⇔  [Sz,Mx] = i My
    assert_eq!(comm(&szm, &mx), my.scale(I));
    let casimir = &(&(&mx * &mx) + &(&my * &my)).scale(r(1, 2)) + &(&szm * &szm);
    assert_eq!(casimir, M::id(3).scale(r(2, 1)));
}
fn bloch_pure_state() {
    let v = psi();
    assert_eq!(inner(&v, &v), ONE);
    let rho = outer(&v);
    let bloch: Vec<C> = paulis().iter().map(|s| expect(s, &v)).collect();
    assert_eq!(bloch, vec![ZERO, r(24, 25), r(-7, 25)]);
    let sum_sq = bloch.iter().fold(ZERO, |acc, &x| acc + x * x);
    assert_eq!(sum_sq, ONE);
    let mut rebuilt = M::id(2);
    for (s, &b) in paulis().iter().zip(&bloch) {
        rebuilt = &rebuilt + &s.scale(b);
    }
    assert_eq!(rebuilt.scale(r(1, 2)), rho);
    assert_eq!((&rho * &rho).tr(), ONE);
}
fn born_rule_example() {
    let v = psi();
    let p0 = outer(&[ONE, ZERO]);
    let p1 = outer(&[ZERO, ONE]);
    let probs = [expect(&p0, &v), expect(&p1, &v)];
    assert_eq!(probs, [r(9, 25), r(16, 25)]);
    assert_eq!(probs[0] + probs[1], ONE);
    assert_eq!(&p0 + &p1, M::id(2));
    // |+⟩⟨+| = ½ outer(1,1)
    let plus = outer(&[ONE, ONE]).scale(r(1, 2));
    for p in [&p0, &plus] {
        assert!(p.is_hermitian());
        assert_eq!(p * p, p.clone());
    }
    // |+⟩ = (1,1)/√2: p(±1) = ½ |⟨±|(1,1)⟩|²
    assert_eq!(expect(&p0, &[ONE, ONE]).0 * Q::new(1, 2), Q::new(1, 2));
    assert_eq!(expect(&p1, &[ONE, ONE]).0 * Q::new(1, 2), Q::new(1, 2));
}
fn expectation_values_example() {
    let plus = [ONE, ONE]; // √2|+⟩
    assert_eq!(expect(&sx(), &plus).scale_half(), ONE);
    assert_eq!(expect(&sz(), &plus).scale_half(), ZERO);
    assert_eq!(expect(&sz(), &[ONE, ZERO]), ONE);
    let v = psi();
    assert_eq!(expect(&sz(), &v), r(-7, 25));
    assert_eq!(r(9, 25) - r(16, 25), r(-7, 25));
}
trait Half {
    fn scale_half(self) -> C;
}
impl Half for C {
    fn scale_half(self) -> C {
        self * r(1, 2)
    }
}
fn rational_unitary_example() {
    let u = M::of(&[&[r(3, 5), c((0, 1), (4, 5))], &[c((0, 1), (4, 5)), r(3, 5)]]);
    assert!(u.is_unitary());
    let out = u.apply(&psi());
    assert_eq!(out, vec![r(-7, 25), c((0, 1), (24, 25))]);
    assert_eq!(inner(&out, &out), ONE);
}
fn hadamard_gate() {
    let h = h_scaled();
    assert!(h.is_hermitian());
    assert_eq!(&h * &h, M::id(2).scale(r(2, 1))); // H² = I
    assert_eq!(h.apply(&[ONE, ZERO]), vec![ONE, ONE]); // H|0⟩ = |+⟩
    assert_eq!(&(&h * &sx()) * &h, sz().scale(r(2, 1)));
    assert_eq!(&(&h * &sz()) * &h, sx().scale(r(2, 1)));
}
fn phase_gate() {
    let s = M::of(&[&[ONE, ZERO], &[ZERO, I]]);
    assert!(s.is_unitary());
    assert_eq!(&s * &s, sz());
    assert_eq!(&(&s * &sx()) * &s.dag(), sy());
}
fn cnot_gate() {
    let g = cnot();
    assert!(g.is_hermitian() && g.is_unitary());
    assert_eq!(&g * &g, M::id(4));
}
fn bell_basis() {
    let b = bell_scaled();
    for (i, x) in b.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            assert_eq!(inner(x, y), if i == j { r(2, 1) } else { ZERO });
        }
    }
}
fn bell_circuit() {
    let ket00 = vec![ONE, ZERO, ZERO, ZERO];
    let out = cnot().apply(&h_scaled().kron(&M::id(2)).apply(&ket00));
    assert_eq!(out, bell_scaled()[0]);
}
fn bell_entangled() {
    let rho = outer(&bell_scaled()[0]).scale(r(1, 2));
    assert_eq!(rho.tr(), ONE);
    assert_eq!((&rho * &rho).tr(), ONE);
    let rho_a = rho.partial_trace_b();
    assert_eq!(rho_a, M::id(2).scale(r(1, 2)));
    assert_eq!((&rho_a * &rho_a).tr(), r(1, 2));
    // coefficient matrix c_ij = Φ+_{2i+j} = (1/√2)[[1,0],[0,1]], det = 1/2
    let v = &bell_scaled()[0];
    let det = (v[0] * v[3] - v[1] * v[2]) * r(1, 2);
    assert_eq!(det, r(1, 2));
}
fn superdense_coding() {
    let iy = sy().scale(I); // iσy = [[0,1],[-1,0]]
    let ops = [M::id(2), sx(), iy, sz()];
    let phi = &bell_scaled()[0];
    let encoded: Vec<Vec<C>> = ops.iter().map(|o| o.kron(&M::id(2)).apply(phi)).collect();
    let b = bell_scaled();
    // Φ+, Ψ+, Ψ−, Φ− up to a global sign
    for (e, target) in encoded.iter().zip([&b[0], &b[2], &b[3], &b[1]]) {
        assert!(*e == *target || e.iter().map(|&x| -x).collect::<Vec<_>>() == *target);
    }
    for i in 0..4 {
        for j in 0..4 {
            assert_eq!(
                inner(&encoded[i], &encoded[j]),
                if i == j { r(2, 1) } else { ZERO }
            );
        }
    }
}
fn no_cloning_cnot_example() {
    let g = cnot();
    let (k0, k1) = (vec![ONE, ZERO], vec![ZERO, ONE]);
    assert_eq!(g.apply(&kron_v(&k0, &k0)), kron_v(&k0, &k0));
    assert_eq!(g.apply(&kron_v(&k1, &k0)), kron_v(&k1, &k1));
    // |+⟩|0⟩ (scaled by √2) → √2 Φ+; compare normalized density matrices.
    let plus = vec![ONE, ONE];
    let out = outer(&g.apply(&kron_v(&plus, &k0))).scale(r(1, 2));
    let clone = outer(&kron_v(&plus, &plus)).scale(r(1, 4));
    assert_eq!(out.tr(), ONE);
    assert_eq!(clone.tr(), ONE);
    assert_ne!(out, clone);
}
fn chsh_local_bound() {
    let signs = [-1i128, 1];
    let mut values = BTreeSet::new();
    for a0 in signs {
        for a1 in signs {
            for b0 in signs {
                for b1 in signs {
                    values.insert(a0 * b0 + a0 * b1 + a1 * b0 - a1 * b1);
                }
            }
        }
    }
    assert_eq!(values, BTreeSet::from([-2, 2]));
}
fn chsh_quantum_violation() {
    let (a0, a1) = (sz(), sx());
    // √2 B0 = Z + X, √2 B1 = Z − X
    let (b0, b1) = (&sz() + &sx(), &sz() - &sx());
    for b in [&b0, &b1] {
        assert_eq!(b * b, M::id(2).scale(r(2, 1))); // B² = I
    }
    let s_scaled = &(&(&a0.kron(&b0) + &a0.kron(&b1)) + &a1.kron(&b0)) - &a1.kron(&b1);
    let phi = &bell_scaled()[0];
    // ⟨Φ+|√2 S|Φ+⟩ = ½ ⟨√2Φ+|√2 S|√2Φ+⟩ = 4  ⇒  ⟨S⟩ = 2√2
    let value = expect(&s_scaled, phi) * r(1, 2);
    assert_eq!(value, r(4, 1));
    let s_sq = value * value * r(1, 2);
    assert_eq!(s_sq, r(8, 1));
    assert!(s_sq.0 .0 > Q::int(4).0 * s_sq.0 .1);
}
fn robertson_pauli_example() {
    // normalized rational pure states from Pythagorean triples and rational phases
    let states: Vec<Vec<C>> = vec![
        psi(),
        vec![ONE, ZERO],
        vec![r(3, 5), r(4, 5)],
        vec![r(5, 13), c((0, 1), (12, 13))],
        vec![r(8, 17), c((9, 17), (12, 17))], // (8/17, (15/17)(3/5 + 4i/5))
        vec![r(3, 5), c((-12, 25), (-16, 25))],
    ];
    let mut equality_seen = false;
    for v in &states {
        assert_eq!(inner(v, v), ONE);
        let mean_x = expect(&sx(), v);
        let mean_y = expect(&sy(), v);
        let var_x = ONE - mean_x * mean_x; // σx² = I
        let var_y = ONE - mean_y * mean_y;
        let rhs = expect(&comm(&sx(), &sy()), v);
        let bound = rhs.conj() * rhs * r(1, 4);
        let (lhs, rhs) = ((var_x * var_y).0, bound.0);
        assert!(lhs.0 * rhs.1 >= rhs.0 * lhs.1, "Robertson violated");
        if *v == psi() {
            assert_eq!(var_x, ONE);
            assert_eq!(var_y, r(49, 625));
            assert_eq!(lhs, rhs);
            equality_seen = true;
        }
    }
    assert!(equality_seen);
}

const CHECKS: &[(&str, fn())] = &[
    ("pauli_hermitian_unitary", pauli_hermitian_unitary),
    ("pauli_product", pauli_product),
    ("pauli_commutator", pauli_commutator),
    ("spin_half_algebra", spin_half_algebra),
    ("spin_one_algebra", spin_one_algebra),
    ("bloch_pure_state", bloch_pure_state),
    ("born_rule_example", born_rule_example),
    ("expectation_values_example", expectation_values_example),
    ("rational_unitary_example", rational_unitary_example),
    ("hadamard_gate", hadamard_gate),
    ("phase_gate", phase_gate),
    ("cnot_gate", cnot_gate),
    ("bell_basis", bell_basis),
    ("bell_circuit", bell_circuit),
    ("bell_entangled", bell_entangled),
    ("superdense_coding", superdense_coding),
    ("no_cloning_cnot_example", no_cloning_cnot_example),
    ("chsh_local_bound", chsh_local_bound),
    ("chsh_quantum_violation", chsh_quantum_violation),
    ("robertson_pauli_example", robertson_pauli_example),
];

// ------------------------------------------------------------------ pack loading
fn knowledge_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("knowledge")
}
fn load(path: PathBuf) -> Vec<KnowledgeUnit> {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn qm_units() -> Vec<KnowledgeUnit> {
    load(knowledge_dir().join("qm_v0_1/catalogue/quantum_mechanics.json"))
}
fn usm_space() -> KnowledgeSpace {
    let mut space = KnowledgeSpace::empty();
    space
        .register_batch(load(
            knowledge_dir().join("usm_math_v0_1_1/catalogue/all_with_legacy.json"),
        ))
        .unwrap();
    space
}
fn full_space() -> KnowledgeSpace {
    let mut space = usm_space();
    space.register_batch(qm_units()).unwrap();
    space
}

#[test]
fn every_exact_check_passes_and_is_claimed_by_a_unit() {
    let units = qm_units();
    let claimed: BTreeSet<String> = units
        .iter()
        .filter_map(|u| {
            let m = &u.activation_metadata;
            assert_eq!(m["formal_status"], "UNKNOWN", "{}", u.knowledge_id);
            let exact = m["verification_status"] == "EXACT_FINITE_CHECKED";
            assert_eq!(exact, m.get("exact_check").is_some(), "{}", u.knowledge_id);
            exact.then(|| m["exact_check"]["check_id"].as_str().unwrap().to_string())
        })
        .collect();
    let defined: BTreeSet<String> = CHECKS.iter().map(|(id, _)| id.to_string()).collect();
    assert_eq!(claimed, defined);
    for (_, check) in CHECKS {
        check();
    }
}

#[test]
fn pack_registers_on_top_of_usm_knowledge() {
    let units = qm_units();
    assert_eq!(units.len(), 60);
    // The pack cannot stand alone: anchors refer to kernel-checked USM propositions.
    assert!(KnowledgeSpace::empty()
        .register_batch(units.clone())
        .is_err());
    let base = usm_space();
    assert_eq!(base.len(), 245);
    let space = full_space();
    assert_eq!(space.len(), 305);
    for unit in &units {
        assert_eq!(unit.category, "PHYSICS");
        if let Some(anchor) = unit.activation_metadata.get("formal_anchor") {
            let id = anchor["knowledge_id"].as_str().unwrap();
            assert!(unit.dependencies.iter().any(|d| d == id));
            let anchored = space.get(id).unwrap();
            assert_eq!(
                anchored.activation_metadata["formal_status"], "KERNEL_CHECKED",
                "{id}"
            );
        }
    }
    // Retrieval by goal inside the PHYSICS domain.
    let result = space.query(&KnowledgeQuery {
        domain: "PHYSICS".into(),
        goal: "UNCERTAINTY".into(),
        symbols: vec![],
        relations: vec![],
        required_properties: vec![],
        constraints: vec![],
        semantic_state_refs: vec![],
        limit: 50,
    });
    let found: BTreeSet<_> = result
        .candidates
        .iter()
        .map(|c| c.knowledge_id.as_str())
        .collect();
    for id in [
        "QM_ROBERTSON_UNCERTAINTY",
        "QM_HEISENBERG_UNCERTAINTY",
        "QM_CCR_NO_FINITE_DIMENSION",
        "QM_PAULI_COMMUTATOR",
    ] {
        assert!(found.contains(id), "{id}");
    }
}

#[test]
fn memory_space_round_trip_keeps_quantum_knowledge() {
    let memory = MemorySpace::with_knowledge_space(full_space()).unwrap();
    let path = std::env::temp_dir().join(format!("qm-knowledge-{}.json", std::process::id()));
    memory.save(&path).unwrap();
    let loaded = MemorySpace::load(&path).unwrap();
    std::fs::remove_file(&path).ok();
    assert_eq!(loaded.knowledge_space().len(), 305);
    assert!(loaded
        .knowledge_space()
        .get("QM_POSTULATE_BORN_RULE")
        .is_some());
}

fn sym(name: &str) -> RuleTerm {
    RuleTerm::Symbol(name.into())
}
fn fact(state: &mut SemanticState, predicate: &str, args: &[&str]) {
    let mut object = Object::Predicate(PredicateFact {
        common: Common::unidentified(Status::Known, Provenance::input("test", predicate)),
        predicate: predicate.into(),
        arguments: args.iter().map(|a| sym(a)).collect(),
        negated: false,
    });
    object.normalize_identity(predicate);
    state
        .apply(SemanticDelta {
            added: vec![object],
            ..Default::default()
        })
        .unwrap();
}
fn goal(predicate: &str, args: &[&str]) -> RulePredicate {
    RulePredicate {
        predicate: predicate.into(),
        arguments: args.iter().map(|a| sym(a)).collect(),
        negated: false,
    }
}

#[test]
fn conditional_quantum_rules_derive_only_with_all_premises() {
    let with = MemorySpace::with_knowledge_space(full_space()).unwrap();
    let without = MemorySpace::with_knowledge_space(usm_space()).unwrap();
    type Case<'a> = (
        &'a str,
        Vec<(&'a str, Vec<&'a str>)>,
        (&'a str, Vec<&'a str>),
    );
    let cases: Vec<Case> = vec![
        (
            "QM_UNITARY_PRESERVES_NORM",
            vec![
                ("UNITARY", vec!["U"]),
                ("EVOLVES", vec!["U", "psi0", "psi1"]),
                ("NORMALIZED", vec!["psi0"]),
            ],
            ("NORMALIZED", vec!["psi1"]),
        ),
        (
            "QM_GLOBAL_PHASE_SAME_DENSITY",
            vec![("PHASE_EQUIVALENT", vec!["psi", "phi"])],
            ("SAME_DENSITY", vec!["psi", "phi"]),
        ),
        (
            "QM_HERMITIAN_REAL_SPECTRUM",
            vec![("HERMITIAN", vec!["A"])],
            ("REAL_SPECTRUM", vec!["A"]),
        ),
        (
            "QM_COMMUTING_OBSERVABLES_COMPATIBLE",
            vec![
                ("COMMUTE", vec!["A", "B"]),
                ("HERMITIAN", vec!["A"]),
                ("HERMITIAN", vec!["B"]),
            ],
            ("COMPATIBLE", vec!["A", "B"]),
        ),
        (
            "QM_CONSERVED_QUANTITY",
            vec![("HAMILTONIAN", vec!["H"]), ("COMMUTE", vec!["H", "Lz"])],
            ("CONSERVED", vec!["Lz"]),
        ),
        (
            "QM_TIME_EVOLUTION_UNITARY",
            vec![("HAMILTONIAN", vec!["H"]), ("GENERATES", vec!["H", "U"])],
            ("UNITARY", vec!["U"]),
        ),
        (
            "QM_ORTHOGONAL_STATES_DISTINGUISHABLE",
            vec![("ORTHOGONAL", vec!["up", "down"])],
            ("PERFECTLY_DISTINGUISHABLE", vec!["up", "down"]),
        ),
    ];
    for (id, facts, (predicate, args)) in cases {
        let target = goal(predicate, &args);
        let build = |skip: Option<usize>| {
            let mut state = SemanticState::default();
            for (i, (p, a)) in facts.iter().enumerate() {
                if Some(i) != skip {
                    fact(&mut state, p, a);
                }
            }
            state
        };
        let report = with.infer_generic(&mut build(None), &target, InferenceLimits::default());
        assert_eq!(report.status, InferenceStatus::Derived, "{id}");
        assert!(report.traces.iter().any(|t| t.knowledge_id == id), "{id}");
        let report = without.infer_generic(&mut build(None), &target, InferenceLimits::default());
        assert_ne!(report.status, InferenceStatus::Derived, "{id} without pack");
        for skip in 0..facts.len() {
            let report =
                with.infer_generic(&mut build(Some(skip)), &target, InferenceLimits::default());
            assert_ne!(
                report.status,
                InferenceStatus::Derived,
                "{id} missing premise {skip}"
            );
        }
    }
}

#[test]
fn generated_files_match_the_pack_manifest() {
    use sha2::{Digest, Sha256};
    let pack = knowledge_dir().join("qm_v0_1");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(pack.join("package_manifest.json")).unwrap())
            .unwrap();
    let files = manifest["files"].as_object().unwrap();
    assert!(files.contains_key("catalogue/quantum_mechanics.json"));
    for (path, digest) in files {
        let bytes = std::fs::read(pack.join(path)).unwrap();
        let actual: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(&actual, digest.as_str().unwrap(), "{path}");
    }
}
