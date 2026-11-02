module
public import USMVerification.Density
@[expose] public section
noncomputable section
open scoped TensorProduct
namespace USMVerification
abbrev M1 (n : ℕ) := EuclideanSpace ℝ (Fin n)
abbrev M2 := ComplexState
abbrev M3 (𝕜 : Type*) [RCLike 𝕜] (n m : ℕ) :=
  EuclideanSpace 𝕜 (Fin n) ⊗[𝕜] EuclideanSpace 𝕜 (Fin m)
abbrev M4 := DensityState
-- A hybrid lives on the joint space, not on an unrelated tuple of states.
structure M5 (n m : ℕ) where
  density : DensityState (n*m)
  extraction : ComplexState (n*m) →ₗ[ℂ] ComplexState (n*m)
  idempotent : extraction.comp extraction = extraction

def weight {n : ℕ} (x : ComplexState n) (i : Fin n) : ℝ := ‖x i‖ ^ 2
def Certain {n : ℕ} (x : ComplexState n) : Prop := ∃ i, weight x i = 1
def Uncertain {n : ℕ} (x : ComplexState n) : Prop := ∀ i, weight x i < 1

theorem U02 {n : ℕ} (x : ComplexState n) : ¬ (Certain x ∧ Uncertain x) := by
  rintro ⟨⟨i, hi⟩, hu⟩
  have := hu i
  linarith

theorem phase_weight {n : ℕ} (x : ComplexState n) (z : ℂ) (hz : ‖z‖ = 1)
    (i : Fin n) : weight (z • x) i = weight x i := by
  simp [weight, PiLp.smul_apply, hz]

theorem U05 {n : ℕ} (x : ComplexState n) (z : ℂ) (hz : ‖z‖ = 1) :
    Certain (z • x) ↔ Certain x := by
  simp only [Certain, phase_weight x z hz]

def e0 : ComplexState 2 := EuclideanSpace.single 0 1
def ambiguous : ComplexState 2 := (Real.sqrt 2)⁻¹ •
  (EuclideanSpace.single 0 (1 : ℂ) + EuclideanSpace.single 1 (1 : ℂ))

theorem certain_witness : ‖e0‖ = 1 ∧ Certain e0 := by
  constructor
  · simp [e0, EuclideanSpace.single, PiLp.norm_single]
  · refine ⟨0, ?_⟩
    simp [weight, e0, EuclideanSpace.single]

theorem uncertain_witness : ‖ambiguous‖ = 1 ∧ Uncertain ambiguous := by
  have hs : (Real.sqrt 2)^2 = 2 := Real.sq_sqrt (by norm_num)
  have hp : 0 < Real.sqrt 2 := Real.sqrt_pos.2 (by norm_num)
  have hcoord (i : Fin 2) : weight ambiguous i = (Real.sqrt 2)⁻¹ ^ 2 := by
    fin_cases i <;> simp [ambiguous, weight, EuclideanSpace.single,
      PiLp.smul_apply, Complex.norm_real, abs_of_pos hp]
  have half (i : Fin 2) : weight ambiguous i = 1/2 := by
    rw [hcoord, inv_pow, hs]; norm_num
  constructor
  · have hn : ‖ambiguous‖ ^ 2 = 1 := by
      rw [EuclideanSpace.norm_sq_eq]
      change (∑ i : Fin 2, weight ambiguous i) = 1
      simp [half]
    nlinarith [norm_nonneg ambiguous]
  · intro i; rw [half]; norm_num

-- A transition is a norm-preserving rotation of coordinates, not a tag change.
def hadamard (x : ComplexState 2) : ComplexState 2 :=
  WithLp.toLp 2 ![((Real.sqrt 2)⁻¹ : ℂ)*(x 0 + x 1),
    ((Real.sqrt 2)⁻¹ : ℂ)*(x 0 - x 1)]

theorem U06_forward : hadamard e0 = ambiguous := by
  ext i; fin_cases i <;> simp [hadamard, e0, ambiguous, EuclideanSpace.single,
    PiLp.smul_apply, Complex.real_smul]

theorem U06_backward : hadamard ambiguous = e0 := by
  have hs : (Real.sqrt 2)^2 = 2 := Real.sq_sqrt (by norm_num)
  have hn : (Real.sqrt 2 : ℂ) ≠ 0 := by exact_mod_cast (ne_of_gt (Real.sqrt_pos.2 (by norm_num : (0:ℝ)<2)))
  have hc : (Real.sqrt 2 : ℂ)^2 = 2 := by exact_mod_cast hs
  ext i; fin_cases i <;> simp [hadamard, e0, ambiguous, EuclideanSpace.single,
    PiLp.smul_apply, Complex.real_smul]
  field_simp
  linear_combination -hc

-- No truth assignment is derived from certainty.
theorem U07 : ∃ x : ComplexState 2, Certain x ∧
    ∃ t f : ComplexState 2 → Prop, t x ∧ ¬ f x :=
  ⟨e0, certain_witness.2, fun _ => True, fun _ => False, trivial, not_false⟩
end USMVerification
