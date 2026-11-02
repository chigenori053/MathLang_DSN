module
public import USMVerification.Foundations
@[expose] public section
noncomputable section
open scoped ComplexOrder
namespace USMVerification
structure DensityState (n : ℕ) where
  matrix : Matrix (Fin n) (Fin n) ℂ
  positive : matrix.PosSemidef
  trace_one : matrix.trace = 1

def outer {n : ℕ} (x : ComplexState n) : Matrix (Fin n) (Fin n) ℂ :=
  Matrix.vecMulVec x.ofLp (star x.ofLp)

theorem outer_positive {n : ℕ} (x : ComplexState n) : (outer x).PosSemidef :=
  Matrix.posSemidef_vecMulVec_self_star _

theorem outer_trace {n : ℕ} (x : ComplexState n) :
    (outer x).trace = (‖x‖ ^ 2 : ℝ) := by
  rw [outer, Matrix.trace_vecMulVec]
  rw [← EuclideanSpace.inner_eq_star_dotProduct x x]
  exact_mod_cast (inner_self_eq_norm_sq_to_K (𝕜 := ℂ) x)

def pureDensity {n : ℕ} (x : NormalizedState n) : DensityState n where
  matrix := outer x.val
  positive := outer_positive x.val
  trace_one := by rw [outer_trace, x.property]; norm_num

def mix {n : ℕ} (a : ℝ) (ha : 0 ≤ a) (ha1 : a ≤ 1)
    (r s : DensityState n) : DensityState n where
  matrix := a • r.matrix + (1-a) • s.matrix
  positive := (r.positive.smul ha).add (s.positive.smul (sub_nonneg.mpr ha1))
  trace_one := by
    rw [Matrix.trace_add, Matrix.trace_smul, Matrix.trace_smul, r.trace_one, s.trace_one]
    simp [Complex.real_smul]

theorem p0_04 {n : ℕ} (a : ℝ) (ha : 0 ≤ a) (ha1 : a ≤ 1)
    (r s : DensityState n) :
    (mix a ha ha1 r s).matrix.PosSemidef ∧
    (mix a ha ha1 r s).matrix.IsHermitian ∧ (mix a ha ha1 r s).matrix.trace = 1 :=
  ⟨(mix a ha ha1 r s).positive, (mix a ha ha1 r s).positive.isHermitian,
    (mix a ha ha1 r s).trace_one⟩

theorem p0_06 {n : ℕ} (x : ComplexState n) (z : ℂ) (hz : ‖z‖ = 1) :
    outer (z • x) = outer x := by
  have h : z * star z = 1 := by
    rw [mul_comm, Complex.star_def, Complex.conj_mul']; simp [hz]
  ext i j
  simp only [outer, Matrix.vecMulVec_apply, Pi.star_apply, PiLp.smul_apply, smul_eq_mul,
    star_mul]
  calc z * x i * (star (x j) * star z) = (z * star z) * (x i * star (x j)) := by ring
       _ = x i * star (x j) := by rw [h, one_mul]
theorem phase_exp (θ : ℝ) : ‖Complex.exp ((θ : ℂ) * Complex.I)‖ = 1 := by
  simp [Complex.norm_exp]

theorem phase_exp_density {n : ℕ} (x : ComplexState n) (θ : ℝ) :
    outer (Complex.exp ((θ : ℂ) * Complex.I) • x) = outer x := p0_06 x _ (phase_exp θ)
end USMVerification
