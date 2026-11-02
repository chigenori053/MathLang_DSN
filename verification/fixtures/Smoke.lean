module
import Foundation
set_option warningAsError true
namespace Generated
public theorem target : ∀ (n : ℕ) (U : Foundation.Operator n), U.adjoint.comp U = 1 → ∀ x : Foundation.State n, ‖U x‖ = ‖x‖ := by
  exact Foundation.unitary_norm
#print axioms Generated.target
end Generated
