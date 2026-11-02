# USM-QSMV-001 Comparison

参照元はローカルの `MRA_Test/USM_QSMV`。その既存EvidenceやPASSは引き継いでいない。
元の `Foundation/StateSpace.lean`, `SemanticSpace/Encoding.lean`, `Knowledge/Update.lean`, `Integration/UnifiedComputation.lean`, `DensityState.lean` を読み、必要な性質を新しいIRと新しいLeanモジュールへ記述した。

| ID | 今回の対象 / 参照 | 正式判定 | 適用範囲と前提 |
|---|---|---|---|
| MV-01 | unitary norm / `p0_03` | PROVED | 全Nat寸法・全複素連続線形自己作用素・全状態。`U†U=1` を保持 |
| MV-02 | 任意線形作用素のnorm保存 | REFUTED | 元の全称命題の否定。n=1、A=2I、単位basis状態による厳密な反例 |
| MV-03 | faithful codec / basis codecを参考 | PROVED | 具体的Bool identity encoder/decoder。USMの全basis codecを証明したとはしない |
| MV-04 | encoding collision / `p2_06`, `p3_08` | PROVED | 任意の型A,B、E,D,x,y。衝突・x≠yなら両方の完全復元は不可能という条件付き不可能性定理 |
| MV-05 | retention / `p3_03` | PROVED | Nat→Nat registryで別key `j≠i` の観測を保持。一般USM Update/DeltaKの保持はUNKNOWN |
| MV-06 | completeness / `p1_03`, `p4_05` | PROVED | knownをpositive∧validとする限定命題モデル。positive欠落ならknownでない。実運用の全Evidenceポリシーの証明ではない |

追加: 同一keyへの更新順序が非可換（PROVED）、unit modulus位相のnorm観測保存（PROVED）、norm観測が複素状態を単射識別するという主張は1と−1でREFUTED。
位相の結果は明示したnorm観測についてのもので、任意の観測器や物理測定モデルの一般定理ではない。
Evidence不足の誤確定はMV-06とregistryの不完全証跡拒否で検査する。

各MVの生成命題を `Independent.lean` の別記述の型とKernelで照合し、同ファイルの単独証明も同じ固定toolchainで再検査する。
旧USMの全命題を移植・再検証したという主張はしない。P5で要求する最初の2件と、MV-03〜06の条件・限定構築・未検証部分を明示する。
