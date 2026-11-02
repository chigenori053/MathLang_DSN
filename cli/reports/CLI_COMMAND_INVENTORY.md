# Existing Command Inventory

| 系統 | 既存経路（全dispatcher分岐） |
|---|---|
| MIRP | solve-problem、gke-infer、memory-store、memory-restore、`<state> <namespace> <scope> <nl|ja|math|code> <input>` |
| Verification | verify-obligation、verify-usm、check-translations |
| ReasonScript プロジェクト経路 | check、project-validate、run、build。全て維持 |
| ReasonScript インストール全体 | 既存 `reason --help` の全コマンドを `../evidence/baseline/reason-help.log` に保存。これは外部Runtimeの機能で統一CLIへの再実装対象外 |
| 検証Python | prepare_lean.py（明示cache準備）、regression.py（原baseline）、negative_bridge.py（Lean負例）、audit.py（既存原証跡再集計）、lean_runner.py（固定隔離実行） |

ReasonScriptのhelpにはcluster、RUO/Tensor、vision等も含まれる。棚卸し対象として原一覧を保存し、今回の必須CLI範囲との混同を避ける。
既存実行ファイルとdispatcherは変更・削除していない。
MIRPの固定契約は `mirp/0.1-si2`、既存ProofObligation/VerificationEvidence/ModelRegistryとRU/RUS/RUOはそのまま。
