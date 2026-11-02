# インストールと環境

正式対象は macOS、Rust 1.93.1、ReasonScript 0.5.6.5、Lean 4.35.0、Mathlib `56a005855ea17e0b0958f73ec801ae140b0a1679`。
今回のローカル実測は ReasonScript 0.5.6.10。互換試験は通るが、指定版の受入は未完了。
Linux/Windows は正式受入対象外。

```sh
cargo install --path cli --locked --offline
mathlang --version
mathlang doctor --format json
mathlang --help
```

通常の Cargo bin ディレクトリを PATH に追加する。今回の検証は既存のインストールを変更せず次を使用した。

```sh
cargo install --path cli --locked --offline --root cli/target/local-install
./cli/target/local-install/bin/mathlang --version
./cli/target/local-install/bin/mathlang doctor --format json
```

バイナリだけでは Lean/Mathlib/ReasonScript は導入されない。リポジトリはビルド時と同じ場所に維持する。
既存APIはコンパイル時のソースパスから ReasonScript モジュール、Lean foundation、runner、cache を読む。
リポジトリを移動した場合は再インストールする。単体バイナリの配布には対応しない。

Lean の固定配置は `~/.elan/toolchains/leanprover--lean4---v4.35.0/bin/lean`。
既に用意した固定revisionのLake package cacheを既存手順で配置する。

```sh
python3 verification/scripts/prepare_lean.py /absolute/path/to/project/.lake/packages
```

この準備はdoctorとは独立した明示操作。doctor はネット接続、インストール、設定変更をしない。
macOS の `sandbox-exec` と `/bin/ps` によるRSS監視が必要。外側のsandboxがこれらを拒否した場合はERRORとなる。
隔離なしへ切り替えない。Rust/Cargoの固定版を事前に導入し、依存はCargo.lockで固定する。

ReasonScriptの既存root実行は信頼済みビルド記録が必要。`reason build` 後に実行する。
今回開始時は `target/.reason_build_cache` のみ存在し `.reason/current-build` が欠けていた。
その生成キャッシュを `cli/evidence/baseline/preexisting-reason-build-cache.txt` へ退避して再ビルドした。
Runtime・数理ソースの変更は行っていない。
