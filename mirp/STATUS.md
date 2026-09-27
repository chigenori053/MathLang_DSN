# MIRP Foundation v0.1 status

The bounded Foundation architecture is implemented. The serialized core schema uses
`mirp/1.0` as its version tag; that tag does not claim full MIRP v1.0 acceptance.
The mandatory gates establish **MIRP Foundation v0.1 PASS** within the bounded
input contract below.

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
| M9 MemorySpace | Bounded bridge passes | Canonical `.mirp` survives a process restart; imported ReasonScript codec validates the structure of RU/RUS/RUO traces attached to Rust snapshots. Retention policy, merge, and all retrieval scenarios remain. |

Foundation v0.1 also represents CALL and RESULT as typed objects. A RESULT must
reference a real CALL through the dependency graph. CALL execution is reserved
for a later RU family. Memory retrieval records `MEMORY_RETRIEVAL` evidence and
preserves the original object provenance with a `memory_id`.

The imported DSN_Test modules are listed in `DSN_TEST_IMPORTS.md`. Its older `mirp.rsn` was intentionally excluded because it defines a different schema.

Verification commands: `cargo test --offline --manifest-path mirp/Cargo.toml`,
`reason project-validate . --json`, and
`python3 -m unittest discover -s tests -q`. The Foundation suite covers core
types, scoped identity and aliases, uncertainty and conflicts, transactional
deltas, provenance and dependency validation, both runtime RU families,
cross-domain continuation, persistence, MemorySpace integrity, and CALL / RESULT
representation. The bounded input grammar and remaining MIRP v1.0 work are
described in `README.md`.

Verification result: Rust tests passed three consecutive runs (21 tests per
run), the existing Python API regression suite passed three consecutive runs
(9 tests per run), and three ReasonScript project validations produced identical
canonical output and passed. `cargo clippy --offline --manifest-path mirp/Cargo.toml
--all-targets -- -D warnings`, `cargo fmt --manifest-path mirp/Cargo.toml --check`,
and `git diff --check` also passed. The runtime source has
no fixture-string answer branches; an additional cross-domain arithmetic and
comparison case exercises different identifiers and numbers.
