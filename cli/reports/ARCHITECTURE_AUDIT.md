# Architecture Audit — MLDSN-CLI-001

Reference: c48dc40bf98e83a78d16e18bda21ddfd2b670061。開始時tracked作業ツリーはclean。
既存数理エンジンを変更せず、独立Rust crate `cli` を追加した。workspace再編は不要。

| 境界 | 既存公開API / CLIの責務 |
|---|---|
| MIRP | native::solve_problem、SemanticState、memory/session/gke/knowledge。CLIは呼出しと表示のみ |
| IR | Context::{infer,check}、ir::normalized。CLIはtyped slotの正規化とモデル構造入力検査 |
| モデル | MathematicalModel、ModelRegistry::{register,snapshot,is_current}。更新はしない |
| 計画 | dsn::plan。既存legacy Knowledge・1000ノード。数学的判断は追加しない |
| 反例 | counterexample::search_finite。候補はUNKNOWN。REFUTEDはLean whole-statement negationのみ |
| 形式証明 | lean::translate、VerificationEvidence::{verify,persist,load}。既存recipe、翻訳、公理監査を使用 |
| 実行 | 既存Python runnerのsandbox・固定toolchain・時間/RSS制限を維持。solverは同一バイナリのworkerで実行し300秒でプロセス群終了 |
| Evidence | 既存形式を直接保存・再読込。CLI configuration/environment/manifestを外側へ追加 |

禁止領域: mirp/src、verification/src/scripts/lean、mathlang_dsn、srcの数理実装、固定スキーマ、ReasonScript Runtime、Lean命題・Knowledge。
これらのbaseline hashを独立auditで照合した。ソース変更なし。

依存プロセス: ReasonScript、python3、固定Lean、git、sandbox-exec、ps、Rust/Cargo。
引数配列のみで起動する。doctorはoffline/read-only。CLIにshell文字列実行はない。
新規依存は既存local cacheのclap/toml/libc（プロセス群終了）をlockした。

P0の実行原証跡は `../evidence/baseline/`。Rust baselineは実装着手前に起動し、既存領域が無変更のまま完了した。
最初のReasonScript root実行には既存の信頼ビルド記録欠損があり、生成cacheを退避して再ビルド後に再検査した。
初回の失敗を成功ログで上書きしていない。これらの前提と補正を含めてP0 PASSとする。
