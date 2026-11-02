# Regression Report

独立再集計: [audit-final.json](../evidence/audit-final.json)。単なる過去PASSの流用はしていない。

| 比較 | 原証跡・結果 |
|---|---|
| 旧CLI vs 最終CLI | formal-legacy / formal-final、同じ9義務×3回 |
| Formal Status | 各21 PROVED、6 REFUTED、誤PROVED・義務欠落なし |
| 義務ID・Goal・Assumptions・revision | 全件一致 |
| IR source hash・Lean source hash・翻訳契約・recipe・依存 | 全27結果一致 |
| Plan | 旧CLIの未注釈plan hashと新CLI計画のhashを9件照合、一致。plan-regressionに生出力 |
| 3回再現 | 上記意味フィールドが各9件で3回一致 |
| solver | answer=[2,3]、status、全plan、SemanticStateが新旧一致 |
| 既存ソース | baselineおよび旧formal source_manifestから現物をhash照合、一致 |
| 入力不変 | acceptance-final/installedで入力hash維持 |

Evidence ID、timestamp、elapsed、RSS、sandbox用一時パスは実行固有で、単純なバイト一致は要求しない。
各EvidenceのID・artifact hash・raw logはそれぞれ独立に検査する。正式hash一致を真正性の証明とは呼ばない。
既存CLIは従来のパスで引き続き動作する。
