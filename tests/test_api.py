import unittest

from mathlang_dsn import calculate, evaluate


class ModelIntegrationTest(unittest.TestCase):
    def test_calculate_and_solve(self):
        self.assertEqual(calculate("2*(3+4)")["constant"], 14)
        result = calculate("2*(x+3)=12")
        self.assertEqual((result["status"], result["numerator"], result["denominator"]), ("SOLVED", 3, 1))
        self.assertTrue(all(step["validation"] == "VALID" for step in result["steps"]))

    def test_assessment_and_unsupported_input(self):
        self.assertEqual(evaluate("2*(x+3)=12", "x=3")["status"], "VALID")
        self.assertEqual(evaluate("2*(x+3)=12", "x=4")["status"], "INVALID")
        self.assertEqual(calculate("x*x=4")["status"], "UNSUPPORTED")
        self.assertEqual(calculate("__import__('os').system('echo bad')")["status"], "UNSUPPORTED")

    def test_deterministic_multistep_and_terminal_classes(self):
        first = calculate("2*x+6=3*x+10")
        second = calculate("2*x+6=3*x+10")
        self.assertEqual(first, second)
        self.assertEqual([step["rule"] for step in first["steps"]], ["MOVE_RIGHT_VARIABLE", "MOVE_LEFT_CONSTANT"])
        self.assertEqual(calculate("x+1=x+2")["status"], "CONTRADICTION")
        self.assertEqual(calculate("x+1=x+1")["status"], "INFINITE_SOLUTIONS")


if __name__ == "__main__":
    unittest.main()
