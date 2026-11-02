# Negative Test Report

debug/installedのcommands.jsonにコマンド、期待/実際終了値、stdout/stderrを保存。

- 不正引数・JSON・未知field・型不一致・未対応型/演算を拒否。
- 数学的偽命題とIncomplete recipeは実際のLeanでUNKNOWN、終了0。
- 単体REFUTEDは終了0、反例候補はUNKNOWN。
- 改変Generated.lean、古いRegistry revision、欠損Evidence参照は終了7。
- 出力衝突、存在しない入力、symlink、不正出力パスは拒否。
- runner不在の制御環境はERROR、終了5、部分Evidenceを保存。
- 有限探索予算超過は終了6。子プロセスdeadlineは短い実時間試験で停止を確認。
- 初回Codex sandbox内のRSS監視拒否はERROR、終了5。lean-smokeに原Evidenceを保持。

既存negative_bridge.pyの6ケースを新規実行: 偽命題、Incomplete、sorry、不許可公理、弱めた命題、隔離内fork。
全ケースのソース・kernel結果・stdout/stderrはnegative-lean。証明への不正な昇格なし。

未実施: 実際のLeanプロセスを300秒/2GiBまで消費させる故障注入。通常runでは固定上限以内を全件確認したが、上限到達時のCLI統合受入を完全PASSにしない。
