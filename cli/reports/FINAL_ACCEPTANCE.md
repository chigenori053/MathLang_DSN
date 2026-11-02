# Final Acceptance — PARTIAL

MLDSN-CLI-001 v0.1、2026-11-02 JST。
必須CLI実装とローカルインストールは完了。原証跡からの独立再集計はPASS。
仕様全体の最終受入はPARTIALとする。

| Gate | 判定 | 根拠 |
|---|---|---|
| P0 | PASS | 既存経路棚卸し、Rust baseline、ReasonScript生成cache修復後自己検査、原ソースhash |
| P1 | PASS | Envelope・引数・設定・doctor・エラー契約、clippy/fmt |
| P2 | PASS | model/proof/evidence、Leanの新旧27結果、hash・義務の完全性 |
| P3 | PASS | solverの答え/plan/SemanticState一致、Knowledgeとrevision失効 |
| P4 | PASS | 条件差検出、順位なし、3回意味一致、9計画hash一致 |
| P5 | PARTIAL | 指定Runtime版と実測版が異なり、実Lean資源枯渇の故障注入も未実施 |

実測成功: Rust既存77件・検証基盤33件・CLI6件、debug CLI40実行・install済みCLI40実行、旧/新各27形式結果（21 PROVED、6 REFUTED）、独立変換11件、Lean負例6件。
数理モデル・アルゴリズム・Lean命題・MIRP固定スキーマ・ReasonScript Runtimeの変更なし。

未完了条件:

1. 仕様指定ReasonScript 0.5.6.5に対し導入環境は0.5.6.10。互換実行は成功したが指定版試験とは区別する。
2. 実Leanの300秒/2GiB上限到達を故障注入していない。固定runner、通常runの上限内実行、探索予算超過、子プロセスdeadlineは検査済み。

一次根拠: [audit-final.json](../evidence/audit-final.json)、[COMMAND_COVERAGE](COMMAND_COVERAGE.md)、[REGRESSION_REPORT](REGRESSION_REPORT.md)。
再実行: [docs/cli/EXAMPLES.md](../../docs/cli/EXAMPLES.md)。
