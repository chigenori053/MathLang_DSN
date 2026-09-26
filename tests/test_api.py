import unittest

from mathlang_dsn import calculate, differentiate, evaluate, evaluate_antiderivative, evaluate_derivative, integrate


class ModelIntegrationTest(unittest.TestCase):
    def test_calculate_and_solve(self):
        self.assertEqual(calculate("2*(3+4)")["constant"], 14)
        result = calculate("2*(x+3)=12")
        self.assertEqual((result["status"], result["numerator"], result["denominator"]), ("SOLVED", 3, 1))
        self.assertTrue(all(step["validation"] == "VALID" for step in result["steps"]))

    def test_assessment_and_unsupported_input(self):
        self.assertEqual(evaluate("2*(x+3)=12", "x=3")["status"], "VALID")
        self.assertEqual(evaluate("2*(x+3)=12", "x=4")["status"], "INVALID")
        self.assertEqual(calculate("x*x=4")["status"], "UNSOLVED")
        self.assertEqual(calculate("__import__('os').system('echo bad')")["status"], "UNSUPPORTED")

    def test_deterministic_multistep_and_terminal_classes(self):
        first = calculate("2*x+6=3*x+10")
        second = calculate("2*x+6=3*x+10")
        self.assertEqual(first, second)
        self.assertEqual((first["numerator"], first["denominator"]), (-4, 1))
        self.assertTrue(all(step["validation"] == "VALID" for step in first["steps"]))
        self.assertEqual(calculate("x+1=x+2")["status"], "CONTRADICTION")
        self.assertEqual(calculate("x+1=x+1")["status"], "INFINITE_SOLUTIONS")

    def test_exact_rational_polynomial_and_calculus(self):
        self.assertEqual((calculate("1/2+1/3")["numerator"], calculate("1/2+1/3")["denominator"]), (5, 6))
        self.assertEqual(calculate("(x+1)^2")["coefficients"], [1, 2, 1, 0, 0, 0])
        self.assertEqual(calculate("x/2+1/3=5/6")["numerator"], 1)
        self.assertEqual(evaluate("(x+1)^2", "x^2+2*x+1")["status"], "VALID")
        self.assertEqual(evaluate("x^2-1=0", "2*x^2-2=0")["status"], "VALID")
        self.assertEqual(evaluate("x^2=1", "x=1")["status"], "UNVERIFIED")
        self.assertEqual(differentiate("x^3+2*x")["coefficients"], [2, 0, 3, 0, 0, 0])
        self.assertEqual(integrate("x^2+1")["coefficients"], [0, 3, 0, 1, 0, 0])
        self.assertEqual(evaluate_derivative("x^3+2*x", "3*x^2+2")["status"], "VALID")
        self.assertEqual(evaluate_derivative("x^3+2*x", "3*x^2+3")["status"], "INVALID")
        self.assertEqual(evaluate_antiderivative("x^2+1", "x^3/3+x+7")["status"], "VALID")
        self.assertEqual(evaluate_antiderivative("x^2+1", "x^3/3+2*x")["status"], "INVALID")

    def test_domain_boundaries(self):
        self.assertEqual(calculate("1/0")["status"], "UNSUPPORTED")
        self.assertEqual(calculate("x^6")["status"], "UNSUPPORTED")
        self.assertEqual(calculate("x^5*x")["status"], "RESOURCE_LIMIT")
        self.assertEqual(integrate("x^5")["status"], "RESOURCE_LIMIT")
        self.assertEqual(evaluate_derivative("sin(x)", "cos(x)")["status"], "UNVERIFIED")

    def test_knowledge_rus_and_ruo_trace(self):
        solved = calculate("2*x+6=3*x+10")
        self.assertEqual([step["ruo"]["knowledge_id"] for step in solved["steps"]],
                         ["EQ_MOVE_RIGHT_VARIABLE", "EQ_MOVE_LEFT_CONSTANT"])
        self.assertEqual([step["candidate_ru"] for step in solved["steps"]], [[1, 2], [2]])
        self.assertEqual(solved["ruos"], [step["ruo"] for step in solved["steps"]])
        for step in solved["steps"]:
            self.assertEqual(step["active_rus"], "EquationRUS")
            self.assertIn(step["ruo"]["source_ru"], step["candidate_ru"])
            self.assertEqual(step["ruo"]["source_rus"], 1)
            self.assertEqual(step["ruo"]["source"], "mathlang_dsn/knowledge.rsn")
            self.assertEqual((step["ruo"]["state_before"], step["ruo"]["state_after"]),
                             (step["before"], step["after"]))
            self.assertEqual(step["ruo"]["validation"], "VALID")

        polynomial = calculate("(x+1)^2")
        self.assertEqual([ruo["knowledge_id"] for ruo in polynomial["ruos"]],
                         ["POLY_ADD", "POLY_POWER_BOUNDED"])
        self.assertEqual([ruo["id"] for ruo in polynomial["ruos"]], [1, 2])
        branched = calculate("(x+1)*(x+2)")
        self.assertEqual([ruo["id"] for ruo in branched["ruos"]], [1, 2, 3])
        self.assertEqual(polynomial["ruos"][-1]["after"], polynomial["coefficients"])
        self.assertEqual(polynomial["ruos"][-1]["after_denominator"], polynomial["denominator"])
        self.assertIn("POLY_POWER_BOUNDED", polynomial["knowledge_ids"])
        self.assertEqual(differentiate("x^2")["ruos"][-1]["knowledge_id"], "POLY_DIFFERENTIATE")
        self.assertEqual(integrate("x")["ruos"][-1]["knowledge_id"], "POLY_INTEGRATE")
        assessment = evaluate("x+1", "1+x")
        self.assertEqual((assessment["status"], assessment["knowledge_id"], assessment["active_rus"]),
                         ("VALID", "POLY_EXPRESSION_EQUIVALENCE", "AssessmentRUS"))
        self.assertEqual(assessment["ruo"], {
            "id": 1, "source_ru": 19, "source_rus": "AssessmentRUS", "knowledge_id": assessment["knowledge_id"],
            "source": assessment["source"], "validation": assessment["status"]})


if __name__ == "__main__":
    unittest.main()
