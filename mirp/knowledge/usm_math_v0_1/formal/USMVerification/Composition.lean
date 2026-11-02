module
public import USMVerification.Extraction
public import Mathlib.LinearAlgebra.TensorProduct.Associator
@[expose] public section
noncomputable section
open scoped TensorProduct
namespace USMVerification

theorem R01 {n m : ℕ} (x : ComplexState n) (y : ComplexState m)
    (hx : ‖x‖ = 1) (hy : ‖y‖ = 1) : ‖x ⊗ₜ[ℂ] y‖ = 1 := by
  rw [TensorProduct.norm_tmul,hx,hy,one_mul]

theorem R02 (x y z : ℝ) :
    TensorProduct.assoc ℝ ℝ ℝ ℝ ((x ⊗ₜ[ℝ] y) ⊗ₜ[ℝ] z) =
      x ⊗ₜ[ℝ] (y ⊗ₜ[ℝ] z) := TensorProduct.assoc_tmul _ _ _

theorem R03 (x y : ℝ) : TensorProduct.comm ℝ ℝ ℝ (x ⊗ₜ[ℝ] y) = y ⊗ₜ[ℝ] x :=
  TensorProduct.comm_tmul ℝ ℝ ℝ x y

-- Product binding is ordered and invertible. Tensor binding additionally has scale ambiguity.
theorem R07_ambiguity : ((2:ℝ) ⊗ₜ[ℝ] (1:ℝ)) = ((1:ℝ) ⊗ₜ[ℝ] (2:ℝ)) := by
  have h := TensorProduct.smul_tmul (R := ℝ) (2:ℝ) (1:ℝ) (1:ℝ)
  simpa using h

def role {E : Type*} [DecidableEq E] (e : E) : E → ℝ := Pi.single e 1

def bind {E F : Type*} [DecidableEq E] [DecidableEq F] (e : E) (f : F) :
    (E → ℝ) ⊗[ℝ] (F → ℝ) := role e ⊗ₜ[ℝ] role f

def readRole {E F : Type*} (e : E) (f : F) :
    (E → ℝ) ⊗[ℝ] (F → ℝ) →ₗ[ℝ] ℝ :=
  (TensorProduct.lid ℝ ℝ).toLinearMap.comp
    (TensorProduct.map (LinearMap.proj e) (LinearMap.proj f))

theorem read_bind {E F : Type*} [DecidableEq E] [DecidableEq F]
    (e e' : E) (f f' : F) :
    readRole e f (bind e' f') = role e' e * role f' f := by
  simp [readRole,bind,TensorProduct.map_tmul,TensorProduct.lid_tmul]

theorem R09 {E F : Type*} [DecidableEq E] [DecidableEq F] :
    Function.Injective (fun p : E × F => bind p.1 p.2) := by
  intro a b h
  have hh := congrArg (readRole a.1 a.2) h
  rw [read_bind,read_bind] at hh
  by_cases he : a.1 = b.1
  · by_cases hf : a.2 = b.2
    · exact Prod.ext he hf
    · simp [role,he,hf] at hh
  · simp [role,Pi.single_apply,he] at hh

theorem R04 (x y : ℝ × ℝ) (hx : x.1 = 0) (hy : y.1 = 0) :
    (x+y).1 = 0 := by simp [hx,hy]

theorem R05 : ¬ ∃ s : ℝ, s = 0 ∧ s = 1 := by
  rintro ⟨s,h,k⟩; linarith

theorem R06 : ∃ x y : ℝ × ℝ, x ≠ y := ⟨(1,0),(0,1),by norm_num⟩

-- Tensor formation alone places no restriction on an independent truth valuation.
theorem R10 : ∃ (x y : ℝ) (truth : (ℝ ⊗[ℝ] ℝ) → Prop),
    ¬ truth (x ⊗ₜ[ℝ] y) := ⟨1,1,fun _ => False,not_false⟩
end USMVerification
