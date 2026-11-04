# 量子力学Knowledge — パック v0.1

USM数理Knowledge v0.1.1（245件）の上に積む、量子力学の知識パック。60件のKnowledgeUnitを `PHYSICS` カテゴリで登録する。

- [整備レポート](reports/knowledge_preparation.ja.md)
- [全Knowledge索引](catalogue/INDEX.ja.md)
- [機械可読カタログ](catalogue/quantum_mechanics.json)
- [生成スクリプト](tools/build_catalogue.py)

| 段階 | 内容 | 件数 |
|---|---|---:|
| 1 | 公理（状態・観測量・Born則・射影・Schrödinger方程式・合成系・密度作用素・同種粒子・正準交換関係）、SI定義定数 | 13 |
| 2 | 状態と密度行列、観測量とスペクトル、スピン・Pauli代数・角運動量 | 21 |
| 3 | ユニタリ時間発展と量子ゲート、不確定性関係、エンタングルメント・Bell状態・CHSH・複製不可能定理 | 23 |
| 4 | 調和振動子、無限井戸型ポテンシャル、水素原子のスペクトル | 3 |

## 検証の区分

| `verification_status` | 件数 | 意味 |
|---|---:|---|
| `EXACT_FINITE_CHECKED` | 20 | 具体的な行列・状態の主張を、`mirp/tests/quantum_knowledge.rs` のガウス有理数（i128）厳密演算で検査した。浮動小数点は使わない。 |
| `ANCHORED_TO_KERNEL_CHECKED` | 9 | 数学的内容がUSMパックのLean検査済み命題そのもの（`formal_anchor` と `dependencies` で参照）。物理的な読み替えは公理による。 |
| `POSTULATE` | 11 | 物理的公理・基本関係。数学的に証明する対象ではない。 |
| `SI_DEFINITION` | 2 | 2019年SI改定の定義値。 |
| `STANDARD_RESULT` | 18 | 教科書的な標準結果。このパックでは未検査で、`mathematical_result` は `UNKNOWN`。 |

このパック自身の命題についてはLean検査を実行していないため、全件 `formal_status` は `UNKNOWN`。`EXACT_FINITE_CHECKED` は有限の具体例の厳密計算であり、一般定理の証明の代用にはしない。

## 利用

```sh
cargo test --manifest-path mirp/Cargo.toml --test quantum_knowledge
```

依存の多くがUSMパックの命題を指すため、単独では登録できない。`usm_math_v0_1_1/catalogue/all_with_legacy.json` を `KnowledgeSpace::register_batch` で登録した後に `catalogue/quantum_mechanics.json` を登録する（合計305件）。`gke-infer` で使う場合も両方を1つのJSON配列に結合して渡す。既定CLIの `proof plan` が読むlegacyカタログは切り替わらない。

GKEの条件付き規則は7件（ユニタリ発展のノルム保存、大域位相、エルミート行列の実スペクトル、可換観測量の同時測定、保存量、時間発展演算子のユニタリ性、直交状態の識別）。入力事実の型と解釈が正しいことを前提とした適用であり、GKEの `DERIVED` を証明と同一視しない。

## 再生成

```sh
python3 mirp/knowledge/qm_v0_1/tools/build_catalogue.py
```

カタログ・索引・`package_manifest.json` を決定的に再生成する。マニフェストのhashはテスト `generated_files_match_the_pack_manifest` が照合する。
