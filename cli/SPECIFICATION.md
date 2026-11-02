# MathLang_DSN Unified CLI Foundation v0.1

**Specification ID:** MLDSN-CLI-001  
**Version:** 0.1  
**Status:** PROPOSED / NOT IMPLEMENTED  
**Date:** 2026-11-02  
**Target Repository:** chigenori053/MathLang_DSN  
**Reference Commit:** c48dc40bf98e83a78d16e18bda21ddfd2b670061  
**Implementation Language:** Rust  
**Implementation Agent:** Codex  
**Primary Interface:** Command-Line Interface  
**Related Specification:** MLDSN-MVF-001 v0.1

---

# 1. 目的

## 1.1 開発目的

MathLang_DSNの既存機能を統一されたCLIから操作できるようにする。

現在、MathLang_DSNは以下の独立した実行経路を持つ。

- ReasonScript CLI
- MIRP Rust CLI
- Mathematical Verification Rust CLI
- 検証用Pythonスクリプト

これらをユーザーが直接使い分ける必要がないよう、単一のコマンド体系を構築する。

正式コマンド名は `mathlang` とする。

## 1.2 最重要要件

本開発では、次の原則を必須とする。

1. 数理モデルおよび数理演算アルゴリズムを変更しない。
2. 既存の形式証明結果を変更しない。
3. MIRPの固定スキーマを変更しない。
4. ReasonScript Runtimeを変更しない。
5. 既存の証明・検証エンジンを再実装しない。
6. GUI、Web UI、TUIを実装しない。
7. CLI自体に新しい数学的判断ロジックを持たせない。
8. 全ての検証結果について再現性と監査可能性を維持する。

CLIは、既存機能の呼び出し、入力検証、結果表示、実行管理を担当する。

## 1.3 非目標

今回の開発では以下を実施しない。

- 新しい数理モデルの実装
- DSN推論アルゴリズムの変更
- Lean証明戦略の拡張
- 汎用自動定理証明機能
- 任意のLeanソースコード実行
- GUI開発
- APIサーバー構築
- クラウド実行基盤
- 推論モデルの学習機能追加
- 既存の数学的対応範囲の拡張

---

# 2. 現行アーキテクチャの確認

## 2.1 既存の操作経路

現行実装から確認された主なコマンドは次のとおり。

### MIRP CLI

- `solve-problem`
- `gke-infer`
- `memory-store`
- `memory-restore`
- 名前空間・スコープを指定するセッション入力

対象実装：

`mirp/src/main.rs`

### Mathematical Verification CLI

- `verify-obligation`
- `verify-usm`
- `check-translations`

対象実装：

`verification/src/main.rs`

### ReasonScript CLI

- `reason check`
- `reason project-validate`
- `reason run`

これらは既存機能として維持する。

## 2.2 既存検証基盤の状態

MLDSN-MVF-001 v0.1の保存済み検証報告は、次の結果を示す。

- Phase P0〜P5：PASS
- 新規Rustテスト：33件成功
- 既存Rustテスト：77件成功
- Lean独立変換照合：11件成功
- Lean負例：6件成功
- 正規形式検証：27結果
- 誤PROVED：0件
- 証明義務欠落：0件

これらの値は変更前の回帰基準とする。

ただし、過去の報告値を新CLIの合否判定としてそのまま流用してはならない。

今回の受入試験で改めて原証跡を生成・比較する。

---

# 3. 新アーキテクチャ

## 3.1 全体構成

```text
User / Coding Agent
        |
        v
  mathlang CLI
        |
        v
  Command Parser
        |
        v
  Application Layer
        |
        +-------------------+
        |                   |
        v                   v
  MIRP Adapter       Verification Adapter
        |                   |
        v                   v
  MIRP Core           Mathematical IR
        |                   |
        v                   v
  ReasonScript         DSN Planner
                            |
                            v
                       Lean 4 Bridge
                            |
                            v
                        Lean Kernel
        |                   |
        +---------+---------+
                  |
                  v
          Evidence Registry
```

## 3.2 ディレクトリ構成案

```text
MathLang_DSN/
│
├── cli/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── model.rs
│   │   │   ├── verify.rs
│   │   │   ├── proof.rs
│   │   │   ├── knowledge.rs
│   │   │   ├── compare.rs
│   │   │   ├── evidence.rs
│   │   │   ├── solve.rs
│   │   │   └── doctor.rs
│   │   ├── application/
│   │   ├── adapters/
│   │   ├── output/
│   │   └── errors.rs
│   └── tests/
│
├── mirp/
├── verification/
├── mathlang_dsn/
├── src/
└── docs/
    └── cli/
```

これは論理的な構成案であり、既存コードと同等機能を重複実装しない。

既存Rustライブラリを直接再利用できる場合は、Adapter層を最小化する。

単一Rust Workspaceへの再編成は必須ではない。既存Cargo構成への影響を調査し、必要最小限の変更を選択する。

---

# 4. CLI共通仕様

## 4.1 コマンド名

正式な実行ファイル名：

`mathlang`

基本構文：

```bash
mathlang <command> <subcommand> [options]
```

## 4.2 共通オプション

| オプション | 定義 |
|---|---|
| `--help` | ヘルプ表示 |
| `--version` | CLIバージョン表示 |
| `--format text` | 人間向け出力 |
| `--format json` | 機械処理用JSON |
| `--output PATH` | 結果ファイルの出力先 |
| `--quiet` | 不要な通常メッセージを抑制 |
| `--verbose` | 実行診断情報を表示 |
| `--config PATH` | 明示した設定ファイルを使用 |

`--format`の既定値は `text` とする。

JSONモードでは、標準出力に1個の完全なJSONオブジェクトのみを出力する。進捗、診断、RustやLeanの生ログは標準エラーまたはEvidenceへ送る。

## 4.3 標準終了コード

| Code | 意味 |
|---|---|
| 0 | コマンドが正常に完了 |
| 2 | CLI引数・入力形式エラー |
| 3 | 型検査・モデル構造エラー |
| 4 | 非対応機能・スコープ外 |
| 5 | 依存ツール・実行環境エラー |
| 6 | リソース制限・タイムアウト |
| 7 | Evidenceの不整合・失効 |
| 8 | ファイルシステム・保存エラー |
| 9 | 内部エラー |

**重要：数学的なREFUTEDやUNKNOWNは、それ自体ではCLIの実行失敗を意味しない。**

有効な検証処理が完了し、数学的結果がREFUTEDであっても終了コード0を返す。

UNKNOWNについても、検証処理が正常に完了して未証明と判定された場合は終了コード0とする。

ただし、リソース制限で検証が中断された場合は、結果にUNKNOWNを記録するとともに終了コード6を返す。

基盤異常はERRORとして記録し、対応する非ゼロ終了コードを返す。

## 4.4 JSON出力契約

全コマンドは共通Envelopeを使用する。

```json
{
  "schema_version": "mathlang.cli/0.1",
  "command": "verify",
  "execution_status": "COMPLETED",
  "formal_status": "PROVED",
  "run_id": "example-run-id",
  "model_id": "example-model",
  "model_revision": 1,
  "result": {},
  "evidence": {
    "reference": "relative/evidence/path"
  },
  "error": null
}
```

このJSONはCLI応答の例示スキーマであり、新規に定義する。

次の区分を混同してはならない。

- Execution Status：コマンド実行状態
- Formal Status：数学的証明状態
- Research Gate：検証仕様の受入判定
- Model Validity：モデル全体の成立性

`formal_status`は非証明系コマンドではnullとする。

## 4.5 エラー契約

エラーは最低限以下を保持する。

- error_code
- category
- message
- command
- recoverable
- evidence_reference（存在する場合）

エラーのJSON契約は、正常時と同じEnvelopeを使用する。

Rust panicや未処理例外を通常のユーザー向けエラー処理として使用しない。

---

# 5. コマンド仕様

## 5.1 `mathlang model`

数理モデルの管理と型検査を行う。

```bash
mathlang model check model.json
mathlang model inspect model.json
mathlang model normalize model.json
```

### 必須機能

**check**

- JSONスキーマ検証
- Mathematical IRの型検査
- 変数・次元・添字の整合性検査
- 未対応型・未対応演算の拒否

**inspect**

- モデルID
- revision
- 内容hash
- 前提条件
- 命題
- 検証対象の一覧表示

**normalize**

- 既存IR正規化関数を利用
- 決定論的なシリアライズ
- 厳密有理数の正規化

正規化によって数理的意味を書き換える処理を追加してはならない。

`model add`などの永続Registry更新操作はv0.1の必須範囲に含めない。既存Registry APIで安全に提供できる場合は、独立した拡張候補として報告する。

---

## 5.2 `mathlang verify`

Lean 4による形式証明を実行する。

```bash
mathlang verify obligation.json \
  --output ./evidence/run-001

mathlang verify usm \
  --output ./evidence/usm-run
```

### 必須要件

- 既存ProofObligation形式を受理
- 既存Recipeを使用
- Lean変換を再利用
- Lean Kernel結果を取得
- 公理監査を実施
- Evidenceを保存
- PROVED / REFUTED / UNKNOWN / ERRORを区別

`verify usm`は既存の代表検証セットを使用する。

CLI側が勝手に前提を追加して証明を成功させることは禁止する。

既存のmacOS sandbox、固定toolchain、リソース制限は維持する。

出力ディレクトリが既に存在する場合は、原則として上書きせずエラーを返す。

---

## 5.3 `mathlang proof`

DSNによる証明義務の解析を行う。

```bash
mathlang proof plan obligation.json

mathlang proof counterexample obligation.json
```

### plan

- 前提条件の抽出
- 証明義務の構造分解
- 必要補題の抽出
- Knowledge候補の列挙
- 依存グラフ生成
- 循環・予算超過の検出

既存 `dsn::plan` を利用する。

### counterexample

- 既存の有限探索を使用
- 厳密な候補評価を実施
- 元の命題と前提を保持
- 反例候補と正式否定証明を区別

反例候補の発見のみでREFUTEDを返してはならない。

`verify`と連携してLeanが元命題全体の否定を確認した場合にのみ、正式なREFUTEDを返す。

---

## 5.4 `mathlang knowledge`

既存Knowledgeの内容と依存関係を調査する。

```bash
mathlang knowledge inspect knowledge.json

mathlang knowledge impact \
  model-registry.json \
  --revision 3
```

### 必須機能

- Knowledge情報表示
- revision表示
- occurrence情報表示
- Knowledge依存関係の分析
- revision更新の影響確認
- 失効済みEvidenceの識別

### 制約

v0.1では、Knowledgeの更新そのものを新しい推論機能として実装しない。

既存Registryの依存失効機構を利用し、まず読み取り・影響分析を提供する。

未実装の更新操作は明示的にOUT_OF_SCOPEとする。

---

## 5.5 `mathlang compare`

数理モデルの比較を実施する。

```bash
mathlang compare \
  model-a.json \
  model-b.json \
  --format json
```

### 比較項目

- モデルID
- revision
- IR構造
- 前提条件
- 命題
- ProofObligation
- Knowledge依存
- 利用可能な正式Evidence

### 比較上の制約

本コマンドは、数理モデルの優劣を自動決定するものではない。

数学的命題、前提条件、量化範囲、実行環境が異なる場合、単純なPROVED件数による優劣判定を禁止する。

比較可能性の判定を以下に分類する。

- COMPARABLE
- CONDITIONALLY_COMPARABLE
- NOT_COMPARABLE
- UNKNOWN

数学的に同値な命題であるかどうかを、文字列やhashの一致だけから推定してはならない。

比較条件が一致しない場合は、その差異を明示する。

---

## 5.6 `mathlang evidence`

既存Evidenceを閲覧・検査する。

```bash
mathlang evidence show <evidence-path>

mathlang evidence validate <evidence-path>
```

### show

- 証明状態
- 対象モデル
- revision
- 使用前提
- Lean情報
- 入力hash
- 証明hash
- 実行結果

### validate

- Evidence ID整合性
- artifact hash整合性
- 対象revision整合性
- 参照先の存在
- 必要な生ログの存在
- 既存Registry契約との整合性

保存済みEvidenceを修正しない。

Evidenceの暗号学的hash一致を、独立した真正性証明と誤認してはならない。

---

## 5.7 `mathlang solve`

既存の数学問題処理を呼び出す。

```bash
mathlang solve \
  "Solve x^2 - 5x + 6 = 0."
```

### 要件

- 既存MIRP solverを再利用
- ReasonScript Runtimeを利用
- 正確な結果と推論計画を取得
- MIRP SemanticStateを保持
- 既存対応範囲を越えない

整数解の二次方程式に対する既存の限定的な契約を保持する。

対応外の数学問題について、推測した答えを正常結果として生成しない。

---

## 5.8 `mathlang doctor`

ローカル環境の診断を実施する。

```bash
mathlang doctor
mathlang doctor --format json
```

### 診断項目

- MathLang CLIバージョン
- Rust toolchain
- ReasonScript CLIの存在とバージョン
- Leanの存在とバージョン
- Mathlib revision
- Cargo依存の解決状態
- macOS sandbox利用可否
- 必須実行ファイルへのアクセス
- 設定ファイルの整合性

### 判定

- READY
- PARTIAL
- NOT_READY

`doctor`は環境を自動変更しない。

依存ライブラリの自動インストール、システム設定変更、外部通信を行わない。

---

# 6. 設定管理仕様

## 6.1 設定ファイル

任意の設定ファイルとして以下を採用する。

`mathlang.toml`

設定例：

```toml
[cli]
output_format = "text"

[runtime]
mode = "local"

[verification]
max_rss_mib = 2048
timeout_seconds = 300
parallelism = 1

[evidence]
directory = "./evidence"
```

ただし、数値は無条件に変更可能としない。

既存の検証基盤が固定する資源上限やtoolchain設定を超える指定は拒否する。

## 6.2 設定の優先順位

1. 明示されたCLIオプション
2. `--config`で指定した設定ファイル
3. 既定値

環境変数による上書きはv0.1の必須機能に含めない。

## 6.3 設定の監査

検証結果に影響する設定値はEvidenceに保存する。

同じモデルを異なる設定で実行した場合、その違いを識別できなければならない。

---

# 7. セキュリティと信頼境界

## 7.1 外部コマンド実行

Lean、ReasonScriptなどを呼び出す場合は次を必須とする。

- Shell文字列連結によるコマンド実行禁止
- 引数を明示的な配列で渡す
- 入力値を実行ファイル名として使用しない
- 任意のLeanコードを直接実行しない
- 標準出力・標準エラーを分離
- 実行環境の固定
- タイムアウトと資源制限
- 子プロセスの終了処理

既存のLeanサンドボックスを迂回してはならない。

## 7.2 ファイル操作

- 入力ファイルの変更禁止
- Evidenceの原則不変保存
- 不明なパスへの暗黙的な書込禁止
- 既存証跡の上書き禁止
- 入出力パスの明示的な区別
- シンボリックリンクなどによる意図しない出力先変更を拒否

## 7.3 検証失敗時の処理

異常終了した場合も、可能な範囲で以下を保存する。

- 入力識別子
- エラー区分
- 実行したコマンド
- 実行環境
- stdout/stderr
- 中断理由
- 部分的なEvidence

未完成の結果を正常な証明Evidenceとして登録してはならない。

---

# 8. 後方互換性

## 8.1 基本方針

既存CLIを削除しない。

新CLIは既存CLIの上位インターフェースとして構築する。

以下の操作は、変更前と同じ方法で引き続き実行できること。

```bash
cargo run --manifest-path mirp/Cargo.toml -- \
  solve-problem "Solve x^2 - 5x + 6 = 0."
```

```bash
verification/target/debug/mathlang-verification \
  verify-obligation request.json new-output-directory
```

## 8.2 スキーマ互換性

維持対象：

- MIRP `mirp/0.1-si2`
- Mathematical IR v0.1
- ProofObligation
- VerificationEvidence
- ModelRegistry
- ReasonScript RU/RUS/RUO
- Lean証明命題の変換規則

変更が不可避な場合は、現行仕様との差分と互換性への影響を報告し、本開発の範囲外として扱う。

## 8.3 保存形式

既存Evidenceの再保存・再署名・再計算による上書きを禁止する。

CLIの表示形式が変更されても、既存の保存済み証跡はそのまま読み込めなければならない。

---

# 9. 開発Phase

## Phase 0 — Architecture Audit

### 目的

既存CLIとRust APIの責務を明らかにする。

### 作業

- 既存コマンド一覧の作成
- MIRP/Verification API一覧の作成
- CLIとCoreの責務分析
- 外部プロセス依存の確認
- 後方互換性の確認
- 既存テストのbaseline実行

### 成果物

- `CLI_ARCHITECTURE_AUDIT.md`
- `CLI_COMMAND_INVENTORY.md`
- `CLI_BASELINE_REPORT.md`

### Gate P0

**PASS条件**

- 全既存CLIコマンドの棚卸し完了
- 既存公開契約の特定完了
- baselineを実行し原証跡を保存
- 変更禁止対象を明示

未確認の既存コマンドがある場合はPASSとしない。

---

## Phase 1 — CLI Foundation

### 目的

統一CLIの共通基盤を構築する。

### 作業

- CLI引数解析
- help/version
- 共通JSON Envelope
- text/JSON表示
- エラーコード
- 設定読み込み
- Application Layer
- `doctor`

### Gate P1

**PASS条件**

- 共通オプションが正常動作
- JSON出力が機械解析可能
- エラーコードの契約が成立
- doctorが依存環境を正しく識別
- 既存コードへの回帰がない

---

## Phase 2 — Mathematical Verification Commands

### 目的

既存数理検証基盤をCLIから実行可能にする。

### 作業

- model check/inspect/normalize
- verify
- proof plan
- proof counterexample
- evidence show/validate

### Gate P2

**PASS条件**

- 既存Mathematical IRを正しく扱う
- Lean証明結果が既存CLIと一致
- 証明状態の誤分類がない
- 証明義務を欠落させない
- Evidenceの内容とhashが一致
- 異常系で正常なPROVEDを生成しない

---

## Phase 3 — MIRP / Knowledge Integration

### 目的

MIRPとKnowledge操作を統合する。

### 作業

- solve
- knowledge inspect
- knowledge impact
- MIRP状態の参照
- revision依存の表示

### Gate P3

**PASS条件**

- 既存MIRP solverとの結果一致
- 既存ReasonScriptの計算結果一致
- Knowledge依存の表示が正しい
- revision失効を誤分類しない
- UNKNOWNを保持する

---

## Phase 4 — Comparison & Reproducibility

### 目的

数理モデル比較と検証結果の再現性を確立する。

### 作業

- model compare
- 前提・命題の差分表示
- 検証条件の比較
- JSON結果の決定論性確認
- Evidence再読込
- 3回繰り返し検証

### Gate P4

**PASS条件**

- 比較条件の不一致を検出
- 不適切な優劣判定をしない
- 同一入力で意味的に同一の結果
- 実行ごとの差分を正しく識別
- Evidenceの整合性を維持

タイムスタンプやrun IDのような実行固有情報を除いた正規化結果が一致することを要求する。

---

## Phase 5 — Final Integration & Acceptance

### 目的

CLI全体の受入試験を行う。

### 作業

- 全コマンドの統合試験
- 旧CLI回帰試験
- 数理モデル検証回帰
- 異常系テスト
- Evidence監査
- CLIドキュメント完成
- ローカルインストール検証

### Gate P5

**PASS条件**

- 全必須コマンドが使用可能
- P0〜P4がPASS
- 既存の形式検証結果との一致
- 後方互換性の維持
- 改変されたEvidenceの拒否
- CLI経由の形式検証成功
- ローカル環境での再実行成功
- インストール・使用方法が文書化されている

---

# 10. テスト仕様

## 10.1 テストID

| ID | テスト内容 | 必須 |
|---|---|---|
| CLI-01 | Help/version | Yes |
| CLI-02 | 不正引数の拒否 | Yes |
| CLI-03 | Text/JSON形式 | Yes |
| CLI-04 | 終了コード契約 | Yes |
| CLI-05 | 設定読み込み | Yes |
| CLI-06 | 環境診断 | Yes |
| CLI-07 | IR型検査 | Yes |
| CLI-08 | IR正規化 | Yes |
| CLI-09 | Lean形式証明 | Yes |
| CLI-10 | REFUTED判定 | Yes |
| CLI-11 | UNKNOWN保持 | Yes |
| CLI-12 | 反例候補の区別 | Yes |
| CLI-13 | DSN証明義務分解 | Yes |
| CLI-14 | Knowledge依存 | Yes |
| CLI-15 | Evidence検査 | Yes |
| CLI-16 | revision失効 | Yes |
| CLI-17 | モデル比較 | Yes |
| CLI-18 | MIRP solver | Yes |
| CLI-19 | 旧CLIとの結果一致 | Yes |
| CLI-20 | 3回実行再現性 | Yes |
| CLI-21 | 入力ファイル不変性 | Yes |
| CLI-22 | Evidence上書き拒否 | Yes |
| CLI-23 | 不正パス・権限異常 | Yes |
| CLI-24 | Lean異常終了 | Yes |
| CLI-25 | 資源制限 | Yes |
| CLI-26 | インストール後の実行 | Yes |

## 10.2 重要な負例

少なくとも次のケースを含める。

1. 数学的に偽の命題
2. 未完成のLean証明
3. 未許可公理の使用
4. 型不一致の数理IR
5. 未対応数理演算
6. Evidenceのhash改変
7. 古いKnowledge revision
8. 不完全なモデル比較条件
9. Lean実行環境の欠落
10. 出力ディレクトリの衝突
11. 不正なJSON入力
12. リソース超過
13. 存在しないEvidence参照
14. 不正なファイルパス
15. 既存CLI呼び出しとの結果不一致

## 10.3 回帰判定

過去のPASS報告との一致だけでは受入を認めない。

新旧CLIを同一の入力・Knowledge revision・toolchain・資源設定で実行する。

比較対象は以下とする。

- Formal Status
- Proof Obligation ID
- Goal
- Assumptions
- Generated Lean source hash
- Knowledge dependency
- Evidence内容
- Model revision
- Planの正規化結果

旧CLIと新CLIで出力順序が異なる場合は、意味を変えない正規化を行った後に比較する。

単なるタイムスタンプの不一致は回帰失敗としない。

---

# 11. インストール仕様

## 11.1 対象環境

最初の正式サポート環境：

- macOS
- Rust 1.93.1
- ReasonScript 0.5.6.5
- Lean 4.35.0
- 固定Mathlib revision

LinuxとWindowsは、現行Lean実行sandboxの互換性が確立するまで正式な受入対象としない。

## 11.2 インストール方式

Rustの標準的なインストール方式を使用する。

目標コマンド例：

```bash
cargo install \
  --path cli \
  --locked
```

インストール後：

```bash
mathlang --version
mathlang doctor
mathlang --help
```

既存の固定toolchain、Mathlib cache、実行に必要なリソースの配置方法は文書化する。

CLIバイナリのインストールだけでLean/Mathlibまで導入済みと誤認させてはならない。

## 11.3 ドキュメント

必須成果物：

- `docs/cli/README.md`
- `docs/cli/INSTALL.md`
- `docs/cli/COMMAND_REFERENCE.md`
- `docs/cli/JSON_SCHEMA.md`
- `docs/cli/ERROR_CODES.md`
- `docs/cli/EXAMPLES.md`

---

# 12. 監査・成果物

実装完了時、以下を作成する。

```text
cli/reports/
├── ARCHITECTURE_AUDIT.md
├── IMPLEMENTATION_REPORT.md
├── COMMAND_COVERAGE.md
├── REGRESSION_REPORT.md
├── NEGATIVE_TEST_REPORT.md
├── INSTALLATION_REPORT.md
└── FINAL_ACCEPTANCE.md
```

実行原証跡は専用ディレクトリに保存する。

```text
cli/evidence/
└── <run-id>/
    ├── environment.json
    ├── commands.json
    ├── test_results.json
    ├── regression_results.json
    ├── negative_results.json
    ├── source_manifest.json
    └── artifact_manifest.json
```

成果物のパスとスキーマは実装時に固定し、以後の変更はrevision管理する。

原証跡を生成しない集計報告だけではPASSとしない。

---

# 13. 最終受入判定

## PASS

次の全条件を満たした場合のみ認定する。

- P0〜P5の全GateがPASS
- 必須CLI機能が実装済み
- 既存数理モデルが変更されていない
- 既存検証エンジンの振る舞いが維持されている
- 形式証明結果が一致
- Evidenceの整合性が成立
- 重大な回帰なし
- ローカルインストール成功
- 実行手順と結果が再現可能

## PARTIAL

CLIの一部が正常動作していても、必須コマンドや回帰検証が未完了の場合。

## FAIL

次のいずれかを確認した場合。

- 数理的結果の誤分類
- 偽命題をPROVEDと判定
- 必須の証明義務欠落
- Evidence改変を正常と認定
- 既存数理検証の非互換変更
- 未承認のMIRPスキーマ変更
- 未承認のReasonScript Runtime変更
- 不正な入力による隔離実行の突破

## NOT_VERIFIED

試験を実行できない、証跡が不足する、環境が成立していない場合。

環境不成立をPASSとして扱わない。

---

# 14. Codexへの実装指示

次の順序を厳守すること。

1. 既存実装を監査する。
2. 既存テストを再実行し、baselineを保存する。
3. CLI統一に必要な最小限の責務分離を検討する。
4. CLI共通基盤を実装する。
5. 既存Core APIへ接続する。
6. CLIの正常系・異常系を実装する。
7. 既存CLIとの回帰比較を行う。
8. 数理検証の正式試験を再実行する。
9. 独立監査を行う。
10. 最終判定を報告する。

実装を簡素に保つこと。

コマンドごとに数理処理や証明ロジックを複製しない。

既存のRust Coreを直接再利用できるなら、新しい抽象化層を不要に増やさない。

テストを成功させるために、既存の数理モデル、Lean命題、Knowledge、証明結果を変更してはならない。

**最終目標は、MathLang_DSNをローカルにインストールし、単一のCLIから数理モデルの解析・形式検証・比較・Evidence管理を実行できる状態にすることである。**

本開発では推論モデルの新規実装および数学的表現能力の拡張を行わない。