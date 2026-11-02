module
public import USMVerification.Models
@[expose] public section
noncomputable section
namespace USMVerification
-- Basis coordinates preserve distinct elements without assuming a decoder.
def encode {E : Type*} [Fintype E] [DecidableEq E] (e : E) : EuclideanSpace ℝ E :=
  EuclideanSpace.single e 1

theorem S01 {E : Type*} [Fintype E] [DecidableEq E] :
    Function.Injective (encode (E := E)) := by
  intro a b h
  by_contra hn
  have hh := congrArg (fun x : EuclideanSpace ℝ E => x a) h
  simp [encode, EuclideanSpace.single, hn] at hh

-- A total linear map cannot represent an arbitrary input/output assignment.
theorem arbitrary_relation_not_linear :
    ¬ ∃ A : ℝ →ₗ[ℝ] ℝ, A 0 = 1 := by
  rintro ⟨A,h⟩; simp at h

theorem S06 : (1:ℝ) + 2 = 2 + 1 ∧ (1:ℝ) ≠ 2 := by norm_num

-- Real linear operators already have order effects.
def projX : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun x := (x.1,0)
  map_add' x y := by ext <;> simp
  map_smul' c x := by ext <;> simp

def swap : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun x := (x.2,x.1)
  map_add' x y := by ext <;> simp
  map_smul' c x := by ext <;> simp

theorem S09 : projX (swap (0,1)) ≠ swap (projX (0,1)) := by
  norm_num [projX,swap]

theorem S08 (x y : ℝ × ℝ) (h : x.1 = y.1) :
    (projX x).1 = (projX y).1 := h

-- Recovering the whole state after discarding a coordinate is impossible.
theorem S05_no_decoder : ¬ ∃ d : (ℝ × ℝ) → (ℝ × ℝ),
    ∀ x, d (projX x) = x := by
  rintro ⟨d,hd⟩
  have h0 := hd (0,0); have h1 := hd (0,1)
  have : ((0,0) : ℝ × ℝ) = (0,1) := h0.symm.trans h1
  have := congrArg Prod.snd this
  norm_num at this

-- A shared invariant from a left inverse is sufficient for distinctness.
theorem leftInverse_injective {A B : Type*} (f : A → B) (g : B → A)
    (h : Function.LeftInverse g f) : Function.Injective f := h.injective
end USMVerification
