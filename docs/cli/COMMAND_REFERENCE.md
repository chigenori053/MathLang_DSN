# コマンド参照

共通: `--format text|json`、`--output PATH`、`--config PATH`、`--quiet`、`--verbose`、`--help`、`--version`。
`--version` はルートで指定する。引数エラーも `--format json` で共通Envelopeになる。help/versionは通常のCLIテキスト。

| コマンド | 入力・処理 |
|---|---|
| `model check FILE` | 既存MathematicalModel JSON。既存ModelRegistry.registerで構造・型・次元・添字・義務IDを検査 |
| `model inspect FILE` | モデル全内容、ID/revision/content_hashを表示 |
| `model normalize FILE` | 型付き式の既存ir::normalizedを使用。定義、前提、invariant、全義務を正規化 |
| `verify FILE [--knowledge CATALOGUE]` | 既存 `{obligation,recipe,refutation}` またはProofObligation単体。単体はAuto/非否定。Lean証明は閉じたレシピで生成し、指定Knowledge（既定legacy）による検索計画を `knowledge_plans.json`、Knowledgeの出所・revisionを `environment.json` に記録 |
| `verify usm` | 既存usm + auxiliaryの9件を3回。全27結果を保持 |
| `proof plan FILE [--knowledge CATALOGUE]` | 同じ義務入力。既定はlegacy Knowledge。`--knowledge` はKnowledgeUnit配列または単体で、legacyを置き換える（両方必要なら累積版 `all_with_legacy.json`）。既存dsn::plan、上限1000ノード |
| `proof counterexample FILE` | 既存有限探索、予算1000。候補と元の義務を保持。formal_status=UNKNOWN |
| `knowledge inspect FILE` | KnowledgeUnit単体または配列。既存KnowledgeSpaceで依存・循環を検査。各unitの `formal_retrieval` と、結論形式別の件数 `summary.conclusion_formats` を表示 |
| `knowledge impact FILE --revision N` | ModelRegistryの既存履歴revisionを選択し、各モデルの依存と現在失効している義務を表示 |
| `knowledge update FILE` | 明示的OUT_OF_SCOPE、終了4 |
| `compare A B` | モデルの全構造・前提・義務・依存・Evidence参照を比較。順位付けしない |
| `evidence show DIR` | IDを名前とする既存Evidenceディレクトリ。整合性検査後に生情報を表示 |
| `evidence validate DIR` | 同じ読み取り専用検査。任意 `--obligation FILE`、`--registry FILE` |
| `solve TEXT` | 既存MIRP native solver/ReasonScript。結果・計画・SemanticStateを返す |
| `doctor` | 固定版、ReasonScript、Lean、Mathlib全package revision/cache、offline Cargo、sandbox、必要ファイル、設定を検査 |

モデル・義務の入力形式は既存契約を維持する。未知のfieldは終了2、未知の型/演算enumは終了4。
`operations`の文字列は既存モデルのメタデータであり、実行可能な演算は既存の型付きIRが決める。

Evidenceの対象義務は近傍4階層の `proof_obligations.json` から照合する。見つからなければ `--obligation` が必要。
`--registry` がなければ現在revisionの判定はnull。指定したRegistryで失効していれば終了7。
standalone Knowledgeのrevision=1は初回importの意味で、履歴revisionを推測しない。
impactは読み取り専用の現状分析。更新操作や仮想revisionの登録はしない。

Knowledge検索は、`conclusion` が型付きMathExpr IRのJSONであるunitだけを対象とする。目標と結論をそれぞれ `ir::normalized` で正規化し、束縛変数名を束縛の深さで置き換えて構造比較する（α同値は一致、自由変数名・束縛順序・型が違えば不一致）。JSONの空白やキー順の差は影響しない。
legacyの29件は `validated result of ...` という説明文の操作RULEで、数式目標の検索キーにはならない。Lean精緻化命題の記録（`Lean4.ElaboratedProposition.v1`）も検索キーではない。検索一致は候補であり、formal_statusは常にUNKNOWNのまま。PROVEDはLean kernelのEvidenceだけが与える。

比較では実行環境が与えられないため、条件一致でもCONDITIONALLY_COMPARABLE、条件差はNOT_COMPARABLE。
環境比較と数学的同値性はUNKNOWN。COMPARABLEへの自動昇格や証明数の順位付けをしない。
Evidence参照はモデルファイルからの相対パス。hash照合は真正性の独立証明ではない。

verifyは新規出力先のみ。既定は設定の `evidence.directory/<run-id>`。
`configuration.json`、`environment.json`（バイナリhashを含む）、`source_manifest.json`、`commands.json`、義務、結果、kernelログ、registry、artifact manifestを保存する。
失敗時は可能な範囲で `failure.json` と部分Evidenceを保存する。集合検証のformal_statusはnullで、個々の結果を参照する。
単体はPROVED/REFUTED/UNKNOWN/ERROR。正常に完了したREFUTED/UNKNOWNは終了0。

設定は明示CLI > `--config` > カレントの `mathlang.toml` > 既定値。
既存runner固定のRSS=2048 MiB、timeout=300秒、parallelism=1以外の指定は終了4。小さい値も黙って無視しない。
solve外部プロセス群も300秒、doctorの各probeは30秒で停止する。
出力/Evidenceパスのsymlinkおよび `..` を拒否する。macOSの `/tmp`・`/var` 等のsymlinkは `/private/tmp`・`/private/var` 等の実パスを指定する。
