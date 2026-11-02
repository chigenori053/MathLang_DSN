module
public import Foundation
public import Independent
set_option autoImplicit false
set_option warningAsError true
set_option linter.unusedTactic false
set_option maxRecDepth 1024
set_option maxHeartbeats 2000000
noncomputable section
namespace Generated
public theorem target : (∀ (v_x : Bool), (((id : Bool → Bool) ((id : Bool → Bool) v_x)) = v_x)) := by
  solve | rfl | simp_all | tauto | omega | aesop
#print axioms Generated.target
example : Independent.mv03 := Generated.target
end Generated
