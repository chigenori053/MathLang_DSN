module
public import USMVerification.DensityDynamics
@[expose] public section
namespace USMVerification

theorem projX_selfadjoint_coordinates (x y : ℝ × ℝ) :
    x.1*(projX y).1+x.2*(projX y).2 =
      (projX x).1*y.1+(projX x).2*y.2 := by simp [projX]

theorem projD_selfadjoint_coordinates (x y : ℝ × ℝ) :
    x.1*(projD y).1+x.2*(projD y).2 =
      (projD x).1*y.1+(projD x).2*y.2 := by simp [projD]; ring

theorem projD_residual_orthogonal (x : ℝ × ℝ) :
    (projD x).1*(x.1-(projD x).1)+(projD x).2*(x.2-(projD x).2) = 0 := by
  simp [projD]; ring
end USMVerification
