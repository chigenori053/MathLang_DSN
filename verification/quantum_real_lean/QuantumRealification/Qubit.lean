import QuantumRealification.Basic

/-!
# Qubit tomography and counterexamples

* SEPARATING / information completeness of `PAULI_TOMOGRAPHY` for every qubit
  state: `qubit_reconstruction`, `qubit_separating`.
* BOUNDED: `qubit_purity`, `realify_sq_le_one`.
* Counterexamples matching the ReasonScript evaluation:
  `raw_expectation_not_real`, `real_part_not_ray_invariant`,
  `born_not_separating`, and the Bell-state facts for `LOCAL_BLOCH` and
  `BORN_COMPUTATIONAL`.
-/

open scoped Matrix

namespace QuantumRealification

/-- Pauli X, Y, Z and `σ₊ = |0⟩⟨1|`. -/
def σx : Matrix (Fin 2) (Fin 2) ℂ := !![0, 1; 1, 0]
def σy : Matrix (Fin 2) (Fin 2) ℂ := !![0, -Complex.I; Complex.I, 0]
def σz : Matrix (Fin 2) (Fin 2) ℂ := !![1, 0; 0, -1]
def σp : Matrix (Fin 2) (Fin 2) ℂ := !![0, 1; 0, 0]

theorem σx_hermitian : σx.IsHermitian := by
  ext i j
  fin_cases i <;> fin_cases j <;> simp [σx, Matrix.conjTranspose_apply]

theorem σy_hermitian : σy.IsHermitian := by
  ext i j
  fin_cases i <;> fin_cases j <;> simp [σy, Matrix.conjTranspose_apply, Complex.star_def]

theorem σz_hermitian : σz.IsHermitian := by
  ext i j
  fin_cases i <;> fin_cases j <;> simp [σz, Matrix.conjTranspose_apply]

/-! ## PAULI_TOMOGRAPHY is information complete on a qubit -/

/-- `⟨ψ|ψ⟩ I + Σ_P ⟨ψ|P|ψ⟩ P = 2 |ψ⟩⟨ψ|` for every qubit vector. -/
theorem qubit_reconstruction (ψ : Fin 2 → ℂ) :
    nrm ψ • (1 : Matrix (Fin 2) (Fin 2) ℂ) + expect σx ψ • σx + expect σy ψ • σy +
        expect σz ψ • σz = (2 : ℂ) • outer ψ := by
  ext i j
  fin_cases i <;> fin_cases j <;>
    simp [nrm, expect, outer, σx, σy, σz, Fin.sum_univ_two] <;>
    ring_nf <;> simp only [Complex.I_sq] <;> ring

/-- Normalized density matrix rebuilt from the realified Bloch vector. -/
theorem qubit_density_of_bloch (ψ : Fin 2 → ℂ) (h : nrm ψ ≠ 0) :
    (nrm ψ)⁻¹ • outer ψ = (2 : ℂ)⁻¹ • ((1 : Matrix (Fin 2) (Fin 2) ℂ) +
      (expect σx ψ / nrm ψ) • σx + (expect σy ψ / nrm ψ) • σy +
      (expect σz ψ / nrm ψ) • σz) := by
  have hr := qubit_reconstruction ψ
  have ho : outer ψ = (2 : ℂ)⁻¹ • (nrm ψ • (1 : Matrix (Fin 2) (Fin 2) ℂ) +
      expect σx ψ • σx + expect σy ψ • σy + expect σz ψ • σz) := by
    rw [hr, smul_smul]
    norm_num
  rw [ho]
  match_scalars <;> field_simp

/-- Equal Pauli expectations force equal physical states (density matrices). -/
theorem qubit_separating (ψ φ : Fin 2 → ℂ) (hψ : nrm ψ ≠ 0) (hφ : nrm φ ≠ 0)
    (hx : expect σx ψ / nrm ψ = expect σx φ / nrm φ)
    (hy : expect σy ψ / nrm ψ = expect σy φ / nrm φ)
    (hz : expect σz ψ / nrm ψ = expect σz φ / nrm φ) :
    (nrm ψ)⁻¹ • outer ψ = (nrm φ)⁻¹ • outer φ := by
  rw [qubit_density_of_bloch ψ hψ, qubit_density_of_bloch φ hφ, hx, hy, hz]

/-! ## BOUNDED -/

/-- Pure-state purity: `Σ_P ⟨P⟩² = ⟨ψ|ψ⟩²`. -/
theorem qubit_purity (ψ : Fin 2 → ℂ) :
    expect σx ψ ^ 2 + expect σy ψ ^ 2 + expect σz ψ ^ 2 = nrm ψ ^ 2 := by
  simp [nrm, expect, σx, σy, σz, Fin.sum_univ_two]
  ring_nf
  simp only [Complex.I_sq, Complex.I_pow_four]
  ring

theorem real_of_im_zero {z : ℂ} (h : z.im = 0) : z = (z.re : ℂ) :=
  Complex.ext rfl (by simp [h])

/-- The realified Bloch vector of a nonzero qubit state lies on the unit sphere. -/
theorem realify_bloch_unit (ψ : Fin 2 → ℂ) (h : (nrm ψ).re ≠ 0) :
    realify σx ψ ^ 2 + realify σy ψ ^ 2 + realify σz ψ ^ 2 = 1 := by
  have hp := qubit_purity ψ
  rw [real_of_im_zero (expect_im_eq_zero σx_hermitian ψ),
    real_of_im_zero (expect_im_eq_zero σy_hermitian ψ),
    real_of_im_zero (expect_im_eq_zero σz_hermitian ψ),
    real_of_im_zero (nrm_im_eq_zero ψ)] at hp
  have hr : (expect σx ψ).re ^ 2 + (expect σy ψ).re ^ 2 + (expect σz ψ).re ^ 2 =
      (nrm ψ).re ^ 2 := by exact_mod_cast hp
  unfold realify
  field_simp
  linarith

theorem realify_sq_le_one (ψ : Fin 2 → ℂ) (h : (nrm ψ).re ≠ 0) :
    realify σx ψ ^ 2 ≤ 1 ∧ realify σy ψ ^ 2 ≤ 1 ∧ realify σz ψ ^ 2 ≤ 1 := by
  have hu := realify_bloch_unit ψ h
  refine ⟨?_, ?_, ?_⟩ <;> nlinarith [sq_nonneg (realify σx ψ), sq_nonneg (realify σy ψ),
    sq_nonneg (realify σz ψ)]

/-! ## Counterexamples for rejected candidates -/

/-- `RAW_EXPECTATION`: `⟨ψ|σ₊|ψ⟩ = i` for `ψ = (1, i)`. -/
theorem raw_expectation_not_real :
    expect σp ![1, Complex.I] = Complex.I ∧ (expect σp ![1, Complex.I]).im ≠ 0 := by
  have h : expect σp ![1, Complex.I] = Complex.I := by
    simp [expect, σp, Fin.sum_univ_two]
  exact ⟨h, by rw [h]; simp⟩

/-- `REAL_PART`: multiplying by the global phase `i` changes `Re ψ₀`. -/
theorem real_part_not_ray_invariant :
    ((Complex.I • ![(1 : ℂ), 0]) 0).re ≠ (![(1 : ℂ), 0] 0).re := by
  simp

/-- `BORN_COMPUTATIONAL`: `(1, 1)` and `(1, -1)` share outcome probabilities but
are different physical states. -/
theorem born_not_separating :
    (∀ k, star (![(1 : ℂ), 1] k) * ![(1 : ℂ), 1] k =
      star (![(1 : ℂ), -1] k) * ![(1 : ℂ), -1] k) ∧
    outer ![(1 : ℂ), 1] ≠ outer ![(1 : ℂ), -1] := by
  refine ⟨fun k => by fin_cases k <;> simp, fun h => ?_⟩
  have h01 := congrFun (congrFun h 0) 1
  simp [outer] at h01
  norm_num at h01

/-! ## Two qubits: Bell states -/

abbrev Q2 := Fin 2 × Fin 2

/-- `|Φ±⟩ = |00⟩ ± |11⟩` (unnormalized). -/
def phiPlus : Q2 → ℂ := fun x => if x = (0, 0) then 1 else if x = (1, 1) then 1 else 0
def phiMinus : Q2 → ℂ := fun x => if x = (0, 0) then 1 else if x = (1, 1) then -1 else 0

def onFirst (P : Matrix (Fin 2) (Fin 2) ℂ) : Matrix Q2 Q2 ℂ :=
  Matrix.kronecker P 1
def onSecond (P : Matrix (Fin 2) (Fin 2) ℂ) : Matrix Q2 Q2 ℂ :=
  Matrix.kronecker 1 P

/-- `LOCAL_BLOCH` cannot see entanglement: every local Pauli expectation of
`Φ⁺` and `Φ⁻` is zero. -/
theorem bell_local_bloch_zero :
    ∀ P ∈ [σx, σy, σz],
      expect (onFirst P) phiPlus = 0 ∧ expect (onSecond P) phiPlus = 0 ∧
      expect (onFirst P) phiMinus = 0 ∧ expect (onSecond P) phiMinus = 0 := by
  intro P hP
  simp only [List.mem_cons, List.not_mem_nil, or_false] at hP
  rcases hP with rfl | rfl | rfl <;>
    simp [expect, onFirst, onSecond, phiPlus, phiMinus, σx, σy, σz,
      Fintype.sum_prod_type, Fin.sum_univ_two, Matrix.kronecker, Matrix.kroneckerMap_apply,
      Matrix.one_apply]

/-- `BORN_COMPUTATIONAL` gives identical probabilities for `Φ⁺` and `Φ⁻`. -/
theorem bell_born_equal (x : Q2) :
    star (phiPlus x) * phiPlus x = star (phiMinus x) * phiMinus x := by
  unfold phiPlus phiMinus
  split_ifs <;> simp

/-- `PAULI_TOMOGRAPHY` separates them through the correlator `⟨X ⊗ X⟩`. -/
theorem bell_xx_separates :
    expect (Matrix.kronecker σx σx) phiPlus = 2 ∧
      expect (Matrix.kronecker σx σx) phiMinus = -2 ∧ nrm phiPlus = 2 ∧ nrm phiMinus = 2 := by
  refine ⟨?_, ?_, ?_, ?_⟩ <;>
    simp [expect, nrm, phiPlus, phiMinus, σx, Fintype.sum_prod_type, Fin.sum_univ_two,
      Matrix.kronecker, Matrix.kroneckerMap_apply] <;> norm_num

theorem bell_states_differ : outer phiPlus ≠ outer phiMinus := by
  intro h
  have h' := congrFun (congrFun h (0, 0)) (1, 1)
  simp [outer, phiPlus, phiMinus] at h'
  norm_num at h'

end QuantumRealification
