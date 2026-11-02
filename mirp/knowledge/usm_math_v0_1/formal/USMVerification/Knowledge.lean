module
public import USMVerification.Semantics
@[expose] public section
noncomputable section
namespace USMVerification
-- A valuation is a mathematical state; updates below are maps, not a learner.
def replace (s : ℕ → ℝ) (i : ℕ) (v : ℝ) := Function.update s i v

theorem K02 (s : ℕ → ℝ) (i j : ℕ) (v : ℝ) (h : j ≠ i) :
    replace s i v j = s j := Function.update_of_ne h _ _

theorem K03 (s : ℕ → ℝ) (i : ℕ) (v : ℝ) : replace s i v i = v :=
  Function.update_self _ _ _

theorem K04_distinct (s : ℕ → ℝ) (i j : ℕ) (v w : ℝ) (h : i ≠ j) :
    replace (replace s i v) j w = replace (replace s j w) i v :=
  Function.update_comm h _ _ _

theorem K04_same (s : ℕ → ℝ) :
    replace (replace s 0 1) 0 2 ≠ replace (replace s 0 2) 0 1 := by
  intro h
  have := congrFun h 0
  norm_num [replace] at this

theorem K05 : ¬ Function.Injective (fun s : ℕ → ℝ => replace s 0 0) := by
  intro hi
  have h : replace (fun _ => 0) 0 0 = replace (fun i => if i=0 then 1 else 0) 0 0 := by
    funext i; by_cases h : i=0 <;> simp [replace,h]
  have := congrFun (hi h) 0
  norm_num at this

theorem K06 : ¬ ∃ s : ℕ → ℝ, s 0 = 0 ∧ s 0 = 1 := by
  rintro ⟨s,a,b⟩; linarith

-- Extension genuinely adds a coordinate while retaining all old coordinates.
def extend {n : ℕ} (s : Fin n → ℝ) (v : ℝ) : Fin (n+1) → ℝ := Fin.snoc s v

theorem K10 {n : ℕ} (s : Fin n → ℝ) (v : ℝ) (i : Fin n) :
    extend s v i.castSucc = s i := by simp [extend]

theorem K01 {n : ℕ} (v : ℝ) : Function.Injective (fun s : Fin n → ℝ => extend s v) := by
  intro s t h
  funext i
  have := congrFun h i.castSucc
  simpa [K10] using this

theorem K07 (s : Set ℕ) (i : ℕ) : i ∉ s \ {i} := by simp

-- Iteration preserves a relation when every primitive transformation preserves it.
theorem K11 {S : Type*} (Q : S → Prop) (fs : List (S → S))
    (h : ∀ f ∈ fs, ∀ x, Q x → Q (f x)) (x : S) (hx : Q x) :
    Q (fs.foldl (fun s f => f s) x) := by
  induction fs generalizing x with
  | nil => exact hx
  | cons f fs ih =>
    apply ih (fun g hg => h g (by simp [hg])) (f x) (h f (by simp) x hx)

theorem K12 : ¬ ∃ F : ℝ → ℝ, (∀ x, F x = x) ∧ F 0 = 1 := by
  rintro ⟨F,h,k⟩; have := h 0; linarith

-- Fixed finite state sets have a capacity bound. This is NOT a bound on ℝ^n.
theorem finite_capacity {A B : Type*} [Fintype A] [Fintype B]
    (f : A → B) (h : Function.Injective f) : Fintype.card A ≤ Fintype.card B :=
  Fintype.card_le_of_injective f h
end USMVerification
