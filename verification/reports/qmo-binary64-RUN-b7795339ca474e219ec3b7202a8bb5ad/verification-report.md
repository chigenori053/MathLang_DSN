# USM-QMO-01 Evidence & Artifact Foundation v0.1

Stage A: **PASS**  
Stage B: **NOT_VERIFIED**  
Independent audit: **PASS**

数学モデルの採用判断は全5モデルで **INCONCLUSIVE**。証拠基盤の成功を意味理解・USMの成立・量子優位性の証明とは扱わない。

| Phase | Result |
|---|---|
| P0 | PASS |
| P1 | PARTIAL |
| P2 | NOT_VERIFIED |
| P3 | NOT_VERIFIED |
| P4 | NOT_VERIFIED |
| P5 | NOT_VERIFIED |

## 実行した限定検証

Lean4: 有限次元 outer product の Hermitian、PSD、正規化状態の trace 1、凸混合の有効性、global phase 不変性の5命題。意図した定理名・型・推移的公理依存を照合し、`--trust=0` で独立再実行した。

MathLang DSN: 厳密有理数の密度3例に加え、検証用 numerical CLI のbinary64密度12例を実行。原始IEEE入力からPython Fractionの厳密比で期待行列を再計算し、固定した絶対・相対許容誤差（各1e-12）で独立再実行を照合した。無効入力8件の拒否も再実行した。数値例を一般定理・意味理解の証明には数えない。

改ざん検出・スキーマ等: 29 件。既存 Lean build、MIRP quantum・CLI contracts 回帰試験の原始ログを保管。

## 原始証拠への参照

| Evidence ID | Case | Type | Audited result |
|---|---|---|---|
| EV-7a55b9093b11494f965b9d2fb32c3e47 | QMO-FOUND-hermitian | FORMAL | PASS |
| EV-adf36ffb6e4741ae99e2de2fba5ed5c4 | QMO-FOUND-positive | FORMAL | PASS |
| EV-4287e74e39ce44f6ab7809dc8e5f2c30 | QMO-FOUND-trace_one | FORMAL | PASS |
| EV-819981122c794837bc2137b19cb5c329 | QMO-FOUND-mixture | FORMAL | PASS |
| EV-7e782a4fa5e7442881b0419769327a06 | QMO-FOUND-phase | FORMAL | PASS |
| EV-5908ad83dff34ce89c9cdbfc621408cd | QMO-NUM-DENSITY-0 | NUMERICAL | PASS |
| EV-e9cbf5c6f6f24fffbf06fb50a9a3a532 | QMO-NUM-DENSITY-1 | NUMERICAL | PASS |
| EV-f0d8f1b3cc9e45d697e55a001782ea1f | QMO-NUM-DENSITY-2 | NUMERICAL | PASS |
| EV-3e31f3b5c3184e40beb235075d1a9cb2 | QMO-FLOAT-00 | NUMERICAL | PASS |
| EV-5aefcbab5c3e4de3932fbfe65651c15a | QMO-FLOAT-01 | NUMERICAL | PASS |
| EV-cd34b53f1a7a435d95059dbd6b2114e9 | QMO-FLOAT-02 | NUMERICAL | PASS |
| EV-f74e51a3779b4aaa935d29a1469ada10 | QMO-FLOAT-03 | NUMERICAL | PASS |
| EV-0525aea02cda47c1885665a933c6b1dc | QMO-FLOAT-04 | NUMERICAL | PASS |
| EV-afecb0821b024aa0b88998d2e95538a9 | QMO-FLOAT-05 | NUMERICAL | PASS |
| EV-4b16d4ada63c4e0ca6aa68dcd67a7e65 | QMO-FLOAT-06 | NUMERICAL | PASS |
| EV-7134c350615f422f8c23fe2344feee68 | QMO-FLOAT-07 | NUMERICAL | PASS |
| EV-162adb352b1f4233942682cabd501df5 | QMO-FLOAT-08 | NUMERICAL | PASS |
| EV-d9f37e29e1914e6aa4f9e807743395ec | QMO-FLOAT-09 | NUMERICAL | PASS |
| EV-4d01f227ebd9478e93435ea9db434e3f | QMO-FLOAT-10 | NUMERICAL | PASS |
| EV-696d014f714840b0be00e5b73b739829 | QMO-FLOAT-11 | NUMERICAL | PASS |

各 Evidence の inputs/outputs に Artifact ID と SHA-256、manifest に物理パス・サイズを記録。`final-gates.json` は独立再計算結果で照合される。

## 基盤受入の検査結果

exact_replay: PASS
floating_tolerance_replay: PASS
negative_cli_replay: PASS

## 最終研究判断

Observable-induced abstract semantic states: complete mathematical construction NOT_VERIFIED.
Retention of required semantic relations: NOT_VERIFIED.
Knowledge update preservation as a USM model: NOT_VERIFIED.
Empirical advantage over classical models: NOT_VERIFIED.
Model suitability for UnifiedRU follow-up research: INCONCLUSIVE for every registered model revision.

## 制限

- The evidence foundation passing does not establish USM-QMO-01 mathematical feasibility.
- Five limited foundation lemmas, three exact rational and twelve binary64 dimension-2 fixtures only.
- QMO-MATH-004 pure-state projection/idempotency is not covered by this run.
- No general eigenvalue/eigenvector, tensor, matrix exponential, seeded benchmark, high-dimensional encoding or memory validation.
- No held-out natural-language semantic evaluation, preservation experiment or fair five-family baseline comparison.
- Knowledge-update and semantic collector/auditor interfaces are tested on synthetic infrastructure fixtures; no research claim is established.
- The separately retained anchor is a local checkpoint, not an authenticated signature or an externally trusted CI record. Preserve its hash separately.
- Initial EAS G0-G5 differs from continuation P0-P5: continuation section 11 is the authoritative mapping.
- Regression scope is existing Lean build, MIRP quantum integration tests and CLI contracts; the full repository test suite is not claimed.
- Production USM, UnifiedRU and inference functionality were not modified; a verification-only MathLang numerical CLI command was added.
