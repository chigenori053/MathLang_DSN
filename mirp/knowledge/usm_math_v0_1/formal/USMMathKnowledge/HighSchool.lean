module
public import USMVerification
public import Mathlib.Analysis.Calculus.Deriv.Pow
public import Mathlib.Analysis.SpecialFunctions.Trigonometric.Deriv
public import Mathlib.Analysis.SpecialFunctions.Log.Deriv
public import Mathlib.Analysis.SpecialFunctions.Integrals.Basic
@[expose] public section
noncomputable section
open scoped BigOperators
namespace USMMathKnowledge

theorem HS_EQ_SYMM (a b : ℝ) (h : a=b) : b=a := h.symm
theorem HS_EQ_TRANS (a b c : ℝ) (h : a=b) (k : b=c) : a=c := h.trans k
theorem HS_NE_ZERO_PRODUCT (a b : ℝ) : a*b≠0 ↔ a≠0 ∧ b≠0 := mul_ne_zero_iff
theorem HS_ZERO_PRODUCT (a b : ℝ) : a*b=0 ↔ a=0 ∨ b=0 := mul_eq_zero
theorem HS_ADD_COMM (a b : ℝ) : a+b=b+a := add_comm _ _
theorem HS_ADD_ASSOC (a b c : ℝ) : (a+b)+c=a+(b+c) := add_assoc _ _ _
theorem HS_MUL_COMM (a b : ℝ) : a*b=b*a := mul_comm _ _
theorem HS_DISTRIBUTE (a b c : ℝ) : a*(b+c)=a*b+a*c := mul_add _ _ _
theorem HS_DIFF_SQUARES (a b : ℝ) : (a-b)*(a+b)=a^2-b^2 := by ring
theorem HS_SQUARE_EXPAND (a b : ℝ) : (a+b)^2=a^2+2*a*b+b^2 := by ring
theorem HS_SQUARE_NONNEG (a : ℝ) : 0≤a^2 := sq_nonneg _
theorem HS_ABS_NONNEG (a : ℝ) : 0≤|a| := abs_nonneg _
theorem HS_ABS_TRIANGLE (a b : ℝ) : |a+b|≤|a|+|b| := abs_add_le _ _
theorem HS_ABS_MUL (a b : ℝ) : |a*b|=|a| * |b| := abs_mul _ _
theorem HS_DIV_SELF (a : ℝ) (h : a≠0) : a/a=1 := div_self h
theorem HS_LINEAR_SOLUTION (a b x : ℝ) (ha : a≠0) (h : a*x+b=0) :
    x = -b/a := by apply (eq_div_iff ha).mpr; linarith
theorem HS_QUADRATIC_FACTORS (x r s : ℝ) :
    (x-r)*(x-s)=0 ↔ x=r ∨ x=s := by simp [mul_eq_zero,sub_eq_zero]
theorem HS_SQRT_SQUARE (a : ℝ) (h : 0≤a) : (Real.sqrt a)^2=a := Real.sq_sqrt h
theorem HS_SQRT_NONNEG (a : ℝ) : 0≤Real.sqrt a := Real.sqrt_nonneg _
theorem HS_DISTANCE_PYTHAGOREAN (x y : ℝ) :
    (Real.sqrt (x^2+y^2))^2=x^2+y^2 := Real.sq_sqrt (by positivity)
theorem HS_AMGM_SQUARE_FORM (a b : ℝ) : 4*a*b≤(a+b)^2 := by
  nlinarith [sq_nonneg (a-b)]
theorem HS_ADD_ORDER (a b c : ℝ) (h : a≤b) : a+c≤b+c := by linarith
theorem HS_MUL_ORDER (a b c : ℝ) (h : a≤b) (hc : 0≤c) : a*c≤b*c :=
  mul_le_mul_of_nonneg_right h hc
theorem HS_POW_ADD (a : ℝ) (n m : ℕ) : a^(n+m)=a^n*a^m := pow_add _ _ _
theorem HS_GEOMETRIC_STEP (a : ℝ) (n : ℕ) : a^(n+1)=a^n*a := pow_succ _ _
theorem HS_EXP_ADD (a b : ℝ) : Real.exp (a+b)=Real.exp a*Real.exp b := Real.exp_add _ _
theorem HS_LOG_MUL (a b : ℝ) (ha : 0<a) (hb : 0<b) :
    Real.log (a*b)=Real.log a+Real.log b := Real.log_mul (ne_of_gt ha) (ne_of_gt hb)
theorem HS_SIN_COS_SQUARE (a : ℝ) : Real.sin a^2+Real.cos a^2=1 := Real.sin_sq_add_cos_sq _
theorem HS_SIN_ADD (a b : ℝ) :
    Real.sin (a+b)=Real.sin a*Real.cos b+Real.cos a*Real.sin b := Real.sin_add _ _
theorem HS_COS_ADD (a b : ℝ) :
    Real.cos (a+b)=Real.cos a*Real.cos b-Real.sin a*Real.sin b := Real.cos_add _ _
theorem HS_COMPLEX_COORDINATES (z : ℂ) : (z.re : ℂ)+(z.im : ℂ)*Complex.I=z :=
  Complex.re_add_im _
theorem HS_COMPLEX_NORM_SQUARE (z : ℂ) : ‖z‖^2=z.re^2+z.im^2 := by
  rw [Complex.sq_norm]; simp [Complex.normSq_apply,pow_two]
theorem HS_COMPLEX_CONJ_PRODUCT (z : ℂ) : z*star z=(‖z‖^2 : ℝ) := by
  rw [mul_comm,Complex.star_def,Complex.conj_mul']; norm_cast
theorem HS_DOT_COMM (x y : Fin 3 → ℝ) :
    (∑ i, x i*y i)=∑ i, y i*x i := Finset.sum_congr rfl (fun _ _ => mul_comm _ _)
theorem HS_DOT_NONNEG (x : Fin 3 → ℝ) : 0≤∑ i, x i*x i :=
  Finset.sum_nonneg (fun i _ => mul_self_nonneg (x i))
theorem HS_AFFINE_COMPOSE (a b c d x : ℝ) : a*(c*x+d)+b=(a*c)*x+(a*d+b) := by ring
theorem HS_FINITE_SUM_ADD (n : ℕ) (x y : Fin n → ℝ) :
    (∑ i, (x i+y i))=(∑ i, x i)+(∑ i, y i) := Finset.sum_add_distrib
theorem HS_ARITHMETIC_SUM (n : ℕ) :
    (∑ i ∈ Finset.range n, (i : ℝ))=(n:ℝ)*((n:ℝ)-1)/2 := by
  induction n with
  | zero => simp
  | succ n ih => rw [Finset.sum_range_succ,ih]; push_cast; ring
theorem HS_COUNT_PAIRS (A B : Type*) [Fintype A] [Fintype B] :
    Fintype.card (A×B)=Fintype.card A*Fintype.card B := Fintype.card_prod _ _
theorem HS_FINITE_UNION (A : Type*) [DecidableEq A] (s t : Finset A) :
    (s∪t).card+(s∩t).card=s.card+t.card := Finset.card_union_add_card_inter _ _
theorem HS_PROBABILITY_COMPLEMENT_BOUNDS (p : ℝ) (hp : 0≤p) (hp1 : p≤1) :
    0≤1-p ∧ 1-p≤1 := by constructor <;> linarith
theorem HS_FINITE_EXPECTATION_ADD (n : ℕ) (p x y : Fin n → ℝ) :
    (∑ i, p i*(x i+y i))=(∑ i,p i*x i)+(∑ i,p i*y i) := by
  simp only [mul_add,Finset.sum_add_distrib]
theorem HS_WEIGHTED_VARIANCE_NONNEG (n : ℕ) (p x : Fin n → ℝ) (μ : ℝ)
    (hp : ∀ i, 0≤p i) : 0≤∑ i,p i*(x i-μ)^2 :=
  Finset.sum_nonneg (fun i _ => mul_nonneg (hp i) (sq_nonneg _))
theorem HS_DERIV_SQUARE (x : ℝ) : HasDerivAt (fun t : ℝ => t^2) (2*x) x := by
  simpa using hasDerivAt_pow 2 x
theorem HS_DERIV_SIN (x : ℝ) : HasDerivAt Real.sin (Real.cos x) x := Real.hasDerivAt_sin x
theorem HS_CONTINUOUS_QUADRATIC (a b c : ℝ) :
    Continuous (fun x : ℝ => a*x^2+b*x+c) := by fun_prop
theorem HS_INTEGRAL_POWER (a b : ℝ) (n : ℕ) :
    (∫ x in a..b, x^n)=(b^(n+1)-a^(n+1))/(n+1) := integral_pow n
theorem HS_NAT_DIVISION (a b : ℕ) : a%b+b*(a/b)=a := Nat.mod_add_div _ _
theorem HS_REMAINDER_BOUND (a b : ℕ) (hb : 0<b) : a%b<b := Nat.mod_lt _ hb
theorem HS_GCD_COMM (a b : ℕ) : Nat.gcd a b=Nat.gcd b a := Nat.gcd_comm _ _
theorem HS_GCD_DIVIDES (a b : ℕ) : Nat.gcd a b ∣ a := Nat.gcd_dvd_left _ _
theorem HS_FACTORIAL_STEP (n : ℕ) : (n+1).factorial=(n+1)*n.factorial := Nat.factorial_succ _
theorem HS_PASCAL (n k : ℕ) : (n+1).choose (k+1)=n.choose k+n.choose (k+1) := Nat.choose_succ_succ _ _
theorem HS_EXP_POSITIVE (x : ℝ) : 0<Real.exp x := Real.exp_pos _
theorem HS_LOG_EXP (x : ℝ) : Real.log (Real.exp x)=x := Real.log_exp _
theorem HS_SIN_BOUND (x : ℝ) : |Real.sin x|≤1 := Real.abs_sin_le_one _
theorem HS_QUADRATIC_MINIMUM (x a : ℝ) : a≤(x-a)^2+a := by nlinarith [sq_nonneg (x-a)]
theorem HS_LE_TRANS (a b c : ℝ) (h : a≤b) (k : b≤c) : a≤c := h.trans k
theorem HS_MODUS_PONENS (P Q : Prop) (h : P → Q) (hp : P) : Q := h hp
theorem HS_VECTOR_ADD_ZERO (n : ℕ) (x : Fin n → ℝ) : x+0=x := add_zero _
theorem HS_MUL_ZERO (x : ℝ) : x*0=0 := mul_zero _
end USMMathKnowledge
