# MLDSN-MVF-001 v0.1 Final Verification Report

日付: 2026-11-02 JST。
総合判定は `audit-summary-final.json` の原結果再集計を正とする。
対象範囲は `IR_SPECIFICATION.md` に定義した有限次元・型付き・有界のv0.1実装である。
一般数学証明器の完成、全USMモデルの正しさ、Rust変換器自体の形式証明を主張しない。

## 実装成果

独立したRust crateに、型付きIR、型検査、モデル/証明義務registry、有界DSN planner、Lean生成・実行・公理監査、厳密反例候補評価、revision依存失効、不変Evidence保存を実装した。
JSON入力で個別obligationを検証でき、USM代表命題の3回実行と独立変換照合をCLIから実行できる。
既存MIRPの固定schema・ReasonScript Runtime・公開APIへの変更はない。

## Phase Gate

| Phase | 判定対象と原証跡 | 結果 |
|---|---|---|
| P0 | Architecture Audit、capability/dependency/reuse/impact、既存Rust77件とReasonScript baseline、既存source hash一致 | PASS |
| P1 | IR-01〜10: Complex、Vector、Matrix、内積、adjoint、座標tensor、命題、前提、型拒否、決定論性。モデルrevision/hash保存 | PASS |
| P2 | LV-01〜10: typed変換、前提/量化、正命題、偽命題、穴、追加公理、revision、独立契約、Evidence | PASS |
| P3 | DP-01〜10: 構造分解、条件・補題抽出、Knowledge候補/依存、循環、正式結果との照合、3回一致 | PASS |
| P4 | CV-01〜10: 有限/代数候補、前提違反、UNKNOWN分離、Knowledge/model revision、推移的失効、原証跡整合性 | PASS |
| P5 | MV-01/02の期待判定、MV-03〜06の限定範囲・前提、独立Lean、誤PROVED/欠落0、USM比較 | PASS |

当該kernel run単独の `phase_results.json` は、別実行の負例・変換・回帰を含めない暫定結果である。
最終監査はそれらを合わせてGateを判定する。研究GateのPASSをモデルの全命題のPROVEDと混同しない。

## 検証結果

- 新規Rust: 33 passed、0 failed。Clippy（warnings denied）、fmt、diff whitespace検査も成功。
- 既存Rust: 77 passed、0 failed、1 ignored。ReasonScript check/project-validate/5 self-check成功。
- Lean独立変換照合: 11件成功。型、演算、量化、前提と手書きの期待命題をKernelで照合。
- Lean負例: 6件成功。偽命題、未完成証明、sorry、追加公理、命題の弱化、外部プロセス起動を成功扱いしない。
- USM6件＋追加3件: 各3回、計27結果が同一入力・revision・生成コード・planner hash・判定で一致。
- MV-01 PROVED、MV-02 REFUTED。MV-03〜06は `USM_COMPARISON.md` に明記した具体的構築/条件/限定モデルでPROVED。
- 追加: 同一key更新の非可換性PROVED、位相のnorm保存PROVED、normによる複素状態の単射識別REFUTED。
- 正規runの誤PROVED 0、証明義務欠落0。正式証跡・原出力・公理・revision・SHA-256を独立再監査。

RSSと最長時間の正確な値は `audit-summary-final.json` を参照。各ケースは2 GiB・300秒の予算内。
生成Leanは専用一時領域でOS sandbox実行し、外部書込・network・子プロセス起動を制限する。
Lean4.35.0、Mathlib `56a005855ea17e0b0958f73ec801ae140b0a1679` とLake依存revisionを固定した。
許可する公理は `propext`, `Classical.choice`, `Quot.sound`。実際の対象ごとの依存を原ログに保存する。

## 必須試験の対応

| 要求 | 主な検査 |
|---|---|
| IR-01〜10 | `tests/foundation.rs` の同名IRテスト、`rejected_typecheck.json`、11件の独立変換 |
| LV-01〜03 | `lv_01_03_*`、`translation_checks()`、MV独立契約 |
| LV-04〜07 | 正規27結果、negative_bridgeのfalse/incomplete/sorry/axiom、公理判定unit test |
| LV-08〜10 | revision/コード改変拒否、独立期待命題、全registry artifact hash監査 |
| DP-01〜10 | 構造planner、定義展開、必要補題edge、Knowledge候補、循環、別obligation Evidence拒否、Plan正式更新と3回一致 |
| CV-01〜04 | Bool有限探索、厳密有理代数、範囲外/前提違反拒否、候補とREFUTEDの分離 |
| CV-05〜10 | model/Knowledge変更の失効、推移的依存、旧Evidence保持、改変/上書き/欠落拒否、原結果再集計、再現性 |

## 研究上の問い

| 問い | 判定 | 根拠と限界 |
|---|---|---|
| Q1 型付きモデル記述 | EXPERIMENTALLY VERIFIED | 対応する全型・型拒否・正規化をRustとLean照合で確認。型検査器全体の形式証明ではない |
| Q2 意味を保持したLean変換 | CONDITIONALLY VERIFIED | 対応表、再生成、11代表例＋MV6件の独立Kernel型照合。全IR入力に関するcompiler correctness theoremはない |
| Q3 Leanの正当性検査 | FORMALLY VERIFIED | 対象の証明項・否定証明・許可公理をKernel検査。記載した限定命題に適用 |
| Q4 義務/前提/依存生成 | EXPERIMENTALLY VERIFIED | bounded構造規則、Knowledge候補、グラフを検査 |
| Q5 不足条件の検出 | EXPERIMENTALLY VERIFIED | missing premise/lemma、未証明origin、予算/unsupportedを区別。完全な前提推定ではない |
| Q6 反例探索・検証 | FORMALLY VERIFIED / EXPERIMENTALLY VERIFIED | 線形反例と位相識別反例は形式否定証明。一般有限/有理探索は厳密候補評価であり単独REFUTEDにしない |
| Q7 revision追随 | EXPERIMENTALLY VERIFIED | model/Knowledge更新、旧証跡保持、推移的失効と再検証要求 |
| Q8 USMとの整合 | CONDITIONALLY VERIFIED | 移植した代表6件は独立Leanと一致。一般USM retention、全basis codec、全物理観測モデルはNOT VERIFIED |
| Q9 継続拡張の技術的根拠 | EXPERIMENTALLY VERIFIED | 既存経路を変更せずtyped IR→候補→Kernel→Evidenceを実行・再現できた |

## 未達・適用限界

無限次元Hilbert空間、抽象completed tensor積、一般tensor縮約、未指定predicate signature、一般自動証明探索はOUT OF SCOPE。
一般USMモデルのretentionは必要条件が個別に与えられない限りUNKNOWN。MV-05の別key条件を黙って一般化しない。
固定toolchainのビルドcacheとRust/Pythonの実装は信頼境界に残る。代表例の意味照合を変換器全体の形式証明へ読み替えない。
macOSのsandboxを使用する実装であり、別OSは対応sandboxがない場合ERRORとして停止する。
既存の100k性能試験、開始時から削除済みのPython APIは未実行。既存変更を今回の実装成果として数えていない。

## 原証跡と探索中の失敗

正規run、負例、独立変換、回帰、Rust検査は別に保存し、最終監査で統合する。
探索中のRSS超過、sandbox起動拒否、import/共有モジュール/linter失敗を含む旧runは削除していない。
それらを最終PASSの根拠に含めない。最後の正規runのsource manifestと実行binary hashが最終版を識別する。

成果物案内: `ARCHITECTURE_AUDIT.md`, `IR_SPECIFICATION.md`, `REGISTRY_CONTRACTS.md`, `USM_COMPARISON.md`, `REGRESSION_TEST.md`、Rust実装、Lean bridge、原証跡と最終監査JSON。
