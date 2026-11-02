module
public import Mathlib.Analysis.InnerProductSpace.Adjoint
public import Mathlib.Analysis.InnerProductSpace.PiL2
public import Mathlib.Analysis.InnerProductSpace.TensorProduct
public import Mathlib.LinearAlgebra.Matrix.PosDef
public import Mathlib.LinearAlgebra.Matrix.Trace
public import Mathlib.Tactic

@[expose] public section
noncomputable section
open scoped TensorProduct
namespace USMVerification
abbrev ComplexState (n : ℕ) := EuclideanSpace ℂ (Fin n)
def NormalizedState (n : ℕ) := {x : ComplexState n // ‖x‖ = 1}
def InnerProduct {n : ℕ} (x y : ComplexState n) : ℂ := inner ℂ x y
abbrev LinearOperator (n : ℕ) := ComplexState n →L[ℂ] ComplexState n
abbrev TensorProductState (n m : ℕ) := ComplexState n ⊗[ℂ] ComplexState m
abbrev StateTransition (n : ℕ) := ComplexState n → ComplexState n
def StateDistance {n : ℕ} (x y : ComplexState n) : ℝ := dist x y
def StateEquivalence {n : ℕ} (x y : ComplexState n) : Prop :=
  ∃ z : ℂ, ‖z‖ = 1 ∧ y = z • x

theorem p0_01 {n : ℕ} (x : ComplexState n) : 0 ≤ ‖x‖ ^ 2 := sq_nonneg _

theorem p0_02 {n : ℕ} (x : ComplexState n) (hx : x ≠ 0) :
    ∃ y : ComplexState n, ‖y‖ = 1 := by
  refine ⟨(‖x‖⁻¹ : ℝ) • x, ?_⟩
  rw [norm_smul, Real.norm_eq_abs, abs_of_nonneg (inv_nonneg.mpr (norm_nonneg _))]
  exact inv_mul_cancel₀ (norm_ne_zero_iff.mpr hx)

theorem p0_03 {n : ℕ} (U : LinearOperator n)
    (h : U.adjoint.comp U = 1) (x : ComplexState n) : ‖U x‖ = ‖x‖ :=
  U.norm_map_iff_adjoint_comp_self.mpr h x

theorem p0_05 {n m : ℕ} (x : ComplexState n) (y : ComplexState m) :
    ‖x ⊗ₜ[ℂ] y‖ = ‖x‖ * ‖y‖ := TensorProduct.norm_tmul x y

theorem phase_refl {n : ℕ} (x : ComplexState n) : StateEquivalence x x := by
  exact ⟨1, norm_one, by simp⟩

theorem phase_symm {n : ℕ} {x y : ComplexState n} (h : StateEquivalence x y) :
    StateEquivalence y x := by
  obtain ⟨z, hz, rfl⟩ := h
  refine ⟨z⁻¹, by simp [hz], ?_⟩
  have hn : z ≠ 0 := by intro h; simp [h] at hz
  simp [smul_smul, hn]

theorem phase_trans {n : ℕ} {x y w : ComplexState n}
    (h : StateEquivalence x y) (k : StateEquivalence y w) : StateEquivalence x w := by
  obtain ⟨a, ha, rfl⟩ := h
  obtain ⟨b, hb, rfl⟩ := k
  exact ⟨b*a, by simp [ha, hb], by simp [smul_smul]⟩

-- Observation is an explicit argument: no assumption that phase carries semantics.
theorem p0_07 {S O : Type*} (observe : S → O) (x y : S) :
    (∃ o, observe x = o ∧ observe y ≠ o) ↔ observe x ≠ observe y := by
  constructor
  · rintro ⟨o, hx, hy⟩ he; exact hy (he ▸ hx)
  · intro h; exact ⟨observe x, rfl, Ne.symm h⟩
end USMVerification
