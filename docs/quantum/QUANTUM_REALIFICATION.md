# 量子状態の実数化関数: 数理モデル検討と評価

対象: `mathlang_dsn/quantum_real.rsn`(`module QuantumReal`)、`src/main.rsn` の
`QuantumRealSelfCheck` / `QuantumRealEvaluation` / `QuantumRealReport`、
独立参照モデル `verification/scripts/quantum_real_reference.py`。

## 1. 問題設定

量子状態 ψ ∈ ℂ^d の演算結果(振幅・内積・作用素の行列要素)は一般に複素数になる。
これを実数へ写す関数 R: ℂ^d → ℝ^k(実数化関数)のうち、物理量として意味を持つものを選ぶ。

物理状態は射線 [ψ] = {cψ | c ∈ ℂ∖{0}} であり、混合状態は密度行列 ρ で表される。
したがって実数化関数には次の要件を課す。

| 記号 | 要件 | 数式 |
| --- | --- | --- |
| REAL | 出力が実数 | Im R(ψ) = 0 |
| RAY_INVARIANT | 大域位相・規格化に依存しない | R(cψ) = R(ψ), c ≠ 0 |
| SEPARATING | 異なる物理状態を区別できる | [ψ] ≠ [φ] ⇒ R(ψ) ≠ R(φ) |
| BOUNDED | 有界で確率・期待値として解釈できる | R(ψ) ∈ [-1, 1]^k |
| ENSEMBLE_INDEPENDENT | 同じ ρ を与えるアンサンブルで結果が一致(ρ について線形) | Σ pᵢ R(ψᵢ) が ρ = Σ pᵢ ψᵢψᵢ†/‖ψᵢ‖² のみに依存 |

## 2. 厳密計算モデル

- 状態は Gaussian 整数振幅ベクトル `amps = [re0, im0, re1, im1, ...]` と正整数分母 `den`。
  行列は同じ交互配置の行優先 d×d 複素行列。d = 2(1 qubit)と d = 4(2 qubit)を扱う。
- 二次形式型の関数は ⟨ψ|M|ψ⟩ / ⟨ψ|ψ⟩ で計算するため、規格化に √ が不要で、
  出力は常に既約有理数 `Rational { numerator, denominator }` として**厳密**に得られる。浮動小数は使わない。
- 射線同値は Cauchy–Schwarz の等号条件 |⟨φ|ψ⟩|² = ‖φ‖²‖ψ‖² により整数演算で厳密判定する。
- 入力は振幅の絶対値 10000 以下、次元 2 または 4、零ベクトル以外に制限し、範囲外は `UNSUPPORTED`。

## 3. 候補関数

| ID | 名前 | 定義 | Knowledge ID |
| --- | --- | --- | --- |
| 1 | REAL_PART | (Re ψₖ)ₖ | QM_REAL_PART_PROJECTION |
| 2 | REALIFICATION | (Re ψₖ, Im ψₖ)ₖ ∈ ℝ^{2d} | QM_REALIFICATION_ISOMORPHISM |
| 3 | BORN_COMPUTATIONAL | \|ψₖ\|² / ⟨ψ\|ψ⟩ | QM_BORN_RULE |
| 4 | RAW_EXPECTATION | ⟨ψ\|A\|ψ⟩ / ⟨ψ\|ψ⟩, A = σ₊ = \|0⟩⟨1\|(非エルミート) | QM_EXPECTATION_VALUE |
| 5 | HERMITIAN_EXPECTATION | ⟨ψ\|(A + A†)/2\|ψ⟩ / ⟨ψ\|ψ⟩ = Re⟨ψ\|A\|ψ⟩ / ⟨ψ\|ψ⟩ | QM_HERMITIAN_PART_EXPECTATION |
| 6 | LOCAL_BLOCH | 部分系ごとの Pauli 期待値のみ(2 qubit では P⊗I, I⊗P の 6 成分) | QM_REDUCED_BLOCH_VECTOR |
| 7 | PAULI_TOMOGRAPHY | 単位行列以外の全 Pauli 文字列 P の期待値 r_P = ⟨ψ\|P\|ψ⟩ / ⟨ψ\|ψ⟩(d=2 で 3 成分、d=4 で 15 成分) | QM_PAULI_TOMOGRAPHY |

d = 4 の A は σ₊ ⊗ I。

## 4. 評価方法

`QEvaluateCandidate` が各候補について次を厳密に検査する(`QEvaluate` が 7 候補を集計)。

- 試験状態(各 8 件)
  - 1 qubit: \|0⟩, \|1⟩, (1,1), (1,−1), (1,i), (1,−i), (1,2), (2+i, 1−i)
  - 2 qubit: \|00⟩, \|01⟩, Bell 状態 Φ⁺, Φ⁻, Ψ⁺, Ψ⁻, (1,1,1,1), (1,i,0,0)
- 射線不変性: c ∈ {i, −1, 2, 1+2i, 3+4i} を掛けて出力が一致するか。
- 分離性: 試験状態の全組(各 28 組)のうち射線非同値な組で出力が異なるか。
- アンサンブル独立性: いずれも ρ = I/2 を与える 3 つの等確率アンサンブル
  {\|0⟩, \|1⟩}, {(3,4)/5, (4,−3)/5}, {(3,4i)/5, (4i,3)/5} の平均出力が一致するか。
- 判定: 5 項目すべて満たせば `RECOMMENDED`、REAL と RAY_INVARIANT を満たせば `PARTIAL`、それ以外は `REJECTED`。

加えて `QTomographyCheck` が PAULI_TOMOGRAPHY の情報完全性を構成的に検査する。
r_P = n_P / N(N = ⟨ψ|ψ⟩)とおくと、純粋状態では

- 純度: Σ_P n_P² = (d − 1) N²
- 再構成: N δᵢⱼ + Σ_P n_P P_ij = d ψᵢ ψ̄ⱼ(すなわち ρ = (I + Σ_P r_P P)/d が ψψ†/N に一致)

をすべての成分について整数等式で確認する。

## 5. 評価結果

DSN の RSN モジュールと独立参照モデル(Python `fractions.Fraction`)で一致した結果:

| 候補 | REAL | RAY | SEP | BOUND | ENS | 点 | 判定 |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | --- |
| REAL_PART | ✓ | ✗ | ✗ | ✗ | ✗ | 1 | REJECTED |
| REALIFICATION | ✓ | ✗ | ✓ | ✗ | ✗ | 2 | REJECTED |
| BORN_COMPUTATIONAL | ✓ | ✓ | ✗ | ✓ | ✓ | 4 | PARTIAL |
| RAW_EXPECTATION | ✗ | ✓ | ✗ | ✓ | ✓ | 3 | REJECTED |
| HERMITIAN_EXPECTATION | ✓ | ✓ | ✗ | ✓ | ✓ | 4 | PARTIAL |
| LOCAL_BLOCH | ✓ | ✓ | ✗ | ✓ | ✓ | 4 | PARTIAL |
| **PAULI_TOMOGRAPHY** | ✓ | ✓ | ✓ | ✓ | ✓ | **5** | **RECOMMENDED** |

トモグラフィ検査: 1 qubit 8 状態・2 qubit 8 状態とも純度・再構成が成立。

### 反例と解釈

- **REAL_PART / REALIFICATION**: ψ → iψ で出力が変わる(大域位相は観測不能なのに結果が変わる)。
  REALIFICATION は ℂ^d ≅ ℝ^{2d} の等長同型で情報は失わないが、物理状態ではなく状態ベクトルの表現に依存する。
  アンサンブル平均も ρ だけで決まらない({\|0⟩,\|1⟩} と {(3,4)/5,(4,−3)/5} で異なる)。
- **BORN_COMPUTATIONAL**: 物理的に正当(確率、和は 1、ρ について線形)だが相対位相を失う。
  (1,1) と (1,−1)、さらに Bell 状態 Φ⁺ と Φ⁻ がともに (1/2, 0, 0, 1/2) となり区別できない。
- **RAW_EXPECTATION**: ψ = (1, i) に対し ⟨ψ|σ₊|ψ⟩ = i。非エルミート作用素の期待値は実数にならない。
- **HERMITIAN_EXPECTATION**: エルミート部分 (A + A†)/2 を取れば常に実数になり、物理量として正当。
  ただし 1 つのスカラーなので状態を特定できない(\|0⟩ と \|1⟩ がともに 0)。
- **LOCAL_BLOCH**: 1 qubit では完全だが、2 qubit ではエンタングルメントを失う。
  Bell 状態はすべて局所 Bloch ベクトル 0 となり区別できない。
- **PAULI_TOMOGRAPHY**: Φ⁺ と Φ⁻ は相関 ⟨XX⟩ = +1, −1 で区別される。純度・再構成が厳密に成立し、
  出力から ρ を一意に復元できる。

### 結論

量子状態の演算結果を実数化する関数としては、**エルミート作用素の期待値の組
R(ψ) = (Tr(ρ P))_{P ≠ I}, ρ = ψψ†/⟨ψ|ψ⟩(一般化 Bloch 表現 / Pauli トモグラフィ)** を推奨する。

- 実数性はエルミート性から、射線不変性は ρ への依存のみから、アンサンブル独立性は Tr(ρ·) の線形性から従う。
- Pauli 文字列はエルミート行列空間の直交基底(Tr(PQ) = d δ_PQ)なので、出力から ρ が一意に復元でき分離性を持つ。
- 単一の実数値が必要な用途では、HERMITIAN_EXPECTATION(非エルミート演算子は (A + A†)/2 で実数化)と
  BORN_COMPUTATIONAL(測定確率)を、目的の観測量に合わせて PAULI_TOMOGRAPHY の線形汎関数として使う。

## 6. 実行

```sh
reason check
reason run --entry QuantumRealSelfCheck --json --trace=off
reason run --entry QuantumRealEvaluation --json --trace=off
reason run --entry QuantumRealReport --json --trace=off
python3 verification/scripts/quantum_real_reference.py
```

`QuantumRealSelfCheck` と `QuantumRealEvaluation` は `runtime_result.result` が `true` のとき合格。
`QuantumRealReport` は評価表全体を JSON で返す。公開関数 `QRealify`、`QBornProbabilities`、
`QBlochVector`、`QExpectation`、`QEvaluateCandidate`、`QTomographyCheck`、`QEvaluate` は型付き引数を持つため
`reason run --entry <name> --inputs request.json` でも呼び出せる。

### 実行記録(2026-11-03)

ReasonScript 0.5.6.9(Linux、ソースからインストール)で実行した。証跡は
[`verification/evidence/quantum-real-20261103/`](../../verification/evidence/quantum-real-20261103/)。

| 手順 | 結果 |
| --- | --- |
| `reason check` | 13 files passed |
| `reason project-validate . --json` | 全フェーズ passed、決定性 3 回一致 |
| `reason run --entry QuantumRealSelfCheck` | `true` |
| `reason run --entry QuantumRealEvaluation` | `true` |
| `reason run --entry QuantumRealReport` | 第 5 節の表と参照モデルが 7 候補すべてで一致 |
| 既存の calculation(SelfCheck ほか 7 件) | すべて `true` |
| `reason run --entry QRealify --inputs`(Φ⁻, 候補 7) | ⟨XX⟩ = −1 |
| `cargo test --manifest-path mirp/Cargo.toml` | 全件 ok |

プロジェクトが指定する ReasonScript の版は 0.5.6.5 で、今回使ったのは 0.5.6.9。指定版での受入はまだ行っていない。

## 7. Lean 形式化(未検査)

`verification/quantum_real_lean/` に、Lean 4.35.0 + Mathlib `56a0058…` 用の証明を置いた。
**まだ Lean で検査していない。** 検査環境から `release.lean-lang.org` と Mathlib キャッシュの配布ホストに
接続できなかったため。

| 要件 | 定理 | 範囲 |
| --- | --- | --- |
| REAL | `expect_im_eq_zero`、`hermitianPart_expect` | 任意の有限次元 |
| RAY_INVARIANT | `realify_smul` | 任意の有限次元、c ≠ 0 |
| ENSEMBLE_INDEPENDENT | `ensemble_independent` | 任意の有限次元・任意のアンサンブル |
| SEPARATING | `qubit_reconstruction`、`qubit_separating` | 1 qubit の全状態 |
| BOUNDED | `realify_bloch_unit`、`realify_sq_le_one` | 1 qubit の全状態 |
| 反例 | `raw_expectation_not_real`、`real_part_not_ray_invariant`、`born_not_separating`、`bell_local_bloch_zero`、`bell_born_equal`、`bell_xx_separates`、`bell_states_differ` | 具体的な状態 |

検査手順:

```sh
cd verification/quantum_real_lean
lake exe cache get
lake build
lake env lean PrintAxioms.lean
```

## 8. 範囲と限界

- 有界な試験集合での厳密検査であり、一般の d や全状態についての形式証明ではない。
  第 5 節の一般論は第 7 節で Lean 化したが、まだ検査していない。2 qubit の一般状態の再構成は形式化していない。
- 対象は 1・2 qubit の純粋状態入力と、アンサンブル独立性検査用の 1 qubit 混合状態に限る。
- 振幅は Gaussian 有理数に限る(√2 を含む規格化状態は非規格化の代表元で与える)。
- Knowledge ID は報告用の識別子であり、`Knowledge::KCatalog` には未登録(既存カタログ件数の契約を変えないため)。
