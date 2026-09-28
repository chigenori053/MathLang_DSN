# ProblemIntent Native Integration v1.0

Status: PASS / FROZEN for the bounded quadratic solve contract.

## Native path

`mirp solve-problem '<problem>'` returns a JSON object containing `result`, `plan`, and the validated MIRP `semantic_state`. The path is ProblemIntent parser → runtime goal bridge → planner → ReasonScript factorization or quadratic formula RU/RUS/RUO → MIRP transitions → intent evaluation. The immutable `ProblemDefinition` stays separate from mathematical state and runtime statuses. MIRP remains `mirp/0.1-si2`; PIF remains `mathlang/problem-intent/0.1`.

The factorization RU searches integer candidates in `[-100, 100]`. An exact independent equation check rejects incomplete candidate sets. Unrequired factorization falls back to the existing ReasonScript formula RU. Irrational or nonintegral roots remain `UNKNOWN` under this stage's exact integer answer contract. Factor, Expand, and Verify have goal bridge support and return `UNSUPPORTED` for execution, as specified for this stage.

## Validation

- `cargo test --offline --manifest-path mirp/Cargo.toml`: all existing MIRP/PIF tests and the 520-case integration matrix passed.
- `cargo clippy --offline --manifest-path mirp/Cargo.toml --all-targets -- -D warnings`: passed.
- `cargo fmt --manifest-path mirp/Cargo.toml --check`: passed.
- `reason project-validate . --json`: passed.
- `python3 -m pytest -q tests/test_api.py`: 9 passed.
- `git diff --check`: passed.
- `mirp/target/debug/examples/pini_digest` run in three independent processes over 100 generated end-to-end inputs each: SHA-256 `58f914cf6e6f8befad9684782ad47e5e1daa5755d2b5a27171debaa76a8c8cbf` in all three.

The matrix covers 40 parameterized cases in each group A–L, plus 20 English and 20 Japanese end-to-end cases. It also checks method mismatch in both directions, partial submissions, wrong targets, constraints, evidence references, dynamically generated coefficients, and scaled-equation equivalence. The five required specification examples have separate assertions.
