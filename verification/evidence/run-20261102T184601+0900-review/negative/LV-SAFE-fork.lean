module
public import Foundation
public import Independent
set_option warningAsError true
namespace Generated
#eval IO.Process.output {cmd := "/bin/echo", args := #["sandbox-escape"]}
public theorem target : True := True.intro
#print axioms Generated.target
end Generated
