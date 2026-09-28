# ProblemIntent Foundation Completion v1.0 — validation report

**Result: PASS for the bounded foundation contract.** The existing MIRP state
format remains `mirp/0.1-si2`. The new layer uses
`mathlang/problem-intent/0.1` and does not serialize intent into MIRP state.
The supported executable domain is exact integer-root quadratic equations;
factor, expand, and verify are structurally parsed but not generally executed.

## Tickets and case accounting

PIF-1 through PIF-5 and their verification files are listed in
`PIF_TICKETS.md`. The mandatory matrix has 18 groups. Its printed total of 380
is arithmetically inconsistent with the group minima: A–D = 80, E = 40,
F–Q = 240, R = 40, totaling **400**. We applied all 18 stated group minima.

| Group | Cases | Evidence |
| --- | ---: | --- |
| A schema | 20 | `pif_schema.rs` positive and negative variants |
| B goal graph | 20 | `pif_schema.rs` alternative graphs and invalid links |
| C immutable definition | 20 | `pif_runtime.rs` definition equality through execution |
| D criteria runtime | 20 | `pif_runtime.rs` unresolved, partial, complete |
| E extraction | 40 | `pif_parser.rs` 20 English, 20 Japanese |
| F language equivalence | 20 | `pif_parser.rs` paired semantic signatures |
| G alternative strategies | 20 | `pif_matrix.rs` factor and formula trajectories |
| H executed methods | 20 | `pif_runtime.rs` emitted RU/RUS/RUO traces |
| I mismatch | 20 | `pif_runtime.rs` both declaration directions |
| J local/global validity | 20 | `pif_runtime.rs` three-transition chains |
| K error origin | 20 | `pif_runtime.rs` primary and downstream records |
| L constraints | 20 | `pif_matrix.rs` inequalities, excluded values, intervals |
| M partial answers | 20 | `pif_matrix.rs` valid but missing one root |
| N wrong target | 20 | `pif_matrix.rs` valid but irrelevant mathematics |
| O ambiguity and unknown | 20 | `pif_matrix.rs` ambiguous, unsupported and out-of-scope cases |
| P evidence integrity | 20 | `pif_matrix.rs` reference checks and dangling rejection |
| Q path independence | 20 | `pif_matrix.rs` distinct traces, same semantic completion |
| R end to end | 40 | `pif_matrix.rs` 20 English, 20 Japanese |

These are **400 parameterized validation cases**, not 400 independent
mathematical problem types. They use 20 coefficient variants and six broad
forms: solve, factor, expand, verify, constrained solve, and ambiguous or
unsupported instructions. The end-to-end executable form is quadratic solve.

## Dimension-level results

The rates below are pass counts against explicit expected assertions in this
bounded matrix. They are not estimates of accuracy on unseen student work.

| Metric | Verified / tested |
| --- | ---: |
| Schema validation accuracy | 40 / 40 A accept/reject checks |
| Intent extraction accuracy | 40 / 40 |
| Cross-language intent equivalence | 20 / 20 |
| Mathematical validity accuracy | 40 / 40 alternative-path executions |
| Problem relevance accuracy | 20 / 20 wrong-target cases |
| Intent alignment accuracy | 20 / 20 partial + 20 / 20 wrong-target cases |
| Method compliance accuracy | 20 / 20 required-method cases |
| Goal completion accuracy | 40 / 40 end-to-end cases |
| Completion criterion accuracy | 20 / 20 partial-to-complete trajectories |
| Alternative-strategy acceptance | 40 / 40 trajectories |
| Method execution recognition | 40 / 40 factor/formula traces |
| Declared/executed mismatch detection | 40 / 40 directional mismatches |
| Local validity accuracy | 60 / 60 three-step chain transitions |
| Global derivational validity accuracy | 60 / 60 chain transitions |
| Primary error localization | 20 / 20 chains |
| Constraint application accuracy | 20 / 20 cases |
| Partial-correctness classification | 20 / 20 cases |
| UNKNOWN preservation | 20 / 20 ambiguous cases plus unsupported variants |
| Evidence integrity | 20 / 20 cases; dangling references rejected |
| Determinism | 3 / 3 independent processes |

Safety controls: false invalid **0/60** across 40 valid alternative-path
executions and 20 valid partial answers; false complete **0/60** across partial,
wrong-target, and ambiguous cases; false method compliance **0/40** directional
mismatches; false intent resolution **0/20** ambiguous cases.

## Reproduction and boundaries

Three independent `cargo test --offline --manifest-path mirp/Cargo.toml -q`
processes passed. Three independent `cargo run --offline --manifest-path
mirp/Cargo.toml --example pif_digest -q` processes produced the identical hash:

```text
2d22dc4892fd077b241dc088f9fb11851ccfc056c12ddcf86cc898e74810f01b
```

`cargo clippy --offline --manifest-path mirp/Cargo.toml --all-targets -- -D
warnings`, `cargo fmt --manifest-path mirp/Cargo.toml --check`, `git diff
--check`, `reason project-validate . --json`, and the existing Python API
regression suite (9 tests) passed. The old identity, persistence, MemorySpace,
cross-domain, and CALL/RESULT Rust tests remain unchanged and passed. The
anti-cheating source gate rejects fixture-ID, expected-answer, and reference
trajectory branches in the new production layer.

Method evidence comes from executed bounded Rust quadratic operations with
explicit RU/RUS/RUO records. The declared method field is ignored for
compliance. General natural-language parsing, non-integer roots, other
operation execution, geometry, and curriculum-wide coverage remain outside
this stage.
