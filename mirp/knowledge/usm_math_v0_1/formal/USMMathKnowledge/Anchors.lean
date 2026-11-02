module
public import USMMathKnowledge.Specialist
@[expose] public section
noncomputable section
open scoped ComplexOrder
namespace USMMathKnowledge

theorem IR_RET_K02 : (∀ (v_s : (ℕ → ℝ)), (∀ (v_i : ℕ), (∀ (v_j : ℕ), (∀ (v_x : ℝ), ((¬ (v_j = v_i)) → (((Function.update v_s v_i v_x) v_j) = (v_s v_j))))))) := by simp_all
example : ∀ (s : ℕ → ℝ) (i j : ℕ) (x : ℝ), j ≠ i → Function.update s i x j = s j := IR_RET_K02

theorem IR_RET_K03 : (∀ (v_s : (ℕ → ℝ)), (∀ (v_i : ℕ), (∀ (v_x : ℝ), (((Function.update v_s v_i v_x) v_i) = v_x)))) := by simp_all
example : ∀ (s : ℕ → ℝ) (i : ℕ) (x : ℝ), Function.update s i x i = x := IR_RET_K03

theorem IR_RET_K04 : (∀ (v_s : (ℕ → ℝ)), (¬ (((Function.update (Function.update v_s (0 : ℕ) (1 : ℝ)) (0 : ℕ) ((1 : ℝ) + (1 : ℝ))) (0 : ℕ)) = ((Function.update (Function.update v_s (0 : ℕ) ((1 : ℝ) + (1 : ℝ))) (0 : ℕ) (1 : ℝ)) (0 : ℕ))))) := by simp_all
example : ∀ (s : ℕ → ℝ), Function.update (Function.update s 0 1) 0 (1+1) 0 ≠ Function.update (Function.update s 0 (1+1)) 0 1 0 := IR_RET_K04

theorem IR_RET_K06 : (¬ (∃ (v_s : (ℕ → ℝ)), (((v_s (0 : ℕ)) = (0 : ℝ)) ∧ ((v_s (0 : ℕ)) = (1 : ℝ))))) := by simp_all
example : ¬ ∃ s : ℕ → ℝ, s 0 = 0 ∧ s 0 = 1 := IR_RET_K06

theorem IR_RET_K12 : (¬ (∃ (v_f : (ℝ → ℝ)), ((∀ (v_x : ℝ), ((v_f v_x) = v_x)) ∧ ((v_f (0 : ℝ)) = (1 : ℝ))))) := by simp_all
example : ¬ ∃ f : ℝ → ℝ, (∀ x, f x = x) ∧ f 0 = 1 := IR_RET_K12

theorem IR_RET_R04 : (∀ (v_a : (Fin 2 → ℝ)), (∀ (v_b : (Fin 2 → ℝ)), ((((v_a 0) = (0 : ℝ)) ∧ ((v_b 0) = (0 : ℝ))) → (((v_a + v_b) 0) = (0 : ℝ))))) := by simp_all
example : ∀ (a b : Fin 2 → ℝ), a 0 = 0 ∧ b 0 = 0 → (a+b) 0 = 0 := IR_RET_R04

theorem IR_RET_E03 : (∀ (v_p : (EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2))), ((v_p (0 : EuclideanSpace ℂ (Fin 2))) = (0 : EuclideanSpace ℂ (Fin 2)))) := by simp_all
example : ∀ p : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2), p 0 = 0 := IR_RET_E03

theorem IR_RET_E04 : (∀ (v_p : (EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2))), (∀ (v_x : EuclideanSpace ℂ (Fin 2)), (∀ (v_y : EuclideanSpace ℂ (Fin 2)), (∀ (v_z : EuclideanSpace ℂ (Fin 2)), (((v_y = (v_p v_x)) ∧ (v_z = (v_p v_x))) → (v_y = v_z)))))) := by simp_all
example : ∀ (p : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2)) (x y z : EuclideanSpace ℂ (Fin 2)), y = p x ∧ z = p x → y = z := IR_RET_E04

theorem IR_RET_R10 : (∀ (v_a : Prop), (∀ (v_b : Prop), ((v_a → v_b) → (v_a → v_b)))) := by simp_all
example : ∀ a b : Prop, (a → b) → a → b := IR_RET_R10

theorem IR_RET_C08 : (¬ (∃ (v_a : ℝ), ((v_a = (0 : ℝ)) ∧ (v_a = (1 : ℝ))))) := by simp_all
example : ¬ ∃ a : ℝ, a = 0 ∧ a = 1 := IR_RET_C08

theorem IR_RET_I03_NEC : (∀ (v_p : (ℝ → ℝ)), (∀ (v_f : (ℝ → ℝ)), (∀ (v_g : (ℝ → ℝ)), ((∀ (v_x : ℝ), ((v_p (v_f v_x)) = (v_g (v_p v_x)))) → (∀ (v_x : ℝ), (∀ (v_y : ℝ), (((v_p v_x) = (v_p v_y)) → ((v_p (v_f v_x)) = (v_p (v_f v_y)))))))))) := by simp_all
example : ∀ (p f g : ℝ → ℝ), (∀ x, p (f x) = g (p x)) → ∀ x y, p x = p y → p (f x) = p (f y) := IR_RET_I03_NEC

theorem IR_RET_I03_OBSTRUCTION : (∀ (v_p : (ℝ → ℝ)), (∀ (v_f : (ℝ → ℝ)), (∀ (v_x : ℝ), (∀ (v_y : ℝ), ((((v_p v_x) = (v_p v_y)) ∧ (¬ ((v_p (v_f v_x)) = (v_p (v_f v_y))))) → (¬ (∃ (v_g : (ℝ → ℝ)), (∀ (v_z : ℝ), ((v_p (v_f v_z)) = (v_g (v_p v_z))))))))))) := by
  rintro p f x y ⟨he,hne⟩ ⟨g,hg⟩
  apply hne
  rw [hg,hg,he]
example : ∀ (p f : ℝ → ℝ) (x y : ℝ), p x = p y ∧ p (f x) ≠ p (f y) → ¬ ∃ g : ℝ → ℝ, ∀ z, p (f z) = g (p z) := IR_RET_I03_OBSTRUCTION

theorem IR_RET_M1_ZERO_ADD : (∀ (v_x : (Fin 2 → ℝ)), ((v_x + (0 : (Fin 2 → ℝ))) = v_x)) := by simp_all
example : ∀ x : Fin 2 → ℝ, x + 0 = x := IR_RET_M1_ZERO_ADD

theorem IR_RET_M2_INNER_ZERO : (∀ (v_x : EuclideanSpace ℂ (Fin 2)), ((inner ℂ v_x (0 : EuclideanSpace ℂ (Fin 2))) = (0 : ℂ))) := by simp_all
example : ∀ x : EuclideanSpace ℂ (Fin 2), inner ℂ x 0 = 0 := IR_RET_M2_INNER_ZERO

theorem IR_RET_M3_ZERO : (∀ (v_a : (Fin 2 → ℝ)), (((fun (tensor_i_0 : Fin 2) (tensor_i_1 : Fin 2) => (v_a tensor_i_0) * ((0 : (Fin 2 → ℝ)) tensor_i_1)) 0 1) = (0 : ℝ))) := by simp_all
example : ∀ a : Fin 2 → ℝ, a 0 * (0 : Fin 2 → ℝ) 1 = 0 := IR_RET_M3_ZERO

theorem IR_RET_M4_TRACE : (∀ (v_rho : { m : Matrix (Fin 2) (Fin 2) ℂ // m.PosSemidef ∧ m.trace = 1 }), ((v_rho).val.trace = (1 : ℂ))) := by simp_all
example : ∀ rho : {m : Matrix (Fin 2) (Fin 2) ℂ // m.PosSemidef ∧ m.trace = 1}, rho.val.trace = 1 := IR_RET_M4_TRACE

theorem IR_RET_M4_ZERO_PSD : ((0 : Matrix (Fin 2) (Fin 2) ℂ)).PosSemidef := by exact Matrix.PosSemidef.zero
example : (0 : Matrix (Fin 2) (Fin 2) ℂ).PosSemidef := IR_RET_M4_ZERO_PSD

theorem IR_RET_M4_ZERO_TRACE : (((0 : Matrix (Fin 2) (Fin 2) ℂ)).trace = (0 : ℂ)) := by simp_all
example : (0 : Matrix (Fin 2) (Fin 2) ℂ).trace = 0 := IR_RET_M4_ZERO_TRACE
end USMMathKnowledge
