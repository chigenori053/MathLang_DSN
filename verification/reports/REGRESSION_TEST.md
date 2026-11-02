# Regression Test Report

既存領域と新規領域を別々に集計した。タスク開始前からの変更は維持し、その作業ツリーをbaselineとする。

| 対象 | コマンド / 原結果 | 結果 |
|---|---|---|
| 既存Rust | `cargo test --manifest-path mirp/Cargo.toml --offline` / baseline `00.*.log` | 77 passed, 0 failed, 1 ignored |
| ReasonScript構文 | `reason check` / `01.*.log` | 12 files validated |
| ReasonScript統合 | `reason project-validate . --json` / `02.*.log` | passed、内部3回のdeterminism一致 |
| SelfCheck | `03.*.log` | runtime_result.result=true |
| PolynomialSelfCheck | `04.*.log` | true |
| KnowledgeSelfCheck | `05.*.log` | true |
| JuniorHighSelfCheck | `06.*.log` | true |
| QuadraticRuntimeSelfCheck | `07.*.log` | true |
| 新規Rust | `rust-final-v3.log` | 33 passed, 0 failed |
| 新規静的検査 | `clippy-final-v3.log`、fmt --check | 成功、warnings denied |
| 既存差分 | `git diff --check` | 成功 |

baselineは最初の全件実行に加え、`scripts/regression.py` で原ログを保存して再実行した。
新規crate実装中も既存ソースは変更しておらず、最終監査でbaselineのファイルhashを現物と照合する。
数学API・MIRP・ReasonScriptの既存公開経路はこれらのテスト範囲で回帰なし。
開始時点で削除済みのPython APIとそのテストは実行対象として存在しないため、今回の成功数へ含めない。
ignoredの100k Knowledge性能試験は実行していない。一般性能・全教材範囲の再認定も今回の範囲に含めない。
