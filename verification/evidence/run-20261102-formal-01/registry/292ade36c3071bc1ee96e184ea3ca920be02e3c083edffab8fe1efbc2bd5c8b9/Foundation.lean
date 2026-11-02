module
public import Mathlib.Analysis.InnerProductSpace.Adjoint
public import Mathlib.Analysis.InnerProductSpace.PiL2
public import Mathlib.Tactic.NormNum
public import Mathlib.Tactic.Tauto

@[expose] public section

set_option warningAsError true
namespace Foundation
noncomputable section
abbrev State (n : ℕ) := EuclideanSpace ℂ (Fin n)
abbrev Operator (n : ℕ) := State n →L[ℂ] State n

theorem unitary_norm (n : ℕ) (U : Operator n)
    (h : U.adjoint.comp U = 1) (x : State n) : ‖U x‖ = ‖x‖ :=
  U.norm_map_iff_adjoint_comp_self.mpr h x

-- A concrete member of the original universal domain, with exact arithmetic.
theorem arbitrary_norm_false :
    ¬ (∀ (n : ℕ) (A : Operator n) (x : State n), ‖A x‖ = ‖x‖) := by
  intro h
  have hx : ‖(EuclideanSpace.single (0 : Fin 1) (1 : ℂ))‖ = 1 := by
    simp [EuclideanSpace.single, PiLp.norm_single]
  have bad := h 1 ((2 : ℂ) • (1 : Operator 1))
    (EuclideanSpace.single (0 : Fin 1) (1 : ℂ))
  simp only [smul_apply, one_apply_eq_self,
    norm_smul, hx] at bad
  norm_num at bad

theorem collision {A B : Type} (E : A → B) (D : B → A) (x y : A)
    (hne : x ≠ y) (hc : E x = E y) : ¬ (D (E x) = x ∧ D (E y) = y) := by
  rintro ⟨hx, hy⟩
  exact hne (hx.symm.trans ((congrArg D hc).trans hy))

-- Auxiliary counterexamples remain separately named and audited.
theorem updates_noncommute :
    Function.update (Function.update (fun _ : ℕ => 0) 0 1) 0 2 ≠
    Function.update (Function.update (fun _ : ℕ => 0) 0 2) 0 1 := by
  intro h
  have := congrFun h 0
  norm_num at this

theorem phase_observation (z x : ℂ) (hz : ‖z‖ = 1) : ‖z * x‖ = ‖x‖ := by
  rw [norm_mul, hz, one_mul]

theorem missing_evidence (positive valid : Prop) :
    ¬ positive → ¬ (positive ∧ valid) := by tauto
end
end Foundation
