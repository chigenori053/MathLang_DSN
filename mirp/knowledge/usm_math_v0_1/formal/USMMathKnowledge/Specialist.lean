module
public import USMMathKnowledge.University
@[expose] public section
noncomputable section
open scoped ComplexOrder
namespace USMMathKnowledge
theorem SP_UNITARY_NORM_FORM {n : ℕ} (U : USMVerification.LinearOperator n)
    (x y : USMVerification.ComplexState n) (r : ℝ)
    (hU : U.adjoint.comp U=1) (hy : y=U x) (hr : ‖x‖=r) : ‖y‖=r := by
  rw [hy,USMVerification.p0_03 U hU,hr]
theorem SP_DENSITY_TRACE_SUBTYPE {n : ℕ}
    (rho : {m : Matrix (Fin n) (Fin n) ℂ // m.PosSemidef ∧ m.trace=1}) :
    rho.val.trace=1 := rho.property.2
end USMMathKnowledge
