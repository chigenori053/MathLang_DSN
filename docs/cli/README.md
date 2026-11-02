# MathLang Unified CLI v0.1

MLDSN-CLI-001 の Rust CLI。数学処理・証明・MIRP/ReasonScript は既存 API を呼び出す。
実装仕様は [cli/SPECIFICATION.md](../../cli/SPECIFICATION.md)、実測判定は [FINAL_ACCEPTANCE](../../cli/reports/FINAL_ACCEPTANCE.md)。

```sh
cargo install --path cli --locked --offline
mathlang --help
mathlang doctor --format json
mathlang verify verification/fixtures/MV-01.json --output ./evidence/example --format json
```

`verify` の出力先は新規ディレクトリ。他のコマンドの `--output` は新規JSON応答ファイル。
保存先も入力も上書きしない。text は整形済みEnvelope、json は標準出力に単一オブジェクト。
`--quiet` はtext通常出力を抑制し、JSONは維持する。診断はstderr。

[インストール](INSTALL.md) / [コマンド](COMMAND_REFERENCE.md) / [JSON](JSON_SCHEMA.md) / [終了コード](ERROR_CODES.md) / [例](EXAMPLES.md)
