module
public import USMMathKnowledge.HighSchool
@[expose] public section
noncomputable section
open scoped BigOperators ComplexOrder
namespace USMMathKnowledge

theorem UNI_LEFT_INVERSE_INJECTIVE {A B : Type*} (f : A → B) (g : B → A)
    (h : Function.LeftInverse g f) : Function.Injective f := h.injective
theorem UNI_INJECTIVE_COMPOSE {A B C : Type*} (f : B → C) (g : A → B)
    (hf : Function.Injective f) (hg : Function.Injective g) :
    Function.Injective (f ∘ g) := hf.comp hg
theorem UNI_EQUIV_INJECTIVE {A B : Type*} (e : A ≃ B) : Function.Injective e := e.injective
theorem UNI_COMPOSITION_ASSOC {A B C D : Type*} (f : C → D) (g : B → C) (h : A → B) :
    (f ∘ g) ∘ h=f ∘ (g ∘ h) := rfl
theorem UNI_CONTRAPOSITION (P Q : Prop) (h : P → Q) (hq : ¬Q) : ¬P :=
  fun hp => hq (h hp)
theorem UNI_LINEAR_ZERO {n m : ℕ}
    (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m) : A 0=0 := A.map_zero
theorem UNI_LINEAR_ADD {n m : ℕ}
    (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m) (x y) :
    A (x+y)=A x+A y := A.map_add _ _
theorem UNI_LINEAR_SMUL {n m : ℕ}
    (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m) (c : ℂ) (x) :
    A (c • x)=c • A x := A.map_smul _ _
theorem UNI_NORM_NONNEG {n : ℕ} (x : USMVerification.M1 n) : 0≤‖x‖ := norm_nonneg _
theorem UNI_NORM_ZERO {n : ℕ} (x : USMVerification.M1 n) : ‖x‖=0 ↔ x=0 := norm_eq_zero
theorem UNI_NORM_TRIANGLE {n : ℕ} (x y : USMVerification.M1 n) :
    ‖x+y‖≤‖x‖+‖y‖ := norm_add_le _ _
theorem UNI_NORM_SMUL {n : ℕ} (c : ℂ) (x : USMVerification.ComplexState n) :
    ‖c • x‖=‖c‖*‖x‖ := norm_smul _ _
theorem UNI_INNER_SELF {n : ℕ} (x : USMVerification.ComplexState n) :
    inner ℂ x x=(‖x‖^2 : ℝ) := by simp
theorem UNI_CAUCHY_SCHWARZ {n : ℕ} (x y : USMVerification.ComplexState n) :
    ‖inner ℂ x y‖≤‖x‖*‖y‖ := norm_inner_le_norm _ _
theorem UNI_SUBSPACE_ADD {n : ℕ} (S : Submodule ℝ (USMVerification.M1 n))
    (x y : USMVerification.M1 n) (hx : x∈S) (hy : y∈S) : x+y∈S := S.add_mem hx hy
theorem UNI_SUBSPACE_SMUL {n : ℕ} (S : Submodule ℝ (USMVerification.M1 n))
    (c : ℝ) (x : USMVerification.M1 n) (hx : x∈S) : c • x∈S := S.smul_mem c hx
theorem UNI_TRACE_ADD {n : ℕ} (A B : Matrix (Fin n) (Fin n) ℂ) :
    (A+B).trace=A.trace+B.trace := Matrix.trace_add _ _
theorem UNI_TRACE_CYCLIC {n : ℕ} (A B : Matrix (Fin n) (Fin n) ℂ) :
    (A*B).trace=(B*A).trace := Matrix.trace_mul_comm _ _
theorem UNI_MATRIX_ID_LEFT {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ) :
    (1:Matrix (Fin n) (Fin n) ℂ)*A=A := Matrix.one_mul _
theorem UNI_ADJOINT_INVOLUTION {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ) :
    A.conjTranspose.conjTranspose=A := Matrix.conjTranspose_conjTranspose _
theorem UNI_PSD_ZERO {n : ℕ} : (0:Matrix (Fin n) (Fin n) ℂ).PosSemidef :=
  Matrix.PosSemidef.zero
theorem UNI_UPDATE_SELF {A B : Type*} [DecidableEq A] (f : A → B) (a : A) (b : B) :
    Function.update f a b a=b := Function.update_self _ _ _
theorem UNI_UPDATE_OTHER {A B : Type*} [DecidableEq A] (f : A → B) (a i : A) (b : B)
    (h : i≠a) : Function.update f a b i=f i := Function.update_of_ne h _ _
theorem UNI_SURJECTIVE_COMPOSE {A B C : Type*} (f : B → C) (g : A → B)
    (hf : Function.Surjective f) (hg : Function.Surjective g) :
    Function.Surjective (f ∘ g) := hf.comp hg
theorem UNI_KERNEL_ADD_FORM {n m : ℕ}
    (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m)
    (x y z : USMVerification.ComplexState n) (hx : A x=0) (hy : A y=0) (hz : z=x+y) :
    A z=0 := by rw [hz,A.map_add,hx,hy,zero_add]
theorem UNI_INNER_ZERO {n : ℕ} (x : USMVerification.ComplexState n) : inner ℂ x 0=0 := by simp
theorem UNI_TRACE_ZERO {n : ℕ} : (0:Matrix (Fin n) (Fin n) ℂ).trace=0 := Matrix.trace_zero _ _
theorem UNI_FIBER_OBSTRUCTION {S T : Type*} (P : S → T) (F : S → S) (x y : S)
    (h : P x=P y) (hne : P (F x)≠P (F y)) :
    ¬ ∃ G : T → T, ∀ z, P (F z)=G (P z) := by
  rintro ⟨G,hG⟩
  apply hne
  rw [hG,hG,h]
end USMMathKnowledge
