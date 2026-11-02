module
import Foundation
import Independent
set_option autoImplicit false
set_option warningAsError true
set_option maxRecDepth 1024
set_option maxHeartbeats 2000000
noncomputable section
namespace Generated
public theorem target : (∀ (v_positive : Prop), (∀ (v_valid : Prop), ((¬ v_positive) → (¬ (v_positive ∧ v_valid))))) := by
  first | rfl | simp_all | tauto | omega | aesop
#print axioms Generated.target
example : Independent.mv06 := Generated.target
end Generated
