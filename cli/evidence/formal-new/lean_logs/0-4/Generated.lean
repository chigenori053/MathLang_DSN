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
public theorem target : (∀ (v_S : (ℕ → ℕ)), (∀ (v_i : ℕ), (∀ (v_j : ℕ), (∀ (v_delta : ℕ), ((¬ (v_j = v_i)) → (((Function.update v_S v_i v_delta) v_j) = (v_S v_j))))))) := by
  simp_all
set_option pp.proofs true in
#print Generated.target
#print axioms Generated.target
example : Independent.mv05 := Generated.target
end Generated
