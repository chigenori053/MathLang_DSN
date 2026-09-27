"""Small JSON CLI for the MathLang_DSN model."""

import argparse
import json

from .api import (calculate, circle_angle, classical_probability, data_summary,
                  differentiate, evaluate, evaluate_antiderivative,
                  evaluate_derivative, function_value, hypotenuse, integrate,
                  inverse_proportion, polygon_angle_sum, prime_factors,
                  relative_frequency, similarity_ratios, simplify_sqrt,
                  solve_quadratic, solve_system)


def main() -> None:
    parser = argparse.ArgumentParser(description="MathLang_DSN mathematical inference model")
    commands = parser.add_subparsers(dest="command", required=True)
    calc = commands.add_parser("calculate")
    calc.add_argument("expression")
    diff = commands.add_parser("differentiate")
    diff.add_argument("expression")
    integ = commands.add_parser("integrate")
    integ.add_argument("expression")
    assess = commands.add_parser("evaluate")
    assess.add_argument("before")
    assess.add_argument("after")
    derivative = commands.add_parser("evaluate-derivative")
    derivative.add_argument("expression")
    derivative.add_argument("candidate")
    antiderivative = commands.add_parser("evaluate-antiderivative")
    antiderivative.add_argument("expression")
    antiderivative.add_argument("candidate")
    for name in ("prime-factors", "simplify-sqrt", "polygon-angle-sum", "circle-angle"):
        command = commands.add_parser(name)
        command.add_argument("value", type=int)
    quadratic = commands.add_parser("solve-quadratic")
    quadratic.add_argument("equation")
    system = commands.add_parser("solve-system")
    for name in ("a", "b", "c", "d", "e", "f"):
        system.add_argument(name, type=int)
    function = commands.add_parser("function-value")
    function.add_argument("expression")
    function.add_argument("x_numerator", type=int)
    function.add_argument("x_denominator", type=int, nargs="?", default=1)
    for name in ("hypotenuse", "similarity-ratios", "classical-probability", "relative-frequency"):
        command = commands.add_parser(name)
        command.add_argument("first", type=int)
        command.add_argument("second", type=int)
    inverse = commands.add_parser("inverse-proportion")
    for name in ("a_numerator", "a_denominator", "x_numerator"):
        inverse.add_argument(name, type=int)
    inverse.add_argument("x_denominator", type=int, nargs="?", default=1)
    data = commands.add_parser("data-summary")
    data.add_argument("values", type=int, nargs="+")
    args = parser.parse_args()
    if args.command == "calculate":
        result = calculate(args.expression)
    elif args.command == "differentiate":
        result = differentiate(args.expression)
    elif args.command == "integrate":
        result = integrate(args.expression)
    elif args.command == "evaluate-derivative":
        result = evaluate_derivative(args.expression, args.candidate)
    elif args.command == "evaluate-antiderivative":
        result = evaluate_antiderivative(args.expression, args.candidate)
    elif args.command == "solve-quadratic":
        result = solve_quadratic(args.equation)
    elif args.command == "solve-system":
        result = solve_system(args.a, args.b, args.c, args.d, args.e, args.f)
    elif args.command == "function-value":
        result = function_value(args.expression, args.x_numerator, args.x_denominator)
    elif args.command == "inverse-proportion":
        result = inverse_proportion(args.a_numerator, args.a_denominator, args.x_numerator, args.x_denominator)
    elif args.command == "data-summary":
        result = data_summary(args.values)
    elif args.command in {"prime-factors", "simplify-sqrt", "polygon-angle-sum", "circle-angle"}:
        result = {"prime-factors": prime_factors, "simplify-sqrt": simplify_sqrt,
                  "polygon-angle-sum": polygon_angle_sum, "circle-angle": circle_angle}[args.command](args.value)
    elif args.command in {"hypotenuse", "similarity-ratios", "classical-probability", "relative-frequency"}:
        result = {"hypotenuse": hypotenuse, "similarity-ratios": similarity_ratios,
                  "classical-probability": classical_probability,
                  "relative-frequency": relative_frequency}[args.command](args.first, args.second)
    else:
        result = evaluate(args.before, args.after)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
