"""Small JSON CLI for the MathLang_DSN model."""

import argparse
import json

from .api import calculate, evaluate


def main() -> None:
    parser = argparse.ArgumentParser(description="MathLang_DSN mathematical inference model")
    commands = parser.add_subparsers(dest="command", required=True)
    calc = commands.add_parser("calculate")
    calc.add_argument("expression")
    assess = commands.add_parser("evaluate")
    assess.add_argument("before")
    assess.add_argument("after")
    args = parser.parse_args()
    result = calculate(args.expression) if args.command == "calculate" else evaluate(args.before, args.after)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
