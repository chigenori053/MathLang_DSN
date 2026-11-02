# 実行例

リポジトリrootから実行する。`OUT` は未使用パスに変更する。

```sh
mathlang --help
mathlang doctor --format json
mathlang solve "Solve x^2 - 5x + 6 = 0." --format json
mathlang verify verification/fixtures/MV-01.json --output ./evidence/proof-001 --format json
mathlang verify verification/fixtures/MV-02.json --output ./evidence/refutation-001 --format json
mathlang verify usm --output ./evidence/usm-001 --format json
mathlang proof plan verification/fixtures/MV-01.json --format json
mathlang knowledge inspect mirp/knowledge/legacy.json --format json
```

verify usmで生成される既存モデルは `cli/evidence/formal-legacy/model_registry.json` の
`models.USM-MVF.1` にある。受入fixtureも `cli/evidence/acceptance-final/model.json` として保存している。

```sh
mathlang model check cli/evidence/acceptance-final/model.json
mathlang model inspect cli/evidence/acceptance-final/model.json --format json
mathlang model normalize cli/evidence/acceptance-final/model.json --output ./normalized-envelope.json
mathlang compare cli/evidence/acceptance-final/model.json cli/evidence/acceptance-final/different.json --format json
mathlang knowledge impact cli/evidence/formal-legacy/model_registry.json --revision 1 --format json
mathlang proof counterexample cli/evidence/acceptance-final/finite.json --format json
mathlang evidence validate ./evidence/proof-001/registry/EVIDENCE_ID --format json
```

任意設定 `mathlang.toml`:

```toml
[cli]
output_format = "text"
[runtime]
mode = "local"
[verification]
max_rss_mib = 2048
timeout_seconds = 300
parallelism = 1
[evidence]
directory = "./evidence"
```

検証手順:

```sh
cargo test --manifest-path cli/Cargo.toml --locked --offline
cargo clippy --manifest-path cli/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo fmt --manifest-path cli/Cargo.toml -- --check
python3 cli/scripts/acceptance.py ./cli/evidence/NEW-RUN --binary cli/target/debug/mathlang
python3 cli/scripts/audit.py --output /private/tmp/NEW-AUDIT.json
```

acceptanceは固定版の既存Lean sandboxが使用できる実行経路で実施する。
auditは今回保存した原証跡の再集計用で、証明を再実行しない。
