# MathLang_DSN

## Problem intent aware validation (bounded Rust layer)

`mirp::intent` stores `Problem` and `ProblemIntent` separately from `SemanticState`: sourced
quadratic givens, sourced constraints, a MIRP semantic target, the requested
operation and output type, method constraints, and completion criteria. A
`ReasoningContext` evaluates a candidate as five independent dimensions and
returns sourced elimination records and educational error evidence. Accepted
transitions retain the same intent while applying a MIRP `SemanticDelta`.

The executable contract currently proves exact integer-root quadratic cases
with coefficients of magnitude at most 1,000,000. A correct root subset remains
mathematically valid but incomplete. An excluded root is recorded with the
constraint and its provenance. Different solution methods are allowed unless a
method is required. A method label is an attestation by the adapter, not proof
that a particular algorithm was executed. Non-integral roots, non-quadratic
equations, other operation intents, and unsupported mathematical claims preserve
`UNKNOWN` where proof is unavailable. Structured polynomial coefficients and
exact expansion of algebra syntax trees are the checked semantics; display
strings are never used as a correctness oracle.

`mirp/tests/intent.rs` checks 10 groups × 20 parameterized algebra cases, then
repeats the matrix three times. The supplied specification ends midway through
its mandatory acceptance gate, so this is a bounded implementation and does
not claim full stage acceptance or Mathematics III/C coverage.

## MIRP Foundation v0.1

The `mirp/` Rust crate owns MIRP's typed objects, scoped entity bindings,
validation, canonical JSON, semantic deltas, conflict and unknown states, and
`.mirp` persistence. The bounded natural-language, Japanese, mathematics, and code
adapters build one `SemanticState`. Arithmetic uses the existing ReasonScript
polynomial RU/RUS/RUO path; integer comparisons use a ReasonScript comparison
RU selected through `Knowledge::KMirpComparisonRus`. The Rust bridge invokes
ReasonScript directly; the existing Python API remains for the 0.3 interface.
CALL and RESULT are typed MIRP representations with a validated dependency;
their execution is a later RU family. Memory retrieval adds explicit evidence
while retaining source provenance and the memory ID. The status and acceptance
boundary are documented in `mirp/STATUS.md`.

In the identity-separated state format, `Common.id` is a source-independent
Semantic ID and `Common.occurrence_id` identifies one assertion or derivation.
Object references form the semantic graph; `SemanticState.dependencies` and
`Provenance.parent_occurrence_ids` form the occurrence-level derivation graph.
Each object persists an `occurrence_key`, so both IDs can be recomputed and
validated when a state is loaded. An existing occurrence cannot change its
meaning, origin, key, or parent set through `SemanticDelta.updated`.
Two source histories can share semantic IDs while their complete canonical
`.mirp` files differ. The current format is `mirp/0.1-si2`; older `mirp/0.1-si`
and `mirp/1.0` snapshots are rejected and must be regenerated.

```sh
cargo test --offline --manifest-path mirp/Cargo.toml
cargo run --offline --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global nl 'x is five.'
cargo run --offline --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global math 'y = x + 2'
cargo run --offline --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global code $'if y > 6:\n    result = y'
cargo run --offline --manifest-path mirp/Cargo.toml -- /tmp/story.mirp story global ja '太郎は学生である。'
```

The three commands run in separate processes. The final state contains a
dependency path from the natural-language `x` fact through the mathematical
`y` result and the code condition to `result = 7`. The implemented text
adapters intentionally accept a small fragment: numeric `name is number`
facts; named `is`, `is not`, `may be`, `must be`, and `If X, A is B` relations;
integer arithmetic assignments; and simple code assignments with a
single-level `if`. The copied DSN_Test Stage 2A math and code adapters add
bounded assignment, relation, arithmetic equality, and conditional forms.
The copied DSN_Test Japanese semantic adapter adds its
bounded Japanese entity, state, quantity, and event forms. Its output enters
the same Rust `SemanticState`. The copied ReasonScript MemorySpace codec stores
the RU/RUS/RUO trace alongside a versioned Rust state snapshot. Unsupported
syntax returns an explicit error. General equations/functions/matrices, calls
and control flow, geometry observations, and full MemorySpace retention policy
are not yet implemented. The Rust schema can represent several of these structures,
but MIRP v1.0 acceptance is not claimed for them.

MathLang_DSN is a **mathematical computation and assessment model** for MathLang. It is not a programming language. ReasonScript implements the deterministic model; a small Python host accepts expressions and returns JSON. The design follows the state, activation, transition, and validation cycle explored in [DSN_Test](https://github.com/chigenori053/DSN_Test). MIRP now includes selected DSN_Test ReasonScript modules with the project owner's authorization; see `mirp/DSN_TEST_IMPORTS.md`.

## Current scope (0.3)

- Exact rational arithmetic using `+`, `-`, `*`, `/`, integer powers 0–5, and parentheses.
- One-variable rational polynomials in `x` through degree five. Results use a reduced common denominator and coefficients in ascending power order.
- Exact polynomial differentiation, indefinite integration through degree four, derivative assessment, and antiderivative assessment. The integration constant is reported separately.
- One-variable linear equations with rational coefficients, including no solution and infinitely many solutions. `calculate` reports higher-degree equations as `UNSOLVED`; `solve_quadratic` separately handles bounded quadratic equations with exact real radical roots.
- Assessment of two expressions or two equations. Polynomial expressions are compared by exact coefficients. Linear equations are compared by solution set. Higher-degree equations return `VALID` when proportionality proves the same zero set; otherwise they return `UNVERIFIED` unless a stronger proof exists.
- A bounded, deterministic rule sequence for equation solving. Each committed transition is checked for solution-set preservation.
- A `knowledge.rsn` catalogue for arithmetic, polynomial, calculus, equation, and assessment rules. Each rule has a stable Knowledge ID, source, RU number, and RUS family.

Explicit multiplication is required (`2*x`, not `2x`). `^` and `**` both denote powers. Decimal literals, negative powers, general non-polynomial expressions, symbolic multivariable expressions, assumptions, matrices, and `.mlang` documents are not yet accepted by the expression interface. Structured middle-school operations below take integer or rational parameters instead. MathLang can pass normalized expression strings to the Python API. Integer literals, reduced coefficients, and denominators are limited to 1,000,000 in magnitude; expressions are limited to 64 nodes. The model does not silently interpret an unsupported expression as an incorrect mathematical step.

## Japanese junior-high Knowledge

The 13 new Knowledge entries (`JH_*`, IDs 21–33) map to selected calculations in the four areas of the [Japanese junior-high mathematics curriculum](https://www.mext.go.jp/component/a_menu/education/micro_detail/__icsFiles/afieldfile/2019/03/18/1387018_004.pdf). Together with the previous 14 entries, the catalogue contains 27 rules. This is **partial curriculum coverage**; each operation has a bounded input contract and an `UNSUPPORTED` or `RESOURCE_LIMIT` outcome outside it.

| Area | New Knowledge | Public operations |
| --- | --- | --- |
| Numbers and expressions | Prime factorization; exact square-root simplification; quadratic formula; two-variable linear systems | `prime_factors`, `simplify_sqrt`, `solve_quadratic`, `solve_system` |
| Functions | Exact polynomial value at a rational input; inverse proportion `y=a/x` | `function_value`, `inverse_proportion` |
| Geometry | Polygon interior-angle sum; Pythagorean hypotenuse; similarity length/area/volume ratios; inscribed angle from central angle | `polygon_angle_sum`, `hypotenuse`, `similarity_ratios`, `circle_angle` |
| Data | Mean, median, quartiles and range; probability for equally likely cases; relative frequency | `data_summary`, `classical_probability`, `relative_frequency` |

`solve_quadratic("x^2-2=0")` returns two roots in the exact form `(numerator + radical_coefficient*sqrt(radicand))/denominator`; only **real** roots are returned. `data_summary` splits sorted data into lower and upper halves, excluding the middle item when the count is odd, then takes each half's median. Geometry functions require their named assumptions: simple polygons, right triangles, similar figures, and an inscribed angle subtending the supplied central angle. `classical_probability` assumes equally likely outcomes.

Current operation limits: factor inputs up to 1,000,000; square-root inputs up to 200,000,000; quadratic integer coefficients up to 10,000 in magnitude and discriminant up to 200,000,000; system coefficients up to 10,000; function input numerator and denominator up to 20; polygon sides up to 10,000; right-triangle legs up to 10,000; similarity ratio components up to 1,000; data sets of 2–64 integers each within ±1,000,000; and probability/frequency totals up to 1,000,000. Inputs outside these contracts return `UNSUPPORTED` or `RESOURCE_LIMIT`.

The public APIs do not yet parse systems written as two equations, construct or interpret diagrams, prove congruence or similarity, factor symbolic polynomials into factors, solve general word problems, process histograms or box plots, or infer probability from a described experiment. These remain outside the implemented Knowledge despite being part of the curriculum.

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
python3 -m mathlang_dsn prime-factors 360
python3 -m mathlang_dsn simplify-sqrt 72
python3 -m mathlang_dsn solve-quadratic 'x^2-2=0'
python3 -m mathlang_dsn solve-system 1 1 5 1 -1 1
python3 -m mathlang_dsn function-value '(x+1)^2' 3 2
python3 -m mathlang_dsn inverse-proportion 6 1 2
python3 -m mathlang_dsn polygon-angle-sum 5
python3 -m mathlang_dsn hypotenuse 3 4
python3 -m mathlang_dsn similarity-ratios 2 3
python3 -m mathlang_dsn circle-angle 90
python3 -m mathlang_dsn data-summary 1 2 3 4 5 6
python3 -m mathlang_dsn classical-probability 2 6
python3 -m mathlang_dsn relative-frequency 3 12
```

The operations are available as Python functions in `mathlang_dsn`. They return dictionaries with `schema_version: "mathlang-dsn/0.3"`. Polynomial results include `coefficients`, `denominator`, ordered `knowledge_ids`/`knowledge_sources`, and `ruos` for arithmetic or calculus operations. Each polynomial RUO records its input and output coefficients and denominators, source RU, active RUS family, Knowledge ID, source, and application status. Equation results include `status`, reduced rational `numerator`/`denominator`, `final_state`, committed `steps`, and `ruos`. Each equation step embeds the matching RUO, active RUS, candidate RUs, before and after states, and validation result. Assessment and junior-high results include their Knowledge ID, source, active RUS, and a RUO; junior-high RUOs also record integer inputs and outputs. Equation traces start from the input equation with denominators cleared when each side is linear. The Python host creates an isolated temporary ReasonScript project for each call. Only parsed integer literals and fixed model calls are written to generated source; user text never becomes ReasonScript code.

## Model boundaries

MathLang owns input syntax and learning logs. This package owns expression conversion and the ReasonScript model. The model owns arithmetic, rule candidates, transition validation, and terminal classification. `knowledge.rsn` converts catalogue entries into RUs; equation state activates applicable RUs in `EquationRUS`, and the solver selects a validated RU for each step. Polynomial and calculus operations use the corresponding catalogued RUs. RUOs in the public result are the stable mathematical trace. They are model data, as in DSN_Test's ReasonScript structures; ReasonScript runtime counters such as `ruo_created_count` are not populated by these structures.

The arithmetic evaluator works on a bounded expression DAG. Its output is a canonical rational polynomial. Equation assessment compares linear solution sets by exact integer cross products. The solver chooses among applicable equation rules using a deterministic distance measure, validates each transition, and stops at `SOLVED`, `CONTRADICTION`, `INFINITE_SOLUTIONS`, `UNSOLVED`, or `RESOURCE_LIMIT`. Rational answers are represented exactly as reduced numerator and denominator. No floating-point sampling is used to turn an unknown case into `VALID` or `INVALID`.

## Coverage plan

The goal is to cover mathematics taught through graduate study, but coverage is accepted by **domain and proof contract**, rather than by school level. Finish the remaining middle-school contracts before claiming full junior-high coverage; later domains include complex arithmetic, multivariable polynomials, linear algebra, and bounded transcendental calculus. Each domain needs exact representation, stated assumptions, positive and negative cases, transition checks, and a documented `UNVERIFIED` boundary before it is exposed as supported. General integration, limits, differential equations, and advanced algebra remain outside the validated 0.3 scope.

## Validation

```sh
reason check
reason build
reason run --entry SelfCheck --json --trace=off
reason run --entry PolynomialSelfCheck --json --trace=off
reason run --entry KnowledgeSelfCheck --json --trace=off
reason run --entry JuniorHighSelfCheck --json --trace=off
reason project-validate . --json
python3 -m unittest discover -s tests -v
```

Each self-check calculation must return `true` in `runtime_result.result`; a successful process exit alone does not establish that. The Python tests execute the native ReasonScript runtime through the public API and check RUO links against returned results.

## Provenance and status

This project is informed by the locally developed DSN_Test prototype and by [MathLang](https://github.com/chigenori053/mathlang). It is a focused new implementation, not a release of DSN_Test. Selected ReasonScript files were imported from DSN_Test with the project owner's authorization and are listed in `mirp/DSN_TEST_IMPORTS.md`. The model uses ReasonScript as a required runtime and is licensed under Apache-2.0, matching MathLang. Version 0.3 does not claim the broader capabilities reported by DSN_Test.
