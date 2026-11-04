# 量子力学Knowledge索引 v0.1

生成元: `tools/build_catalogue.py`。数式中の ħ を省いた行列は ħ = 1 単位。

## 段階1: 量子力学の公理（状態・観測量・Born則・射影・時間発展・合成系・混合状態）（`QM_POSTULATES`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_POSTULATE_STATE` | AXIOM | 物理的公理 | 孤立系の純粋状態は複素Hilbert空間 H の単位ベクトル ψ（‖ψ‖ = 1）で表され、ψ と e^{iθ}ψ は同じ物理状態を表す。 |
| `QM_POSTULATE_OBSERVABLE` | AXIOM | 物理的公理 | 観測可能量は H 上の自己共役作用素 A で表され、測定値は A のスペクトルの元である。 |
| `QM_POSTULATE_BORN_RULE` | AXIOM | 物理的公理 | 固有値 a に対応する射影を P_a とすると、状態 ψ で a を得る確率は p(a) = ⟨ψ\|P_a\|ψ⟩ = ‖P_a ψ‖²。 |
| `QM_POSTULATE_PROJECTION` | AXIOM | 物理的公理 | 射影測定で a を得た直後の状態は P_a ψ / ‖P_a ψ‖。 |
| `QM_POSTULATE_SCHRODINGER` | AXIOM | 物理的公理 | 閉じた系の状態は iħ dψ/dt = H ψ に従う。H はハミルトニアン（自己共役）。 |
| `QM_POSTULATE_COMPOSITE` | AXIOM | 物理的公理 | 部分系 A, B からなる合成系の状態空間はテンソル積 H_A ⊗ H_B。 |
| `QM_POSTULATE_DENSITY` | AXIOM | 物理的公理 | 一般の（混合）状態は密度作用素 ρ（ρ ⪰ 0, tr ρ = 1）で表され、p(a) = tr(ρ P_a)。純粋状態は ρ = \|ψ⟩⟨ψ\|。 |
| `QM_POSTULATE_SYMMETRIZATION` | AXIOM | 物理的公理 | 同種粒子の状態はボソンなら交換で対称、フェルミオンなら反対称。反対称性から2個のフェルミオンは同じ1粒子状態を占有できない（Pauli排他律）。 |
| `QM_CANONICAL_COMMUTATION` | AXIOM | 物理的公理 | 位置と運動量は正準交換関係 [x̂, p̂] = iħ を満たす。 |

## 段階1: SI定義定数と基本関係（`QM_CONSTANTS`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_CONST_PLANCK` | DEFINITION | SI定義 | Planck定数 h = 6.62607015 × 10⁻³⁴ J·s（2019年SI改定以降、定義値で不確かさなし）。 |
| `QM_CONST_REDUCED_PLANCK` | DEFINITION | SI定義 | 換算Planck定数 ħ = h / (2π) ≈ 1.054571817 × 10⁻³⁴ J·s。 |
| `QM_PLANCK_EINSTEIN` | RULE | 物理的公理 | 光子のエネルギーと振動数の関係 E = hν = ħω。 |
| `QM_DE_BROGLIE` | RULE | 物理的公理 | 運動量 p の粒子のde Broglie波長は λ = h / p（波数 k = p/ħ）。 |

## 段階2: 状態・密度行列・位相・Bloch表現（`QM_STATES`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_GLOBAL_PHASE_SAME_DENSITY` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_RAY_OUTER`） | ψ と φ = zψ（\|z\| = 1）が位相同値なら \|ψ⟩⟨ψ\| = \|φ⟩⟨φ\|。したがって大域位相はどの測定確率にも現れない。 |
| `QM_GLOBAL_PHASE_BORN_WEIGHTS` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_PHASE_WEIGHT`） | \|z\| = 1 なら各基底成分の確率重み \|(zψ)_i\|² = \|ψ_i\|² は不変。 |
| `QM_PURE_STATE_DENSITY` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_OUTER_POSITIVE`） | 任意の ψ について \|ψ⟩⟨ψ\| は半正定値で tr \|ψ⟩⟨ψ\| = ‖ψ‖²。‖ψ‖ = 1 なら密度行列である。 |
| `QM_MIXTURE_IS_DENSITY` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_P0_04`） | 密度行列 ρ, σ と 0 ≤ a ≤ 1 について aρ + (1−a)σ も密度行列（混合状態の凸性）。 |
| `QM_MAXIMALLY_MIXED_NOT_PURE` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_C07_NOT_PURE`） | 量子ビットの最大混合状態 I/2 はどの純粋状態 \|ψ⟩⟨ψ\| とも等しくない。 |
| `QM_ENSEMBLE_NOT_UNIQUE` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_ENSEMBLE_AMBIGUITY`） | 異なる純粋状態の集団（{\|0⟩,\|1⟩} と {\|+⟩,\|−⟩} の等確率混合）が同じ密度行列 I/2 を与える。 |
| `QM_BLOCH_PURE_STATE` | THEOREM | 厳密計算で検査（検査: `bloch_pure_state`） | ψ = (3/5, 4i/5) について ρ = \|ψ⟩⟨ψ\| = (I + r·σ)/2、r = (⟨σx⟩, ⟨σy⟩, ⟨σz⟩) = (0, 24/25, −7/25)、\|r\|² = 1、tr ρ² = 1。 |
| `QM_BLOCH_GENERAL` | THEOREM | 標準的結果（未検査） | 任意の量子ビット密度行列は ρ = (I + r·σ)/2（r ∈ ℝ³, \|r\| ≤ 1）と一意に書け、純粋 ⇔ \|r\| = 1 ⇔ tr ρ² = 1。 |

## 段階2: 観測量・スペクトル・期待値・同時測定（`QM_OBSERVABLES`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_HERMITIAN_REAL_SPECTRUM` | THEOREM | 標準的結果（未検査） | 有限次元のエルミート行列 A（A† = A）の固有値はすべて実数で、異なる固有値の固有ベクトルは直交し、A は正規直交固有基底で対角化できる（スペクトル定理）。 |
| `QM_EXPECTATION_REAL` | THEOREM | 標準的結果（未検査） | A† = A なら期待値 ⟨ψ\|A\|ψ⟩ は実数。 |
| `QM_BORN_PROBABILITIES_SUM_TO_ONE` | THEOREM | 標準的結果（未検査） | Σ_a P_a = I かつ tr ρ = 1 なら Σ_a tr(ρ P_a) = tr(ρ) = 1。 |
| `QM_BORN_RULE_EXAMPLE` | THEOREM | 厳密計算で検査（検査: `born_rule_example`） | ψ = (3/5, 4i/5) を σz 基底で測ると p(+1) = 9/25, p(−1) = 16/25（和 1）。\|+⟩ = (1,1)/√2 では p(±1) = 1/2。射影 \|0⟩⟨0\|, \|+⟩⟨+\| は冪等かつエルミートで、\|0⟩⟨0\| + \|1⟩⟨1\| = I。 |
| `QM_EXPECTATION_VALUES_EXAMPLE` | THEOREM | 厳密計算で検査（検査: `expectation_values_example`） | ⟨+\|σx\|+⟩ = 1, ⟨+\|σz\|+⟩ = 0, ⟨0\|σz\|0⟩ = 1。ψ = (3/5, 4i/5) では ⟨σz⟩ = −7/25 = 9/25 − 16/25（Born確率による平均と一致）。 |
| `QM_COMMUTING_OBSERVABLES_COMPATIBLE` | THEOREM | 標準的結果（未検査） | 有限次元で A, B がエルミートかつ [A, B] = 0 なら、共通の正規直交固有基底が存在し同時測定可能。 |
| `QM_ORTHOGONAL_STATES_DISTINGUISHABLE` | THEOREM | 標準的結果（未検査） | 直交する状態 ⟨ψ\|φ⟩ = 0 は射影測定 {\|ψ⟩⟨ψ\|, I − \|ψ⟩⟨ψ\|} で確実に区別でき、非直交状態は単一測定で誤りなく区別できない。 |

## 段階2: スピン・Pauli行列・角運動量代数（`QM_SPIN`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_PAULI_HERMITIAN_UNITARY` | IDENTITY | 厳密計算で検査（検査: `pauli_hermitian_unitary`） | Pauli行列 σx = [[0,1],[1,0]], σy = [[0,−i],[i,0]], σz = [[1,0],[0,−1]] はエルミートかつユニタリで、σ_k² = I, tr σ_k = 0, det σ_k = −1。 |
| `QM_PAULI_PRODUCT` | IDENTITY | 厳密計算で検査（検査: `pauli_product`） | σ_j σ_k = δ_jk I + i ε_jkl σ_l。特に σxσy = iσz, σyσz = iσx, σzσx = iσy。 |
| `QM_PAULI_COMMUTATOR` | IDENTITY | 厳密計算で検査（検査: `pauli_commutator`） | [σ_j, σ_k] = 2i ε_jkl σ_l、{σ_j, σ_k} = 2 δ_jk I。 |
| `QM_SPIN_HALF_ALGEBRA` | THEOREM | 厳密計算で検査（検査: `spin_half_algebra`） | S = σ/2 は [S_x, S_y] = iS_z（巡回）を満たし、S² = S_x² + S_y² + S_z² = (3/4) I = s(s+1) I（s = 1/2）。S_z の固有値は ±1/2。 |
| `QM_SPIN_ONE_ALGEBRA` | THEOREM | 厳密計算で検査（検査: `spin_one_algebra`） | スピン1行列 S_x = (1/√2)[[0,1,0],[1,0,1],[0,1,0]], S_y = (1/√2)[[0,−i,0],[i,0,−i],[0,i,0]], S_z = diag(1,0,−1) は [S_x,S_y] = iS_z（巡回）、S² = 2I = s(s+1)I（s = 1）を満たしエルミート。 |
| `QM_ANGULAR_MOMENTUM_ALGEBRA` | THEOREM | 標準的結果（未検査） | [J_x, J_y] = iħJ_z（巡回）を満たす有限次元既約表現は j = 0, 1/2, 1, … で次元 2j+1、J² = j(j+1)ħ²、J_z の固有値は mħ（m = −j, …, j）。 |

## 段階3: ユニタリ時間発展・保存量・量子ゲート（`QM_DYNAMICS`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_UNITARY_PRESERVES_NORM` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `SP_UNITARY_NORM_FORM`） | U†U = I かつ φ = Uψ なら ‖φ‖ = ‖ψ‖。ユニタリ時間発展は正規化（確率の総和 1）を保存する。 |
| `QM_UNITARY_DENSITY_UPDATE` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_DENSITYUPDATE_TRACE`） | 密度行列 ρ とユニタリ U について UρU† は密度行列（半正定値・トレース1）。 |
| `QM_TIME_EVOLUTION_UNITARY` | THEOREM | 標準的結果（未検査） | H が自己共役で時間に依存しなければ U(t) = exp(−iHt/ħ) はユニタリで ψ(t) = U(t)ψ(0) はSchrödinger方程式を解く。U(t)U(s) = U(t+s)。 |
| `QM_STATIONARY_STATE` | THEOREM | 標準的結果（未検査） | Hψ = Eψ なら ψ(t) = e^{−iEt/ħ}ψ で、あらゆる測定確率は時間に依存しない（定常状態）。 |
| `QM_EHRENFEST` | THEOREM | 標準的結果（未検査） | 時間に陽に依存しない観測量 A について d⟨A⟩/dt = (i/ħ)⟨[H, A]⟩。 |
| `QM_CONSERVED_QUANTITY` | THEOREM | 標準的結果（未検査） | [H, A] = 0 で A が時間に陽に依存しなければ ⟨A⟩ と A の測定確率分布は保存される。 |
| `QM_RATIONAL_UNITARY_EXAMPLE` | THEOREM | 厳密計算で検査（検査: `rational_unitary_example`） | U = (1/5)[[3, 4i],[4i, 3]] は U†U = UU† = I を満たし、ψ = (3/5, 4i/5) に作用させると Uψ = (−7/25, 24i/25) で ‖Uψ‖² = 1。 |
| `QM_HADAMARD_GATE` | IDENTITY | 厳密計算で検査（検査: `hadamard_gate`） | H = (1/√2)[[1,1],[1,−1]] は H† = H, H² = I、H\|0⟩ = \|+⟩、H σx H = σz、H σz H = σx。 |
| `QM_PHASE_GATE` | IDENTITY | 厳密計算で検査（検査: `phase_gate`） | 位相ゲート S = diag(1, i) はユニタリで S² = σz、S σx S† = σy。 |
| `QM_CNOT_GATE` | IDENTITY | 厳密計算で検査（検査: `cnot_gate`） | CNOT = \|0⟩⟨0\| ⊗ I + \|1⟩⟨1\| ⊗ σx はエルミートかつユニタリで CNOT² = I。 |

## 段階3: 交換関係と不確定性関係（`QM_UNCERTAINTY`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_ROBERTSON_UNCERTAINTY` | THEOREM | 標準的結果（未検査） | エルミートな A, B と正規化された ψ について σ_A σ_B ≥ \|⟨[A, B]⟩\| / 2（Robertsonの不確定性関係）。 |
| `QM_ROBERTSON_PAULI_EXAMPLE` | THEOREM | 厳密計算で検査（検査: `robertson_pauli_example`） | A = σx, B = σy で、検査した各有理純粋状態について Var(σx)·Var(σy) ≥ \|⟨[σx, σy]⟩\|²/4 = ⟨σz⟩²。例: ψ = (3/5, 4i/5) では Var(σx) = 1, Var(σy) = 49/625 で等号成立。 |
| `QM_HEISENBERG_UNCERTAINTY` | THEOREM | 標準的結果（未検査） | 位置と運動量について σ_x σ_p ≥ ħ/2。 |
| `QM_CCR_NO_FINITE_DIMENSION` | THEOREM | 標準的結果（未検査） | n ≥ 1 の n×n 行列 X, P で XP − PX = cI（c ≠ 0）となるものは存在しない。tr(XP − PX) = 0 だが tr(cI) = cn ≠ 0。 |

## 段階3: 合成系・エンタングルメント・Bell状態・非局所性（`QM_COMPOSITE`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_PRODUCT_STATE_NORMALIZED` | THEOREM | Lean検査済みUSM命題に依拠（アンカー: `USM_R01`） | ‖x‖ = ‖y‖ = 1 なら ‖x ⊗ y‖ = 1（一般に ‖x ⊗ y‖ = ‖x‖‖y‖）。 |
| `QM_BELL_BASIS` | THEOREM | 厳密計算で検査（検査: `bell_basis`） | Bell状態 Φ± = (\|00⟩ ± \|11⟩)/√2, Ψ± = (\|01⟩ ± \|10⟩)/√2 は C²⊗C² の正規直交基底をなす。 |
| `QM_BELL_CIRCUIT` | THEOREM | 厳密計算で検査（検査: `bell_circuit`） | CNOT (H ⊗ I) \|00⟩ = Φ+。 |
| `QM_BELL_ENTANGLED` | THEOREM | 厳密計算で検査（検査: `bell_entangled`） | ρ = \|Φ+⟩⟨Φ+\| は tr ρ² = 1（全体は純粋）だが、部分トレース tr_B ρ = I/2 で tr(ρ_A²) = 1/2。係数行列の行列式は 1/2 ≠ 0 なので Φ+ は積状態 a ⊗ b ではない。 |
| `QM_SUPERDENSE_CODING` | THEOREM | 厳密計算で検査（検査: `superdense_coding`） | (σ ⊗ I)Φ+ は σ = I, σx, iσy, σz に対してそれぞれ Φ+, Ψ+, Ψ−, Φ−（符号は大域位相）を与え、4状態は直交する（超高密度符号化）。 |
| `QM_NO_CLONING` | THEOREM | 標準的結果（未検査） | 任意の未知状態 ψ に対し U(ψ ⊗ e) = ψ ⊗ ψ となるユニタリ U は存在しない（ψ が2つ以上の非直交・非同一状態を含む集合を動く場合）。 |
| `QM_NO_CLONING_CNOT_EXAMPLE` | THEOREM | 厳密計算で検査（検査: `no_cloning_cnot_example`） | CNOT は \|0⟩\|0⟩ → \|0⟩\|0⟩, \|1⟩\|0⟩ → \|1⟩\|1⟩ と基底状態を複製するが、\|+⟩\|0⟩ は Φ+ に写り \|+⟩\|+⟩ にならない（密度行列が異なる）。 |
| `QM_CHSH_LOCAL_BOUND` | THEOREM | 厳密計算で検査（検査: `chsh_local_bound`） | ±1 の決定論的な値 a0, a1, b0, b1 について S = a0b0 + a0b1 + a1b0 − a1b1 ∈ {−2, 2}。したがって局所隠れた変数模型では \|⟨S⟩\| ≤ 2。 |
| `QM_CHSH_QUANTUM_VIOLATION` | THEOREM | 厳密計算で検査（検査: `chsh_quantum_violation`） | Φ+ と A0 = σz, A1 = σx, B0 = (σz + σx)/√2, B1 = (σz − σx)/√2 で ⟨S⟩ = 2√2 > 2（⟨S⟩² = 8 を厳密に検査）。2√2 はTsirelson限界。 |

## 段階4: 代表的な束縛系のスペクトル（`QM_MODELS`）

| ID | 種別 | 検証 | 内容 |
|---|---|---|---|
| `QM_HARMONIC_OSCILLATOR` | THEOREM | 標準的結果（未検査） | H = p²/(2m) + mω²x²/2 の固有値は E_n = ħω(n + 1/2)（n = 0, 1, 2, …）。a = √(mω/2ħ)(x + ip/(mω)) について [a, a†] = 1, H = ħω(a†a + 1/2)。 |
| `QM_INFINITE_SQUARE_WELL` | THEOREM | 標準的結果（未検査） | 幅 L の無限井戸型ポテンシャル中の粒子は E_n = n²π²ħ²/(2mL²)、ψ_n(x) = √(2/L) sin(nπx/L)（n = 1, 2, …）。 |
| `QM_HYDROGEN_SPECTRUM` | THEOREM | 標準的結果（未検査） | 非相対論的水素原子の束縛状態のエネルギーは E_n = −m_e e⁴ / (2(4πε₀)²ħ² n²) ≈ −13.6057 eV / n²、縮退度 n²（スピンを除く）。 |
