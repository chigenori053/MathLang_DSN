# ProblemIntent Foundation Completion tickets

The MIRP `mirp/0.1-si2` state format remains frozen. Each ticket has an
independent validation gate; the stage is PASS only after all five gates and
the complete matrix pass.

| Ticket | Scope | Gate | Result |
| --- | --- | --- | --- |
| PIF-1 | Versioned intent schema; goals, qualifiers, output and criterion identity; reference and DAG validation | `tests/pif_schema.rs`; no MIRP serialization change | PASS |
| PIF-2 | Immutable problem definition; runtime goal/criterion status; local/global validity; primary error origin | `tests/pif_runtime.rs`; downstream steps preserve origin | PASS |
| PIF-3 | Bounded English/Japanese problem-to-intent parser | `tests/pif_parser.rs`; ambiguity and unsupported inputs explicit | PASS |
| PIF-4 | Executed method evidence and strategy recognition | `tests/pif_runtime.rs`; factorization, quadratic formula, mismatch and missing trace | PASS |
| PIF-5 | End-to-end pipeline, matrix, metrics, determinism, regressions and anti-cheating check | `tests/pif_matrix.rs` and `PIF_REPORT.md` | PASS |

The matrix counts parameterized variants separately from independent problem
forms. The final report records both.

The specification's group totals contain an arithmetic error: F–Q is 12
groups, so 12 × 20 = 240 and the per-group minimum totals **400**, not 380.
The gate uses the stricter 400-case total.
