# MathLang DSN Mathematical Verification Foundation v0.1

**Formal Development & Verification Specification**

## Document Control

| Item | Definition |
|---|---|
| Specification ID | MLDSN-MVF-001 |
| Version | 0.1 |
| Date | 2026-11-02 |
| Status | PROPOSED / NOT VERIFIED |
| Target Repository | `chigenori053/MathLang_DSN` |
| Implementation Agent | Codex |
| Core Implementation | Rust |
| Formal Verification | Lean 4 + Mathlib |
| Reasoning Engine | DSN |
| Execution Engine | ReasonScript |
| Semantic Protocol | MIRP |
| Reference Test | USM-QSMV-001 |
| Primary Objective | Mathematical Model Verification |

---

# 1. 目的

## 1.1 研究目的

MathLang DSNを、数学的演算・評価システムから、数理モデルの構築、数学的推論、形式証明、反例分析、検証結果管理を支援する基盤へ拡張する。

主な研究目標は次の5項目である。

1. 数理モデルを型付きの数学的中間表現（Mathematical IR）として記述できること。
2. DSNによって数理モデルの前提条件、推論経路、証明義務を構造化できること。
3. Lean 4と連携し、数学的命題の形式証明を実行できること。
4. 数理モデルの不成立条件、反例、未証明条件を明示できること。
5. 数理モデルを変更した場合、その変更が既存証明や依存する推論経路へ及ぼす影響を追跡できること。

本プロジェクトでは、Lean 4の定理証明機構を置き換えることは目的としない。

MathLang DSNは、数理モデルの検証を計画・構造化・管理するシステムとして開発する。

## 1.2 最終的に実現したい処理

入力された数理モデルに対して、以下を実行できることを目指す。

```text
Mathematical Model
        |
        v
Mathematical IR
        |
        v
Model Validation
        |
        v
DSN Structural Reasoning
        |
        v
Proof Obligation Generation
        |
        v
Lean 4 Verification
        |
        +---- PROVED
        |
        +---- REFUTED
        |
        +---- UNKNOWN
        |
        +---- OUT_OF_SCOPE
        |
        v
Evidence Registry
        |
        v
Model Evaluation
```

REFUTEDは反例または否定命題の有効な証明がある場合のみ使用する。

Lean 4の証明探索が失敗しただけではREFUTEDにしてはならない。

## 1.3 本バージョンの到達範囲

v0.1では、以下を必須とする。

- 型付き数理IRの最小実装
- Lean 4への数理命題変換
- Lean 4証明実行と結果検査
- DSNによる限定的な証明義務分解
- 既存Knowledgeとの依存関係管理
- 反例候補の生成・検証
- Evidence Registry
- USMの代表的な数学命題による検証

一般的な自動定理証明システムの完成は目標としない。

---

# 2. 現行システムとの関係

## 2.1 既存構成

現行MathLang DSNは、以下の構成を持つ。

- MathLang数学演算インターフェース
- DSN構造化推論
- RU/RUS/RUO
- MIRP SemanticState
- ProblemIntent
- Knowledge Registry
- ReasonScript Runtime
- Python Host
- Rust MIRP Implementation

これらの既存機能を、利用可能な範囲で再利用する。

ただし、現在の数学インターフェースで一般的な複素行列、Hilbert空間、密度演算子、テンソル関係演算が検証済みであるとは仮定しない。

## 2.2 既存機能の保護

本拡張により、既存の数学計算・ReasonScript・MIRP機能を破壊してはならない。

次の項目を要求する。

- 既存APIの動作維持
- MIRPの既存固定フォーマットを無断変更しない
- ReasonScript Runtimeの不要な変更を禁止
- 既存Knowledge IDとEvidenceの整合性維持
- 既存のテスト結果と新規テスト結果の分離

既存システムへの変更が必要な場合は、変更理由、互換性、代替方法を記録すること。

## 2.3 モジュール独立性

Mathematical Verification機能は独立したRust crateとして実装することを原則とする。

推奨パス：

`verification/`

MIRP本体への変更は、既存の型や識別子では表現できないことが確認された場合に限定する。

---

# 3. 検証アーキテクチャ

## 3.1 Mathematical Verification Core

以下の6コンポーネントを定義する。

### Component A — Mathematical IR

数理対象、演算、命題、制約、前提条件を型付きで保持する。

### Component B — Mathematical Model Registry

モデル定義、revision、依存定理、証明義務を管理する。

### Component C — DSN Proof Planning

推論規則を利用して証明義務と候補証明経路を構築する。

### Component D — Lean Verification Adapter

型付きIRをLean 4へ変換し、証明コードを検査する。

### Component E — Counterexample Verification

反例候補を生成し、対象命題の否定事例として成立するか確認する。

### Component F — Verification Evidence Registry

証明・反例・未証明条件を不変の証拠として保存する。

## 3.2 信頼境界

最も重要な設計条件は、DSNによる推論とLeanによる証明を区別することである。

| Component | 信頼上の役割 |
|---|---|
| DSN | 証明候補・証明義務の生成 |
| Mathematical IR | 数理命題の構造的記述 |
| Lean Adapter | IRからLeanへの変換 |
| Lean Kernel | Lean命題と証明項の検査 |
| Counterexample Engine | 反例候補の探索 |
| Evidence Registry | 検査結果と原証跡の保持 |

Lean Kernelが検査するのは、Lean上で形式化された命題である。

MathLang IRとLean定義の意味的一致は別の検証義務として管理する。

Leanで証明が成功しても、IRとの対応が未確認ならモデル全体の検証PASSにはしない。

---

# 4. Mathematical IR Foundation

## 4.1 基本型

次の型を定義する。

```text
MathType
├── Bool
├── Nat
├── Int
├── Rational
├── Real
├── Complex
├── Vector
├── Matrix
├── Tensor
├── HilbertSpace
├── LinearOperator
├── DensityOperator
├── Proposition
└── Predicate
```

RealおよびComplexは、v0.1では主として記号的・公理的な数学対象として扱う。

浮動小数点近似を実数の厳密証明と混同してはならない。

## 4.2 数式ノード

Mathematical IRには、最低限以下の演算ノードを実装する。

```text
MathExpr
├── Constant
├── Variable
├── Add
├── Subtract
├── Multiply
├── Divide
├── Power
├── Conjugate
├── Norm
├── InnerProduct
├── MatrixMultiply
├── Adjoint
├── TensorProduct
├── Trace
├── Equality
├── Inequality
├── LogicalAnd
├── LogicalOr
├── LogicalNot
├── Implies
├── ForAll
└── Exists
```

量子状態の演算に必要な以下の表現も追加する。

- PositiveSemidefinite
- Normalized
- Unitary
- Hermitian
- LinearMap
- StateTransition

## 4.3 型検査要件

以下の不正な数理式を拒否すること。

- 次元の異なるベクトルの加算
- 不適切な行列積
- 定義域が一致しない作用素の合成
- 実数と複素数の無定義な型変換
- 前提なしでの非ゼロ除算
- 不正なテンソル添字
- 定義されていない数学的関数の利用

型検査結果はEvidenceとして保存する。

## 4.4 数学的意味の保存

各MathIRノードについて、以下を定義する。

1. 型
2. 入力
3. 出力
4. 数学的意味
5. 必要な前提条件
6. Leanへの変換規則
7. 対応するMathlib定義

数学的意味が確定していないノードは、検証済みノードとして登録してはならない。

---

# 5. Proof Obligation Model

## 5.1 Proof Obligationの定義

数理モデルの検証要求を、独立した証明義務として管理する。

基本構造：

```text
ProofObligation
├── id
├── model_id
├── model_revision
├── goal
├── assumptions
├── required_lemmas
├── dependencies
├── verification_method
├── proof_status
└── evidence_refs
```

## 5.2 証明区分

以下の3分類を必須とする。

### Construction Proof

具体的な数学的構築物について性質を証明する。

例：

正規化状態から密度演算子を構築し、正値性とtrace=1を証明する。

### Conditional Proof

明示した前提条件の下で命題を証明する。

例：

Faithful Codecであることを前提に復号一致を証明する。

### Impossibility Proof

特定の条件下で対象モデルの成立が不可能であることを証明する。

例：

異なる2つの意味が同一の符号化状態に対応する場合、決定論的な復号器が両方を完全復元することはできない。

## 5.3 前提条件の監査

条件付き証明について、以下を区別する。

- 前提条件が具体的構築から証明済み
- 前提条件が外部仕様として与えられている
- 前提条件が未証明
- 前提条件が成立しない

未証明の前提条件を暗黙に補完することを禁止する。

---

# 6. Lean 4 Verification Bridge

## 6.1 目的

MathLang DSNが定義した数理モデルをLean 4で形式的に検証できるようにする。

## 6.2 必須機能

- MathIRからLean 4構文への変換
- Mathlib定義への対応
- 証明義務ごとのLeanモジュール生成
- Lean実行
- 証明結果の解析
- 依存公理の監査
- 失敗理由の保持
- Evidence Registryとの連携

## 6.3 変換契約

各変換には次を保存する。

```text
TranslationRecord
├── source_ir_hash
├── source_model_revision
├── translation_version
├── lean_module_hash
├── mathlib_revision
├── assumptions
├── semantic_mapping
└── validation_status
```

Lean変換器が数学的な命題を変更してはならない。

具体的には以下を禁止する。

- 必要条件を証明なしに削除
- 全称量化を特定例だけに限定
- より強い前提条件を無断追加
- 証明対象を自明な命題に置換
- 不正な型変換による証明の成立
- 未定義演算を無監査の公理へ変換

## 6.4 Lean証明の成立条件

Proof StatusをPROVEDとするには次が必要である。

1. Leanの対象モジュールが正しく検査される。
2. 証明対象の命題が期待する定義と一致する。
3. 未解決の証明穴が存在しない。
4. 未監査の追加公理を使用していない。
5. 証明依存関係が保存されている。
6. 対象モデルrevisionと証明revisionが一致する。

Leanの既存公理依存は、許可リストと監査記録に従って判定する。

`propext`、`Classical.choice`、`Quot.sound`等の利用は、依存関係を明示して監査する。

## 6.5 Lean実行の安全性

生成されたLeanコードは実行対象コードとして扱う。

- 専用の隔離された実行環境を使用する。
- 証明検査に不要なファイル書込や外部コマンドの実行を制限する。
- MathlibとLeanのrevisionを固定する。
- 実行時間・メモリの上限を設定する。
- 動的に生成した証明コードを信頼済みライブラリへ自動登録しない。

Lean Kernelによる証明項の検査と、生成コード全体の安全性を同一視してはならない。

---

# 7. DSN Mathematical Proof Planning

## 7.1 目的

DSNを利用して、数学的命題を推論可能な構造へ分解する。

## 7.2 基本処理

```text
Mathematical Goal
       |
       v
Goal Analysis
       |
       v
Required Conditions
       |
       v
Knowledge Retrieval
       |
       v
Candidate Reasoning Paths
       |
       v
Proof Obligation Graph
       |
       v
Lean Verification
```

## 7.3 Proof Obligation Graph

以下の情報を保持する。

- Goal
- Assumptions
- Lemmas
- Definitions
- Dependency Edges
- Proof Candidate
- Verification Status
- Source Knowledge
- Evidence

このGraphはMIRPの既存依存構造を参考に設計する。

## 7.4 推論規則

初期バージョンでは、以下の推論規則だけを対象とする。

- 定義展開
- 命題分解
- 論理積分解
- 含意導入
- 全称量化導入
- 必要補題の抽出
- 既存定理との照合
- 前提条件不足の検出

一般的な証明探索は本バージョンでは対象外とする。

## 7.5 証明候補の評価

DSNが生成した候補は、以下で分類する。

```text
CANDIDATE
VERIFIED
REJECTED
BLOCKED
UNKNOWN
```

DSN内部のVERIFIEDは、Lean KernelによるPROVEDと同義にしてはならない。

そのため、正式な証明状態はLean検査結果を参照する独立フィールドとして管理する。

## 7.6 UNKNOWN管理

次の状態を区別する。

- 必要な定義が存在しない
- 前提条件が不足している
- 必要な補題が未証明
- Lean変換に対応していない
- 証明探索予算を超過した
- 反例候補が存在する
- 数学的に不成立と証明済み

これらを単一の失敗状態に統合してはならない。

---

# 8. Counterexample Verification Foundation

## 8.1 目的

数理モデルの不成立条件を探索・検証する。

## 8.2 検証方式

最低限、次を実装する。

### Finite Counterexample

有限集合・有限次元・有限構造における具体的反例。

### Algebraic Counterexample

代数的演算による明示的な反例。

### Constraint Violation

必要な前提条件を満たさない入力の検出。

## 8.3 反例の証明要件

反例候補をREFUTEDの根拠として採用するには、次を確認する。

1. 対象が元の命題の量化範囲に含まれている。
2. 命題の前提条件を満たしている。
3. 命題の結論が成立しない。
4. 対象のrevisionが一致している。
5. 反例の評価に必要な演算が検証済みである。

可能な場合はLeanによって反例成立を証明する。

浮動小数点計算による不一致のみの場合は、数値的反例候補として保持し、数学的REFUTEDと区別する。

## 8.4 必須反例

USM-QSMV-001から、以下を移植する。

- 任意線形作用素がノルムを保存するという偽命題
- 符号化衝突による復号不能
- Knowledge更新の非可換性
- グローバル位相だけによる状態識別不能
- Evidence欠落による誤確定

ただし、すべてをv0.1の必須実装Gateに含めるわけではない。

最初の必須Gateは、線形作用素と符号化衝突の2件とする。

---

# 9. Model Registry & Evidence

## 9.1 Model Registry

数学的モデルはrevision付きで登録する。

```text
MathematicalModel
├── model_id
├── revision
├── definitions
├── assumptions
├── operations
├── invariants
├── proof_obligations
├── dependencies
└── content_hash
```

## 9.2 Evidence Registry

証明結果・反例・未検証条件を不変のEvidenceとして保存する。

```text
VerificationEvidence
├── evidence_id
├── model_id
├── model_revision
├── obligation_id
├── source_ir_hash
├── lean_source_hash
├── proof_result
├── counterexample
├── assumptions
├── toolchain
├── timestamp
└── artifact_hashes
```

## 9.3 Revision更新

モデル定義が変更された場合、依存する証明結果を検査する。

変更された定義に依存する証明は、再検証が完了するまで有効な証明として扱わない。

旧Evidenceは削除せず、旧revisionに紐付けて保持する。

## 9.4 Evidence監査

次を必須とする。

- 原証跡の保存
- SHA-256整合性
- 証明対象とのrevision一致
- 依存定理の追跡
- 未監査公理の検出
- 判定と原結果の一致
- 不完全なEvidenceをPASSに使用しない

---

# 10. Phase構成

## Phase 0 — Existing Architecture Audit

### 目的

既存MathLang DSNの構成を確認し、再利用可能な機能と不足機能を特定する。

### 必須検証

- DSN推論構造
- RU/RUS/RUO
- MIRP SemanticState
- ProblemIntent
- Knowledge Registry
- ReasonScript Runtime
- 数学演算API
- Evidence関連機能

### 成果物

- Architecture Audit
- Capability Matrix
- Dependency Map
- Reuse Decision
- Change Impact Report

### Gate P0

PASS条件：

- 対象モジュールの責務を確認
- 既存機能と新規機能の重複を識別
- MIRP互換性への影響を記録
- Mathematical Verification機能の独立実装方針を確定
- 既存テストのbaselineを取得

未確認の機能を実装済みと扱わない。

---

## Phase 1 — Mathematical IR

### 目的

数理モデルを正確に記述できる型付きIRを実装する。

### 必須試験

| Test ID | 内容 |
|---|---|
| IR-01 | 複素数の型定義 |
| IR-02 | ベクトル型 |
| IR-03 | 行列型 |
| IR-04 | 内積演算 |
| IR-05 | 共役転置 |
| IR-06 | テンソル積 |
| IR-07 | 数理命題 |
| IR-08 | 前提条件 |
| IR-09 | 型不一致拒否 |
| IR-10 | 正規化されたIRの決定論的生成 |

### Gate P1

- IR-01〜10すべてPASS
- 型不一致を拒否
- 数理的意味が未定義な演算を黙認しない
- 同一入力から同一正規化IRを生成
- モデルID・revision・hashを保存できる

---

## Phase 2 — Lean Verification Bridge

### 目的

Mathematical IRをLean 4で証明できるようにする。

### 必須試験

| Test ID | 内容 |
|---|---|
| LV-01 | 数式のLean変換 |
| LV-02 | 型の対応 |
| LV-03 | 前提条件の対応 |
| LV-04 | 正命題の証明 |
| LV-05 | 偽命題の拒否 |
| LV-06 | 未完成証明の拒否 |
| LV-07 | 未監査公理の検出 |
| LV-08 | model revision検証 |
| LV-09 | 変換意味の一致監査 |
| LV-10 | Evidence生成 |

### Gate P2

- LV-01〜10すべてPASS
- 有効な証明を正しく検査
- 偽命題をPROVEDにしない
- 型不一致を拒否
- 変換前後の前提条件を監査
- 未解決証明を成功扱いしない

変換意味の一致監査は、対応表だけでなく、代表的な具体例・型・量化・前提条件の独立照合を含む。

---

## Phase 3 — DSN Proof Planning

### 目的

DSNが数学的命題の構造化推論と証明義務生成を行えるか検証する。

### 必須試験

| Test ID | 内容 |
|---|---|
| DP-01 | Goal構造化 |
| DP-02 | 命題分解 |
| DP-03 | 前提条件抽出 |
| DP-04 | 補題依存関係生成 |
| DP-05 | 既存Knowledge検索 |
| DP-06 | 推論経路候補生成 |
| DP-07 | 未証明条件の検出 |
| DP-08 | 循環依存検出 |
| DP-09 | Lean結果による候補更新 |
| DP-10 | 決定論的再実行 |

### Gate P3

- DP-01〜10すべてPASS
- 生成された証明義務を追跡可能
- 既存定理と未証明条件を区別
- 不足条件を無断で補完しない
- DSNの推論結果のみでPROVEDにしない
- 同一入力・同一Knowledge revisionで再現性を確保

---

## Phase 4 — Counterexample & Knowledge Verification

### 目的

不成立条件とモデル変更の影響を検証できるようにする。

### 必須試験

| Test ID | 内容 |
|---|---|
| CV-01 | 有限反例 |
| CV-02 | 代数的反例 |
| CV-03 | 前提不成立の識別 |
| CV-04 | 証明失敗とREFUTEDの区別 |
| CV-05 | Knowledge revision更新 |
| CV-06 | 旧Evidence失効 |
| CV-07 | 依存証明の再検証要求 |
| CV-08 | Evidence整合性 |
| CV-09 | 未解決条件の保持 |
| CV-10 | 再実行一致 |

### Gate P4

- CV-01〜10すべてPASS
- 未証明をREFUTEDと誤判定しない
- 有効な反例が対象命題と一致する
- モデル変更後の旧証明を無条件採用しない
- 証拠の破壊・上書きを行わない

---

## Phase 5 — USM Mathematical Verification Feasibility

### 目的

実際のUSM数理モデルを用いて、MathLang DSNの検証基盤としての有効性を評価する。

### 対象

USM-QSMV-001の数理モデルを参考にする。

ただし、既存の証明結果を新しいMathLang DSNのPASSへ読み替えてはならない。

### 必須試験

**MV-01：Unitary Norm Preservation**

\[
U^\dagger U=I
\Rightarrow
\|U\psi\|=\|\psi\|
\]

期待結果：PROVED

**MV-02：Arbitrary Linear Operator Norm Preservation**

\[
\forall A,\forall\psi,\quad
\|A\psi\|=\|\psi\|
\]

期待結果：REFUTED

反例候補：

\[
A=2I,\quad \|\psi\|=1
\]

**MV-03：Faithful Codec**

\[
D(E(x))=x
\]

具体的符号化器と復号器について成立条件を検証する。

期待結果：CONDITIONALまたはPROVED

**MV-04：Encoding Collision**

\[
E(x)=E(y),\quad x\ne y
\]

この条件で完全復号が不可能となることを検証する。

期待結果：PROVED（条件付き不可能性定理）

**MV-05：Knowledge Retention**

\[
D_k(Update(S,\Delta K))=D_k(S)
\]

保持対象について条件付きRetention定理を検証する。

期待結果：必要な前提条件の抽出と検証。

未証明の場合はUNKNOWNとする。

**MV-06：Evidence Completeness**

Evidenceが欠落した場合に誤ったKNOWN判定を防ぐための条件を検証する。

期待結果：必要条件の構造化と、限定モデルでの形式証明。

### Gate P5

- MV-01とMV-02の期待判定が成立
- MV-03〜MV-06で前提・証明・未証明部分を明示
- Lean単独検証と生成命題の整合性を確認
- 誤ったPROVED判定が0件
- Proof Obligationの欠落を検出
- 反例と未証明を区別
- 既存USM-QSMV-001との比較報告を作成

MV-03〜MV-06のすべてを完全証明することはv0.1の必須条件としない。

本Phaseで検証するのは、MathLang DSNが証明可能性・不足条件・反例を正しく扱えるかである。

---

# 11. 判定仕様

## 11.1 Formal Verification Status

以下を使用する。

```text
PROVED
REFUTED
UNKNOWN
OUT_OF_SCOPE
ERROR
```

各状態の意味：

- PROVED：対象命題の有効な形式証明が成立
- REFUTED：対象命題の有効な反例または否定証明が成立
- UNKNOWN：証明・反証が未成立
- OUT_OF_SCOPE：現在の対応範囲外
- ERROR：検証基盤の実行・変換・監査に異常がある

## 11.2 Research Gate

研究上のPhase Gateは別に管理する。

```text
PASS
PARTIAL
FAIL
NOT_VERIFIED
```

LeanによるPROVEDと研究GateのPASSを同一視してはならない。

数学的に誤りである命題を正しくREFUTEDと判定できた場合、その検証テストはPASSとなり得る。

---

# 12. 実装ディレクトリ

推奨構成：

```text
MathLang_DSN/
├── verification/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ir/
│   │   │   ├── types.rs
│   │   │   ├── expr.rs
│   │   │   ├── model.rs
│   │   │   └── validation.rs
│   │   ├── proof/
│   │   │   ├── obligation.rs
│   │   │   ├── dependency.rs
│   │   │   └── status.rs
│   │   ├── dsn/
│   │   │   ├── planner.rs
│   │   │   └── rules.rs
│   │   ├── lean/
│   │   │   ├── translator.rs
│   │   │   ├── runner.rs
│   │   │   └── audit.rs
│   │   ├── counterexample/
│   │   │   ├── finite.rs
│   │   │   └── algebraic.rs
│   │   └── evidence/
│   │       ├── registry.rs
│   │       └── manifest.rs
│   ├── lean/
│   ├── tests/
│   ├── fixtures/
│   ├── evidence/
│   └── reports/
├── mirp/
├── mathlang_dsn/
└── ...
```

既存リポジトリ構成との不整合がある場合はPhase 0で確認し、変更理由を記録すること。

---

# 13. 実行条件

## 13.1 初期環境

- Rust stable
- Cargo
- Lean 4
- Mathlib
- Lake
- ReasonScript（既存MathLang DSNの対応版）

実行時は各バージョンを固定すること。

既存のMathLang DSNについては、最初に現在の依存関係と実行可能なテストを監査する。

## 13.2 資源予算

v0.1の初期検証予算：

- 最大RSS：2 GiB
- 単一検証ケース：300秒以内
- 証明探索ノード数：設定可能な上限を設ける
- 並列実行数：ホスト資源に応じて制限
- 再現性：固定された入力・revision・実行設定で最低3回照合

予算超過を数学的な不成立と判定してはならない。

タイムアウト時はUNKNOWNまたは実行状態に応じたERRORとして記録する。

## 13.3 回帰テスト

既存の数学計算API、MIRP、ReasonScript連携についてbaselineを取得する。

新規検証機能のPASSで既存機能の回帰FAILを相殺してはならない。

---

# 14. Evidence & Audit Requirements

各実行で以下を保存する。

```text
evidence/
└── run-<timestamp>/
    ├── environment.json
    ├── source_manifest.json
    ├── model_registry.json
    ├── proof_obligations.json
    ├── proof_results.json
    ├── counterexamples.json
    ├── dependency_graph.json
    ├── phase_results.json
    ├── commands.json
    ├── lean_logs/
    ├── rust_logs/
    └── artifact_manifest.json
```

必須監査：

1. ソースとモデルrevisionの一致
2. Lean証明対象の一致
3. 未監査公理の検出
4. DSN推論結果とLean結果の分離
5. 証明義務の欠落検出
6. 反例の成立条件の確認
7. 旧Evidenceの失効処理
8. 原結果からの独立再集計
9. 保存済みログと集計結果の一致
10. 決定論的再実行の一致

---

# 15. 受入基準

本仕様の総合PASS条件は以下とする。

### Mathematical IR

- 型付き数理モデルを構築できる。
- 数学的な型不整合を検出できる。
- 同一モデルの正規化表現が決定論的である。

### Lean Integration

- 数理命題をLean 4へ変換できる。
- 正しい命題の形式証明が成立する。
- 偽命題をPROVEDにしない。
- 数理IRとLeanへの変換対応を監査できる。

### DSN Mathematical Reasoning

- 証明義務を構造化できる。
- 不足する前提条件を識別できる。
- 依存関係を追跡できる。
- 証明候補と正式証明を区別できる。

### Counterexample Verification

- 反例候補を生成できる。
- 反例が成立することを検査できる。
- 未証明と反証済みを区別できる。

### Evidence

- すべての正式判定に原証跡が存在する。
- 旧revisionの証明を誤採用しない。
- 同一条件で検証結果を再現できる。

### Regression

- 既存MathLang DSNの公開機能に意図しない回帰がない。

全必須Phase GateがPASSした場合のみ、MLDSN-MVF-001 v0.1を総合PASSとする。

---

# 16. Codex実装指示

## 16.1 開発順序

以下の順序で実行すること。

1. 現行MathLang DSNの構造監査
2. 数理IRの設計
3. Mathematical IR Foundation実装
4. 型検証
5. Lean 4 Adapter実装
6. Lean 4形式証明の最小試験
7. DSN Proof Planning実装
8. 反例検証機能実装
9. Model/Evidence Registry実装
10. USM代表命題の移植試験
11. 回帰テスト
12. Evidence監査
13. 最終判定

各Phase完了時に、Phase Gateと未達事項を報告すること。

## 16.2 禁止事項

- Leanと同等の証明器を独自に再実装しない。
- 証明不能を数学的な偽と扱わない。
- DSNの推論結果を正式証明と扱わない。
- 未証明の前提条件を自動承認しない。
- 生成Leanコードのビルド成功だけで数理モデルを承認しない。
- 原仕様を満たさない弱い定理へ無断変更しない。
- 未解決の証明を `sorry` 等で通過させない。
- Rustの浮動小数点結果を厳密な数学的証明と扱わない。
- 既存MIRP固定仕様を無断変更しない。
- 既存ReasonScript Runtimeを不必要に複雑化しない。
- 失敗したEvidenceを削除しない。

## 16.3 最終成果物

以下を提出すること。

1. Architecture Audit Report
2. Mathematical IR Specification
3. Mathematical IR Rust Implementation
4. Lean 4 Verification Bridge
5. DSN Proof Planning Implementation
6. Counterexample Verification Implementation
7. Model Registry
8. Evidence Registry
9. Proof Obligation Registry
10. USM-QSMV-001 Comparison Report
11. Regression Test Report
12. Final Verification Report

---

# 17. 研究成果の判定

本検証の完了時には、以下の問いへ回答すること。

**Q1.** MathLang DSNは型付き数学モデルを正確に記述できるか。

**Q2.** 数理モデルをLean 4へ意味を保持して変換できるか。

**Q3.** Lean 4により数学的命題の正当性を検査できるか。

**Q4.** DSNは証明義務・前提条件・依存関係を生成できるか。

**Q5.** DSNは証明に不足する条件を検出できるか。

**Q6.** 数理モデルの反例を探索・検証できるか。

**Q7.** Knowledgeや数理モデルのrevision更新に追随して証明を再検証できるか。

**Q8.** 既存のUSM数理モデルについて、Lean単独検証と整合する検証結果を生成できるか。

**Q9.** 数理モデル検証基盤としてMathLang DSNを継続拡張する技術的根拠が得られたか。

各問いについて、以下を明確に区別すること。

- FORMALLY VERIFIED
- EXPERIMENTALLY VERIFIED
- CONDITIONALLY VERIFIED
- REFUTED
- NOT VERIFIED
- OUT OF SCOPE

**本仕様の目的はMathLang DSNの汎用数学証明能力を主張することではなく、数学的モデルを安全かつ再現可能に検証するための最小基盤を成立させることである。**

---

**End of Specification**

**MLDSN-MVF-001 v0.1 — PROPOSED / NOT VERIFIED**
