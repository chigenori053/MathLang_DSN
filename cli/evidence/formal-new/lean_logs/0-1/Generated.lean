module
public import Foundation
public import Independent
set_option autoImplicit false
set_option warningAsError true
set_option maxRecDepth 1024
set_option maxHeartbeats 2000000
noncomputable section
open scoped ComplexOrder
namespace Generated
public theorem target : (¬ (∀ (v_n : ℕ), (∀ (v_A : (EuclideanSpace ℂ (Fin v_n) →L[ℂ] EuclideanSpace ℂ (Fin v_n))), (∀ (v_x : EuclideanSpace ℂ (Fin v_n)), (‖(v_A v_x)‖ = ‖v_x‖))))) := by
  exact Foundation.arbitrary_norm_false
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
example : Independent.mv02 := Generated.target
end Generated
