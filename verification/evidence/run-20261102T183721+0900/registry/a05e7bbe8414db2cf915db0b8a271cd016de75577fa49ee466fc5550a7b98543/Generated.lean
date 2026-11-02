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
public theorem target : (¬ (∀ (v_x : ℂ), (∀ (v_y : ℂ), ((‖v_x‖ = ‖v_y‖) → (v_x = v_y))))) := by
  exact Foundation.phase_collision
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
end Generated
