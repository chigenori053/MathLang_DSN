# MathLang_DSN

MathLang_DSN is a **mathematical computation and assessment model** for MathLang. It is not a programming language. ReasonScript implements the deterministic model; a small Python host accepts expressions and returns JSON. The design follows the state, activation, transition, and validation cycle explored in [DSN_Test](https://github.com/chigenori053/DSN_Test). No DSN_Test source code is copied into this repository.

## Current scope (0.1)

- Integer arithmetic using `+`, `-`, `*`, exact `/`, and parentheses.
- Affine expressions in one variable, `x`, with integer coefficients.
- One-variable linear equations over the rational numbers, including no solution and infinitely many solutions.
- Assessment of two expressions or two equations. `VALID` means equal expressions or equal solution sets within this fragment; `INVALID` means they differ. Unsupported input returns `UNVERIFIED` for assessment and `UNSUPPORTED` for calculation.
- A bounded, deterministic rule sequence for equation solving. Each committed transition is checked for solution-set preservation.

Explicit multiplication is required (`2*x`, not `2x`). Decimal values, powers, nonlinear terms, functions, assumptions, and `.mlang` documents are not yet accepted by this interface. MathLang can pass its normalized expression strings to the Python API. All integer literals and intermediate affine coefficients are limited to 1,000,000 in magnitude; expressions are limited to 64 nodes. The model does not silently interpret an unsupported expression as an incorrect mathematical step.

## Requirements and use

Install [ReasonScript 0.5.6.2](https://github.com/chigenori053/ReasonScript) so `reason` is on `PATH`. The Python host needs Python 3.11 or newer and no runtime Python dependencies.

```sh
python3 -m mathlang_dsn calculate '2*(3+4)'
python3 -m mathlang_dsn calculate '2*(x+3)=12'
python3 -m mathlang_dsn evaluate '2*(x+3)=12' 'x=3'
python3 -m mathlang_dsn evaluate '2*(x+3)=12' 'x=4'
```

The same operations are available as `mathlang_dsn.calculate(expression)` and `mathlang_dsn.evaluate(before, after)`. They return dictionaries with `schema_version: "mathlang-dsn/0.1"`. Equation results include `status`, reduced rational `numerator`/`denominator`, `final_state`, and committed `steps`. Each step contains its rule, before and after states, and validation result. The Python host creates an isolated temporary ReasonScript project for each call. Only parsed integer literals and fixed model calls are written to generated source; user text never becomes ReasonScript code.

## Model boundaries

MathLang owns input syntax and learning logs. This package owns expression conversion and the ReasonScript model. The model owns arithmetic, rule candidates, transition validation, and terminal classification. ReasonScript's runtime JSON trace is an implementation detail; the model's `steps` are the stable mathematical trace.

The arithmetic evaluator works on a bounded expression DAG. Its output is a canonical affine term `a*x+b`. Equation assessment compares solution sets by exact integer cross products. The solver chooses among applicable equation rules using a deterministic distance measure, validates each transition, and stops at `SOLVED`, `CONTRADICTION`, `INFINITE_SOLUTIONS`, `UNSOLVED`, or `RESOURCE_LIMIT`. Rational answers are represented exactly as reduced numerator and denominator.

## Validation

```sh
reason check
reason build
reason run --entry SelfCheck --json --trace=off
python3 -m unittest discover -s tests -v
```

`SelfCheck` must return `true` in `runtime_result.result`; a successful process exit alone does not establish that. The Python tests execute the native ReasonScript runtime through the public API.

## Provenance and status

This project is informed by the locally developed DSN_Test prototype and by [MathLang](https://github.com/chigenori053/mathlang). It is a focused new implementation, not a release of DSN_Test. DSN_Test currently has no license file in its checkout, so its source is not redistributed here. The model uses ReasonScript as a required runtime and is licensed under Apache-2.0, matching MathLang. Version 0.1 is a bounded foundation for further verified mathematical domains; it does not claim the broader capabilities reported by DSN_Test.
