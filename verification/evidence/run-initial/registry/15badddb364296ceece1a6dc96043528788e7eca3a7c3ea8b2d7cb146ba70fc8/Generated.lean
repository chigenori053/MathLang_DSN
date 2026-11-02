module
import Foundation
import Independent
set_option autoImplicit false
set_option warningAsError true
set_option maxRecDepth 1024
set_option maxHeartbeats 2000000
noncomputable section
namespace Generated
public theorem target : (∀ (v_n : ℕ), (∀ (v_U : (EuclideanSpace ℂ (Fin v_n) →L[ℂ] EuclideanSpace ℂ (Fin v_n))), (((v_U).adjoint.comp v_U = 1) → (∀ (v_x : EuclideanSpace ℂ (Fin v_n)), (‖(v_U v_x)‖ = ‖v_x‖))))) := by
  exact Foundation.unitary_norm
#print axioms Generated.target
example : Independent.mv01 := Generated.target
end Generated
