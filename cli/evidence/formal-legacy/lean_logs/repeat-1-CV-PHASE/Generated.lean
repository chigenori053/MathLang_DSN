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
public theorem target : (∀ (v_z : ℂ), (∀ (v_x : ℂ), ((‖v_z‖ = 1) → (‖(v_z * v_x)‖ = ‖v_x‖)))) := by
  exact Foundation.phase_observation
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
end Generated
