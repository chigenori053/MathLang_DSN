# MathLang_DSN

MathLang_DSN is a **mathematical computation and assessment model** for MathLang. It is not a programming language. ReasonScript implements the deterministic model; a small Python host accepts expressions and returns JSON. The design follows the state, activation, transition, and validation cycle explored in [DSN_Test](https://github.com/chigenori053/DSN_Test). No DSN_Test source code is copied into this repository.

## Current scope (0.2)

- Exact rational arithmetic using `+`, `-`, `*`, `/`, integer powers 0–5, and parentheses.
- One-variable rational polynomials in `x` through degree five. Results use a reduced common denominator and coefficients in ascending power order.
- Exact polynomial differentiation, indefinite integration through degree four, derivative assessment, and antiderivative assessment. The integration constant is reported separately.
- One-variable linear equations with rational coefficients, including no solution and infinitely many solutions. Higher-degree equations are recognized but reported as `UNSOLVED`.
- Assessment of two expressions or two equations. Polynomial expressions are compared by exact coefficients. Linear equations are compared by solution set. Higher-degree equations return `VALID` when proportionality proves the same zero set; otherwise they return `UNVERIFIED` unless a stronger proof exists.
- A bounded, deterministic rule sequence for equation solving. Each committed transition is checked for solution-set preservation.
- A `knowledge.rsn` catalogue for arithmetic, polynomial, calculus, equation, and assessment rules. Each rule has a stable Knowledge ID, source, RU number, and RUS family.

Explicit multiplication is required (`2*x`, not `2x`). `^` and `**` both denote powers. Decimal literals, negative powers, non-polynomial functions, multivariable expressions, assumptions, matrices, and `.mlang` documents are not yet accepted by this interface. MathLang can pass normalized expression strings to the Python API. Integer literals, reduced coefficients, and denominators are limited to 1,000,000 in magnitude; expressions are limited to 64 nodes. The model does not silently interpret an unsupported expression as an incorrect mathematical step.

## Requirements and use

Install [ReasonScript 0.5.6.2](https://github.com/chigenori053/ReasonScript) so `reason` is on `PATH`. The Python host needs Python 3.11 or newer and no runtime Python dependencies.

```sh
python3 -m mathlang_dsn calculate '2*(3+4)'
python3 -m mathlang_dsn calculate '2*(x+3)=12'
python3 -m mathlang_dsn calculate '1/2+1/3'
python3 -m mathlang_dsn calculate '(x+1)^2'
python3 -m mathlang_dsn evaluate '2*(x+3)=12' 'x=3'
python3 -m mathlang_dsn evaluate '2*(x+3)=12' 'x=4'
python3 -m mathlang_dsn differentiate 'x^3+2*x'
python3 -m mathlang_dsn integrate 'x^2+1'
python3 -m mathlang_dsn evaluate-derivative 'x^3+2*x' '3*x^2+2'
python3 -m mathlang_dsn evaluate-antiderivative 'x^2+1' 'x^3/3+x+7'
```

The operations are available as Python functions in `mathlang_dsn`. They return dictionaries with `schema_version: "mathlang-dsn/0.2"`. Polynomial results include `coefficients`, `denominator`, ordered `knowledge_ids`/`knowledge_sources`, and `ruos` for arithmetic or calculus operations. Each polynomial RUO records its input and output coefficients and denominators, source RU, active RUS family, Knowledge ID, source, and validation. Equation results include `status`, reduced rational `numerator`/`denominator`, `final_state`, committed `steps`, and `ruos`. Each equation step embeds the matching RUO, active RUS, candidate RUs, before and after states, and validation result. Assessment results include the assessment Knowledge ID, source, active RUS, and assessment RUO. Equation traces start from the input equation with denominators cleared when each side is linear. The Python host creates an isolated temporary ReasonScript project for each call. Only parsed integer literals and fixed model calls are written to generated source; user text never becomes ReasonScript code.

## Model boundaries

MathLang owns input syntax and learning logs. This package owns expression conversion and the ReasonScript model. The model owns arithmetic, rule candidates, transition validation, and terminal classification. `knowledge.rsn` converts catalogue entries into RUs; equation state activates applicable RUs in `EquationRUS`, and the solver selects a validated RU for each step. Polynomial and calculus operations use the corresponding catalogued RUs. RUOs in the public result are the stable mathematical trace. They are model data, as in DSN_Test's ReasonScript structures; ReasonScript runtime counters such as `ruo_created_count` are not populated by these structures.

The arithmetic evaluator works on a bounded expression DAG. Its output is a canonical rational polynomial. Equation assessment compares linear solution sets by exact integer cross products. The solver chooses among applicable equation rules using a deterministic distance measure, validates each transition, and stops at `SOLVED`, `CONTRADICTION`, `INFINITE_SOLUTIONS`, `UNSOLVED`, or `RESOURCE_LIMIT`. Rational answers are represented exactly as reduced numerator and denominator. No floating-point sampling is used to turn an unknown case into `VALID` or `INVALID`.

## Coverage plan

The goal is to cover mathematics taught through graduate study, but coverage is accepted by **domain and proof contract**, rather than by school level. The next independent domains are complex arithmetic, multivariable polynomials, linear algebra, then bounded transcendental calculus. Each domain needs exact representation, stated assumptions, positive and negative cases, transition checks, and a documented `UNVERIFIED` boundary before it is exposed as supported. General integration, limits, differential equations, and advanced algebra remain outside the validated 0.2 scope.

## Validation

```sh
reason check
reason build
reason run --entry SelfCheck --json --trace=off
reason run --entry PolynomialSelfCheck --json --trace=off
reason run --entry KnowledgeSelfCheck --json --trace=off
reason project-validate . --json
python3 -m unittest discover -s tests -v
```

Each self-check calculation must return `true` in `runtime_result.result`; a successful process exit alone does not establish that. The Python tests execute the native ReasonScript runtime through the public API and check RUO links against returned results.

## Provenance and status

This project is informed by the locally developed DSN_Test prototype and by [MathLang](https://github.com/chigenori053/mathlang). It is a focused new implementation, not a release of DSN_Test. DSN_Test currently has no license file in its checkout, so its source is not redistributed here. The model uses ReasonScript as a required runtime and is licensed under Apache-2.0, matching MathLang. Version 0.2 does not claim the broader capabilities reported by DSN_Test.
