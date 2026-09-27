# MIRP implementation status

This crate is an executable MIRP foundation, not a claim of full v1.0 acceptance.

| Specification phase | Status | Evidence / remaining work |
| --- | --- | --- |
| M1 core | Implemented | Rust types, provenance, scoped identity, canonical JSON; `tests/core.rs`. |
| M2 state and delta | Implemented | Transactional deltas, reference and cycle validation, conflict and unknown preservation; `tests/core.rs`. |
| M3 runtime bridge | Bounded | Rust invokes ReasonScript arithmetic and comparison RU/RUS/RUO directly; `tests/foundation.rs`. Additional RU families need adapters. |
| M4 mathematics | Partial | Integer arithmetic and comparisons plus the imported Stage 2A math adapter for bounded assignment, arithmetic equality, and inequality forms. General equations, functions, vectors, and matrices remain. |
| M5 natural language | Partial | Numeric facts, named relations, and the imported DSN_Test Japanese semantic adapter mapped into Rust MIRP. General reference resolution and English NLSRV coverage remain. |
| M6 code | Partial | Rust assignments and one-level `if`, plus imported Stage 2A code assignment and conditional forms. Calls, returns, and general control flow remain. |
| M7 cross-domain | Bounded proof passes | Natural language `x=5` → mathematics `y=x+2` → code `if y>6: result=y`, one shared state and provenance graph. |
| M8 geometry | Representation only | Geometric entity and spatial relation types can be stored; geometry adapter and reasoning bridge remain. |
| M9 MemorySpace | Bounded bridge passes | Canonical `.mirp` survives a process restart; imported ReasonScript codec protects RU/RUS/RUO traces attached to Rust snapshots. Retention policy, merge, and all retrieval scenarios remain. |

The imported DSN_Test modules are listed in `DSN_TEST_IMPORTS.md`. Its older `mirp.rsn` was intentionally excluded because it defines a different schema.

Verification: `cargo test --offline --manifest-path mirp/Cargo.toml`, `reason project-validate . --json`, and `python3 -m unittest discover -s tests -q`. The new Rust tests and existing Python regression suite passed three consecutive runs; ReasonScript project validation independently reports three identical canonical hashes.
