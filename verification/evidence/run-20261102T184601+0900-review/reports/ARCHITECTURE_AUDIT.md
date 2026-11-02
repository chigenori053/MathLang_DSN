# Architecture Audit / Phase 0

対象: タスク開始時の作業ツリー。既存の変更・削除・未追跡ファイルを含めて監査した。これらを初期コミットの状態へ戻していない。
baseline: `../evidence/baseline-20261102/`。既存ソースのハッシュは `source_manifest.json` にある。

| 対象 | 確認した責務・能力 | 再利用判断 / 不足 |
|---|---|---|
| `mathlang_dsn/model.rsn`, `knowledge.rsn` | 有界有理演算、RU候補、RUS選択、RUOと状態前後の記録 | 既存計算を維持。一般複素線形代数の証明には転用しない |
| `polynomial.rsn`, `junior_high.rsn` | 多項式・教育数学・整数根検証 | 数値計算を形式証明とは扱わない |
| `mirp/src/lib.rs` | `SemanticState`, occurrence、Provenance、依存・矛盾検査、既存Evidence | 固定 `mirp/0.1-si2` を変更しない。数学IR/Evidenceは別構造 |
| `mirp/src/intent.rs`, `pif/*` | ProblemIntent、目標依存、条件、評価、限定構文 | 一般数学型と量化子がないため新IRを独立実装 |
| `mirp/src/knowledge.rs` | KnowledgeSpace、決定論的取得、登録時の循環検査、occurrence hash | crate の公開APIを利用し、ID/occurrenceを読み取り専用で参照 |
| `mirp/src/gke.rs` | 有界構造推論、RU/RUS/RUO traces、推論予算 | 既存 predicate inference を証明器にしない。新plannerで型付き目標を分解 |
| `mirp/src/native.rs`, `capability.rs` | ReasonScript呼出、quadratic能力選択、実行結果との一致 | 新verificationへの呼出を強制しない。既存公開APIを維持 |
| `mirp/src/memory.rs`, `session.rs` | 記憶・復元、プロセス間永続性、失効検査 | 数学revisionは独立registryで管理 |
| Python Host | `api.py`, `__main__.py`, `__init__.py`, Python用project/test定義が開始時点で削除済み | 現存しないAPIを実装済み・検証済みと報告しない。復元は行わない |
| ReasonScript Runtime | installed 0.5.6.10; project指定0.5.6.5; 既存検査が成功 | Runtime自体は変更しない。差異は環境記録に保存 |
| Lean / Mathlib | Lean4.35.0、参照USM側に固定Mathlib cache | 実行時に全Lake packageのrevisionとtracked変更なしを検査 |

依存関係:

```text
verification (Rust)
  ├── mathlang-mirp::knowledge::KnowledgeSpace (public read-only API)
  ├── serde / serde_json / sha2 (既存と同じ依存系列、独立Cargo.lock)
  └── Python stdlib process supervisor
        └── macOS sandbox-exec
              └── fixed Lean + fixed Lake package cache
                    ├── Foundation.lean (新しい証明)
                    ├── Independent.lean (手書きの独立命題・単独証明)
                    └── generated target (IRから生成)
```

変更影響: 新規ファイルは `verification/` に限定。既存MIRPのserde format、Knowledge ID、ReasonScript Runtime、API実装を変更していない。
Rust crateを分離することで、既存利用者の通常の計算にLean依存や実行時間を追加しない。
ReasonScript検査は `.reason/` に再生成可能な実行キャッシュを作成した。これはソースの変更ではない。

baselineは Rust 77成功・1 ignored、ReasonScript check、project-validate、5 self-check成功。既存のignored性能試験は新機能の検証数に加算しない。
変更前後の既存ソースhashの一致を最終監査で検査する。同一ソースの既存回帰を新規テストの成功数で相殺しない。
Phase 0 Gateはbaselineと最終原証跡監査を条件にPASS。
