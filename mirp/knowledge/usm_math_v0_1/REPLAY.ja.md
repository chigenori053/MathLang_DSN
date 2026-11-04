# 配布パックの再確認

このパックはデータと数理ソースで構成し、実行バイナリ・ビルドキャッシュは含まない。

```sh
cd formal
mkdir -p verification
lake exe cache get
lake build
lake env lean -DwarningAsError=true AuditKnowledge.lean
```

すべてのMathlib依存はmanifestで固定される。上記はLean証明の再検査であり、Knowledgeを既定CLIへ自動登録するコマンドではない。

`catalogue/all_with_legacy.json`はKnowledgeSpaceへ登録する累積版。
`catalogue/memoryspace.json`はMemorySpace::load用の保存済みデータ。
`catalogue/stage*.json`は前段階の依存を含めて読み込む。

元の登録・検索の回帰確認用ハーネスは、作業リポジトリのUSM_Math_Knowledge/checks/にある。
MathLang_DSNのソース本体とlegacy.jsonは変更していない。
