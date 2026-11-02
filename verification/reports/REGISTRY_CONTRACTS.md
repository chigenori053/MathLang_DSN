# Model / Proof Obligation / Evidence Registry

`src/proof.rs` のProofObligationはID、モデルrevision、元のgoal、typed context、監査済み前提origin、必要補題、依存snapshot、証明区分、実行方法、正式状態とEvidence参照を持つ。
Construction / Conditional / Impossibility を区別し、前提originは構築Evidence・外部仕様・未証明・偽の4区分。
含意中の前提もIRとして保存し、plannerが抽出した条件をLeanに勝手に追加する処理はない。

`src/dsn.rs` のPlanは、対象obligation hashとKnowledge revision、候補状態、正式状態、子目標、必要補題の依存グラフを持つ。
定義展開・含意導入・全称導入・論理積分解・必要補題/Knowledge候補・不足条件・循環と予算超過を記録する。
閉じた型付き定義の展開は候補subgoalに限定し、元の目標を保存する。formal状態を更新するには、そのPlanと同一のobligationに一致するEvidenceが必要。

`src/evidence.rs` のModelRegistryは、同じIDへの登録をrevisionの単調増加に制限する。
古いrevisionを削除せず、モデル/Knowledgeの依存snapshotが現行と一致しないEvidenceを失効させる。
親モデルの依存も推移的に確認する。Knowledgeのoccurrenceが変わればrevisionが増え、削除も依存不一致となる。
モデル内容hashは `model_hashes.json`、revision別内容は `model_registry.json` に保存する。

VerificationEvidenceの正式な生成経路は、IR再生成照合→隔離Lean→対象固有の公理監査→Evidence封印である。
Source IR hash、生成Lean source hash、Foundation/独立命題source、原stdout/stderr、toolchain、前提、依存、timestamp、artifact hashを含む。
Evidence IDはIDフィールドを空にした全内容のSHA-256。保存は新規のcontent-addressed directoryのみ許可し、読込時にID・対象revision・原artifactを再照合する。
起動失敗・protocol異常でもERRORと原出力を保存する。タイプ検査失敗はValidationEvidenceとして別に保存できる。

有限・代数候補は量化範囲、前提、結論、revision、厳密評価可能な演算を再検査する。
`verify_counterexample` はその候補を再計算し、元の命題全体の否定をLeanに検査させる。
反例候補があっても、否定証明が成功するまではREFUTEDとしない。数値近似候補を厳密な反例へ昇格しない。

不完全なEvidence、未証明・偽・外部指定の前提をモデル全体PASSに使わない。
構築Evidenceを根拠にする場合も、参照先が同一命題の有効な無条件証明であることを確認する。
保存hashは整合性の検出であり、悪意ある保存領域全体の再署名に対する暗号学的な真正性証明ではない。
この基盤の信頼境界は型/変換実装、固定Lean/Mathlibとそのビルドcache、実行supervisor、保存領域である。

数学的に偽の命題を含むテストモデルをモデル全体PASSにしない。
MV-02をREFUTEDにできたことによる研究テストPASSと、そのモデルの全invariant成立とは独立に扱う。
