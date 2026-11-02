module
public import USMVerification.Comparison
@[expose] public section
noncomputable section
namespace USMVerification

def e1 : ComplexState 2 := EuclideanSpace.single 1 1

theorem e1_normalized : ‖e1‖ = 1 := by
  simp [e1, EuclideanSpace.single, PiLp.norm_single]

def maximallyMixed : DensityState 2 :=
  mix (1/2) (by norm_num) (by norm_num)
    (pureDensity ⟨e0,certain_witness.1⟩) (pureDensity ⟨e1,e1_normalized⟩)

theorem mixed_entries : maximallyMixed.matrix = !![1/2,0;0,1/2] := by
  ext i j; fin_cases i <;> fin_cases j <;>
    norm_num [maximallyMixed,mix,pureDensity,outer,e0,e1,EuclideanSpace.single,Matrix.vecMulVec_apply]

theorem C07_not_pure : ¬ ∃ x : ComplexState 2, outer x = maximallyMixed.matrix := by
  rintro ⟨x,h⟩
  have h00 := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ => A 0 0) h
  have h01 := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ => A 0 1) h
  have h11 := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ => A 1 1) h
  simp only [mixed_entries,outer,Matrix.vecMulVec_apply,Pi.star_apply] at h00 h01 h11
  rcases mul_eq_zero.mp h01 with hx | hx
  · rw [hx,zero_mul] at h00; norm_num at h00
  · have hx' : x 1 = 0 := star_eq_zero.mp hx
    rw [hx',zero_mul] at h11; norm_num at h11

-- Realification of a complex matrix is a real block action preserving matvec.
def realMatVec {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ)
    (x : Fin n → ℝ × ℝ) : Fin n → ℝ × ℝ := fun i =>
  (∑ j, ((A i j).re * (x j).1 - (A i j).im * (x j).2),
   ∑ j, ((A i j).re * (x j).2 + (A i j).im * (x j).1))

theorem C11_matrix {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ) (x : Fin n → ℂ) :
    realify (A.mulVec x) = realMatVec A (realify x) := by
  funext i; apply Prod.ext <;>
    simp [realify,realMatVec,Matrix.mulVec,dotProduct,Complex.mul_re,Complex.mul_im]

-- A concrete nonidentity idempotent on the SAME complex state space as the density.
def coordinateProjection : ComplexState 2 →ₗ[ℂ] ComplexState 2 where
  toFun x := EuclideanSpace.single 0 (x 0)
  map_add' x y := by ext i; fin_cases i <;> simp [EuclideanSpace.single]
  map_smul' c x := by ext i; fin_cases i <;> simp [EuclideanSpace.single,PiLp.smul_apply]

theorem coordinateProjection_idempotent :
    coordinateProjection.comp coordinateProjection = coordinateProjection := by
  ext x i; fin_cases i <;> simp [coordinateProjection,EuclideanSpace.single]

def hybridWitness : M5 1 2 where
  density := maximallyMixed
  extraction := coordinateProjection
  idempotent := coordinateProjection_idempotent

theorem hybrid_nontrivial : hybridWitness.extraction e0 = e0 ∧
    hybridWitness.extraction e1 = 0 ∧ e1 ≠ 0 := by
  constructor
  · ext i; fin_cases i <;> simp [hybridWitness,coordinateProjection,e0,EuclideanSpace.single]
  constructor
  · ext i; fin_cases i <;> simp [hybridWitness,coordinateProjection,e1,EuclideanSpace.single]
  · intro h; have := congrArg (fun x : ComplexState 2 => x 1) h
    norm_num [e1,EuclideanSpace.single] at this

-- A normalized input may be annihilated: total normalized extraction is impossible.
theorem E09_zero : ‖e1‖ = 1 ∧ coordinateProjection e1 = 0 := by
  exact ⟨e1_normalized,hybrid_nontrivial.2.1⟩

-- Extraction of the uncertain witness selects a definite state if nonzero.
theorem E09_select : coordinateProjection ambiguous = (Real.sqrt 2)⁻¹ • e0 := by
  ext i; fin_cases i <;> simp [coordinateProjection,ambiguous,e0,EuclideanSpace.single,PiLp.smul_apply]

-- Projecting a pure state is not trace preserving without postselection/renormalization.
theorem I08_trace_loss : (outer (coordinateProjection e1)).trace ≠ (1:ℂ) := by
  rw [E09_zero.2,outer_trace]; norm_num
end USMVerification
