# MathLang_DSN

MathLang_DSN is a bounded mathematical reasoning model for MathLang. ReasonScript provides the existing mathematical rules, calculations, and quadratic answer checks. Exact rational statistics, certified finite radical-sum comparisons, plane geometry, unit-aware classical physics, and finite-dimensional quantum mechanics also use registered Rust executors with independent arithmetic checks. Rust provides MIRP's typed semantic state, persistence, provenance, problem parsing, planning, capability dispatch, and the command-line bridge to ReasonScript. The project follows the state and rule model explored in [DSN_Test](https://github.com/chigenori053/DSN_Test); imported modules are listed in [mirp/DSN_TEST_IMPORTS.md](mirp/DSN_TEST_IMPORTS.md).

## Run

Install [ReasonScript 0.5.6.5](https://github.com/chigenori053/ReasonScript) so `reason` is on `PATH`, and install Rust and Cargo. From the repository root:

```sh
reason --version
reason check
reason run --entry JuniorHighSelfCheck --json --trace=off
reason run --entry QuadraticRuntimeSelfCheck --json --trace=off
cargo run --manifest-path mirp/Cargo.toml -- solve-problem 'Solve x^2 - 5x + 6 = 0.'
```

A ReasonScript self-check passes when `runtime_result.result` is `true`. The Rust command emits JSON containing the result, reasoning plan, and validated MIRP semantic state. The supported native solve path handles bounded quadratic equations with exact integer roots. It may try factorization and fall back to the quadratic formula. Irrational and nonintegral roots remain `UNKNOWN` under this answer contract.

MIRP can also ingest bounded natural language, Japanese, mathematical assignments, and simple code into a persistent state:

```sh
cargo run --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global nl 'x is five.'
cargo run --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global math 'y = x + 2'
cargo run --manifest-path mirp/Cargo.toml -- /tmp/lesson.mirp lesson global code $'if y > 6:\n    result = y'
```

The commands run in separate processes and continue from the same `.mirp` file. The implemented text adapters accept a small grammar; unsupported syntax produces an explicit error. MIRP's current state schema is `mirp/0.1-si2`. Older snapshots must be regenerated.

## ReasonScript model

The `mathlang_dsn/*.rsn` modules implement exact rational arithmetic, one-variable rational polynomials through degree five, differentiation, bounded integration, linear equation solving, equation and expression assessment, and a Knowledge catalogue with RU/RUS/RUO traces. `JuniorHigh` also provides bounded prime factorization, square-root simplification, quadratic roots, two-variable systems, function values, geometry calculations, data summaries, and probability calculations. These are model functions, not a general expression-string command-line interface. The `src/main.rsn` calculations exercise representative operations.

The native quadratic runtime additionally uses `JuniorHigh::JExactIntegerRoots`, `JCheckIntegerRoots`, `JAnyIntegerRoot`, `JEquivalentQuadratics`, and `JProcessQuadratic` for exact solution checks, equation equivalence, root validation, and constraint filtering. Rust converts typed MIRP and ProblemIntent data into bounded ReasonScript calls and records the returned results and provenance. Structured coefficients and algebra syntax trees are used for claims; display strings are not treated as proof.

The quadratic intent contract accepts coefficients with magnitude at most 1,000,000. A correct subset of roots can be mathematically valid but incomplete, and an excluded root is recorded against its sourced constraint. Other mathematical claims preserve `UNKNOWN` where proof is unavailable. General equations, matrices, diagrams, broad natural-language understanding, and curriculum-wide coverage are outside the validated scope.

## Verification

```sh
reason check
reason project-validate . --json
reason run --entry SelfCheck --json --trace=off
reason run --entry PolynomialSelfCheck --json --trace=off
reason run --entry KnowledgeSelfCheck --json --trace=off
reason run --entry JuniorHighSelfCheck --json --trace=off
reason run --entry QuadraticRuntimeSelfCheck --json --trace=off
cargo test --manifest-path mirp/Cargo.toml
git diff --check
```

The MIRP foundation, ProblemIntent validation, and knowledge-native runtime contracts are documented in [mirp/STATUS.md](mirp/STATUS.md), [mirp/PIF_REPORT.md](mirp/PIF_REPORT.md), and [mirp/KNOWLEDGE_RUNTIME_VALIDATION.md](mirp/KNOWLEDGE_RUNTIME_VALIDATION.md). Their historical validation figures describe the versions tested at the time. The project is licensed under Apache-2.0.

## Unified CLI v0.1

Rust製 `mathlang` CLIの実装・インストール・検証結果は [docs/cli/README.md](docs/cli/README.md) を参照。

平方根式の厳密簡約・等値判定・大小比較、平均・中央値・四分位数・範囲、母分散・不偏分散、共分散、重みの正規化・有限離散分布の期待値は `mathlang compute FILE` でKnowledgeから実行できる。
既存ReasonScript関数と多倍長の有理数計算器を使い、独立検算・説明・MIRPの証拠を返す。[入力例と対応範囲](docs/cli/EXACT_CALCULATIONS.md)。

座標平面の距離・内積・直交・三角形の面積、直線同士・直線と円・円同士の全交点も同じ `compute` 経路で利用できる。
単位付きの等加速度運動、合力と加速度、一定力の仕事、運動エネルギー、仕事と運動エネルギーの整合判定も利用できる。1〜3次元の慣性系・直交Cartesian座標とNewtonian質点モデルを要求し、単位・座標系・必要な仮定を検査する。
量子力学は1〜32次元（USM-QRM-001拡張で8から拡大）の純粋状態・混合状態、正規化、内積、射影測定のBorn則、期待値、密度行列、ユニタリ変換を扱う。複素有理数と正の共通平方根係数で厳密計算し、共役・エルミート性・半正定値性・確率総和・トレースを検査する。
座標の次元・退化・接触・一致を検査し、交点の完全性を元の方程式への代入と個数判定で確認する。
