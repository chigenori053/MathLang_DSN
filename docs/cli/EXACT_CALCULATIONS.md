# 厳密式・統計・座標平面のKnowledge実行

リポジトリrootでビルドしたCLIから、構造化JSONを入力する。

```sh
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/sqrt.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/summary.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/variance.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/covariance.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/weights.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/expectation.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/exact_expression.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/exact_equal.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/exact_compare.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/plane_distance.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/plane_dot.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/plane_orthogonal.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/triangle_area.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/line_line.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/line_circle.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/circle_circle.json --format json
```

| 段階 | operation | 入力 | 結果 |
|---|---|---|---|
| 1 | `SIMPLIFY_SQRT` | `{"radicand":72}` | `6√2`を `{"kind":"RADICAL","coefficient":6,"radicand":2}` として返す |
| 2 | `DESCRIBE_DATA` | `{"values":[3,1,2]}` | 個数3、平均2、中央値2、第1四分位数1、第3四分位数3、範囲2 |
| 3 | `DATA_VARIANCE` | `{"values":[1,2,3]}` | 平均2、母分散2/3、不偏分散1 |
| 4 | `DATA_COVARIANCE` | 同数の有理数配列 `x`, `y` | 平均x・平均y、分母nの母共分散、分母n−1の不偏共分散 |
| 4 | `NORMALIZE_WEIGHTS` | 有理数配列 `weights` | 総重みと正規化済みの確率 |
| 4 | `DISCRETE_EXPECTATION` | 有理数配列 `values`, `weights`、任意 `normalize_weights` | 有限離散分布の期待値と実際に使った確率 |
| 5 | `NORMALIZE_EXACT` | 平方根式 `expression` | 同類項統合・約分済みの厳密な正規形 |
| 5 | `EXACT_EQUAL` | 平方根式 `left`, `right` | `equal` と順序、検証可能な証明書 |
| 5 | `EXACT_COMPARE` | 平方根式 `left`, `right` | `LESS` / `EQUAL` / `GREATER` と検証可能な証明書 |
| 6 | `PLANE_DISTANCE` | 点 `a`, `b` | 距離の二乗・非負の厳密距離・一致判定 |
| 6 | `PLANE_DOT_PRODUCT` | ベクトル `u`, `v` | 有理数の内積 |
| 6 | `PLANE_ORTHOGONAL` | ベクトル `u`, `v` | 直交・零ベクトル・角の定義可能性 |
| 6 | `TRIANGLE_AREA` | 頂点 `a`, `b`, `c` | 面積・向き付き倍面積・退化判定 |
| 6 | `LINE_LINE_INTERSECTIONS` | 直線 `first`, `second` | 交点1点・平行・一致 |
| 6 | `LINE_CIRCLE_INTERSECTIONS` | `line`, `circle` | 0・1・2点の全列挙、接点・点円の判定 |
| 6 | `CIRCLE_CIRCLE_INTERSECTIONS` | 円 `first`, `second` | 0・1・2点または無限、同心・一致・点円の判定 |
| 7 | `CONSTANT_ACCELERATION` | 物理context・初期位置・初速度・加速度・時間 | 位置・速度・変位 |
| 7 | `NEWTON_FORCE` / `NEWTON_ACCELERATION` | 物理context・質量と加速度または合力 | F=ma / a=F合力/m |
| 7 | `CONSTANT_FORCE_WORK` | 物理context・一定力・変位 | 仕事W=F·Δx |
| 7 | `KINETIC_ENERGY` | 物理context・質量・速度 | 運動エネルギー |
| 7 | `WORK_ENERGY` | 物理context・質量・前後速度・総仕事 | ΔK・残差・整合判定 |

入力JSONは `operation` と上表の入力フィールドを持つ。未知のフィールド・浮動小数・自由記述の数式文字列は受け付けない。
有理数は `{"numerator":2,"denominator":3}` のような既約分数で表す。
従来の3操作では、平方根の入力は0以上200,000,000以下の整数、統計は1〜64個・絶対値1,000,000以下の整数。
追加した6操作の有理数・式の上限は下記を参照。
範囲外・統計の空配列は終了4、JSONの構造エラーは終了2、ReasonScript実行エラーは終了5、検算不一致は終了9。比較の精度予算不足は終了6で、等値や近似結果を返さない。
共通CLIはReasonScriptを含む子プロセス群に300秒の期限を設定する。

## 計算と検算

`compute` はlegacy 29件、`mirp/knowledge/exact_statistics_v0_1.json` の9件、`mirp/knowledge/plane_geometry_v0_1.json` の7件、`mirp/knowledge/classical_physics_v0_1.json` の6件、計51件のKnowledgeSpaceを明示的に使う。
既存の `solve` の二次方程式用KnowledgeSpaceと、USM配布パックの既定読み込みは変更しない。

処理順序は、入力検証 → MIRPへの入力登録 → Knowledge検索・適用条件確認 → Capability選択 → 登録済実行器による計算 → 独立検算 → 結果・説明・証拠の登録。
検索、優先順位、明示的な競合、Capabilityの存在確認は既存の二次方程式経路と共通の選択処理を使う。
入力の型・上限は検索前とExecutor内で検査する。

- 平方根は既存の `JuniorHigh::JSimplifySqrt`。Rust側で非負性、二乗による復元、平方因子除去を検算する。
- 要約統計は `Descriptive::SummaryOf` から既存の `JuniorHigh::JDataSummary` を呼ぶ。Rust側は別にソートして平均・中央値・四分位数・範囲を検算する。
- 分散は `Descriptive::VarianceOf`。既存の `JuniorHigh::Fraction` で約分し、`nΣx²−(Σx)²` から計算する。Rust側は別の式 `Σ(i<j)(xᵢ−xⱼ)²` で検算する。

四分位数は、昇順データの上下半分の中央値を使う。奇数個では全体の中央値を上下半分から除く。
1個のデータでは平均・中央値・両四分位数をその値とし、範囲・母分散を0とする。不偏分散は未定義のため `null`。
不偏分散という名称は計算式の区別を示す。入力データの独立性や同一分布性を自動的に認定しない。

`result` は `answer`、`verification`、日本語の `explanation`、`knowledge_decision`、`trace`、`semantic_state` を持つ。
TraceにはKnowledgeの内容識別子、依存Knowledge、入力・結果参照、RU/RUS/RUO、実行した関数、検算内容を含む。
SemanticStateにも構造化した厳密値とEvidenceを保持し、既存の保存・再読込経路を使える。
`verification.status=VERIFIED` は実行結果の整数算術による検算を意味する。
`verification.formal_status=NOT_CHECKED` とし、Leanの `PROVED` には昇格させない。

## 共分散・離散分布・重み

追加操作の有理数は `{"numerator":"1","denominator":"2"}`。符号付き64ビットの範囲内なら入力にJSON整数も使える。それより大きい整数には文字列を使う。
浮動小数による精度損失を防ぐため、追加操作の結果は分子・分母を常に10進整数の文字列で返す。
入力の分子は符号を除き最大80桁、分母は正の整数で最大80桁。統計の配列は1〜64要素。
これらの操作は多倍長の `num-rational` / `num-bigint` Capabilityで実行し、実行器名を証拠に記録する。

```json
{"operation":"DATA_COVARIANCE","x":[{"numerator":1,"denominator":1},{"numerator":2,"denominator":1}],"y":[{"numerator":2,"denominator":1},{"numerator":4,"denominator":1}]}
```

共分散は対応する2配列の長さ一致を要求する。分母nとn−1の違いを保持し、n=1の不偏共分散は `null`。
計算は `nΣxy−ΣxΣy`、独立検算は中心化した積の和を使う。不偏性に必要な標本抽出の仮定を自動認定するものではない。

`NORMALIZE_WEIGHTS` は非負・総和が正の重みに対して `pᵢ=wᵢ/Σw` を返す。ゼロの重みは許容し、負の重みや全ゼロは拒否する。
検算では合計1と、元の重みへの比例関係を厳密に確認する。

`DISCRETE_EXPECTATION` の `normalize_weights` は既定で `false`。
この場合 `weights` を確率として扱い、合計が**厳密に1**でない入力を拒否する。
`true` を明示した場合だけ重みを正規化する。値と重みは同数を要求し、値には負数・分数も使える。
期待値は `Σpᵢxᵢ`、独立検算は `Σwᵢ(xᵢ−期待値)=0`。結果には総重み、実際に使った確率、正規化の指定を保持する。

## 平方根を含む厳密式・等値・大小

式は `Σ coefficient × √radicand` という有限和で表す。
有理数部分には `radicand=1`、ゼロには空の項列を使う。例えば `(1+√5)/2` は次のとおり。

```json
{"terms":[{"coefficient":{"numerator":"1","denominator":"2"},"radicand":1},{"coefficient":{"numerator":"1","denominator":"2"},"radicand":5}]}
```

1式につき最大32項。係数は上記の80桁の有理数、根号内は0以上200,000,000以下の整数。
負の根号内は係数が0でも拒否する。根号は非負の主平方根として解釈し、係数の符号を保持する。
有限和は和・差・有理数倍を表せる。入れ子の根号、任意式での除算、文字変数、自由記述の式パーサーはこの入力形式に含まない。

簡約は既存の `JuniorHigh::JSimplifySqrt` をまとめて呼び、同じ平方因子除去済みの根号の係数を厳密に加算する。
ゼロ係数を除去し、根号内の値順に並べる。検算ではRustで素因数分解から別に正規形を再構成する。

`EXACT_EQUAL` と `EXACT_COMPARE` は `EXACT_COMPARISON` の結果を返す。
`comparison.equal` と `comparison.ordering`、正規化済みの両辺、`comparison.certificate` を持つ。
差（左辺−右辺）が正規形で0なら厳密に等しい。それ以外は平方根を包含する有理数区間を精密化して符号を確定する。
証明書には精度、各平方根の整数境界、差の下限・上限を保持し、整数の二乗不等式から検証する。
小数の近似一致、許容誤差による等値判定は使わない。

精度は8ビットから倍増し、最大4096ビット。符号を保証できなければ `RESOURCE_LIMIT` / 終了6とし、判定結果を返さない。
`comparison_limit.json` は許容入力の範囲内でこの制限に達する回帰確認用fixture。
この実行時証明書はLean形式証明とは区別し、`formal_status=NOT_CHECKED` を維持する。

## 座標平面の距離・内積・直交・面積・全交点

幾何学は同一の直交Cartesian座標系を前提とする2次元の有理数計算。
点・ベクトルは、上記の `ExactRational` 2個の配列で表す。例えば原点は次のとおり。

```json
[{"numerator":"0","denominator":"1"},{"numerator":"0","denominator":"1"}]
```

各分子・分母は統計と同じ最大80桁。1・3成分などの次元不一致、分母0、負の分母は終了4。
浮動小数・未知フィールド・不正なJSON構造は終了2。座標系の変換や異なる単位の混在を自動推定する機能は含まない。

直線は `ax+by+c=0` の `{"a":有理数,"b":有理数,"c":有理数}`。
`a=b=0` は、cが0でも非零でも直線ではないため、退化した直線入力として終了4で拒否する。
垂直・水平な直線、有理数係数、係数の負の定数倍を扱える。
円は円周として扱い、`{"center":点,"radius":有理数}`。負半径は拒否し、半径0は中心の1点とする。

距離と交点座標には、有理数を根号内に持てる次の厳密値を使う。

```json
{"rational":{"numerator":"0","denominator":"1"},"coefficient":{"numerator":"1","denominator":"2"},"radicand":{"numerator":"3","denominator":"1"}}
```

これは `rational + coefficient × √radicand`、例では `√3/2` を意味する。
有理数になる場合は `coefficient=0, radicand=0` にまとめる。
平方数の根号は有理数へ戻し、小さい数の平方因子も除去する。大きい有理数の根号は、因数分解を強制せず厳密なまま保持する。
この表現により、大きい座標や分数から導かれる距離・交点でも小数への丸めを使わない。

距離は差ベクトルのノルムから求め、展開した内積の恒等式と非負の二乗根で検算する。
内積は偏極恒等式、面積は靴紐公式で別に検算する。
三角形の `degeneracy` は `NONE`、`REPEATED_VERTEX`、`COLLINEAR` を返す。重複頂点のある場合は `REPEATED_VERTEX` を優先する。
零ベクトルは代数的には全ベクトルと直交するため `orthogonal=true` だが、`zero_vector_present=true, angle_defined=false` として角が未定義であることを明示する。

### 交点の列挙と完全性

`answer.kind=GEOMETRY_INTERSECTIONS` の `intersection` は次のフィールドを持つ。

- `relation`: `CROSSING`（2直線の1交点）、`PARALLEL`、`SECANT`（2点）、`TANGENT`（1点）、`POINT_INTERSECTION`（点円の1点）、`DISJOINT`、`COINCIDENT`。
- `points`: 有限交点すべて。各点は `{"coordinates":[厳密値x,厳密値y]}`。x、次にyの厳密な大小で昇順に並べ、重複を許さない。
- `infinite`: 一致直線・一致する正半径の円ならtrue。この場合 `points=[]` は無限集合を有限列で表さない意味で、交点0個を意味しない。
- `input_degeneracies`: 半径0の円を `circle:ZERO_RADIUS_POINT`、`first_circle:ZERO_RADIUS_POINT` などで記録する。

同一中心・半径0の2円は無限集合ではなく、その中心1点。
同心でも異なる半径なら共通点なし。内接・外接は1点、分離・厳密な包含は0点と判定する。

直線同士は行列式と整合性、直線と円は射影と判別量、円同士は根軸で求める。
検証器では元の両方程式への厳密代入に加え、階数、判別量、中心間距離²と半径の和・差²から個数を確認する。
点が両方程式を満たすだけでは十分とせず、個数・相異性・整列・無限判定も検査する。
誤座標、片方の交点の欠落、同じ点の重複、同値な別表現での重複、有限集合の誤った無限宣言は拒否する。
説明にも行列式・判別量・中心間距離等の実際の値を記録する。

```sh
# 同一の正半径円: infinite=true
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/coincident_circles.json --format json
# 同一の点円: finiteな1点
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/point_circles.json --format json
# 10万候補と末尾の競合を含めて幾何学の試験を実行
cargo test --manifest-path mirp/Cargo.toml --offline --test geometry -- --include-ignored
```

実行器は既存の多倍長有理数・平方根の補助処理を使う `Rust/num-rational plane geometry`。
7件の `GEOMETRY_*` Knowledgeは、選択・計算・検算・説明・証拠まで接続している。
Knowledge IDの差し替え、欠落、競合、係数倍、円の入れ替え、平行移動でも試験する。
`verification.status=VERIFIED` は厳密算術の検算であり、Lean形式証明の追加を意味しない。

## 単位付きの古典物理学

`classical_physics_v0_1.json` の6件の `PHYSICS_*` Knowledgeを同じ選択・計算・検算・説明・証拠の経路で使う。
実行器は既存の多倍長有理数を使う `Rust/num-rational classical physics`。

| operation | context以外の入力 | 計算と出力 |
|---|---|---|
| `CONSTANT_ACCELERATION` | `initial_position`, `initial_velocity`, `acceleration`, `duration` | v=v₀+at、x=x₀+v₀t+at²/2、変位、経過時間 |
| `NEWTON_FORCE` | `mass`, `acceleration` | 合力F=ma |
| `NEWTON_ACCELERATION` | `mass`, `net_force` | a=F合力/m |
| `CONSTANT_FORCE_WORK` | `force`, `displacement` | W=F·Δx。負・0の仕事も保持 |
| `KINETIC_ENERGY` | `mass`, `velocity` | K=m||v||²/2 |
| `WORK_ENERGY` | `mass`, `initial_velocity`, `final_velocity`, `net_work` | K前、K後、ΔK、W合力−ΔKの残差、`consistent` |

スカラー物理量は `{"value":{"numerator":1500,"denominator":1},"unit":"g"}`。
ベクトル物理量は `{"values":[{"numerator":36,"denominator":1}],"unit":"km/h","frame_id":"lab"}`。
有理数の分子・分母は整数または10進整数文字列、各80桁以内で、分母0は拒否する。
浮動小数点や暗黙の単位は使わない。出力は約分された分子・分母の文字列でSI単位に統一する。
ベクトルと `net_work` には同じ `frame_id` が必須。`mass` と `duration` では省略できるが、指定時は一致を要求する。
出力の仕事・エネルギーにも座標系を記録する。

全入力で次の `context` を明示する。軸の順序がベクトル成分の順序であり、原点と軸の意味は呼出側が宣言する。
異なる座標系の自動変換は行わない。

```json
{
  "model": "NEWTONIAN_POINT_PARTICLE",
  "frame": {
    "id": "lab", "origin": "fixed laboratory origin",
    "dimension": 1, "axes": ["x"],
    "kind": "INERTIAL", "basis": "ORTHONORMAL_CARTESIAN"
  },
  "assumptions": ["NONRELATIVISTIC", "CONSTANT_ACCELERATION"]
}
```

次元は1〜3、軸名は次元数と一致し重複不可。各ベクトルもその成分数を要求する。
非慣性系・斜交基底・相対論粒子・剛体モデルは拒否する。質量は正、経過時間は0以上。
入力速度と等加速度運動の終速度が光速以上なら拒否する。この検査は必要条件に過ぎず、低速近似の精度を保証しない。

必要な仮定は全操作の `NONRELATIVISTIC` に加え、運動では `CONSTANT_ACCELERATION`、
力では `CONSTANT_MASS`、加速度では `CONSTANT_MASS` と `NET_FORCE`、
仕事では `CONSTANT_FORCE`、運動エネルギーでは `CONSTANT_MASS`、
仕事とエネルギーの比較では `CONSTANT_MASS` と `NET_WORK`。
欠落・重複は拒否し、宣言したcontextと必要な仮定の一覧を結果・説明に残す。

受理する単位記号は `1, m, cm, mm, km, kg, g, mg, s, ms, min, h, N, kN, mN, J, kJ, mJ`。
大文字小文字を区別する。`*` と `/` を左から評価し、各因子に整数指数 `^-4`〜`^4` を指定できる。
例: `m/s^2`, `kg*m/s/s`, `N*m`, `km/h`。括弧や暗黙の積は未対応。
単位式はASCII 80文字、8因子まで。長さ・質量・時間の指数を照合して、厳密な有理数でSI換算する。
位置・変位はm、速度はm/s、加速度はm/s²、力はN、仕事・エネルギーはJで出力する。

運動は速度変化と平均速度による変位、各軸のv²−v₀²=2aΔxで検算する。
力と加速度は逆算、仕事は成分の寄与の総和、運動エネルギーは非負性と速度の恒等式を検査する。
仕事とエネルギーでは両端のK・差分・残差・一致判定をそれぞれ検査する。
`answer.kind=PHYSICS` の `result.outcome` に各結果を格納する。

`applicability=CONDITIONAL_ON_DECLARED_ASSUMPTIONS` は、現実の系への適用が宣言に条件付きであることを表す。
仮定の物理的な真偽を自動証明するものではない。`WORK_ENERGY` の `consistent=false` は正常な評価結果で、
`verification.status=VERIFIED` はその不整合判定を含む算術の検算成功を意味する。
位置エネルギー・重力モデル・可変質量・変動力の積分・回転・相対論・量子力学は扱わない。Lean形式証明は追加していない。

6個の入力例は同じ1次元慣性系を使い、36 km/h、200 cm/s²、2秒から変位24 m・終速度14 m/sを得る。
質量1500 gなら合力3 N、仕事72 J、K前75 J・K後147 Jで一致する。

```sh
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_constant_acceleration.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_newton_force.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_newton_acceleration.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_constant_force_work.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_kinetic_energy.json --format json
cargo run --manifest-path cli/Cargo.toml --offline -- compute mirp/fixtures/calculation/physics_work_energy.json --format json
# 10万候補と末尾の競合を含めた試験
cargo test --manifest-path mirp/Cargo.toml --offline --test physics -- --include-ignored --nocapture
```

## APIと追加Knowledge

Rust APIは `calculation::compute`、独自のKnowledgeSpaceを使う場合は `compute_with_knowledge`。
`calculation::calculation_knowledge()` でこの拡張を含むKnowledgeSpaceを作成できる。
別のKnowledgeを既存Capabilityへ接続する場合も、適用条件と入力契約を満たす必要がある。
独自の式を文字列として実行することはできない。

MIRPバイナリでも同じJSONを扱う。任意のMemorySpaceを読み込む場合、その中に必要なKnowledgeを登録しておく。

```sh
cargo run --manifest-path mirp/Cargo.toml --offline -- compute mirp/fixtures/calculation/variance.json
# 既存のMemorySpace JSONを使う場合
# cargo run --manifest-path mirp/Cargo.toml --offline -- compute request.json memory.json
```

検証コマンド:

```sh
reason check
reason build
reason run --entry DescriptiveSelfCheck --json --trace=off
cargo test --manifest-path mirp/Cargo.toml --offline
cargo test --manifest-path cli/Cargo.toml --offline
# 適用可能な候補10万件での計算・再現性・末尾の競合検出を明示実行
cargo test --manifest-path mirp/Cargo.toml --offline --test calculation dense_calculation_candidates_scale_and_preserve_late_conflicts -- --ignored --exact --nocapture
```

上記の負荷試験は、同じ優先度の適用可能候補を10万件用意し、64件のデータを3回計算する。
全候補を省略せず確認し、結果・検算・MIRP状態の一致を検査した後、末尾に追加した競合候補の検出を確認する。
ローカルのdebug実行で約47秒だった二乗的な走査の再発を検出するため、1回の計算に20秒の上限を置く。
この上限は負荷試験の回帰判定用であり、製品の応答時間保証ではない。

今回追加したKnowledgeは `EXACT_DATA_COVARIANCE`、`EXACT_WEIGHT_NORMALIZATION`、`EXACT_DISCRETE_EXPECTATION`、
`EXACT_RADICAL_NORMAL_FORM`、`EXACT_RADICAL_EQUALITY`、`EXACT_RADICAL_COMPARISON`。
各操作の実行・独立検算・説明・証拠に加え、別IDへの差し替え、Knowledge欠落、競合検出をテストしている。

一般の無理数全体、入れ子の根号、無理数根への `solve` 拡張、標準偏差、重み付き分散・共分散、統計的推定は未対応。
幾何学は上記の座標平面の範囲に対応する。幾何学の3次元、線分・半直線、任意曲線、量子力学の新しい実行経路は未対応。
