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
public theorem target : (∀ (v_positive : Prop), (∀ (v_valid : Prop), ((¬ v_positive) → (¬ (v_positive ∧ v_valid))))) := by
  solve | rfl | simp_all | tauto | omega | aesop
#print axioms Generated.target
example : Independent.mv06 := Generated.target
end Generated
