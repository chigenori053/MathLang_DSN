module
public import USMVerification.Knowledge
@[expose] public section
noncomputable section
namespace USMVerification

theorem E01 (x : ℝ × ℝ) : projX (projX x) = projX x := rfl

theorem E02 {n : ℕ} (P : ComplexState n →ₗ[ℂ] ComplexState n)
    (x : ComplexState n) (h : P x ≠ 0) :
    ‖(‖P x‖⁻¹ : ℝ) • P x‖ = 1 := by
  rw [norm_smul, Real.norm_eq_abs, abs_of_nonneg (inv_nonneg.mpr (norm_nonneg _))]
  exact inv_mul_cancel₀ (norm_ne_zero_iff.mpr h)

theorem E03 : projX (0,1) = 0 := rfl

theorem E04 (x : ℝ × ℝ) (y z : ℝ × ℝ)
    (hy : y = projX x) (hz : z = projX x) : y = z := hy.trans hz.symm

def projY : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) :=
  (LinearMap.id : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ)) - projX

theorem E05 : projX (1,1) ≠ projY (1,1) ∧ projY (projY (1,1)) = projY (1,1) := by
  norm_num [projX,projY,LinearMap.sub_apply]

-- Two orthogonal projections (one onto the diagonal) need not commute.
def projD : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun x := ((x.1+x.2)/2,(x.1+x.2)/2)
  map_add' x y := by ext <;> simp <;> ring
  map_smul' c x := by ext <;> simp <;> ring

theorem projD_idempotent (x : ℝ × ℝ) : projD (projD x) = projD x := by
  ext <;> simp [projD]

theorem E08 : projX (projD (0,1)) ≠ projD (projX (0,1)) := by
  norm_num [projX,projD]

theorem E07 : (projX (1,1)).1 = (1:ℝ) ∧ (projX (1,1)).2 ≠ (1:ℝ) := by
  norm_num [projX]

-- A rescaled projection is not idempotent but normalized direction is unchanged.
theorem E10 (r : ℝ) (hr : 0 < r) (x : ℝ) :
    (r*x) / |r*x| = x / |x| := by
  rw [abs_mul, abs_of_pos hr]
  by_cases hx : x = 0
  · simp [hx]
  · field_simp

-- Linear P always acts on the updated ambient vector, even if it yields zero.
theorem I02 {n : ℕ} (F : ComplexState n → ComplexState n)
    (P : ComplexState n →ₗ[ℂ] ComplexState n) (x : ComplexState n) :
    (P (F x) = 0) ∨ (∃ y : ComplexState n, ‖y‖ = 1) := by
  by_cases h : P (F x) = 0
  · exact Or.inl h
  · exact Or.inr (p0_02 _ h)

-- An induced extracted update exists exactly when P∘F is constant on P's fibers.
theorem I03_factor {S T : Type*} [Nonempty S] (P : S → T) (F : S → S)
    (hP : Function.Surjective P)
    (hf : ∀ x y, P x = P y → P (F x) = P (F y)) :
    ∃ G : T → T, ∀ x, P (F x) = G (P x) := by
  choose lift hl using hP
  exact ⟨fun t => P (F (lift t)), fun x => hf x (lift (P x)) (hl (P x)).symm⟩

theorem I03_necessary {S T : Type*} (P : S → T) (F : S → S)
    (G : T → T) (h : ∀ x, P (F x) = G (P x)) :
    ∀ x y, P x = P y → P (F x) = P (F y) := by
  intro x y hxy; rw [h,h,hxy]

-- Swapping an erased coordinate into a retained one cannot factor through P.
theorem I03_counterexample : ¬ ∃ G : (ℝ × ℝ) → (ℝ × ℝ),
    ∀ x, projX (swap x) = G (projX x) := by
  rintro ⟨G,h⟩
  have hh := (h (0,0)).trans (h (0,1)).symm
  have := congrArg Prod.fst hh
  norm_num [projX,swap] at this
end USMVerification
