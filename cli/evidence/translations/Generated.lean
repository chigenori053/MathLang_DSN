module
public import Foundation
set_option warningAsError true
set_option autoImplicit false
noncomputable section
open scoped ComplexOrder
namespace Generated
public theorem case_0 : (∀ (v_x : EuclideanSpace ℂ (Fin 2)), (∀ (v_y : EuclideanSpace ℂ (Fin 2)), ((inner ℂ v_x v_y) = (inner ℂ v_x v_y)))) := by
  intros
  rfl
example : ∀ (v_x v_y : EuclideanSpace ℂ (Fin 2)), inner ℂ v_x v_y = inner ℂ v_x v_y := case_0
#print axioms Generated.case_0
public theorem case_1 : (∀ (v_x : Matrix (Fin 2) (Fin 2) ℂ), (∀ (v_y : Matrix (Fin 2) (Fin 2) ℂ), ((((v_x).conjTranspose * v_y)).trace = (((v_x).conjTranspose * v_y)).trace))) := by
  intros
  rfl
example : ∀ (v_x v_y : Matrix (Fin 2) (Fin 2) ℂ), (v_x.conjTranspose * v_y).trace = (v_x.conjTranspose * v_y).trace := case_1
#print axioms Generated.case_1
public theorem case_2 : (∀ (v_x : (Fin 2 → ℂ)), (∀ (v_y : (Fin 3 → ℂ)), (((fun (tensor_i_0 : Fin 2) (tensor_i_1 : Fin 3) => (v_x tensor_i_0) * (v_y tensor_i_1)) 1 2) = ((fun (tensor_i_0 : Fin 2) (tensor_i_1 : Fin 3) => (v_x tensor_i_0) * (v_y tensor_i_1)) 1 2)))) := by
  intros
  rfl
example : ∀ (v_x : Fin 2 → ℂ) (v_y : Fin 3 → ℂ), v_x 1 * v_y 2 = v_x 1 * v_y 2 := case_2
#print axioms Generated.case_2
public theorem case_3 : (∀ (v_x : Matrix (Fin 2) (Fin 2) ℂ), ((((v_x).PosSemidef ∧ (v_x).IsHermitian) ∧ ((v_x).conjTranspose * v_x = 1)) = (((v_x).PosSemidef ∧ (v_x).IsHermitian) ∧ ((v_x).conjTranspose * v_x = 1)))) := by
  intros
  rfl
example : ∀ (v_x : Matrix (Fin 2) (Fin 2) ℂ), ((v_x.PosSemidef ∧ v_x.IsHermitian) ∧ v_x.conjTranspose * v_x = 1) = ((v_x.PosSemidef ∧ v_x.IsHermitian) ∧ v_x.conjTranspose * v_x = 1) := case_3
#print axioms Generated.case_3
public theorem case_4 : (∀ (v_x : { m : Matrix (Fin 2) (Fin 2) ℂ // m.PosSemidef ∧ m.trace = 1 }), ((v_x).val.trace = (v_x).val.trace)) := by
  intros
  rfl
example : ∀ (v_x : {m : Matrix (Fin 2) (Fin 2) ℂ // m.PosSemidef ∧ m.trace = 1}), v_x.val.trace = v_x.val.trace := case_4
#print axioms Generated.case_4
public theorem case_5 : (∀ (v_f : (EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2))), ((IsLinearMap ℂ v_f) = (IsLinearMap ℂ v_f))) := by
  intros
  rfl
example : ∀ (v_f : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2)), IsLinearMap ℂ v_f = IsLinearMap ℂ v_f := case_5
#print axioms Generated.case_5
public theorem case_6 : (∀ (v_x : EuclideanSpace ℂ (Fin 2)), ((‖v_x‖ = 1) = (‖v_x‖ = 1))) := by
  intros
  rfl
example : ∀ (v_x : EuclideanSpace ℂ (Fin 2)), (‖v_x‖ = 1) = (‖v_x‖ = 1) := case_6
#print axioms Generated.case_6
public theorem case_7 : (∀ (v_x : (Fin 2 → ℂ)), (∀ (v_y : (Fin 2 → ℂ)), ((v_x + v_y) = (v_x + v_y)))) := by
  intros
  rfl
example : ∀ (v_x v_y : Fin 2 → ℂ), v_x + v_y = v_x + v_y := case_7
#print axioms Generated.case_7
public theorem case_8 : (∀ (v_f : (ℝ → ℝ)), (∀ (v_x : ℝ), ((v_f v_x) = (v_f v_x)))) := by
  intros
  rfl
example : ∀ (v_f : ℝ → ℝ) (v_x : ℝ), v_f v_x = v_f v_x := case_8
#print axioms Generated.case_8
public theorem case_9 : (∀ (v_p : Prop), (∀ (v_q : Prop), ((v_p → (v_q ∨ (¬ v_q))) = (v_p → (v_q ∨ (¬ v_q)))))) := by
  intros
  rfl
example : ∀ (v_p v_q : Prop), (v_p → v_q ∨ ¬ v_q) = (v_p → v_q ∨ ¬ v_q) := case_9
#print axioms Generated.case_9
public theorem case_10 : (∀ (v_n : ℕ), (∀ (v_x : ℂ), (∀ (v_y : ℂ), ((¬ (v_y = (0 : ℂ))) → (((star ((((v_x + v_y) - v_x) * v_y) / v_y)) ^ v_n) = ((star ((((v_x + v_y) - v_x) * v_y) / v_y)) ^ v_n)))))) := by
  intros
  rfl
example : ∀ (v_n : ℕ) (v_x v_y : ℂ), v_y ≠ 0 → (star (((v_x + v_y - v_x) * v_y) / v_y)) ^ v_n = (star (((v_x + v_y - v_x) * v_y) / v_y)) ^ v_n := case_10
#print axioms Generated.case_10
public theorem target : True := True.intro
#print axioms Generated.target
end Generated
