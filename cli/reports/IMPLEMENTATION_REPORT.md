# Implementation Report

独立crate `cli` に正式名mathlangを実装。入口/Envelope/設定はlib.rs・main.rs、既存APIの呼出しはcommands.rs、外部処理期限はprocess.rs。
コマンドごとに証明ロジックを複製せず、すべて既存Rust Coreへ接続した。

実装対象: model check/inspect/normalize、verify FILE/usm、proof plan/counterexample、knowledge inspect/impact、compare、evidence show/validate、solve、doctor。
Knowledge updateは明示OUT_OF_SCOPE。共通JSON/エラー契約、未知field拒否、設定優先順位、固定上限拒否、出力新規作成、symlink/parent traversal拒否を実装した。

verifyは固定9件を3回実行し、入力・configuration・バイナリhash・ソースhash・kernel生ログと既存形式Evidenceを保存する。
反例候補のみでREFUTEDにしない。条件付き命題とwhole-statement negationを既存翻訳に委ねる。
モデル比較は実行環境未指定のため条件一致時もCONDITIONALLY_COMPARABLE。数学的同値・優劣は主張しない。

設定の資源値は既存runnerの固定値のみ受理。別の値は小さい値も含めて終了4。
install済みバイナリはビルド時リポジトリの資源を使う。リポジトリを移動したら再installが必要。

検証: Rust 6件（契約5 + プロセスdeadline1）、clippy warnings denied、fmt、debug/installed各40実行、既存Rust77+33、Lean27新旧比較、11独立変換、6負例。
実測集計は `../evidence/audit-final.json`。初回テストで見つかった一時パスのsymlink拒否・同時run ID衝突を修正し、失敗ログは別名で保全した。
