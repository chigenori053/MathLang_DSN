# Mathematical Verification Foundation v0.1

MLDSN-MVF-001 の独立 Rust crate。既存 MIRP の schema/API と ReasonScript Runtime は変更しない。
元の要求は `SPECIFICATION.md`、対応範囲は `reports/IR_SPECIFICATION.md`、最終判定は `reports/FINAL_VERIFICATION.md` を参照。

## 実行

Rust 1.93.1、Lean 4.35.0、Mathlib `56a005855ea17e0b0958f73ec801ae140b0a1679` を使用する。
依存は `Cargo.lock` と `lean/lake-manifest.json`、実測環境は `toolchain.json` と各 run の `environment.json` に保存する。

```sh
cargo test --manifest-path verification/Cargo.toml --locked --offline
cargo clippy --manifest-path verification/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo fmt --manifest-path verification/Cargo.toml -- --check

# 固定 revision に対応する、ビルド済み Lake package cache を読み取り専用で再利用する。
python3 verification/scripts/prepare_lean.py /path/to/project/.lake/packages
cargo build --manifest-path verification/Cargo.toml --locked --offline
verification/target/debug/mathlang-verification verify-obligation verification/fixtures/MV-01.json /tmp/new-obligation-run
verification/target/debug/mathlang-verification verify-usm /tmp/new-usm-run
verification/target/debug/mathlang-verification check-translations /tmp/new-semantic-run
python3 verification/scripts/negative_bridge.py /tmp/new-negative-run
```

各出力ディレクトリは新規でなければならない。既存 Evidence は上書きしない。
`verify-usm` は9件を3回、並列数1で実行する。通常の Lean 証明検査に加え、MV-01〜06 は手書きの独立命題との定義的な型一致を Lean に検査させる。
`check-translations` は行列・座標テンソル・密度演算子など11件の別命題と照合する。

Lean は macOS の `sandbox-exec` 内で実行し、ネットワーク、外部プロセス起動、作業領域外への書込を禁止する。
OS sandbox と RSS 計測を起動できない実行環境では `ERROR` とし、隔離なしの実行へ切り替えない。
Codex の外側の sandbox に制限される場合は、OS sandbox を起動できる実行経路が必要となる。
RSS 2 GiB、実時間300秒、Lean allocator 1536 MiB、単一スレッドで実行する。
2 GiB 超過は `UNKNOWN`。Lean の import・変換などの基盤異常は `ERROR`。

## 入力と API

`verify-obligation` の入力は `{ "obligation": ProofObligation, "recipe": Recipe, "refutation": bool }`。
`fixtures/MV-01.json`〜`MV-06.json` が実例となる。文字列の任意 Lean コードは受け付けず、型付き IR と閉じた証明レシピから生成する。
`Auto` は bounded `simp_all`、他のレシピは監査対象の既知の補題に限定する。`Incomplete` は負例検査用。
`refutation: true` は、前提と量化を含めた元の命題全体の否定を検査する。

Rust API:

- `ir::Context::{infer,check}`: 型検査と保存可能な検査 Evidence。
- `ir::normalized`: 厳密有理数の既約化を含む決定論的な表現。
- `dsn::{plan,plan_model}`: 有界の構造分解、閉じた定義展開、Knowledge候補と依存グラフ。
- `lean::{translate,audit_translation,run}`: 変換、再生成照合、隔離された Kernel 検査。
- `counterexample::{search_finite,check_candidate}`: 有限探索と厳密な候補評価。
- `evidence::VerificationEvidence`: 正式証跡、SHA-256、不変保存、読込時の整合性検査。
- `evidence::ModelRegistry`: revision履歴、Knowledge occurrence snapshot、推移的依存の失効、モデル評価。

DSN候補、正式証明状態、研究Gateは独立して保持する。Rust の候補評価だけでは `REFUTED` にしない。
条件のないモデル `PASS` に未証明・外部指定・偽の前提を使用しない。構築から得た前提は、同一命題の有効な Evidence が必要。

## 独立監査

```sh
python3 verification/scripts/audit.py verification/evidence/run-20261102T184232+0900 \
  --baseline verification/evidence/baseline-20261102 \
  --negative verification/evidence/negative-20261102-03 \
  --semantics verification/evidence/semantics-20261102-03 \
  --rust-log verification/evidence/rust-final-v3.log \
  --output /tmp/new-audit-summary.json
```

この監査は原結果から判定を再集計し、IR/revision、モデル内容 hash、全 artifact hash、目標固有の公理、9件×3回の完全性、独立命題契約、既存ソースの不変性と負例の原結果を照合する。
run内の `phase_results.json` は当該runのみの暫定評価であり、別実行の負例・変換・回帰を合わせた最終評価は `reports/audit-summary-final.json` にある。
