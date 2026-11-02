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
public theorem target : (∀ (v_x : Bool), (((id : Bool → Bool) ((id : Bool → Bool) v_x)) = v_x)) := by
  simp_all
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
example : Independent.mv03 := Generated.target
end Generated
