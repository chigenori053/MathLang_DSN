module
public import Foundation
@[expose] public section

namespace Independent
noncomputable section
-- Written separately from the Rust emitter and checked by definitional equality.
def mv01 : Prop := ∀ (n : ℕ) (U : Foundation.Operator n),
  U.adjoint.comp U = 1 → ∀ x : Foundation.State n, ‖U x‖ = ‖x‖
def mv02 : Prop := ¬ ∀ (n : ℕ) (A : Foundation.Operator n) (x : Foundation.State n),
  ‖A x‖ = ‖x‖
def mv03 : Prop := ∀ x : Bool, id (id x) = x
def mv04 : Prop := ∀ (A B : Type) (E : A → B) (D : B → A) (x y : A),
  x ≠ y → E x = E y → ¬ (D (E x) = x ∧ D (E y) = y)
def mv05 : Prop := ∀ (S : ℕ → ℕ) (i j delta : ℕ),
  j ≠ i → Function.update S i delta j = S j
def mv06 : Prop := ∀ positive valid : Prop, ¬ positive → ¬ (positive ∧ valid)

theorem standalone01 : mv01 := Foundation.unitary_norm
theorem standalone02 : mv02 := Foundation.arbitrary_norm_false
theorem standalone03 : mv03 := by intro x; rfl
theorem standalone04 : mv04 := by
  intro A B E D x y hn hc
  exact Foundation.collision E D x y hn hc
theorem standalone05 : mv05 := by
  intro S i j delta h
  exact Function.update_of_ne h delta S
theorem standalone06 : mv06 := by tauto
end
end Independent
