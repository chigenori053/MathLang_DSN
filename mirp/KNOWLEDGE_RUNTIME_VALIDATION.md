# Knowledge-Native Reasoning Runtime Integration v1.0 — Validation

## Result

The knowledge-native path passes the v1.0 acceptance gates for the quadratic runtime. The 29 bundled legacy Knowledge units retain their numeric IDs only for compatibility and have explicit Capability IDs. The native registry currently implements `FACTOR_QUADRATIC_INTEGER` and `SOLVE_QUADRATIC`; the other legacy operations continue through their existing ReasonScript APIs.

## Implementation checked

- `KnowledgeUnit.capabilities` accepts zero, one, or multiple IDs; legacy JSON without the field loads through the bundled migration table.
- Applicable Knowledge selects a registered Capability and RUS without a numeric Rule ID. The registry rejects duplicate Capability IDs and has deterministic ordering.
- The Capability executor reads MIRP state and returns a result. The runtime validates it, creates a Knowledge-native RUO, and records derived MIRP state and evidence.
- RUO carries Knowledge ID, occurrence, activation ID, Capability ID, RU/RUS refs, source state, and dependency refs. MIRP evidence also records the RUO ref and result state. New RU refs include semantic state identity.
- Explicit `conflicts_with` metadata on equally ranked applicable Knowledge produces `KNOWLEDGE_CONFLICT`. Missing Capability and missing applicable Knowledge remain distinct.

## Validation counts

| Test group | Independent scenario families | Parameter variations | Counted checks |
|---|---:|---:|---:|
| Knowledge, retrieval, applicability, activation, registry, persistence matrix | 1 | 40 quadratic inputs | 800 |
| Native intent, runtime, trace, MIRP transition matrix | 1 | 40 quadratic inputs and 20 English/Japanese pairs | 520 |
| Cross-cutting matrices total | 2 | 100 input/language variations | **1,320** |

Additional targeted tests cover seven Knowledge types with zero/one/multiple capabilities (21 variants), all 29 legacy migration mappings, three new Knowledge IDs sharing one Capability, missing Capability, explicit Knowledge conflict, persisted Knowledge execution, and semantic equivalence with the legacy quadratic result. Determinism is checked with 20 inputs across three fresh processes and three registration orders (60 process results), plus a separate three-process persisted Knowledge check. These targeted assertions are not included in the 1,320 count.

## Scale

Indexed retrieval was checked at 100, 1,000, 10,000, and 50,000 Knowledge units. At 50,000 units, the query examined 910 units (1.82%), evaluated three final candidates, and returned three candidates. The scale test uses examined/total as its gate; timing is logged for observation only.

## Commands

```text
cd mirp && cargo test
reason project-validate . --json
git diff --check
rg -n 'match activation.selected_ru|match knowledge_id|match runtime_rule_id|expected_answer|fixture_filename|test_index' mirp/src mirp/knowledge
```

The Rust suite, ReasonScript validation, whitespace check, and anti-cheating source scan pass. The last scan has no matches in the native execution source or bundled Knowledge.

## Scope of the current registry

The native `MathProblemContext` handles quadratic solving. The 27 other migrated Capability references are catalog metadata until corresponding domain runtimes are integrated; their existing ReasonScript behavior remains available. Knowledge registration alone increases executable coverage when the selected Capability is already in the registry.

The separate `calculation::compute` path now registers the three original operations plus `DATA_COVARIANCE`, `NORMALIZE_WEIGHTS`, `DISCRETE_EXPECTATION`, `NORMALIZE_EXACT`, `EXACT_EQUAL`, and `EXACT_COMPARE` alongside the quadratic executors. It loads nine exact-number/statistics Knowledge units explicitly and reuses the same retrieval, applicability, conflict, and activation logic. The existing `MathProblemContext` and its legacy catalogue remain unchanged in scope. Exact calculation inputs, runtime verification, evidence, and staged examples are documented in [EXACT_CALCULATIONS](../docs/cli/EXACT_CALCULATIONS.md).

The same calculation registry also exposes seven exact plane-geometry operations: distance, dot product, orthogonality, triangle area, and line/line, line/circle, circle/circle intersections. Seven additional Knowledge units are loaded from `plane_geometry_v0_1.json` (45 units before the classical physics extension). Inputs are two-dimensional exact rational coordinates; result checks include degeneracy, exact equation membership, independent intersection counts, distinct sorted points and explicit infinite intersections.

Six unit-aware classical physics Knowledge units in `classical_physics_v0_1.json` extend the calculation catalogue to 51 units (22 calculation capabilities). They cover constant acceleration, net force/acceleration, constant-force work, kinetic energy and work–kinetic-energy consistency. Exact rational SI conversion checks dimensions, matching 1..3 dimensional inertial orthonormal frames, positive mass, nonnegative elapsed time, Newtonian point-particle model and required declared assumptions. Given velocities and motion endpoints at or above light speed are rejected; this necessary check does not certify the nonrelativistic approximation. Physical applicability remains conditional on the declarations. A verified `consistent=false` energy balance reports an accurately detected inconsistency, not physical validity. Tests exercise all six selections, replacement Knowledge, missing Knowledge, conflict detection, altered answers, CLI error contracts and a 100,000-candidate load gate.
