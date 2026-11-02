module
public import USMVerification.Orthogonality
@[expose] public section
noncomputable section
namespace USMVerification

def ambiguousMinus : ComplexState 2 := (Real.sqrt 2)⁻¹ •
  (EuclideanSpace.single 0 (1 : ℂ) - EuclideanSpace.single 1 (1 : ℂ))

theorem ambiguousMinus_normalized : ‖ambiguousMinus‖ = 1 := by
  have hs : (Real.sqrt 2)^2 = 2 := Real.sq_sqrt (by norm_num)
  have hp : 0 < Real.sqrt 2 := Real.sqrt_pos.2 (by norm_num)
  have hcoord (i : Fin 2) : ‖ambiguousMinus i‖ ^ 2 = (Real.sqrt 2)⁻¹ ^ 2 := by
    fin_cases i <;> simp [ambiguousMinus, EuclideanSpace.single,
      PiLp.smul_apply, Complex.norm_real, abs_of_pos hp]
  have half (i : Fin 2) : ‖ambiguousMinus i‖ ^ 2 = 1/2 := by
    rw [hcoord,inv_pow,hs]; norm_num
  have hn : ‖ambiguousMinus‖ ^ 2 = 1 := by
    rw [EuclideanSpace.norm_sq_eq]; simp [half]
  nlinarith [norm_nonneg ambiguousMinus]

def alternativeEnsemble : DensityState 2 :=
  mix (1/2) (by norm_num) (by norm_num)
    (pureDensity ⟨ambiguous,uncertain_witness.1⟩)
    (pureDensity ⟨ambiguousMinus,ambiguousMinus_normalized⟩)

theorem ensemble_ambiguity : alternativeEnsemble.matrix = maximallyMixed.matrix := by
  have hs : (Real.sqrt 2)^2 = 2 := Real.sq_sqrt (by norm_num)
  have hc : (Real.sqrt 2 : ℂ)⁻¹^2 = (1/2 : ℂ) := by
    have h : (Real.sqrt 2 : ℂ)^2 = 2 := by exact_mod_cast hs
    rw [inv_pow,h]; norm_num
  rw [mixed_entries]
  ext i j; fin_cases i <;> fin_cases j <;>
    simp [alternativeEnsemble,mix,pureDensity,outer,ambiguous,ambiguousMinus,
      EuclideanSpace.single,Matrix.vecMulVec_apply,Complex.real_smul,Complex.ofReal_inv] <;>
    ring_nf <;> exact hc

-- The ensembles differ even modulo global phase: e₀ has no second-coordinate mass.
theorem ensemble_components_distinct : outer ambiguous ≠ outer e0 := by
  intro h
  have hh := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ => A 1 1) h
  simp [outer,Matrix.vecMulVec_apply,ambiguous,e0,EuclideanSpace.single,
    Complex.real_smul] at hh
end USMVerification
