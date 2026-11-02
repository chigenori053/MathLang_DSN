# Mathematical IR / Semantic Contract v0.1

IRは `src/ir.rs` のenumと構造体で表す。外部関数名をLeanの識別子として実行しない。
名前はASCII英字開始の英数字・underscore（最大80文字）、Leanには `v_` を付ける。
shadowing、未宣言関数、未束縛の寸法、不正な添字、無定義の型変換を拒否する。

| 型 | v0.1の数学的意味 / Lean |
|---|---|
| Bool, Nat, Int, Rational | `Bool`, `ℕ`, `ℤ`, `ℚ`。有理リテラルの分母は正、正規化時に既約化 |
| Real, Complex | 記号的 `ℝ`, `ℂ`。浮動小数点近似を使わない |
| Vector(T,n) | 固定座標の `Fin n → T`。Euclidean normは別のHilbertSpace型を使用 |
| Matrix(T,n,m) | `Matrix (Fin n) (Fin m) T` |
| Tensor(T,[n,…]) | 固定基底の多重線形座標配列 `Fin n → … → T` |
| HilbertSpace(n) | 有限次元複素Hilbert空間 `EuclideanSpace ℂ (Fin n)` |
| LinearOperator(H,K) | 有限次元複素Hilbert空間間の `H →L[ℂ] K` |
| DensityOperator(n) | PSDかつtrace=1の複素正方行列のsubtype。単なる行列へ弱めない |
| Proposition | `Prop` |
| Predicate | 型名を予約。引数signatureのない裸のPredicateは未定義として拒否 |
| Finite(n), Function(A,B), Universe, Abstract(A) | 有限反例とcodec命題に必要な `Fin n`, 関数型、`Type`、束縛された型変数 |

寸法は正の固定整数または宣言済みのNat変数。記号寸法は0を含むNat全体を保持し、量化範囲を勝手に縮めない。

| ノード | 意味、条件、Lean対応 |
|---|---|
| Constant / Variable | 明示した型の厳密定数・束縛変数。Identityは正方行列/自己作用素に限定 |
| Add / Subtract | 同型・同寸法のscalar/座標配列/行列/Hilbert状態/作用素の加減算。Nat減算はNatのmonus |
| Multiply | 同型scalar積、またはComplex scalar × Hilbert状態のscalar作用。行列積と区別 |
| Divide | 同じfieldの除算。非零リテラル、または明示した `denominator ≠ 0` 前提が必要 |
| Power | scalarのNat冪 |
| Conjugate | Complexの `star` |
| Norm | Real、Complex、Euclidean Hilbert状態のnorm。coordinate Vectorのsup normをEuclidean normと混同しない |
| InnerProduct | 同じ複素Hilbert空間、`inner ℂ`（第1引数を共役） |
| MatrixMultiply | 行列内側寸法の一致。作用素の場合は定義域・値域を照合して `.comp` |
| Adjoint | Real/Complex行列の `.conjTranspose`、またはHilbert作用素の `.adjoint` |
| TensorProduct | 座標ベクトル/座標テンソルのpure tensor、成分は `a_i * b_j`。抽象Hilbert空間のcompleted tensor積は対応外 |
| TensorIndex | 固定寸法と添字範囲を検査してcoordinateへ適用。Vector/Matrixのcoordinateも同じ検査 |
| Trace | 正方行列の `.trace`、DensityOperatorの `.val.trace` |
| Equality / Inequality | 同型の等号、ordered scalarの `≤`。Complexのscalar不等号は拒否 |
| And / Or / Not / Implies | 命題論理。含意の結論ではその前提を型検査に利用できる |
| ForAll / Exists | 型と順序を保持した量化。型変数/記号寸法のscopeも検査 |
| PositiveSemidefinite | 複素正方行列の `Matrix.PosSemidef`。Mathlib `ComplexOrder` scopeを明示 |
| Hermitian | 複素正方行列の `Matrix.IsHermitian` |
| Unitary | 正方行列の `Aᴴ A = 1`、自己作用素の `A.adjoint.comp A = 1`。有限次元のisometry条件 |
| Normalized | `‖x‖ = 1` |
| LinearMap | 型付き複素線形作用素の `IsLinearMap ℂ` |
| StateTransition / Apply | 型付き関数・線形作用素の適用。無定義な関数は追加公理へ変換しない |
| Update | 関数型の値を同型のkey/valueで更新。`Function.update` |

正規化はJSONの決定論性と有理数の既約化。交換則・同値命題を証明なしで書き換える処理は含めない。
モデル・証明義務のhashは完全な構造を対象とし、異なるが数学的に同値な式を同一hashにする主張はない。

変換はIRから再生成して原コード・record・revision・前提を比較する。
代表例11件とUSM6件は別記述のLean型/命題への `example : expected := generated` で定義的一致を検査する。
これは代表例におけるKernel照合であり、Rust変換器そのものをLeanで形式証明したものではない。

対応外: 無限次元Hilbert空間、一般のテンソル縮約・添字記法、未指定のpredicate signature、一般自動証明探索、外部の任意Leanコード。
これらを既知の数学的意味として黙認しない。拡張時は型規則・変換規則・独立照合を同時に追加する。
