# Command / Test Coverage

CLI必須8系統は全て実装。debug/installed各40実行の原証跡はacceptance-final/acceptance-installed。
Rust契約試験は5件の試験関数に複数ケースを集約し、process deadlineを別の1件で検査する。

| ID | 結果 | 原根拠 |
|---|---|---|
| CLI-01 | PASS | help/version、生stdout、install後にも実行 |
| CLI-02 | PASS | bad-args、bad-json、Rust未知field拒否 |
| CLI-03 | PASS | text、quiet-json、verboseのstderr分離 |
| CLI-04 | PASS | 0/2/3/4/5/6/7/8実測。9は内部障害用予約で故意のpanicを使わない |
| CLI-05 | PASS | config、config-override、bad-config、Rust設定負例 |
| CLI-06 | PASS | doctorが指定版差を検出しPARTIAL |
| CLI-07 | PASS | model-check、bad-model、unsupported |
| CLI-08 | PASS | normalize、厳密有理数2/4→1/2、入力hash不変 |
| CLI-09 | PASS | formal-finalの9件×3回 |
| CLI-10 | PASS | refutedの単体終了0、集合6 REFUTED |
| CLI-11 | PASS | false/incompleteでUNKNOWN終了0 |
| CLI-12 | PASS | finite候補をUNKNOWN、元の義務保持 |
| CLI-13 | PASS | proof-plan、plan-regression9hash一致 |
| CLI-14 | PASS | knowledge inspect/impact、既存Knowledge依存契約 |
| CLI-15 | PASS | evidence show/validate、全formal artifact hash audit |
| CLI-16 | PASS | stale-registry、Rust契約でKnowledge occurrence更新後も終了7・impact失効表示 |
| CLI-17 | PASS | compare-same/different、未指定環境は条件付き、順位なし |
| CLI-18 | PASS | solver答え[2,3]、Runtime状態保持 |
| CLI-19 | PASS | 新旧27結果、9計画、solver全plan/状態一致 |
| CLI-20 | PASS | formal9件×3回、normalize3回 |
| CLI-21 | PASS | input hash不変 |
| CLI-22 | PASS | collision終了8、既存Evidenceの変更なし |
| CLI-23 | PASS | missing-input、symlink、parent traversal、Rust出力先書込権限拒否 |
| CLI-24 | PASS | missing-runner、実RSS監視拒否、既存Lean負例・公理拒否。実Lean crashの注入は未実施 |
| CLI-25 | PARTIAL | finite budget終了6、実child timeout、固定Lean上限の維持。実Lean300秒/2GiB枯渇は未実施 |
| CLI-26 | PASS | locked offline cargo install、install済み40実行 |

スキーマ/数学処理の拡張や指定外Runtimeへの更新をして合格させていない。
CLI-25未完了とRuntime指定版差により最終GateはPARTIAL。
