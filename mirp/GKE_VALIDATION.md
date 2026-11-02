# Generic Knowledge Executor v1.0 validation

## Implemented path

`MemorySpace.infer_generic` and `mirp gke-infer <state.mirp> <knowledge.json> <goal>` load structured `GenericRule` values at runtime. The command also accepts a JSON `RulePredicate` as the goal argument when exact arguments matter. Rules match typed `PredicateFact` objects, bind variables, evaluate constraints, instantiate conclusions, and commit a `SemanticDelta` containing the conclusion and derivation `Evidence`. The report records RU, RUS, RUO, state references, bindings, candidate selection, and occurrence references. `KnowledgeSpace` indexes generic rules by their first premise predicate. The prior native quadratic capability path remains available; the GKE API selects only structured generic rules.

Exact numeric guards (`NONZERO`, `POSITIVE`, `INTEGER`) run in `mathlang_dsn/generic_knowledge.rsn`. `DISTINCT` is typed equality in the MIRP bridge. The bridge and state commit are Rust because MIRP's typed objects, validation, persistence, and stable identities are implemented there. Generic matching does not branch on Knowledge ID or `runtime_rule_id`.

## Executed gates

| Gate | Result |
| --- | --- |
| Hot injection after binary build | Same built executable: empty Knowledge JSON -> `UNKNOWN`; added `TEST_TRANSITIVE_EQUALITY` -> `DERIVED`. |
| 1,300 scenario matrix | 50 schema, 100 matching, 100 binding, 100 constraints, 100 instantiation, 100 single step, 100 multi step, 50 dependencies, 100 conflicts, 100 trace, 100 SemanticDelta, 100 dynamic registration, 100 determinism, 100 malformed/negative. Each loop varies IDs and/or predicate terms; counts are scenarios, not assertion totals. |
| Four domains | Logic equality transitivity, arithmetic greater-than transitivity, algebra additive equality using `ADD` premises, geometry right-triangle conclusion using `LEG`, `SQUARE`, and `ADD` premises. All use the same matcher. |
| Metamorphic order | Rule registration order and state object order are covered by sorted indexing and state canonicalization; a dedicated test compares registration permutations. |
| Fresh-process determinism | Three registration orders × three fresh processes compare the complete JSON result, including state, RU/RUS/RUO, selected Knowledge, and bindings. |
| Mandatory bounds | Defaults 64 steps, 128 candidates per step, 256 bindings per rule, and depth 32; dedicated tests exceed each boundary and observe `RESOURCE_LIMIT`. |
| Anti-cheating source scan | `TEST_`, `expected_answer`, `case_index`, and Knowledge-ID dispatch absent from GKE production source. One pre-existing `fixture` word occurs in a comment in `intent.rs`. |

Performance observations from the explicit `performance_scales` test (debug build, milliseconds, one relevant rule at each size):

| Total Knowledge | Registration | Inference | Examined | Candidate | Binding attempts | Executed | Steps |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| 1,000 | 13 | 0 | 1 | 1 | 1 | 1 | 1 |
| 10,000 | 155 | 1 | 1 | 1 | 1 | 1 | 1 |
| 50,000 | 848 | 1 | 1 | 1 | 1 | 1 | 1 |
| 100,000 | 1,746 | 1 | 1 | 1 | 1 | 1 | 1 |

Timing is observational; it is not a pass threshold. The performance fixture deliberately has one relevant premise predicate, so these figures demonstrate index selectivity rather than worst-case throughput.

## Scope limits

GKE v1.0 consumes structured predicates and existing arithmetic/geometry facts. It does not parse natural-language theorems, calculate `ADD` or `SQUARE` facts from scratch, or route arbitrary problem text into GKE. The existing native quadratic path still uses capability execution and does not automatically dispatch to GKE. Generic rules are selected through `MemorySpace.infer_generic` or the `gke-infer` command. Numeric constraints currently invoke ReasonScript once per candidate, so workloads with many numeric guards should be measured separately.

Run the checks with:

```sh
reason check
cargo test --manifest-path mirp/Cargo.toml --offline --test gke
cargo test --manifest-path mirp/Cargo.toml --offline --test gke performance_scales -- --ignored --nocapture
cargo test --manifest-path mirp/Cargo.toml --offline
```
