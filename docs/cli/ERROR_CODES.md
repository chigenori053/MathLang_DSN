# 終了コード

| Code | Category | 意味 |
|---|---|---|
| 0 | — | 正常完了。REFUTED、正常に未証明のUNKNOWNも含む |
| 2 | INPUT | 引数、JSON/TOML形式、未知field |
| 3 | MODEL | 型、モデル構造、変数、次元、添字 |
| 4 | OUT_OF_SCOPE | 非対応操作・型・設定・solver問題 |
| 5 | ENVIRONMENT | runner/依存ツール/Lean基盤異常 |
| 6 | RESOURCE_LIMIT | 既存runnerの時間/RSS中断、有限探索/計画予算、子プロセスタイムアウト |
| 7 | EVIDENCE | hash、ID、参照、対象義務、revisionの不整合・失効 |
| 8 | FILESYSTEM | 入出力アクセス、保存先衝突、symlink、不正出力パス |
| 9 | INTERNAL | 内部シリアライズ・プロセス読取スレッド異常 |

error_codeはMLCLI-2〜MLCLI-9。数学的な否定とコマンド異常は独立。
リソース中断はexecution_status=INTERRUPTED、検証のformal_status=UNKNOWNを保持する。
環境異常をPROVEDに昇格しない。doctorのPARTIAL/NOT_READYは診断結果であり、診断処理自体が完了すれば終了0。
