> v0.1.1訂正: 定理本体を含む依存抽出に修正し、推移的な既知補題の依存も登録しました。198命題・18検索ビュー・数学的仮定・証明結果は同一です。v0.1の依存情報は不完全でした。

# USM 数理Knowledge — 段階別パック v0.1.1

高校数学まで → 大学数学 → 専門領域の順に、検査済みの数理命題を整備したデータパック。

- [整備レポート](reports/knowledge_preparation.ja.md)
- [全Knowledge索引](catalogue/INDEX.ja.md)
- [機械可読な登録台帳](catalogue/knowledge_registry.json)
- [検証・登録集計](verification/final_summary.json)

| 段階 | 数理命題 | IR検索ビュー | ファイル |
|---|---:|---:|---|
| 1: 高校数学までの基礎 | 61 | 3 | `catalogue/stage1_high_school.json` |
| 2: 大学数学 | 76 | 14 | `catalogue/stage2_university.json` |
| 3: 専門領域・USM | 61 | 1 | `catalogue/stage3_specialist.json` |
| 合計 | 198 | 18 | 216件の新規KnowledgeUnit |

既存29件を含む全245件は `catalogue/all_with_legacy.json`、登録・保存・再読込済みのKnowledgeSpaceは `catalogue/memoryspace.json`。18ビューは既存命題の検索用表現であり、独立の数学知識として二重計上しない。旧証明3件は別証明のaliasとして保持した。

対象はUSM検証向けの基礎初版。高校・大学数学の全課程の網羅や、USM全71項目の自動解決を宣言するものではない。

## 検証

Lean 4.35.0 / Mathlib `56a005855ea17e0b0958f73ec801ae140b0a1679`。
配布パック内の `formal/` に証明ソースと固定済み依存manifestを収録しています。
Leanビルドと公理監査の手順は [配布パックの再確認](REPLAY.ja.md) を参照してください。

登録・保存・planner・GKE APIの回帰確認用コードは、元のMRA_Testリポジトリの
`USM_Math_Knowledge/checks/` にあります。配布パックには含まれません。

## 利用範囲

MathLang_DSNの `KnowledgeSpace::register_batch`、`MemorySpace::load`、`gke-infer` で明示的に読み込む。既定CLIの `proof plan` が使用するlegacyカタログは自動的には切り替わらない。

GKEの条件付き規則は3件。入力事実と型・解釈が正しいという前提の適用であり、GKEのDERIVEDをLeanのPROVEDと同一視しない。全ての登録命題が汎用GKE規則になっているわけではない。

ステージ別JSONは、その前の段階の依存を含めて登録する。単独で利用する場合は累積版 `all_with_legacy.json` を使う。
