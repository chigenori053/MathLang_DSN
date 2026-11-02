module
public import USMVerification.Structural
@[expose] public section
noncomputable section
open scoped ComplexOrder
namespace USMVerification

theorem density_conjugation_positive {n : ℕ} (r : DensityState n)
    (A : Matrix (Fin n) (Fin n) ℂ) :
    (A*r.matrix*A.conjTranspose).PosSemidef := r.positive.mul_mul_conjTranspose_same A

theorem density_projection_positive {n : ℕ} (r : DensityState n)
    (P : Matrix (Fin n) (Fin n) ℂ) (hP : P.IsHermitian) :
    (P*r.matrix*P).PosSemidef := by
  simpa only [hP.eq] using density_conjugation_positive r P

theorem fullDensityExtraction : FullDensityExtractionTheory := by
  intro n P r hP _hId
  exact density_projection_positive r P hP

def densityUpdate {n : ℕ} (r : DensityState n)
    (U : Matrix (Fin n) (Fin n) ℂ) (hU : U.conjTranspose*U = 1) : DensityState n where
  matrix := U*r.matrix*U.conjTranspose
  positive := density_conjugation_positive r U
  trace_one := by
    rw [Matrix.trace_mul_cycle,hU,Matrix.one_mul,r.trace_one]

def densityPostselect {n : ℕ} (r : DensityState n)
    (P : Matrix (Fin n) (Fin n) ℂ) (hP : P.IsHermitian)
    (t : ℝ) (ht : 0 < t) (htrace : (P*r.matrix*P).trace = (t:ℂ)) : DensityState n where
  matrix := t⁻¹ • (P*r.matrix*P)
  positive := (density_projection_positive r P hP).smul (inv_nonneg.mpr ht.le)
  trace_one := by
    rw [Matrix.trace_smul,htrace]
    simp [Complex.real_smul,ne_of_gt ht]

-- This is weaker than a density channel classification, which remains outside this file.
theorem densityUpdate_trace {n : ℕ} (r : DensityState n)
    (U : Matrix (Fin n) (Fin n) ℂ) (hU : U.conjTranspose*U = 1) :
    (densityUpdate r U hU).matrix.trace = 1 := (densityUpdate r U hU).trace_one
end USMVerification
