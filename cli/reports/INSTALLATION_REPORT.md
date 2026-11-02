# Installation Report

実行済み:

```sh
cargo install --path cli --locked --offline --root cli/target/local-install --force
python3 cli/scripts/acceptance.py cli/evidence/acceptance-installed --binary cli/target/local-install/bin/mathlang
```

releaseビルド/配置成功。原ログinstall-final.stdout.log、install-final.stderr.log。
インストール後のhelp/version/doctorを含む全40実行成功。LeanによるREFUTED/UNKNOWN、Evidence検査、solverの既存結果一致も含む。
配置先は `cli/target/local-install/bin/mathlang`。既存のユーザー全体PATHやReasonScript導入を変更していない。
仕様通りの通常導入は `cargo install --path cli --locked`。Lean等は別途事前準備が必要。
固定ソースパスに依存するため、リポジトリを移動した場合は再installする。
doctorは実測ReasonScript 0.5.6.10を検出してPARTIAL。指定版0.5.6.5での正式受入は未確認。
