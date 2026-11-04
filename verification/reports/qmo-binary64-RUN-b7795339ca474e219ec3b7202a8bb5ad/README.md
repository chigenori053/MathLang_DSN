# MathLang binary64 / QMO Stage A verification results

Recorded run: `RUN-b7795339ca474e219ec3b7202a8bb5ad`. Stage A **PASS**, Stage B **NOT_VERIFIED**; all five model decisions remain **INCONCLUSIVE**.

- 12 binary64 density cases, fixed absolute/relative tolerances of 1e-12, independently checked using exact ratios of original IEEE inputs and replayed.
- 3 exact rational density cases and 5 Lean foundation lemmas.
- 8 invalid-input CLI controls independently replayed and rejected.
- Python integrity tests: 29 passed.
- Rust regression: MIRP quantum 14 passed / 1 ignored; CLI contracts 10 passed; numerical CLI 1 passed; numerical verification 5 passed; existing verification foundation 33 passed.

`logs/` contains byte-preserved original regression output. `independent-audit.json` retains 20 raw replay outputs (12 valid binary64 cases plus 8 rejection controls). `evidence-index.json` links every command to its original artifact ID. `checksums.sha256` covers this publication bundle except itself.

[Complete source artifacts and immutable Registry](https://github.com/chigenori053/MRA_Test/tree/536e780ef20f00ad6cc597d1fd8274c73c34c293/USM_QMO_Evidence/runs/RUN-b7795339ca474e219ec3b7202a8bb5ad) are pinned to MRA_Test commit `536e780ef20f00ad6cc597d1fd8274c73c34c293`. This bundle publishes existing results; it does not claim a new verification run. The MathLang source was uncommitted at execution and is identified by the original source snapshots and hashes, not by retrospectively substituting a commit ID.

Scope: verification-only dimension-2 binary64 density construction. No production inference or UnifiedRU changes, no generic eigensolver, semantic understanding or quantum advantage claim. Subnormal/tiny components can underflow; input reconstruction is not guaranteed.
