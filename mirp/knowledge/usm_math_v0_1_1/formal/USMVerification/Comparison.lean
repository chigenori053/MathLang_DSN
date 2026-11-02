module
public import USMVerification.Composition
public import Mathlib.LinearAlgebra.Complex.Module
@[expose] public section
noncomputable section
namespace USMVerification
-- Coordinate realification uses the Euclidean sum-of-squares metric, not the
-- default max norm of a product of normed spaces.
def realify {ι : Type*} (x : ι → ℂ) : ι → ℝ × ℝ := fun i => ((x i).re,(x i).im)
def complexify {ι : Type*} (x : ι → ℝ × ℝ) : ι → ℂ :=
  fun i => ⟨(x i).1,(x i).2⟩

theorem C02 {ι : Type*} (x : ι → ℂ) : complexify (realify x) = x := by rfl

theorem C03 {ι : Type*} : Function.Injective (realify (ι := ι)) := by
  intro x y h
  have := congrArg complexify h
  simpa only [C02] using this

def realEquiv (ι : Type*) : (ι → ℂ) ≃ (ι → ℝ × ℝ) where
  toFun := realify
  invFun := complexify
  left_inv := C02
  right_inv _ := rfl

-- Transport ALL maps, not only the state values, along the same bijection.
def realOp {ι : Type*} (F : (ι → ℂ) → (ι → ℂ)) :
    (ι → ℝ × ℝ) → (ι → ℝ × ℝ) := fun x => realify (F (complexify x))

theorem C11_map {ι : Type*} (F : (ι → ℂ) → (ι → ℂ)) (x : ι → ℂ) :
    realOp F (realify x) = realify (F x) := rfl

theorem C11_comp {ι : Type*} (F G : (ι → ℂ) → (ι → ℂ)) :
    realOp (F ∘ G) = realOp F ∘ realOp G := by rfl

theorem C11_linear : ∃ e : ℂ ≃ₗ[ℝ] ℝ × ℝ, ∀ z, e z = (z.re,z.im) :=
  ⟨Complex.equivRealProdLm,fun _ => rfl⟩

theorem C11_norm (z : ℂ) : ‖z‖ ^ 2 = z.re^2 + z.im^2 := by
  rw [Complex.sq_norm]; simp [Complex.normSq_apply, pow_two]

def realMul (a b : ℝ × ℝ) : ℝ × ℝ :=
  (a.1*b.1-a.2*b.2,a.1*b.2+a.2*b.1)

theorem C11_mul (a b : ℂ) :
    ((a*b).re,(a*b).im) = realMul (a.re,a.im) (b.re,b.im) := by
  simp [realMul,Complex.mul_re,Complex.mul_im]

-- Relative sign is visible to a cross-coordinate observation but not coordinate magnitudes.
def plus : Fin 2 → ℂ := ![1,1]
def minus : Fin 2 → ℂ := ![1,-1]

theorem U04 : (∀ i, ‖plus i‖ = ‖minus i‖) ∧
    plus 0 * star (plus 1) ≠ minus 0 * star (minus 1) := by
  constructor
  · intro i; fin_cases i <;> simp [plus,minus]
  · norm_num [plus,minus]

theorem no_global_phase : ¬ ∃ z : ℂ, minus = z • plus := by
  rintro ⟨z,h⟩
  have h0 := congrFun h 0
  have h1 := congrFun h 1
  simp [plus,minus] at h0 h1
  rw [← h0] at h1
  norm_num at h1

-- Incompatible added constraints really can destroy consistency.
theorem C08 : ¬ ∃ a : ℝ, a > 0 ∧ a ≤ 0 := by
  rintro ⟨a,h,k⟩; linarith

-- Integration: a nonidentity update and a nonidentity extraction commute.
def diagonalUpdate (a b : ℝ) (x : ℝ × ℝ) := (a*x.1,b*x.2)

theorem I03_commute (a b : ℝ) (x : ℝ × ℝ) :
    projX (diagonalUpdate a b x) = diagonalUpdate a b (projX x) := by
  ext <;> simp [projX,diagonalUpdate]

theorem I06 {n : ℕ} (s : Fin n → ℝ) (v : ℝ) :
    (∀ i, extend s v i.castSucc = s i) ∧ extend s v (Fin.last n) = v := by
  simp [extend]
end USMVerification
