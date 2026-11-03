import Mathlib

/-!
# Quantum state realification: general theorems

Formal counterpart of `mathlang_dsn/quantum_real.rsn`. A state is any vector
`ψ : n → ℂ` over a finite index type, and

* `expect A ψ = ∑ i, ∑ j, star (ψ i) * A i j * ψ j` is the same double sum as
  `QuantumReal::QuadForm`;
* `nrm ψ = ⟨ψ|ψ⟩`;
* `realify A ψ = Re ⟨ψ|A|ψ⟩ / Re ⟨ψ|ψ⟩` is the realification used by
  `HERMITIAN_EXPECTATION` and `PAULI_TOMOGRAPHY`.

The theorems in this file hold for every finite dimension, not only for the
bounded test suites executed by ReasonScript:

* REAL: `expect_im_eq_zero`, `hermitianPart_expect`
* RAY_INVARIANT: `realify_smul`
* ENSEMBLE_INDEPENDENT: `ensemble_independent`
-/

open scoped Matrix

namespace QuantumRealification

variable {n : Type*} [Fintype n]

/-- `⟨ψ|A|ψ⟩`, written exactly as the ReasonScript double sum. -/
def expect (A : Matrix n n ℂ) (ψ : n → ℂ) : ℂ :=
  ∑ i, ∑ j, star (ψ i) * A i j * ψ j

/-- `⟨ψ|ψ⟩`. -/
def nrm (ψ : n → ℂ) : ℂ :=
  ∑ i, star (ψ i) * ψ i

/-- Unnormalized pure density matrix `|ψ⟩⟨ψ|`. -/
def outer (ψ : n → ℂ) : Matrix n n ℂ :=
  fun i j => ψ i * star (ψ j)

/-- `Tr (A ρ)`. -/
def trMul (A ρ : Matrix n n ℂ) : ℂ :=
  ∑ i, ∑ j, A i j * ρ j i

/-- Realified expectation `Re⟨ψ|A|ψ⟩ / Re⟨ψ|ψ⟩`. -/
noncomputable def realify (A : Matrix n n ℂ) (ψ : n → ℂ) : ℝ :=
  (expect A ψ).re / (nrm ψ).re

/-! ## REAL -/

theorem star_expect (A : Matrix n n ℂ) (ψ : n → ℂ) :
    star (expect A ψ) = expect Aᴴ ψ := by
  unfold expect
  simp only [star_sum, star_mul, star_star, Matrix.conjTranspose_apply]
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl fun i _ => Finset.sum_congr rfl fun j _ => ?_
  ring

/-- The expectation of a Hermitian matrix is real, in every dimension. -/
theorem expect_im_eq_zero {A : Matrix n n ℂ} (hA : A.IsHermitian) (ψ : n → ℂ) :
    (expect A ψ).im = 0 := by
  have h : star (expect A ψ) = expect A ψ := by
    rw [star_expect, hA.eq]
  have h2 := congrArg Complex.im h
  rw [Complex.star_def, Complex.conj_im] at h2
  linarith

theorem expect_add (A B : Matrix n n ℂ) (ψ : n → ℂ) :
    expect (A + B) ψ = expect A ψ + expect B ψ := by
  simp only [expect, Matrix.add_apply, mul_add, add_mul, Finset.sum_add_distrib]

/-- Symmetrizing an arbitrary operator, `A + Aᴴ`, always yields the real
number `2 Re⟨ψ|A|ψ⟩` (`HERMITIAN_EXPECTATION`). -/
theorem hermitianPart_expect (A : Matrix n n ℂ) (ψ : n → ℂ) :
    expect (A + Aᴴ) ψ = ((2 * (expect A ψ).re : ℝ) : ℂ) := by
  rw [expect_add, ← star_expect]
  apply Complex.ext
  · simp [Complex.star_def]
    ring
  · simp [Complex.star_def]

theorem nrm_eq_expect_one [DecidableEq n] (ψ : n → ℂ) :
    nrm ψ = expect 1 ψ := by
  unfold nrm expect
  refine Finset.sum_congr rfl fun i _ => ?_
  simp [Matrix.one_apply]

theorem nrm_im_eq_zero [DecidableEq n] (ψ : n → ℂ) : (nrm ψ).im = 0 := by
  rw [nrm_eq_expect_one]
  exact expect_im_eq_zero Matrix.isHermitian_one ψ

/-! ## RAY_INVARIANT -/

theorem star_mul_self_eq_normSq (c : ℂ) : star c * c = (Complex.normSq c : ℂ) := by
  rw [Complex.normSq_eq_conj_mul_self]
  rfl

theorem expect_smul (A : Matrix n n ℂ) (c : ℂ) (ψ : n → ℂ) :
    expect A (c • ψ) = (Complex.normSq c : ℂ) * expect A ψ := by
  rw [← star_mul_self_eq_normSq]
  simp only [expect, Pi.smul_apply, smul_eq_mul, star_mul', Finset.mul_sum]
  refine Finset.sum_congr rfl fun i _ => Finset.sum_congr rfl fun j _ => ?_
  ring

theorem nrm_smul (c : ℂ) (ψ : n → ℂ) :
    nrm (c • ψ) = (Complex.normSq c : ℂ) * nrm ψ := by
  rw [← star_mul_self_eq_normSq]
  simp only [nrm, Pi.smul_apply, smul_eq_mul, star_mul', Finset.mul_sum]
  refine Finset.sum_congr rfl fun i _ => ?_
  ring

/-- Realified values depend only on the ray `[ψ]`: global phase and
normalization are unobservable. -/
theorem realify_smul (A : Matrix n n ℂ) {c : ℂ} (hc : c ≠ 0) (ψ : n → ℂ) :
    realify A (c • ψ) = realify A ψ := by
  unfold realify
  rw [expect_smul, nrm_smul, Complex.re_ofReal_mul, Complex.re_ofReal_mul,
    mul_div_mul_left _ _ (Complex.normSq_pos.mpr hc).ne']

/-! ## ENSEMBLE_INDEPENDENT -/

theorem expect_eq_trMul (A : Matrix n n ℂ) (ψ : n → ℂ) :
    expect A ψ = trMul A (outer ψ) := by
  unfold expect trMul outer
  refine Finset.sum_congr rfl fun i _ => Finset.sum_congr rfl fun j _ => ?_
  ring

theorem trMul_sum {ι : Type*} (s : Finset ι) (p : ι → ℂ)
    (ρ : ι → Matrix n n ℂ) (A : Matrix n n ℂ) :
    trMul A (∑ k ∈ s, p k • ρ k) = ∑ k ∈ s, p k * trMul A (ρ k) := by
  simp only [trMul, Matrix.sum_apply, Matrix.smul_apply, smul_eq_mul, Finset.mul_sum]
  symm
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl fun i _ => ?_
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl fun j _ => Finset.sum_congr rfl fun k _ => ?_
  ring

/-- Two ensembles with the same density matrix give the same averaged
expectation for every operator. -/
theorem ensemble_independent {ι κ : Type*} (s : Finset ι) (t : Finset κ)
    (p : ι → ℂ) (q : κ → ℂ) (ψ : ι → n → ℂ) (φ : κ → n → ℂ)
    (h : ∑ k ∈ s, p k • outer (ψ k) = ∑ l ∈ t, q l • outer (φ l))
    (A : Matrix n n ℂ) :
    ∑ k ∈ s, p k * expect A (ψ k) = ∑ l ∈ t, q l * expect A (φ l) := by
  simp only [expect_eq_trMul]
  rw [← trMul_sum, ← trMul_sum, h]

end QuantumRealification
