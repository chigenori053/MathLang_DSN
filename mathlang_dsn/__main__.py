"""Small JSON CLI for the MathLang_DSN model."""

import argparse
import json

from .api import calculate, differentiate, evaluate, evaluate_antiderivative, evaluate_derivative, integrate


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
    else:
        result = evaluate(args.before, args.after)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
