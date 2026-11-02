module
public import USMVerification.Mixed
public import Mathlib.LinearAlgebra.Dimension.Constructions
public import Mathlib.LinearAlgebra.Complex.FiniteDimensional
public import Mathlib.LinearAlgebra.TensorProduct.Finiteness
@[expose] public section
noncomputable section
open scoped TensorProduct ComplexOrder
namespace USMVerification

theorem U01 : ∃ r u : NormalizedState 2, Certain r.val ∧ Uncertain u.val :=
  ⟨⟨e0,certain_witness.1⟩,⟨ambiguous,uncertain_witness.1⟩,
    certain_witness.2,uncertain_witness.2⟩

def raySetoid (n : ℕ) : Setoid (ComplexState n) where
  r := StateEquivalence
  iseqv := ⟨phase_refl,phase_symm,phase_trans⟩

theorem ray_outer {n : ℕ} {x y : ComplexState n} (h : StateEquivalence x y) :
    outer x = outer y := by
  obtain ⟨z,hz,rfl⟩ := h
  exact (p0_06 x z hz).symm

-- Inner-product relations survive a common change of context by an isometry.
theorem S07 {n m : ℕ} (U : M1 n →ₗᵢ[ℝ] M1 m) (x y : M1 n) :
    inner ℝ (U x) (U y) = inner ℝ x y := U.inner_map_map x y

theorem S03 : bind (0 : Fin 2) (1 : Fin 2) ≠ bind (1 : Fin 2) (0 : Fin 2) := by
  intro h
  have he : ((0 : Fin 2),(1 : Fin 2)) = (1,0) := R09 h
  have := congrArg Prod.fst he
  norm_num at this

theorem K08 (s : ℕ → ℝ) (j : ℕ) (v : ℝ) (updates : List (ℕ × ℝ))
    (h : ∀ p ∈ updates, p.1 ≠ j) (hj : s j = v) :
    (updates.foldl (fun t p => replace t p.1 p.2) s) j = v := by
  induction updates generalizing s with
  | nil => exact hj
  | cons p ps ih =>
    apply ih (replace s p.1 p.2) (fun q hq => h q (by simp [hq]))
    rw [K02 s p.1 j p.2 (Ne.symm (h p (by simp)))]
    exact hj

-- Independence/orthogonality, not raw cardinality of real states, bounds capacity.
theorem K09 {n k : ℕ} (v : Fin k → M1 n) (hv : LinearIndependent ℝ v) : k ≤ n := by
  have h := hv.fintype_card_le_finrank
  simpa using h

theorem tensor_dimension (n m : ℕ) : Module.finrank ℝ (M3 ℝ n m) = n*m := by
  rw [Module.finrank_tensorProduct]
  simp

-- Associativity of normalized pure tensor states does not require equality of types.
theorem R08 {n m k : ℕ} (x : M2 n) (y : M2 m) (z : M2 k)
    (hx : ‖x‖ = 1) (hy : ‖y‖ = 1) (hz : ‖z‖ = 1) :
    ‖(x ⊗ₜ[ℂ] y) ⊗ₜ[ℂ] z‖ = 1 := by
  rw [TensorProduct.norm_tmul,TensorProduct.norm_tmul,hx,hy,hz]; norm_num

theorem R10_modusPonens (A B : Prop) (h : A → B) (a : A) : B := h a

-- A whole compatible chain, with a shared invariant, beyond type inhabitation.
theorem I07 (a b : ℝ) (x : ℝ × ℝ) (hx : x.1 = 0) :
    (projX (diagonalUpdate a b (x + (0,1)))).1 = 0 := by
  simp [projX,diagonalUpdate,hx]

-- This is a concrete coexistence witness, not a consistency proof of Lean.
theorem I01 : ∃ h : M5 1 2, h.extraction e0 = e0 ∧ h.extraction e1 = 0 :=
  ⟨hybridWitness,hybrid_nontrivial.1,hybrid_nontrivial.2.1⟩

-- Research targets are propositions; their proofs must be supplied separately.
def FullDensityExtractionTheory : Prop :=
  ∀ (n : ℕ) (P : Matrix (Fin n) (Fin n) ℂ) (r : DensityState n),
    P.IsHermitian → P*P=P → (P*r.matrix*P).PosSemidef

def ComplexTensorRealification : Prop :=
  ∀ n m : ℕ, Nonempty (M3 ℂ n m ≃ₗ[ℂ] M2 (n*m))

-- Comparing dimensions at equal field is meaningful; bare equal real dimension
-- does not supply a complex structure or the matching allowed operations.
def SameDimensionAdvantage : Prop :=
  ∃ n : ℕ, 0 < n ∧ ¬ Nonempty ((Fin n → ℂ) ≃ₗ[ℝ] (Fin n → ℝ))

theorem U02_exhaustive {n : ℕ} (x : NormalizedState n) :
    Certain x.val ∨ Uncertain x.val := by
  by_cases hc : Certain x.val
  · exact Or.inl hc
  · right
    intro i
    have hn : weight x.val i ≤ 1 := by
      have h := Finset.single_le_sum (fun j (_ : j ∈ (Finset.univ : Finset (Fin n))) =>
        sq_nonneg ‖x.val j‖) (Finset.mem_univ i)
      have hs : (∑ j : Fin n, ‖x.val j‖ ^ 2) = 1 := by
        rw [← EuclideanSpace.norm_sq_eq, x.property]; norm_num
      simpa only [weight,hs] using h
    exact lt_of_le_of_ne hn (fun h => hc ⟨i,h⟩)

theorem C04_dimension (n : ℕ) : Module.finrank ℝ (Fin n → ℂ) = 2*n := by
  simp [Module.finrank_pi_fintype,Nat.mul_comm]

theorem C04_same_dimension (n : ℕ) (hn : 0 < n) :
    ¬ Nonempty ((Fin n → ℂ) ≃ₗ[ℝ] (Fin n → ℝ)) := by
  rintro ⟨e⟩
  have h := e.finrank_eq
  rw [C04_dimension,Module.finrank_pi] at h
  simp only [Fintype.card_fin] at h
  omega

theorem C11_tensor_space (n m : ℕ) :
    Nonempty (M3 ℂ n m ≃ₗ[ℂ] M2 (n*m)) := by
  refine ⟨LinearEquiv.ofFinrankEq _ _ ?_⟩
  rw [Module.finrank_tensorProduct]
  simp
end USMVerification
