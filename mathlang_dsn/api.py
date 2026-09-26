"""Convert bounded MathLang expressions into calls to the ReasonScript model."""

from __future__ import annotations

import ast
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

MODEL = Path(__file__).with_name("model.rsn")
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
        tree = ast.parse(source.strip(), mode="eval").body
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
        raise UnsupportedExpression("supported syntax: integers, x, +, -, *, /, parentheses")

    visit(tree)
    return kinds, values, lefts, rights


def _literal(values: list[int]) -> str:
    return "[" + ", ".join(map(str, values)) + "]"


def _term(name: str, source: str) -> str:
    arrays = _encode(source)
    return f"    let {name} = Model::Evaluate(" + ", ".join(_literal(a) for a in arrays) + ")"


def _run(lines: list[str], result: str) -> object:
    if shutil.which("reason") is None:
        raise RuntimeError("ReasonScript 0.5.6.2 `reason` command is required")
    with tempfile.TemporaryDirectory(prefix="mathlang-dsn-") as directory:
        workspace = Path(directory)
        (workspace / "src").mkdir()
        (workspace / "reason.toml").write_text(MANIFEST.read_text())
        (workspace / "src" / "model.rsn").write_text(MODEL.read_text())
        source = "\n".join(
            ["package mathlang_dsn", "module main {", "  import mathlang_dsn.Model", "  calculation Request {"]
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
    """Calculate an integer affine expression or solve a linear equation."""
    try:
        equation = _split_equation(expression)
        if equation is None:
            term = _run([_term("term", expression)], "term")
            if not term["valid"]:
                return {"schema_version": "mathlang-dsn/0.1", "status": "UNSUPPORTED", "reason": term["reason"]}
            status = "CALCULATED" if term["a"] == 0 else "SYMBOLIC"
            return {"schema_version": "mathlang-dsn/0.1", "status": status, "coefficient": term["a"], "constant": term["b"]}
        left, right = equation
        outcome = _run([_term("left", left), _term("right", right)], "Model::CalculateEquation(left, right)")
        return {"schema_version": "mathlang-dsn/0.1", **outcome}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.1", "status": "UNSUPPORTED", "reason": str(exc)}


def evaluate(before: str, after: str) -> dict:
    """Assess whether two expressions or equations have the same meaning."""
    try:
        first = _split_equation(before)
        second = _split_equation(after)
        if (first is None) != (second is None):
            return {"schema_version": "mathlang-dsn/0.1", "status": "UNVERIFIED", "reason": "different input kinds"}
        if first is None:
            result = _run([_term("before", before), _term("after", after)], "Model::AssessTerms(before, after)")
        else:
            assert second is not None
            result = _run(
                [_term("left", first[0]), _term("right", first[1]), _term("next_left", second[0]), _term("next_right", second[1])],
                "Model::AssessEquationTerms(left, right, next_left, next_right)",
            )
        return {"schema_version": "mathlang-dsn/0.1", "status": result}
    except UnsupportedExpression as exc:
        return {"schema_version": "mathlang-dsn/0.1", "status": "UNVERIFIED", "reason": str(exc)}
