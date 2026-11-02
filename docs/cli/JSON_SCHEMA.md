# JSON出力契約 mathlang.cli/0.1

機械可読の [envelope.schema.json](envelope.schema.json) を同梱する。
通常JSON出力はstdoutに1個のオブジェクトのみ。進捗や診断はstderr、Lean生ログはEvidenceへ保存。
help/versionは通常のテキスト表示。

```json
{"schema_version":"mathlang.cli/0.1","command":"verify","execution_status":"COMPLETED","formal_status":"UNKNOWN","run_id":"run-...","model_id":"example","model_revision":1,"result":{},"evidence":{"reference":"./evidence/run"},"error":null}
```

execution_status: COMPLETED / INTERRUPTED / ERROR。
formal_status: PROVED / REFUTED / UNKNOWN / OUT_OF_SCOPE / ERROR / null。
非証明コマンドと集合検証の集約値はnull。集合の個別証明状態は result.proof_results に保持する。
Research Gateやモデル全体の成立性をformal_statusに混入しない。
resultはコマンド固有のオブジェクト。既存スキーマのデータは変更せず内包する。

errorはnullまたはerror_code/category/message/command/recoverable/evidence_reference。
FS異常等でEvidenceを作成できなければreferenceはnull。
run_id・Evidence ID・timestamp・実行パス・資源計測値は実行ごとに変動する。
再現性は義務、元命題、前提、revision、翻訳hash、依存、計画の意味を比較する。
