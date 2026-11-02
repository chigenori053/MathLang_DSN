# Baseline

| 原結果 | 結果 |
|---|---|
| baseline/legacy/00.stdout.log | MIRP Rust 77 passed、1 ignored |
| baseline/verification.stdout.log | Verification Rust 33 passed |
| baseline/legacy/01.stdout.log | reason check成功 |
| baseline/legacy/02.stdout.log | project-validate成功、決定論性検査成功 |
| baseline/legacy/03〜07.stdout.log | 初回root実行失敗: .reason/current-build欠損 |
| baseline/reason-build.stdout.log | 最初のbuildも既存cacheにより失敗 |
| baseline/preexisting-reason-build-cache.txt | 既存生成cacheを保全・退避 |
| baseline/reason-clean-build.stdout.log | 再build成功、12ファイル |
| baseline/reason-clean-0〜4.stdout.json | 自己検査5件true |

baseline/legacy/summary.jsonのpassed=falseは初回の実測としてそのまま保持する。
補正後の原結果はreason-clean.jsonに分離し、独立auditは初回Rust/check/project-validateと補正後自己検査を個別に確認する。
ReasonScriptは実測0.5.6.10で仕様指定0.5.6.5と異なる。Runtimeを変更していない。
既存ソースはbaseline source_manifestと現物が一致。ignored性能試験を成功数へ算入しない。
