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
public theorem target : (∀ (v_A : Type), (∀ (v_B : Type), (∀ (v_E : (v_A → v_B)), (∀ (v_D : (v_B → v_A)), (∀ (v_x : v_A), (∀ (v_y : v_A), ((¬ (v_x = v_y)) → (((v_E v_x) = (v_E v_y)) → (¬ (((v_D (v_E v_x)) = v_x) ∧ ((v_D (v_E v_y)) = v_y))))))))))) := by
  intro v_A v_B v_E v_D v_x v_y hne hc
  exact Foundation.collision v_E v_D v_x v_y hne hc
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
example : Independent.mv04 := Generated.target
end Generated
