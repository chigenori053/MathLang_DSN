import unittest

from mathlang_dsn import (calculate, circle_angle, classical_probability, data_summary,
                          differentiate, evaluate, evaluate_antiderivative,
                          evaluate_derivative, function_value, hypotenuse,
                          integrate, inverse_proportion, polygon_angle_sum,
                          prime_factors, relative_frequency, similarity_ratios,
                          simplify_sqrt, solve_quadratic, solve_system)


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


class JuniorHighIntegrationTest(unittest.TestCase):
    def test_number_and_equation_knowledge(self):
        self.assertEqual(prime_factors(360)["factors"], [2, 2, 2, 3, 3, 5])
        self.assertEqual((simplify_sqrt(72)["outside"], simplify_sqrt(72)["inside"]), (6, 2))
        self.assertEqual((simplify_sqrt(0)["outside"], simplify_sqrt(0)["inside"]), (0, 1))
        roots = solve_quadratic("x^2-2=0")
        self.assertEqual((roots["status"], roots["discriminant"]), ("SOLVED", 8))
        self.assertEqual([(root["numerator"], root["radical_coefficient"], root["radicand"], root["denominator"])
                          for root in roots["roots"]], [(0, 1, 2, 1), (0, -1, 2, 1)])
        self.assertEqual(solve_quadratic("2*x^2-3*x+1=0")["roots"], [
            {"numerator": 1, "radical_coefficient": 0, "radicand": 1, "denominator": 1},
            {"numerator": 1, "radical_coefficient": 0, "radicand": 1, "denominator": 2}])
        self.assertEqual(solve_quadratic("x^2+1=0")["status"], "NO_REAL_ROOTS")
        self.assertEqual(solve_quadratic("x^2+2*x+1=0")["roots"][0]["numerator"], -1)
        self.assertEqual(solve_quadratic("x^3=0")["status"], "UNSUPPORTED")
        self.assertEqual(solve_quadratic("10000*x^2-10000=0")["status"], "RESOURCE_LIMIT")
        solved = solve_system(1, 1, 5, 1, -1, 1)
        self.assertEqual((solved["x"], solved["y"]), ({"numerator": 3, "denominator": 1}, {"numerator": 2, "denominator": 1}))
        self.assertEqual(solve_system(1, 1, 1, 2, 2, 2)["status"], "INFINITE_SOLUTIONS")
        self.assertEqual(solve_system(0, 0, 1, 0, 0, 2)["status"], "CONTRADICTION")
        self.assertEqual(solve_system(2, 3, 1, 1, -1, 0)["x"], {"numerator": 1, "denominator": 5})

    def test_function_and_geometry_knowledge(self):
        value = function_value("(x+1)^2", 3, 2)
        self.assertEqual(value["value"], {"numerator": 25, "denominator": 4})
        self.assertEqual(inverse_proportion(6, 1, 2)["value"], {"numerator": 3, "denominator": 1})
        self.assertEqual(inverse_proportion(6, 1, 0)["status"], "UNSUPPORTED")
        self.assertEqual(polygon_angle_sum(5)["angle_sum"], 540)
        self.assertEqual((hypotenuse(1, 1)["length_outside"], hypotenuse(1, 1)["length_inside"]), (1, 2))
        similarity = similarity_ratios(2, 3)
        self.assertEqual((similarity["length_ratio"], similarity["area_ratio"], similarity["volume_ratio"]),
                         ({"numerator": 2, "denominator": 3}, {"numerator": 4, "denominator": 9}, {"numerator": 8, "denominator": 27}))
        self.assertEqual(circle_angle(90)["value"], {"numerator": 45, "denominator": 1})
        self.assertEqual(circle_angle(360)["status"], "UNSUPPORTED")

    def test_data_and_provenance(self):
        data = data_summary([6, 1, 4, 2, 5, 3])
        self.assertEqual((data["mean"], data["median"], data["lower_quartile"], data["upper_quartile"], data["range"]),
                         ({"numerator": 7, "denominator": 2}, {"numerator": 7, "denominator": 2},
                          {"numerator": 2, "denominator": 1}, {"numerator": 5, "denominator": 1}, 5))
        self.assertEqual(data_summary([1, 2, 3, 4, 5])["lower_quartile"], {"numerator": 3, "denominator": 2})
        self.assertEqual(classical_probability(2, 6)["value"], {"numerator": 1, "denominator": 3})
        self.assertEqual(relative_frequency(3, 12)["value"], {"numerator": 1, "denominator": 4})
        self.assertEqual(classical_probability(7, 6)["status"], "UNSUPPORTED")
        self.assertEqual(prime_factors(360)["ruo"]["outputs"], [2, 2, 2, 3, 3, 5])
        self.assertEqual(solve_quadratic("x^2-2=0")["ruo"]["inputs"], [1, 0, -2])
        for outcome in (prime_factors(360), solve_quadratic("x^2-2=0"), solve_system(1, 1, 5, 1, -1, 1),
                        function_value("x^2", 2), polygon_angle_sum(5), data):
            self.assertEqual(outcome["ruo"]["knowledge_id"], outcome["knowledge_id"])
            self.assertEqual(outcome["ruo"]["source_rus"], outcome["active_rus"])
            self.assertEqual(outcome["ruo"]["validation"], outcome["status"])
            self.assertIn("mext.go.jp", outcome["ruo"]["curriculum_source"])
            self.assertTrue(outcome["ruo"]["inputs"])
            self.assertTrue(outcome["ruo"]["outputs"])


if __name__ == "__main__":
    unittest.main()
