"""Convert bounded MathLang expressions into calls to the ReasonScript model."""

from __future__ import annotations

import ast
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

MODEL = Path(__file__).with_name("model.rsn")
POLYNOMIAL = Path(__file__).with_name("polynomial.rsn")
KNOWLEDGE = Path(__file__).with_name("knowledge.rsn")
JUNIOR_HIGH = Path(__file__).with_name("junior_high.rsn")
MANIFEST = Path(__file__).with_name("reason.toml")
MAX_NODES = 64
MAX_INTEGER = 1_000_000
OPERATORS = {ast.Add: 2, ast.Sub: 3, ast.Mult: 4, ast.Div: 5}


class UnsupportedExpression(ValueError):
    """Input lies outside the model's documented mathematical fragment."""


def _split_equation(source: str) -> tuple[str, str] | None:
    if "=" not in source:
        return None
    if source.count("=") != 1:
        raise UnsupportedExpression("expected one equality sign")
    left, right = source.split("=", 1)
    if not left.strip() or not right.strip():
        raise UnsupportedExpression("both equation sides are required")
    return left, right


def _encode(source: str) -> tuple[list[int], list[int], list[int], list[int]]:
    try:
        tree = ast.parse(source.strip().replace("^", "**"), mode="eval").body
    except SyntaxError as exc:
        raise UnsupportedExpression("invalid expression syntax") from exc
    kinds: list[int] = []
    values: list[int] = []
    lefts: list[int] = []
    rights: list[int] = []

    def append(kind: int, value: int = 0, left: int = -1, right: int = -1) -> int:
        if len(kinds) >= MAX_NODES:
            raise UnsupportedExpression("expression exceeds 64 nodes")
        kinds.append(kind)
        values.append(value)
        lefts.append(left)
        rights.append(right)
        return len(kinds) - 1

    def visit(node: ast.AST) -> int:
        if isinstance(node, ast.Constant) and type(node.value) is int:
            if abs(node.value) > MAX_INTEGER:
                raise UnsupportedExpression("integer exceeds model limit")
            return append(0, node.value)
        if isinstance(node, ast.Name) and node.id == "x":
            return append(1)
        if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.UAdd, ast.USub)):
            operand = visit(node.operand)
            if isinstance(node.op, ast.UAdd):
                return operand
            zero = append(0)
            return append(3, left=zero, right=operand)
        if isinstance(node, ast.BinOp) and type(node.op) in OPERATORS:
            left = visit(node.left)
            right = visit(node.right)
            return append(OPERATORS[type(node.op)], left=left, right=right)
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Pow):
            if not isinstance(node.right, ast.Constant) or type(node.right.value) is not int or not 0 <= node.right.value <= 5:
                raise UnsupportedExpression("integer exponents must be between 0 and 5")
            base = visit(node.left)
            return append(6, value=node.right.value, left=base)
        raise UnsupportedExpression("supported syntax: integers, x, +, -, *, /, powers 0..5, parentheses")

    visit(tree)
    return kinds, values, lefts, rights


def _literal(values: list[int]) -> str:
    return "[" + ", ".join(map(str, values)) + "]"


def _term(name: str, source: str) -> str:
    arrays = _encode(source)
    return f"    let {name} = Polynomial::PolyEvaluate(" + ", ".join(_literal(a) for a in arrays) + ")"


def _run(lines: list[str], result: str) -> object:
    if shutil.which("reason") is None:
        raise RuntimeError("ReasonScript 0.5.6.2 `reason` command is required")
    with tempfile.TemporaryDirectory(prefix="mathlang-dsn-") as directory:
        workspace = Path(directory)
        (workspace / "src").mkdir()
        (workspace / "reason.toml").write_text(MANIFEST.read_text())
        (workspace / "src" / "model.rsn").write_text(MODEL.read_text())
        (workspace / "src" / "polynomial.rsn").write_text(POLYNOMIAL.read_text())
        (workspace / "src" / "knowledge.rsn").write_text(KNOWLEDGE.read_text())
        (workspace / "src" / "junior_high.rsn").write_text(JUNIOR_HIGH.read_text())
        source = "\n".join(
            ["package mathlang_dsn", "module main {", "  import mathlang_dsn.Model", "  import mathlang_dsn.Polynomial", "  import mathlang_dsn.JuniorHigh", "  calculation Request {"]
            + lines
            + [f"    result = {result}", "  }", "}", ""]
        )
        (workspace / "src" / "main.rsn").write_text(source)
        for command in (["reason", "build"], ["reason", "run", "--entry", "Request", "--json", "--trace=off"]):
            completed = subprocess.run(command, cwd=workspace, capture_output=True, text=True, timeout=30, check=False)
            if completed.returncode:
                raise RuntimeError(completed.stderr.strip() or completed.stdout.strip())
        payload = json.loads(completed.stdout)
        if payload.get("status") != "success" or payload.get("runtime_result", {}).get("status") != "success":
            raise RuntimeError("ReasonScript execution failed")
        return _unwrap(payload["runtime_result"]["result"])


def _unwrap(value: object) -> object:
    if isinstance(value, dict) and "type_name" in value and "fields" in value:
        return {key: _unwrap(item) for key, item in value["fields"].items()}
    if isinstance(value, list):
        return [_unwrap(item) for item in value]
    return value


def calculate(expression: str) -> dict:
    """Calculate a bounded rational polynomial or solve a linear equation."""
    try:
        equation = _split_equation(expression)
        if equation is None:
            term = _run([_term("term", expression)], "term")
            if not term["valid"]:
                return {"schema_version": "mathlang-dsn/0.3", "status": "RESOURCE_LIMIT" if term["reason"] in {"RESOURCE_LIMIT", "DEGREE_LIMIT"} else "UNSUPPORTED", "reason": term["reason"]}
            return _polynomial_result(term)
        left, right = equation
        outcome = _run([_term("left", left), _term("right", right)], "Polynomial::PolySolveEquation(left, right)")
        return {"schema_version": "mathlang-dsn/0.3", **outcome}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": str(exc)}


def _polynomial_result(term: dict, operation: str | None = None) -> dict:
    if not term["valid"]:
        return {"schema_version": "mathlang-dsn/0.3", "status": "RESOURCE_LIMIT" if term["reason"] in {"RESOURCE_LIMIT", "DEGREE_LIMIT"} else "UNSUPPORTED", "reason": term["reason"]}
    coefficients = term["coefficients"]
    degree = max((index for index, value in enumerate(coefficients) if value), default=0)
    status = operation or ("CALCULATED" if degree == 0 else "SYMBOLIC")
    output = {"schema_version": "mathlang-dsn/0.3", "status": status, "coefficients": coefficients, "denominator": term["denominator"], "degree": degree, "knowledge_ids": term["knowledge_ids"], "knowledge_sources": term["knowledge_sources"], "ruos": term["ruos"]}
    if degree == 0:
        output["numerator"] = coefficients[0]
    if degree <= 1 and term["denominator"] == 1:
        output.update(coefficient=coefficients[1], constant=coefficients[0])
    if operation == "INTEGRATED":
        output["integration_constant"] = "C"
    return output


def differentiate(expression: str) -> dict:
    """Differentiate a bounded polynomial exactly with respect to x."""
    try:
        if _split_equation(expression) is not None:
            raise UnsupportedExpression("differentiation requires an expression")
        term = _run([_term("term", expression)], "Polynomial::PolyDifferentiate(term)")
        return _polynomial_result(term, "DIFFERENTIATED")
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": str(exc)}


def integrate(expression: str) -> dict:
    """Find an exact indefinite integral of a bounded polynomial."""
    try:
        if _split_equation(expression) is not None:
            raise UnsupportedExpression("integration requires an expression")
        term = _run([_term("term", expression)], "Polynomial::PolyIntegrate(term)")
        return _polynomial_result(term, "INTEGRATED")
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": str(exc)}


def evaluate_derivative(expression: str, candidate: str) -> dict:
    """Check a proposed polynomial derivative by exact coefficients."""
    return _evaluate_calculus(expression, candidate, derivative_of_candidate=False)


def evaluate_antiderivative(expression: str, candidate: str) -> dict:
    """Check an antiderivative by differentiating the candidate."""
    return _evaluate_calculus(expression, candidate, derivative_of_candidate=True)


def _evaluate_calculus(expression: str, candidate: str, *, derivative_of_candidate: bool) -> dict:
    try:
        if _split_equation(expression) is not None or _split_equation(candidate) is not None:
            raise UnsupportedExpression("calculus assessment requires expressions")
        lines = [_term("source", expression), _term("candidate", candidate)]
        operand, target = ("candidate", "source") if derivative_of_candidate else ("source", "candidate")
        result = _run(lines + [f"    let transformed = Polynomial::PolyDifferentiate({operand})"], f"Polynomial::PolyCheckExpressions(transformed, {target})")
        return {"schema_version": "mathlang-dsn/0.3", **result}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNVERIFIED", "reason": str(exc)}


def evaluate(before: str, after: str) -> dict:
    """Assess whether two expressions or equations have the same meaning."""
    try:
        first = _split_equation(before)
        second = _split_equation(after)
        if (first is None) != (second is None):
            return {"schema_version": "mathlang-dsn/0.3", "status": "UNVERIFIED", "reason": "different input kinds"}
        if first is None:
            result = _run([_term("before", before), _term("after", after)], "Polynomial::PolyCheckExpressions(before, after)")
        else:
            assert second is not None
            result = _run(
                [_term("left", first[0]), _term("right", first[1]), _term("next_left", second[0]), _term("next_right", second[1])],
                "Polynomial::PolyCheckEquations(left, right, next_left, next_right)",
            )
        return {"schema_version": "mathlang-dsn/0.3", **result}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNVERIFIED", "reason": str(exc)}


def _junior_call(function: str, *values: int) -> dict:
    if any(type(value) is not int for value in values):
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": "integer inputs required"}
    if any(abs(value) > 1_000_000_000 for value in values):
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": "integer input exceeds limit"}
    outcome = _run([], f"JuniorHigh::{function}({', '.join(map(str, values))})")
    return {"schema_version": "mathlang-dsn/0.3", **outcome}


def prime_factors(value: int) -> dict:
    """Factor a positive integer into primes, in ascending order."""
    return _junior_call("JPrimeFactors", value)


def simplify_sqrt(value: int) -> dict:
    """Write sqrt(n) as outside*sqrt(inside), exactly."""
    return _junior_call("JSimplifySqrt", value)


def solve_quadratic(equation: str) -> dict:
    """Solve a bounded quadratic equation over the reals with exact surds."""
    try:
        sides = _split_equation(equation)
        if sides is None:
            raise UnsupportedExpression("quadratic equation requires equality")
        lines = [_term("left", sides[0]), _term("right", sides[1]), "    let difference = Polynomial::PolyDifference(left, right)"]
        outcome = _run(lines, "JuniorHigh::JQuadraticFromPolynomial(difference.coefficients)")
        return {"schema_version": "mathlang-dsn/0.3", **outcome}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": str(exc)}


def solve_system(a: int, b: int, c: int, d: int, e: int, f: int) -> dict:
    """Solve a*x+b*y=c and d*x+e*y=f exactly."""
    return _junior_call("JSolveSystem", a, b, c, d, e, f)


def function_value(expression: str, x_numerator: int, x_denominator: int = 1) -> dict:
    """Evaluate a polynomial at a bounded rational x."""
    if type(x_numerator) is not int or type(x_denominator) is not int or abs(x_numerator) > 1_000_000_000 or abs(x_denominator) > 1_000_000_000:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": "integer inputs required"}
    try:
        if _split_equation(expression) is not None:
            raise UnsupportedExpression("function evaluation requires an expression")
        lines = [_term("term", expression)]
        outcome = _run(lines, f"JuniorHigh::JFunctionFromPolynomial(term.valid, term.coefficients, term.denominator, {x_numerator}, {x_denominator})")
        return {"schema_version": "mathlang-dsn/0.3", **outcome}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": str(exc)}


def polygon_angle_sum(sides: int) -> dict:
    """Find the interior angle sum of a polygon in degrees."""
    return _junior_call("JPolygonAngleSum", sides)


def hypotenuse(first: int, second: int) -> dict:
    """Find the exact hypotenuse from positive integer legs."""
    return _junior_call("JHypotenuse", first, second)


def data_summary(values: list[int]) -> dict:
    """Find exact mean, median, quartiles, and range for integer data."""
    if not isinstance(values, list) or len(values) > 64 or any(type(value) is not int or abs(value) > 1_000_000_000 for value in values):
        return {"schema_version": "mathlang-dsn/0.3", "status": "UNSUPPORTED", "reason": "integer list required"}
    outcome = _run([], f"JuniorHigh::JDataSummary({_literal(values)})")
    return {"schema_version": "mathlang-dsn/0.3", **outcome}


def classical_probability(favorable: int, total: int) -> dict:
    """Compute favorable/total for equally likely outcomes."""
    return _junior_call("JClassicalProbability", favorable, total)


def inverse_proportion(a_numerator: int, a_denominator: int, x_numerator: int, x_denominator: int = 1) -> dict:
    """Evaluate y=a/x for rational a and nonzero rational x."""
    return _junior_call("JInverseProportion", a_numerator, a_denominator, x_numerator, x_denominator)


def similarity_ratios(first: int, second: int) -> dict:
    """Calculate length, area, and volume ratios from a positive scale ratio."""
    return _junior_call("JSimilarityRatios", first, second)


def relative_frequency(count: int, total: int) -> dict:
    """Calculate a frequency ratio from observed counts."""
    return _junior_call("JRelativeFrequency", count, total)


def circle_angle(central_angle: int) -> dict:
    """Calculate an inscribed angle from its corresponding central angle."""
    return _junior_call("JCircleAngle", central_angle)
