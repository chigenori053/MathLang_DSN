module
public import Foundation
public import Independent
set_option warningAsError true
namespace Generated
public theorem target : True := True.intro
example : Independent.mv01 := Generated.target
#print axioms Generated.target
end Generated
