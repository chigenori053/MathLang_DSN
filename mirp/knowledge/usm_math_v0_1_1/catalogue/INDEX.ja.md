# 段階別Knowledge索引

証明の量化・前提は下記のLean型およびknowledge_registry.jsonに保存。検索ビューは同一内容の別表現を含むため、数理命題数へ加算しない。

## Stage 1: 高校数学までの基礎

| ID | 区分 | 型・前提を含む命題 |
|---|---|---|
| IR_RET_R10 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_a v_b : Prop), (v_a → v_b) → v_a → v_b` |
| IR_RET_M1_ZERO_ADD | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_x : Fin 2 → ℝ), v_x + 0 = v_x` |
| IR_RET_M3_ZERO | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_a : Fin 2 → ℝ), (fun tensor_i_0 tensor_i_1 => v_a tensor_i_0 * 0 tensor_i_1) 0 1 = 0` |
| HS_EQ_SYMM | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), a = b → b = a` |
| HS_EQ_TRANS | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a = b → b = c → a = c` |
| HS_NE_ZERO_PRODUCT | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), a * b ≠ 0 ↔ a ≠ 0 ∧ b ≠ 0` |
| HS_ZERO_PRODUCT | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), a * b = 0 ↔ a = 0 ∨ b = 0` |
| HS_ADD_COMM | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), a + b = b + a` |
| HS_ADD_ASSOC | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a + b + c = a + (b + c)` |
| HS_MUL_COMM | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), a * b = b * a` |
| HS_DISTRIBUTE | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a * (b + c) = a * b + a * c` |
| HS_DIFF_SQUARES | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), (a - b) * (a + b) = a ^ 2 - b ^ 2` |
| HS_SQUARE_EXPAND | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), (a + b) ^ 2 = a ^ 2 + 2 * a * b + b ^ 2` |
| HS_SQUARE_NONNEG | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), 0 ≤ a ^ 2` |
| HS_ABS_NONNEG | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), 0 ≤ \|a\|` |
| HS_ABS_TRIANGLE | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), \|a + b\| ≤ \|a\| + \|b\|` |
| HS_ABS_MUL | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), \|a * b\| = \|a\| * \|b\|` |
| HS_DIV_SELF | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), a ≠ 0 → a / a = 1` |
| HS_LINEAR_SOLUTION | MATHEMATICAL_PROPOSITION | `∀ (a b x : ℝ), a ≠ 0 → a * x + b = 0 → x = -b / a` |
| HS_QUADRATIC_FACTORS | MATHEMATICAL_PROPOSITION | `∀ (x r s : ℝ), (x - r) * (x - s) = 0 ↔ x = r ∨ x = s` |
| HS_SQRT_SQUARE | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), 0 ≤ a → √a ^ 2 = a` |
| HS_SQRT_NONNEG | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), 0 ≤ √a` |
| HS_DISTANCE_PYTHAGOREAN | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ), √(x ^ 2 + y ^ 2) ^ 2 = x ^ 2 + y ^ 2` |
| HS_AMGM_SQUARE_FORM | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), 4 * a * b ≤ (a + b) ^ 2` |
| HS_ADD_ORDER | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a ≤ b → a + c ≤ b + c` |
| HS_MUL_ORDER | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a ≤ b → 0 ≤ c → a * c ≤ b * c` |
| HS_POW_ADD | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ) (n m : ℕ), a ^ (n + m) = a ^ n * a ^ m` |
| HS_GEOMETRIC_STEP | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ) (n : ℕ), a ^ (n + 1) = a ^ n * a` |
| HS_EXP_ADD | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), Real.exp (a + b) = Real.exp a * Real.exp b` |
| HS_LOG_MUL | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), 0 < a → 0 < b → Real.log (a * b) = Real.log a + Real.log b` |
| HS_SIN_COS_SQUARE | MATHEMATICAL_PROPOSITION | `∀ (a : ℝ), Real.sin a ^ 2 + Real.cos a ^ 2 = 1` |
| HS_SIN_ADD | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), Real.sin (a + b) = Real.sin a * Real.cos b + Real.cos a * Real.sin b` |
| HS_COS_ADD | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ), Real.cos (a + b) = Real.cos a * Real.cos b - Real.sin a * Real.sin b` |
| HS_COMPLEX_COORDINATES | MATHEMATICAL_PROPOSITION | `∀ (z : ℂ), ↑z.re + ↑z.im * Complex.I = z` |
| HS_COMPLEX_NORM_SQUARE | MATHEMATICAL_PROPOSITION | `∀ (z : ℂ), ‖z‖ ^ 2 = z.re ^ 2 + z.im ^ 2` |
| HS_COMPLEX_CONJ_PRODUCT | MATHEMATICAL_PROPOSITION | `∀ (z : ℂ), z * star z = ↑(‖z‖ ^ 2)` |
| HS_DOT_COMM | MATHEMATICAL_PROPOSITION | `∀ (x y : Fin 3 → ℝ), ∑ i, x i * y i = ∑ i, y i * x i` |
| HS_DOT_NONNEG | MATHEMATICAL_PROPOSITION | `∀ (x : Fin 3 → ℝ), 0 ≤ ∑ i, x i * x i` |
| HS_AFFINE_COMPOSE | MATHEMATICAL_PROPOSITION | `∀ (a b c d x : ℝ), a * (c * x + d) + b = a * c * x + (a * d + b)` |
| HS_FINITE_SUM_ADD | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ) (x y : Fin n → ℝ), ∑ i, (x i + y i) = ∑ i, x i + ∑ i, y i` |
| HS_ARITHMETIC_SUM | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ), ∑ i ∈ Finset.range n, ↑i = ↑n * (↑n - 1) / 2` |
| HS_COUNT_PAIRS | MATHEMATICAL_PROPOSITION | `∀ (A : Type u_1) (B : Type u_2) [inst : Fintype A] [inst_1 : Fintype B],   Fintype.card (A × B) = Fintype.card A * Fintype.card B` |
| HS_FINITE_UNION | MATHEMATICAL_PROPOSITION | `∀ (A : Type u_1) [inst : DecidableEq A] (s t : Finset A), (s ∪ t).card + (s ∩ t).card = s.card + t.card` |
| HS_PROBABILITY_COMPLEMENT_BOUNDS | MATHEMATICAL_PROPOSITION | `∀ (p : ℝ), 0 ≤ p → p ≤ 1 → 0 ≤ 1 - p ∧ 1 - p ≤ 1` |
| HS_FINITE_EXPECTATION_ADD | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ) (p x y : Fin n → ℝ), ∑ i, p i * (x i + y i) = ∑ i, p i * x i + ∑ i, p i * y i` |
| HS_WEIGHTED_VARIANCE_NONNEG | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ) (p x : Fin n → ℝ) (μ : ℝ), (∀ (i : Fin n), 0 ≤ p i) → 0 ≤ ∑ i, p i * (x i - μ) ^ 2` |
| HS_DERIV_SQUARE | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), HasDerivAt (fun t => t ^ 2) (2 * x) x` |
| HS_DERIV_SIN | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), HasDerivAt Real.sin (Real.cos x) x` |
| HS_CONTINUOUS_QUADRATIC | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), Continuous fun x => a * x ^ 2 + b * x + c` |
| HS_INTEGRAL_POWER | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ) (n : ℕ), ∫ (x : ℝ) in a..b, x ^ n = (b ^ (n + 1) - a ^ (n + 1)) / (↑n + 1)` |
| HS_NAT_DIVISION | MATHEMATICAL_PROPOSITION | `∀ (a b : ℕ), a % b + b * (a / b) = a` |
| HS_REMAINDER_BOUND | MATHEMATICAL_PROPOSITION | `∀ (a b : ℕ), 0 < b → a % b < b` |
| HS_GCD_COMM | MATHEMATICAL_PROPOSITION | `∀ (a b : ℕ), a.gcd b = b.gcd a` |
| HS_GCD_DIVIDES | MATHEMATICAL_PROPOSITION | `∀ (a b : ℕ), a.gcd b ∣ a` |
| HS_FACTORIAL_STEP | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ), (n + 1).factorial = (n + 1) * n.factorial` |
| HS_PASCAL | MATHEMATICAL_PROPOSITION | `∀ (n k : ℕ), (n + 1).choose (k + 1) = n.choose k + n.choose (k + 1)` |
| HS_EXP_POSITIVE | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), 0 < Real.exp x` |
| HS_LOG_EXP | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), Real.log (Real.exp x) = x` |
| HS_SIN_BOUND | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), \|Real.sin x\| ≤ 1` |
| HS_QUADRATIC_MINIMUM | MATHEMATICAL_PROPOSITION | `∀ (x a : ℝ), a ≤ (x - a) ^ 2 + a` |
| HS_LE_TRANS | MATHEMATICAL_PROPOSITION | `∀ (a b c : ℝ), a ≤ b → b ≤ c → a ≤ c` |
| HS_MODUS_PONENS | MATHEMATICAL_PROPOSITION | `∀ (P Q : Prop), (P → Q) → P → Q` |
| HS_VECTOR_ADD_ZERO | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ) (x : Fin n → ℝ), x + 0 = x` |
| HS_MUL_ZERO | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ), x * 0 = 0` |

## Stage 2: 大学数学

| ID | 区分 | 型・前提を含む命題 |
|---|---|---|
| IR_RET_K02 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_s : ℕ → ℝ) (v_i v_j : ℕ) (v_x : ℝ), ¬v_j = v_i → Function.update v_s v_i v_x v_j = v_s v_j` |
| IR_RET_K03 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_s : ℕ → ℝ) (v_i : ℕ) (v_x : ℝ), Function.update v_s v_i v_x v_i = v_x` |
| IR_RET_K04 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_s : ℕ → ℝ),   ¬Function.update (Function.update v_s 0 1) 0 (1 + 1) 0 = Function.update (Function.update v_s 0 (1 + 1)) 0 1 0` |
| IR_RET_K06 | EXACT_IR_RETRIEVAL_VIEW | `¬∃ v_s, v_s 0 = 0 ∧ v_s 0 = 1` |
| IR_RET_K12 | EXACT_IR_RETRIEVAL_VIEW | `¬∃ v_f, (∀ (v_x : ℝ), v_f v_x = v_x) ∧ v_f 0 = 1` |
| IR_RET_R04 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_a v_b : Fin 2 → ℝ), v_a 0 = 0 ∧ v_b 0 = 0 → (v_a + v_b) 0 = 0` |
| IR_RET_E03 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_p : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2)), v_p 0 = 0` |
| IR_RET_E04 | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_p : EuclideanSpace ℂ (Fin 2) →L[ℂ] EuclideanSpace ℂ (Fin 2)) (v_x v_y v_z : EuclideanSpace ℂ (Fin 2)),   v_y = v_p v_x ∧ v_z = v_p v_x → v_y = v_z` |
| IR_RET_C08 | EXACT_IR_RETRIEVAL_VIEW | `¬∃ v_a, v_a = 0 ∧ v_a = 1` |
| IR_RET_I03_NEC | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_p v_f v_g : ℝ → ℝ),   (∀ (v_x : ℝ), v_p (v_f v_x) = v_g (v_p v_x)) → ∀ (v_x v_y : ℝ), v_p v_x = v_p v_y → v_p (v_f v_x) = v_p (v_f v_y)` |
| IR_RET_I03_OBSTRUCTION | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_p v_f : ℝ → ℝ) (v_x v_y : ℝ),   v_p v_x = v_p v_y ∧ ¬v_p (v_f v_x) = v_p (v_f v_y) → ¬∃ v_g, ∀ (v_z : ℝ), v_p (v_f v_z) = v_g (v_p v_z)` |
| IR_RET_M2_INNER_ZERO | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_x : EuclideanSpace ℂ (Fin 2)), inner ℂ v_x 0 = 0` |
| IR_RET_M4_ZERO_PSD | EXACT_IR_RETRIEVAL_VIEW | `Matrix.PosSemidef 0` |
| IR_RET_M4_ZERO_TRACE | EXACT_IR_RETRIEVAL_VIEW | `Matrix.trace 0 = 0` |
| UNI_LEFT_INVERSE_INJECTIVE | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} (f : A → B) (g : B → A), Function.LeftInverse g f → Function.Injective f` |
| UNI_INJECTIVE_COMPOSE | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} {C : Type u_3} (f : B → C) (g : A → B),   Function.Injective f → Function.Injective g → Function.Injective (f ∘ g)` |
| UNI_EQUIV_INJECTIVE | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} (e : A ≃ B), Function.Injective ⇑e` |
| UNI_COMPOSITION_ASSOC | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} {C : Type u_3} {D : Type u_4} (f : C → D) (g : B → C) (h : A → B),   (f ∘ g) ∘ h = f ∘ g ∘ h` |
| UNI_CONTRAPOSITION | MATHEMATICAL_PROPOSITION | `∀ (P Q : Prop), (P → Q) → ¬Q → ¬P` |
| UNI_LINEAR_ZERO | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m), A 0 = 0` |
| UNI_LINEAR_ADD | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m)   (x y : USMVerification.ComplexState n), A (x + y) = A x + A y` |
| UNI_LINEAR_SMUL | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m) (c : ℂ)   (x : USMVerification.ComplexState n), A (c • x) = c • A x` |
| UNI_NORM_NONNEG | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.M1 n), 0 ≤ ‖x‖` |
| UNI_NORM_ZERO | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.M1 n), ‖x‖ = 0 ↔ x = 0` |
| UNI_NORM_TRIANGLE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x y : USMVerification.M1 n), ‖x + y‖ ≤ ‖x‖ + ‖y‖` |
| UNI_NORM_SMUL | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (c : ℂ) (x : USMVerification.ComplexState n), ‖c • x‖ = ‖c‖ * ‖x‖` |
| UNI_INNER_SELF | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), inner ℂ x x = ↑(‖x‖ ^ 2)` |
| UNI_CAUCHY_SCHWARZ | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x y : USMVerification.ComplexState n), ‖inner ℂ x y‖ ≤ ‖x‖ * ‖y‖` |
| UNI_SUBSPACE_ADD | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (S : Submodule ℝ (USMVerification.M1 n)) (x y : USMVerification.M1 n), x ∈ S → y ∈ S → x + y ∈ S` |
| UNI_SUBSPACE_SMUL | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (S : Submodule ℝ (USMVerification.M1 n)) (c : ℝ), ∀ x ∈ S, c • x ∈ S` |
| UNI_TRACE_ADD | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (A B : Matrix (Fin n) (Fin n) ℂ), (A + B).trace = A.trace + B.trace` |
| UNI_TRACE_CYCLIC | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (A B : Matrix (Fin n) (Fin n) ℂ), (A * B).trace = (B * A).trace` |
| UNI_MATRIX_ID_LEFT | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ), 1 * A = A` |
| UNI_ADJOINT_INVOLUTION | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ), A.conjTranspose.conjTranspose = A` |
| UNI_PSD_ZERO | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ}, Matrix.PosSemidef 0` |
| UNI_UPDATE_SELF | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} [inst : DecidableEq A] (f : A → B) (a : A) (b : B), Function.update f a b a = b` |
| UNI_UPDATE_OTHER | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} [inst : DecidableEq A] (f : A → B) (a i : A) (b : B),   i ≠ a → Function.update f a b i = f i` |
| UNI_SURJECTIVE_COMPOSE | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} {C : Type u_3} (f : B → C) (g : A → B),   Function.Surjective f → Function.Surjective g → Function.Surjective (f ∘ g)` |
| UNI_KERNEL_ADD_FORM | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (A : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState m)   (x y z : USMVerification.ComplexState n), A x = 0 → A y = 0 → z = x + y → A z = 0` |
| UNI_INNER_ZERO | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), inner ℂ x 0 = 0` |
| UNI_TRACE_ZERO | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ}, Matrix.trace 0 = 0` |
| UNI_FIBER_OBSTRUCTION | MATHEMATICAL_PROPOSITION | `∀ {S : Type u_1} {T : Type u_2} (P : S → T) (F : S → S) (x y : S),   P x = P y → P (F x) ≠ P (F y) → ¬∃ G, ∀ (z : S), P (F z) = G (P z)` |
| USM_I03_COMMUTE | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ) (x : ℝ × ℝ),   USMVerification.projX (USMVerification.diagonalUpdate a b x) =     USMVerification.diagonalUpdate a b (USMVerification.projX x)` |
| USM_I06 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (s : Fin n → ℝ) (v : ℝ),   (∀ (i : Fin n), USMVerification.extend s v i.castSucc = s i) ∧ USMVerification.extend s v (Fin.last n) = v` |
| USM_R04 | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ × ℝ), x.1 = 0 → y.1 = 0 → (x + y).1 = 0` |
| USM_R05 | MATHEMATICAL_PROPOSITION | `¬∃ s, s = 0 ∧ s = 1` |
| USM_R06 | MATHEMATICAL_PROPOSITION | `∃ x y, x ≠ y` |
| USM_E01 | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ × ℝ), USMVerification.projX (USMVerification.projX x) = USMVerification.projX x` |
| USM_E02 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (P : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState n)   (x : USMVerification.ComplexState n), P x ≠ 0 → ‖‖P x‖⁻¹ • P x‖ = 1` |
| USM_E03 | MATHEMATICAL_PROPOSITION | `USMVerification.projX (0, 1) = 0` |
| USM_E04 | MATHEMATICAL_PROPOSITION | `∀ (x y z : ℝ × ℝ), y = USMVerification.projX x → z = USMVerification.projX x → y = z` |
| USM_E05 | MATHEMATICAL_PROPOSITION | `USMVerification.projX (1, 1) ≠ USMVerification.projY (1, 1) ∧   USMVerification.projY (USMVerification.projY (1, 1)) = USMVerification.projY (1, 1)` |
| USM_PROJD_IDEMPOTENT | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ × ℝ), USMVerification.projD (USMVerification.projD x) = USMVerification.projD x` |
| USM_E08 | MATHEMATICAL_PROPOSITION | `USMVerification.projX (USMVerification.projD (0, 1)) ≠ USMVerification.projD (USMVerification.projX (0, 1))` |
| USM_E07 | MATHEMATICAL_PROPOSITION | `(USMVerification.projX (1, 1)).1 = 1 ∧ (USMVerification.projX (1, 1)).2 ≠ 1` |
| USM_E10 | MATHEMATICAL_PROPOSITION | `∀ (r : ℝ), 0 < r → ∀ (x : ℝ), r * x / \|r * x\| = x / \|x\|` |
| USM_I03_FACTOR | MATHEMATICAL_PROPOSITION | `∀ {S : Type u_1} {T : Type u_2} [Nonempty S] (P : S → T) (F : S → S),   Function.Surjective P → (∀ (x y : S), P x = P y → P (F x) = P (F y)) → ∃ G, ∀ (x : S), P (F x) = G (P x)` |
| USM_I03_NECESSARY | MATHEMATICAL_PROPOSITION | `∀ {S : Type u_1} {T : Type u_2} (P : S → T) (F : S → S) (G : T → T),   (∀ (x : S), P (F x) = G (P x)) → ∀ (x y : S), P x = P y → P (F x) = P (F y)` |
| USM_I03_COUNTEREXAMPLE | MATHEMATICAL_PROPOSITION | `¬∃ G, ∀ (x : ℝ × ℝ), USMVerification.projX (USMVerification.swap x) = G (USMVerification.projX x)` |
| USM_P0_01 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), 0 ≤ ‖x‖ ^ 2` |
| USM_P0_02 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), x ≠ 0 → ∃ y, ‖y‖ = 1` |
| USM_P0_03 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (U : USMVerification.LinearOperator n),   ContinuousLinearMap.adjoint U ∘SL U = 1 → ∀ (x : USMVerification.ComplexState n), ‖U x‖ = ‖x‖` |
| USM_P0_07 | MATHEMATICAL_PROPOSITION | `∀ {S : Type u_1} {O : Type u_2} (observe : S → O) (x y : S),   (∃ o, observe x = o ∧ observe y ≠ o) ↔ observe x ≠ observe y` |
| USM_K02 | MATHEMATICAL_PROPOSITION | `∀ (s : ℕ → ℝ) (i j : ℕ) (v : ℝ), j ≠ i → USMVerification.replace s i v j = s j` |
| USM_K03 | MATHEMATICAL_PROPOSITION | `∀ (s : ℕ → ℝ) (i : ℕ) (v : ℝ), USMVerification.replace s i v i = v` |
| USM_K04_DISTINCT | MATHEMATICAL_PROPOSITION | `∀ (s : ℕ → ℝ) (i j : ℕ) (v w : ℝ),   i ≠ j →     USMVerification.replace (USMVerification.replace s i v) j w =       USMVerification.replace (USMVerification.replace s j w) i v` |
| USM_K04_SAME | MATHEMATICAL_PROPOSITION | `∀ (s : ℕ → ℝ),   USMVerification.replace (USMVerification.replace s 0 1) 0 2 ≠     USMVerification.replace (USMVerification.replace s 0 2) 0 1` |
| USM_K05 | MATHEMATICAL_PROPOSITION | `¬Function.Injective fun s => USMVerification.replace s 0 0` |
| USM_K06 | MATHEMATICAL_PROPOSITION | `¬∃ s, s 0 = 0 ∧ s 0 = 1` |
| USM_K10 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (s : Fin n → ℝ) (v : ℝ) (i : Fin n), USMVerification.extend s v i.castSucc = s i` |
| USM_K01 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (v : ℝ), Function.Injective fun s => USMVerification.extend s v` |
| USM_K07 | MATHEMATICAL_PROPOSITION | `∀ (s : Set ℕ) (i : ℕ), i ∉ s \ {i}` |
| USM_K11 | MATHEMATICAL_PROPOSITION | `∀ {S : Type u_1} (Q : S → Prop) (fs : List (S → S)),   (∀ f ∈ fs, ∀ (x : S), Q x → Q (f x)) → ∀ (x : S), Q x → Q (List.foldl (fun s f => f s) x fs)` |
| USM_K12 | MATHEMATICAL_PROPOSITION | `¬∃ F, (∀ (x : ℝ), F x = x) ∧ F 0 = 1` |
| USM_FINITE_CAPACITY | MATHEMATICAL_PROPOSITION | `∀ {A : Type u_1} {B : Type u_2} [inst : Fintype A] [inst_1 : Fintype B] (f : A → B),   Function.Injective f → Fintype.card A ≤ Fintype.card B` |
| USM_PROJX_SELFADJOINT_COORDINATES | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ × ℝ),   x.1 * (USMVerification.projX y).1 + x.2 * (USMVerification.projX y).2 =     (USMVerification.projX x).1 * y.1 + (USMVerification.projX x).2 * y.2` |
| USM_PROJD_SELFADJOINT_COORDINATES | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ × ℝ),   x.1 * (USMVerification.projD y).1 + x.2 * (USMVerification.projD y).2 =     (USMVerification.projD x).1 * y.1 + (USMVerification.projD x).2 * y.2` |
| USM_PROJD_RESIDUAL_ORTHOGONAL | MATHEMATICAL_PROPOSITION | `∀ (x : ℝ × ℝ),   (USMVerification.projD x).1 * (x.1 - (USMVerification.projD x).1) +       (USMVerification.projD x).2 * (x.2 - (USMVerification.projD x).2) =     0` |
| USM_S01 | MATHEMATICAL_PROPOSITION | `∀ {E : Type u_1} [inst : Fintype E] [inst_1 : DecidableEq E], Function.Injective USMVerification.encode` |
| USM_ARBITRARY_RELATION_NOT_LINEAR | MATHEMATICAL_PROPOSITION | `¬∃ A, A 0 = 1` |
| USM_S06 | MATHEMATICAL_PROPOSITION | `1 + 2 = 2 + 1 ∧ 1 ≠ 2` |
| USM_S09 | MATHEMATICAL_PROPOSITION | `USMVerification.projX (USMVerification.swap (0, 1)) ≠ USMVerification.swap (USMVerification.projX (0, 1))` |
| USM_S08 | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ × ℝ), x.1 = y.1 → (USMVerification.projX x).1 = (USMVerification.projX y).1` |
| USM_S05_NO_DECODER | MATHEMATICAL_PROPOSITION | `¬∃ d, ∀ (x : ℝ × ℝ), d (USMVerification.projX x) = x` |
| USM_S07 | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (U : USMVerification.M1 n →ₗᵢ[ℝ] USMVerification.M1 m) (x y : USMVerification.M1 n),   inner ℝ (U x) (U y) = inner ℝ x y` |
| USM_K08 | MATHEMATICAL_PROPOSITION | `∀ (s : ℕ → ℝ) (j : ℕ) (v : ℝ) (updates : List (ℕ × ℝ)),   (∀ p ∈ updates, p.1 ≠ j) → s j = v → List.foldl (fun t p => USMVerification.replace t p.1 p.2) s updates j = v` |
| USM_K09 | MATHEMATICAL_PROPOSITION | `∀ {n k : ℕ} (v : Fin k → USMVerification.M1 n), LinearIndependent ℝ v → k ≤ n` |
| USM_I07 | MATHEMATICAL_PROPOSITION | `∀ (a b : ℝ) (x : ℝ × ℝ), x.1 = 0 → (USMVerification.projX (USMVerification.diagonalUpdate a b (x + (0, 1)))).1 = 0` |
| USM_C04_DIMENSION | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ), Module.finrank ℝ (Fin n → ℂ) = 2 * n` |
| USM_C04_SAME_DIMENSION | MATHEMATICAL_PROPOSITION | `∀ (n : ℕ), 0 < n → ¬Nonempty ((Fin n → ℂ) ≃ₗ[ℝ] Fin n → ℝ)` |

## Stage 3: 専門領域・USM

| ID | 区分 | 型・前提を含む命題 |
|---|---|---|
| IR_RET_M4_TRACE | EXACT_IR_RETRIEVAL_VIEW | `∀ (v_rho : { m // m.PosSemidef ∧ m.trace = 1 }), (↑v_rho).trace = 1` |
| SP_UNITARY_NORM_FORM | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (U : USMVerification.LinearOperator n) (x y : USMVerification.ComplexState n) (r : ℝ),   ContinuousLinearMap.adjoint U ∘SL U = 1 → y = U x → ‖x‖ = r → ‖y‖ = r` |
| SP_DENSITY_TRACE_SUBTYPE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (rho : { m // m.PosSemidef ∧ m.trace = 1 }), (↑rho).trace = 1` |
| USM_C02 | MATHEMATICAL_PROPOSITION | `∀ {ι : Type u_1} (x : ι → ℂ), USMVerification.complexify (USMVerification.realify x) = x` |
| USM_C03 | MATHEMATICAL_PROPOSITION | `∀ {ι : Type u_1}, Function.Injective USMVerification.realify` |
| USM_C11_MAP | MATHEMATICAL_PROPOSITION | `∀ {ι : Type u_1} (F : (ι → ℂ) → ι → ℂ) (x : ι → ℂ),   USMVerification.realOp F (USMVerification.realify x) = USMVerification.realify (F x)` |
| USM_C11_COMP | MATHEMATICAL_PROPOSITION | `∀ {ι : Type u_1} (F G : (ι → ℂ) → ι → ℂ),   USMVerification.realOp (F ∘ G) = USMVerification.realOp F ∘ USMVerification.realOp G` |
| USM_C11_LINEAR | MATHEMATICAL_PROPOSITION | `∃ e, ∀ (z : ℂ), e z = (z.re, z.im)` |
| USM_C11_MUL | MATHEMATICAL_PROPOSITION | `∀ (a b : ℂ), ((a * b).re, (a * b).im) = USMVerification.realMul (a.re, a.im) (b.re, b.im)` |
| USM_U04 | MATHEMATICAL_PROPOSITION | `(∀ (i : Fin 2), ‖USMVerification.plus i‖ = ‖USMVerification.minus i‖) ∧   USMVerification.plus 0 * star (USMVerification.plus 1) ≠ USMVerification.minus 0 * star (USMVerification.minus 1)` |
| USM_NO_GLOBAL_PHASE | MATHEMATICAL_PROPOSITION | `¬∃ z, USMVerification.minus = z • USMVerification.plus` |
| USM_C08 | MATHEMATICAL_PROPOSITION | `¬∃ a > 0, a ≤ 0` |
| USM_R01 | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (x : USMVerification.ComplexState n) (y : USMVerification.ComplexState m),   ‖x‖ = 1 → ‖y‖ = 1 → ‖x ⊗ₜ[ℂ] y‖ = 1` |
| USM_R02 | MATHEMATICAL_PROPOSITION | `∀ (x y z : ℝ), (TensorProduct.assoc ℝ ℝ ℝ ℝ) (x ⊗ₜ[ℝ] y ⊗ₜ[ℝ] z) = x ⊗ₜ[ℝ] (y ⊗ₜ[ℝ] z)` |
| USM_R03 | MATHEMATICAL_PROPOSITION | `∀ (x y : ℝ), (TensorProduct.comm ℝ ℝ ℝ) (x ⊗ₜ[ℝ] y) = y ⊗ₜ[ℝ] x` |
| USM_R07_AMBIGUITY | MATHEMATICAL_PROPOSITION | `2 ⊗ₜ[ℝ] 1 = 1 ⊗ₜ[ℝ] 2` |
| USM_READ_BIND | MATHEMATICAL_PROPOSITION | `∀ {E : Type u_1} {F : Type u_2} [inst : DecidableEq E] [inst_1 : DecidableEq F] (e e' : E) (f f' : F),   (USMVerification.readRole e f) (USMVerification.bind e' f') = USMVerification.role e' e * USMVerification.role f' f` |
| USM_R09 | MATHEMATICAL_PROPOSITION | `∀ {E : Type u_1} {F : Type u_2} [inst : DecidableEq E] [inst_1 : DecidableEq F],   Function.Injective fun p => USMVerification.bind p.1 p.2` |
| USM_R10 | MATHEMATICAL_PROPOSITION | `∃ x y truth, ¬truth (x ⊗ₜ[ℝ] y)` |
| USM_OUTER_POSITIVE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), (USMVerification.outer x).PosSemidef` |
| USM_OUTER_TRACE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), (USMVerification.outer x).trace = ↑(‖x‖ ^ 2)` |
| USM_P0_04 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (a : ℝ) (ha : 0 ≤ a) (ha1 : a ≤ 1) (r s : USMVerification.DensityState n),   (USMVerification.mix a ha ha1 r s).matrix.PosSemidef ∧     (USMVerification.mix a ha ha1 r s).matrix.IsHermitian ∧ (USMVerification.mix a ha ha1 r s).matrix.trace = 1` |
| USM_P0_06 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n) (z : ℂ),   ‖z‖ = 1 → USMVerification.outer (z • x) = USMVerification.outer x` |
| USM_PHASE_EXP | MATHEMATICAL_PROPOSITION | `∀ (θ : ℝ), ‖Complex.exp (↑θ * Complex.I)‖ = 1` |
| USM_PHASE_EXP_DENSITY | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n) (θ : ℝ),   USMVerification.outer (Complex.exp (↑θ * Complex.I) • x) = USMVerification.outer x` |
| USM_DENSITY_CONJUGATION_POSITIVE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (r : USMVerification.DensityState n) (A : Matrix (Fin n) (Fin n) ℂ),   (A * r.matrix * A.conjTranspose).PosSemidef` |
| USM_DENSITY_PROJECTION_POSITIVE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (r : USMVerification.DensityState n) (P : Matrix (Fin n) (Fin n) ℂ),   P.IsHermitian → (P * r.matrix * P).PosSemidef` |
| USM_FULLDENSITYEXTRACTION | MATHEMATICAL_PROPOSITION | `USMVerification.FullDensityExtractionTheory` |
| USM_DENSITYUPDATE_TRACE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (r : USMVerification.DensityState n) (U : Matrix (Fin n) (Fin n) ℂ) (hU : U.conjTranspose * U = 1),   (USMVerification.densityUpdate r U hU).matrix.trace = 1` |
| USM_AMBIGUOUSMINUS_NORMALIZED | MATHEMATICAL_PROPOSITION | `‖USMVerification.ambiguousMinus‖ = 1` |
| USM_ENSEMBLE_AMBIGUITY | MATHEMATICAL_PROPOSITION | `USMVerification.alternativeEnsemble.matrix = USMVerification.maximallyMixed.matrix` |
| USM_ENSEMBLE_COMPONENTS_DISTINCT | MATHEMATICAL_PROPOSITION | `USMVerification.outer USMVerification.ambiguous ≠ USMVerification.outer USMVerification.e0` |
| USM_I02 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (F : USMVerification.ComplexState n → USMVerification.ComplexState n)   (P : USMVerification.ComplexState n →ₗ[ℂ] USMVerification.ComplexState n) (x : USMVerification.ComplexState n),   P (F x) = 0 ∨ ∃ y, ‖y‖ = 1` |
| USM_P0_05 | MATHEMATICAL_PROPOSITION | `∀ {n m : ℕ} (x : USMVerification.ComplexState n) (y : USMVerification.ComplexState m), ‖x ⊗ₜ[ℂ] y‖ = ‖x‖ * ‖y‖` |
| USM_PHASE_REFL | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), USMVerification.StateEquivalence x x` |
| USM_PHASE_SYMM | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} {x y : USMVerification.ComplexState n},   USMVerification.StateEquivalence x y → USMVerification.StateEquivalence y x` |
| USM_PHASE_TRANS | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} {x y w : USMVerification.ComplexState n},   USMVerification.StateEquivalence x y → USMVerification.StateEquivalence y w → USMVerification.StateEquivalence x w` |
| USM_E1_NORMALIZED | MATHEMATICAL_PROPOSITION | `‖USMVerification.e1‖ = 1` |
| USM_MIXED_ENTRIES | MATHEMATICAL_PROPOSITION | `USMVerification.maximallyMixed.matrix = !![1 / 2, 0; 0, 1 / 2]` |
| USM_C07_NOT_PURE | MATHEMATICAL_PROPOSITION | `¬∃ x, USMVerification.outer x = USMVerification.maximallyMixed.matrix` |
| USM_C11_MATRIX | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (A : Matrix (Fin n) (Fin n) ℂ) (x : Fin n → ℂ),   USMVerification.realify (A.mulVec x) = USMVerification.realMatVec A (USMVerification.realify x)` |
| USM_COORDINATEPROJECTION_IDEMPOTENT | MATHEMATICAL_PROPOSITION | `USMVerification.coordinateProjection ∘ₗ USMVerification.coordinateProjection = USMVerification.coordinateProjection` |
| USM_HYBRID_NONTRIVIAL | MATHEMATICAL_PROPOSITION | `USMVerification.hybridWitness.extraction USMVerification.e0 = USMVerification.e0 ∧   USMVerification.hybridWitness.extraction USMVerification.e1 = 0 ∧ USMVerification.e1 ≠ 0` |
| USM_E09_ZERO | MATHEMATICAL_PROPOSITION | `‖USMVerification.e1‖ = 1 ∧ USMVerification.coordinateProjection USMVerification.e1 = 0` |
| USM_E09_SELECT | MATHEMATICAL_PROPOSITION | `USMVerification.coordinateProjection USMVerification.ambiguous = (√2)⁻¹ • USMVerification.e0` |
| USM_I08_TRACE_LOSS | MATHEMATICAL_PROPOSITION | `(USMVerification.outer (USMVerification.coordinateProjection USMVerification.e1)).trace ≠ 1` |
| USM_U02 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n), ¬(USMVerification.Certain x ∧ USMVerification.Uncertain x)` |
| USM_PHASE_WEIGHT | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n) (z : ℂ),   ‖z‖ = 1 → ∀ (i : Fin n), USMVerification.weight (z • x) i = USMVerification.weight x i` |
| USM_U05 | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.ComplexState n) (z : ℂ),   ‖z‖ = 1 → (USMVerification.Certain (z • x) ↔ USMVerification.Certain x)` |
| USM_CERTAIN_WITNESS | MATHEMATICAL_PROPOSITION | `‖USMVerification.e0‖ = 1 ∧ USMVerification.Certain USMVerification.e0` |
| USM_UNCERTAIN_WITNESS | MATHEMATICAL_PROPOSITION | `‖USMVerification.ambiguous‖ = 1 ∧ USMVerification.Uncertain USMVerification.ambiguous` |
| USM_U06_FORWARD | MATHEMATICAL_PROPOSITION | `USMVerification.hadamard USMVerification.e0 = USMVerification.ambiguous` |
| USM_U06_BACKWARD | MATHEMATICAL_PROPOSITION | `USMVerification.hadamard USMVerification.ambiguous = USMVerification.e0` |
| USM_U07 | MATHEMATICAL_PROPOSITION | `∃ x, USMVerification.Certain x ∧ ∃ t f, t x ∧ ¬f x` |
| USM_U01 | MATHEMATICAL_PROPOSITION | `∃ r u, USMVerification.Certain ↑r ∧ USMVerification.Uncertain ↑u` |
| USM_RAY_OUTER | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} {x y : USMVerification.ComplexState n},   USMVerification.StateEquivalence x y → USMVerification.outer x = USMVerification.outer y` |
| USM_S03 | MATHEMATICAL_PROPOSITION | `USMVerification.bind 0 1 ≠ USMVerification.bind 1 0` |
| USM_TENSOR_DIMENSION | MATHEMATICAL_PROPOSITION | `∀ (n m : ℕ), Module.finrank ℝ (USMVerification.M3 ℝ n m) = n * m` |
| USM_R08 | MATHEMATICAL_PROPOSITION | `∀ {n m k : ℕ} (x : USMVerification.M2 n) (y : USMVerification.M2 m) (z : USMVerification.M2 k),   ‖x‖ = 1 → ‖y‖ = 1 → ‖z‖ = 1 → ‖x ⊗ₜ[ℂ] y ⊗ₜ[ℂ] z‖ = 1` |
| USM_I01 | MATHEMATICAL_PROPOSITION | `∃ h, h.extraction USMVerification.e0 = USMVerification.e0 ∧ h.extraction USMVerification.e1 = 0` |
| USM_U02_EXHAUSTIVE | MATHEMATICAL_PROPOSITION | `∀ {n : ℕ} (x : USMVerification.NormalizedState n), USMVerification.Certain ↑x ∨ USMVerification.Uncertain ↑x` |
| USM_C11_TENSOR_SPACE | MATHEMATICAL_PROPOSITION | `∀ (n m : ℕ), Nonempty (USMVerification.M3 ℂ n m ≃ₗ[ℂ] USMVerification.M2 (n * m))` |

