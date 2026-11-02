module
import Foundation
import Independent
set_option autoImplicit false
set_option warningAsError true
set_option maxRecDepth 1024
set_option maxHeartbeats 2000000
noncomputable section
namespace Generated
public theorem target : (∀ (v_S : (ℕ → ℕ)), (¬ (((Function.update (Function.update v_S (0 : ℕ) (1 : ℕ)) (0 : ℕ) (2 : ℕ)) (0 : ℕ)) = ((Function.update (Function.update v_S (0 : ℕ) (2 : ℕ)) (0 : ℕ) (1 : ℕ)) (0 : ℕ))))) := by
  first | rfl | simp_all | tauto | omega | aesop
#print axioms Generated.target
end Generated
