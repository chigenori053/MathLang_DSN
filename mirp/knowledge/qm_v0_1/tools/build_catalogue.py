#!/usr/bin/env python3
"""Build the quantum-mechanics KnowledgeUnit pack (QM v0.1).

The pack is data only. Running this script regenerates
catalogue/quantum_mechanics.json, catalogue/INDEX.ja.md and package_manifest.json
deterministically. Exact finite checks live in mirp/tests/quantum_knowledge.rs.
"""
import hashlib
import json
from pathlib import Path

PACK = Path(__file__).resolve().parents[1]
VERSION = "QM-KNOWLEDGE/0.1"
LEVEL = "UNIVERSITY_QUANTUM_MECHANICS"
CHECKER = "mirp/tests/quantum_knowledge.rs"

# verification_status -> (knowledge marker, mathematical_result)
STATUS = {
    "EXACT_FINITE_CHECKED": "EXACT_COMPUTATION_PASSED",
    "ANCHORED_TO_KERNEL_CHECKED": "PROVED_BY_ANCHOR",
    "POSTULATE": "POSTULATE",
    "SI_DEFINITION": "DEFINITION",
    "STANDARD_RESULT": "UNKNOWN",
}

STAGES = {
    "QM_POSTULATES": (1, "量子力学の公理（状態・観測量・Born則・射影・時間発展・合成系・混合状態）"),
    "QM_CONSTANTS": (1, "SI定義定数と基本関係"),
    "QM_STATES": (2, "状態・密度行列・位相・Bloch表現"),
    "QM_OBSERVABLES": (2, "観測量・スペクトル・期待値・同時測定"),
    "QM_SPIN": (2, "スピン・Pauli行列・角運動量代数"),
    "QM_DYNAMICS": (3, "ユニタリ時間発展・保存量・量子ゲート"),
    "QM_UNCERTAINTY": (3, "交換関係と不確定性関係"),
    "QM_COMPOSITE": (3, "合成系・エンタングルメント・Bell状態・非局所性"),
    "QM_MODELS": (4, "代表的な束縛系のスペクトル"),
}

UNITS = []


def rule(variables, premises, conclusions):
    def pred(p):
        name, *args = p
        return {"predicate": name,
                "arguments": [{"type": "VARIABLE", "value": a} for a in args],
                "negated": False}
    return {"variables": [{"name": v, "value_type": "SYMBOL"} for v in variables],
            "premises": [pred(p) for p in premises],
            "constraints": [],
            "conclusions": [pred(p) for p in conclusions]}


def k(kid, sub, ktype, status, statement, *, premises=(), deps=(), goals=("QUANTUM_MECHANICS",),
      symbols=(), note, check=None, anchor=None, gke=None, reference=None, units=None, priority=0):
    assert status in STATUS and sub in STAGES
    assert (status == "EXACT_FINITE_CHECKED") == (check is not None), kid
    assert (status == "ANCHORED_TO_KERNEL_CHECKED") == (anchor is not None), kid
    meta = {
        "mathematical_result": STATUS[status],
        "verification_status": status,
        # No Lean proof of this pack's own statements was run; see reports/.
        "formal_status": "UNKNOWN",
        "interpretation": note,
        "qm_stage": STAGES[sub][0],
        "pack_version": VERSION,
    }
    if check:
        meta["exact_check"] = {"check_id": check, "checker": CHECKER,
                               "arithmetic": "exact Gaussian rationals (i128); no floating point"}
    if anchor:
        meta["formal_anchor"] = {"knowledge_id": anchor, "anchor_formal_status": "KERNEL_CHECKED",
                                 "scope": "the mathematical statement only; the physical reading is a postulate"}
    if units:
        meta["units"] = units
    if reference:
        meta["reference"] = reference
    if gke:
        # GKE only considers units whose applicable_goals name the goal predicate.
        goals = tuple(goals) + tuple(c["predicate"] for c in gke["conclusions"] if c["predicate"] not in goals)
        meta["generic_rule_contract"] = {
            "status": "CONDITIONAL_ON_TYPED_FACT_INTERPRETATION",
            "runtime_rule_does_not_verify_input_facts": True,
            "native_gke_result_is_not_lean_PROVED": True,
        }
    UNITS.append({
        "knowledge_id": kid,
        "category": "PHYSICS",
        "subcategory": sub,
        "knowledge_type": ktype,
        "pattern": {"symbols": list(symbols), "relations": [], "properties": [status]},
        "premises": list(premises),
        "conclusion": statement,
        "constraints": [],
        "dependencies": list(deps),
        "applicable_goals": list(goals),
        "applicable_rus": [],
        "capabilities": [],
        "priority": priority,
        "provenance": {
            "source_type": {"EXACT_FINITE_CHECKED": "exact_finite_computation",
                            "ANCHORED_TO_KERNEL_CHECKED": "kernel_checked_anchor",
                            "POSTULATE": "physical_postulate",
                            "SI_DEFINITION": "si_definition",
                            "STANDARD_RESULT": "standard_reference"}[status],
            "source_id": f"QMKnowledge.{kid}",
            "source_span": None,
            "parent_occurrence_ids": [],
            "transformation": [],
            "memory_id": None,
        },
        "curriculum_level": LEVEL,
        "activation_metadata": meta,
        "runtime_rule_id": None,
        **({"generic_rule": gke} if gke else {}),
    })


TEXTBOOK = "standard textbook result (e.g. Sakurai & Napolitano, Modern Quantum Mechanics; Nielsen & Chuang, Quantum Computation and Quantum Information)"
HBAR1 = "ħ = 1 (spin and angular-momentum matrices are in units of ħ)"

# ---------------------------------------------------------------- Stage 1: postulates
k("QM_POSTULATE_STATE", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "孤立系の純粋状態は複素Hilbert空間 H の単位ベクトル ψ（‖ψ‖ = 1）で表され、ψ と e^{iθ}ψ は同じ物理状態を表す。",
  symbols=("ket", "hilbert_space"), goals=("QUANTUM_MECHANICS", "STATE"),
  note="状態公理。大域位相の同一視の数学的部分は QM_GLOBAL_PHASE_SAME_DENSITY を参照。", reference=TEXTBOOK)
k("QM_POSTULATE_OBSERVABLE", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "観測可能量は H 上の自己共役作用素 A で表され、測定値は A のスペクトルの元である。",
  symbols=("observable", "self_adjoint"), goals=("QUANTUM_MECHANICS", "OBSERVABLE"),
  note="有限次元ではエルミート行列。測定値が実数である数学的根拠は QM_HERMITIAN_REAL_SPECTRUM。", reference=TEXTBOOK)
k("QM_POSTULATE_BORN_RULE", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "固有値 a に対応する射影を P_a とすると、状態 ψ で a を得る確率は p(a) = ⟨ψ|P_a|ψ⟩ = ‖P_a ψ‖²。",
  premises=("‖ψ‖ = 1", "P_a = P_a† = P_a²", "Σ_a P_a = I"),
  symbols=("born_rule", "projector", "probability"), goals=("QUANTUM_MECHANICS", "PROBABILITY"),
  note="Born則。確率の総和が1になることは QM_BORN_PROBABILITIES_SUM_TO_ONE。", reference=TEXTBOOK, priority=10)
k("QM_POSTULATE_PROJECTION", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "射影測定で a を得た直後の状態は P_a ψ / ‖P_a ψ‖。",
  premises=("p(a) = ‖P_a ψ‖² > 0",), deps=("QM_POSTULATE_BORN_RULE",),
  symbols=("measurement", "collapse"), goals=("QUANTUM_MECHANICS", "MEASUREMENT"),
  note="射影仮説（Lüders則）。p(a)=0 の結果には条件付き状態を定義しない。", reference=TEXTBOOK)
k("QM_POSTULATE_SCHRODINGER", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "閉じた系の状態は iħ dψ/dt = H ψ に従う。H はハミルトニアン（自己共役）。",
  symbols=("schrodinger_equation", "hamiltonian"), goals=("QUANTUM_MECHANICS", "DYNAMICS"),
  note="時間に依存しない H では ψ(t) = e^{-iHt/ħ} ψ(0)（QM_TIME_EVOLUTION_UNITARY）。", reference=TEXTBOOK)
k("QM_POSTULATE_COMPOSITE", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "部分系 A, B からなる合成系の状態空間はテンソル積 H_A ⊗ H_B。",
  symbols=("tensor_product", "composite_system"), goals=("QUANTUM_MECHANICS", "COMPOSITE"),
  note="積状態の正規化は QM_PRODUCT_STATE_NORMALIZED（カーネル検査済みアンカー）。", reference=TEXTBOOK)
k("QM_POSTULATE_DENSITY", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "一般の（混合）状態は密度作用素 ρ（ρ ⪰ 0, tr ρ = 1）で表され、p(a) = tr(ρ P_a)。純粋状態は ρ = |ψ⟩⟨ψ|。",
  deps=("QM_POSTULATE_BORN_RULE",), symbols=("density_operator", "mixed_state"),
  goals=("QUANTUM_MECHANICS", "STATE", "PROBABILITY"),
  note="純粋状態の密度行列の正値性・トレースは QM_PURE_STATE_DENSITY。", reference=TEXTBOOK)
k("QM_POSTULATE_SYMMETRIZATION", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "同種粒子の状態はボソンなら交換で対称、フェルミオンなら反対称。反対称性から2個のフェルミオンは同じ1粒子状態を占有できない（Pauli排他律）。",
  symbols=("identical_particles", "pauli_exclusion"), goals=("QUANTUM_MECHANICS", "COMPOSITE"),
  note="スピン統計定理は相対論的場の理論の結果であり、ここでは公理として扱う。", reference=TEXTBOOK)
k("QM_CANONICAL_COMMUTATION", "QM_POSTULATES", "AXIOM", "POSTULATE",
  "位置と運動量は正準交換関係 [x̂, p̂] = iħ を満たす。",
  symbols=("position", "momentum", "commutator"), goals=("QUANTUM_MECHANICS", "UNCERTAINTY"),
  note="有限次元行列では実現できない（QM_CCR_NO_FINITE_DIMENSION）。", reference=TEXTBOOK)

# ---------------------------------------------------------------- Stage 1: constants
k("QM_CONST_PLANCK", "QM_CONSTANTS", "DEFINITION", "SI_DEFINITION",
  "Planck定数 h = 6.62607015 × 10⁻³⁴ J·s（2019年SI改定以降、定義値で不確かさなし）。",
  symbols=("planck_constant",), goals=("QUANTUM_MECHANICS", "PHYSICAL_CONSTANT"),
  note="SIの定義定数。", reference="BIPM, The International System of Units (SI), 9th edition (2019)")
k("QM_CONST_REDUCED_PLANCK", "QM_CONSTANTS", "DEFINITION", "SI_DEFINITION",
  "換算Planck定数 ħ = h / (2π) ≈ 1.054571817 × 10⁻³⁴ J·s。",
  deps=("QM_CONST_PLANCK",), symbols=("reduced_planck_constant",), goals=("QUANTUM_MECHANICS", "PHYSICAL_CONSTANT"),
  note="h から定義される。十進表示は丸め値。", reference="CODATA 2018")
k("QM_PLANCK_EINSTEIN", "QM_CONSTANTS", "RULE", "POSTULATE",
  "光子のエネルギーと振動数の関係 E = hν = ħω。",
  deps=("QM_CONST_PLANCK", "QM_CONST_REDUCED_PLANCK"), symbols=("photon", "energy", "frequency"),
  goals=("QUANTUM_MECHANICS", "PHYSICAL_CONSTANT"), note="Planck–Einstein関係。", reference=TEXTBOOK)
k("QM_DE_BROGLIE", "QM_CONSTANTS", "RULE", "POSTULATE",
  "運動量 p の粒子のde Broglie波長は λ = h / p（波数 k = p/ħ）。",
  deps=("QM_CONST_PLANCK",), symbols=("de_broglie_wavelength", "momentum"),
  goals=("QUANTUM_MECHANICS", "PHYSICAL_CONSTANT"), note="物質波の関係式。", reference=TEXTBOOK)

# ---------------------------------------------------------------- Stage 2: states (anchored)
k("QM_GLOBAL_PHASE_SAME_DENSITY", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "ψ と φ = zψ（|z| = 1）が位相同値なら |ψ⟩⟨ψ| = |φ⟩⟨φ|。したがって大域位相はどの測定確率にも現れない。",
  premises=("∃ z : ℂ, |z| = 1 ∧ φ = z ψ",), deps=("USM_RAY_OUTER", "USM_P0_06", "QM_POSTULATE_DENSITY"),
  symbols=("global_phase", "density_operator"), goals=("QUANTUM_MECHANICS", "STATE"),
  anchor="USM_RAY_OUTER",
  note="数学的内容は USM_RAY_OUTER（Lean検査済み）。確率への帰結は QM_POSTULATE_DENSITY による。",
  gke=rule(["psi", "phi"], [("PHASE_EQUIVALENT", "psi", "phi")], [("SAME_DENSITY", "psi", "phi")]))
k("QM_GLOBAL_PHASE_BORN_WEIGHTS", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "|z| = 1 なら各基底成分の確率重み |(zψ)_i|² = |ψ_i|² は不変。",
  premises=("|z| = 1",), deps=("USM_PHASE_WEIGHT", "QM_POSTULATE_BORN_RULE"),
  symbols=("global_phase", "born_rule"), goals=("QUANTUM_MECHANICS", "PROBABILITY"),
  anchor="USM_PHASE_WEIGHT", note="計算基底での測定確率が大域位相に依存しないことの数学的部分。")
k("QM_PURE_STATE_DENSITY", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "任意の ψ について |ψ⟩⟨ψ| は半正定値で tr |ψ⟩⟨ψ| = ‖ψ‖²。‖ψ‖ = 1 なら密度行列である。",
  deps=("USM_OUTER_POSITIVE", "USM_OUTER_TRACE", "QM_POSTULATE_DENSITY"),
  symbols=("density_operator", "pure_state"), goals=("QUANTUM_MECHANICS", "STATE"),
  anchor="USM_OUTER_POSITIVE", note="正値性は USM_OUTER_POSITIVE、トレースは USM_OUTER_TRACE（いずれもLean検査済み）。")
k("QM_MIXTURE_IS_DENSITY", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "密度行列 ρ, σ と 0 ≤ a ≤ 1 について aρ + (1−a)σ も密度行列（混合状態の凸性）。",
  premises=("0 ≤ a", "a ≤ 1"), deps=("USM_P0_04",), symbols=("mixed_state", "convexity"),
  goals=("QUANTUM_MECHANICS", "STATE"), anchor="USM_P0_04", note="確率的混合が状態であることの数学的部分。")
k("QM_MAXIMALLY_MIXED_NOT_PURE", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "量子ビットの最大混合状態 I/2 はどの純粋状態 |ψ⟩⟨ψ| とも等しくない。",
  deps=("USM_C07_NOT_PURE",), symbols=("mixed_state", "pure_state"), goals=("QUANTUM_MECHANICS", "STATE"),
  anchor="USM_C07_NOT_PURE", note="混合状態が純粋状態の集合より真に大きいことの最小例。")
k("QM_ENSEMBLE_NOT_UNIQUE", "QM_STATES", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "異なる純粋状態の集団（{|0⟩,|1⟩} と {|+⟩,|−⟩} の等確率混合）が同じ密度行列 I/2 を与える。",
  deps=("USM_ENSEMBLE_AMBIGUITY",), symbols=("ensemble", "density_operator"),
  goals=("QUANTUM_MECHANICS", "STATE"), anchor="USM_ENSEMBLE_AMBIGUITY",
  note="密度行列から集団の分解は一意に決まらない。")
k("QM_BLOCH_PURE_STATE", "QM_STATES", "THEOREM", "EXACT_FINITE_CHECKED",
  "ψ = (3/5, 4i/5) について ρ = |ψ⟩⟨ψ| = (I + r·σ)/2、r = (⟨σx⟩, ⟨σy⟩, ⟨σz⟩) = (0, 24/25, −7/25)、|r|² = 1、tr ρ² = 1。",
  deps=("QM_PURE_STATE_DENSITY", "QM_PAULI_HERMITIAN_UNITARY"), symbols=("bloch_vector", "pauli_matrix"),
  goals=("QUANTUM_MECHANICS", "STATE"), check="bloch_pure_state",
  note="Bloch表現の厳密な具体例。一般に量子ビットの密度行列は (I + r·σ)/2（|r| ≤ 1、純粋 ⇔ |r| = 1）。")
k("QM_BLOCH_GENERAL", "QM_STATES", "THEOREM", "STANDARD_RESULT",
  "任意の量子ビット密度行列は ρ = (I + r·σ)/2（r ∈ ℝ³, |r| ≤ 1）と一意に書け、純粋 ⇔ |r| = 1 ⇔ tr ρ² = 1。",
  deps=("QM_BLOCH_PURE_STATE",), symbols=("bloch_vector",), goals=("QUANTUM_MECHANICS", "STATE"),
  note="一般形はこのパックでは未検査。具体例は QM_BLOCH_PURE_STATE。", reference=TEXTBOOK)

# ---------------------------------------------------------------- Stage 2: observables
k("QM_HERMITIAN_REAL_SPECTRUM", "QM_OBSERVABLES", "THEOREM", "STANDARD_RESULT",
  "有限次元のエルミート行列 A（A† = A）の固有値はすべて実数で、異なる固有値の固有ベクトルは直交し、A は正規直交固有基底で対角化できる（スペクトル定理）。",
  premises=("A† = A",), deps=("QM_POSTULATE_OBSERVABLE",), symbols=("hermitian", "eigenvalue", "spectral_theorem"),
  goals=("QUANTUM_MECHANICS", "OBSERVABLE", "EIGENVALUE"),
  note="Mathlib には Matrix.IsHermitian.eigenvalues があるが、このパックではLean検査を実行していない。",
  reference=TEXTBOOK,
  gke=rule(["A"], [("HERMITIAN", "A")], [("REAL_SPECTRUM", "A")]))
k("QM_EXPECTATION_REAL", "QM_OBSERVABLES", "THEOREM", "STANDARD_RESULT",
  "A† = A なら期待値 ⟨ψ|A|ψ⟩ は実数。",
  premises=("A† = A",), deps=("QM_HERMITIAN_REAL_SPECTRUM",), symbols=("expectation_value", "hermitian"),
  goals=("QUANTUM_MECHANICS", "OBSERVABLE"), note="⟨ψ|Aψ⟩ = ⟨Aψ|ψ⟩ = conj⟨ψ|Aψ⟩ による。",
  reference=TEXTBOOK)
k("QM_BORN_PROBABILITIES_SUM_TO_ONE", "QM_OBSERVABLES", "THEOREM", "STANDARD_RESULT",
  "Σ_a P_a = I かつ tr ρ = 1 なら Σ_a tr(ρ P_a) = tr(ρ) = 1。",
  premises=("Σ_a P_a = I", "tr ρ = 1"), deps=("QM_POSTULATE_DENSITY", "UNI_TRACE_ADD"),
  symbols=("born_rule", "completeness"), goals=("QUANTUM_MECHANICS", "PROBABILITY"),
  note="トレースの線形性による。数値例は QM_BORN_RULE_EXAMPLE。", reference=TEXTBOOK)
k("QM_BORN_RULE_EXAMPLE", "QM_OBSERVABLES", "THEOREM", "EXACT_FINITE_CHECKED",
  "ψ = (3/5, 4i/5) を σz 基底で測ると p(+1) = 9/25, p(−1) = 16/25（和 1）。|+⟩ = (1,1)/√2 では p(±1) = 1/2。射影 |0⟩⟨0|, |+⟩⟨+| は冪等かつエルミートで、|0⟩⟨0| + |1⟩⟨1| = I。",
  deps=("QM_POSTULATE_BORN_RULE", "QM_BORN_PROBABILITIES_SUM_TO_ONE"), symbols=("born_rule", "projector"),
  goals=("QUANTUM_MECHANICS", "PROBABILITY"), check="born_rule_example", note="Born則と射影の厳密な具体例。")
k("QM_EXPECTATION_VALUES_EXAMPLE", "QM_OBSERVABLES", "THEOREM", "EXACT_FINITE_CHECKED",
  "⟨+|σx|+⟩ = 1, ⟨+|σz|+⟩ = 0, ⟨0|σz|0⟩ = 1。ψ = (3/5, 4i/5) では ⟨σz⟩ = −7/25 = 9/25 − 16/25（Born確率による平均と一致）。",
  deps=("QM_EXPECTATION_REAL", "QM_BORN_RULE_EXAMPLE"), symbols=("expectation_value",),
  goals=("QUANTUM_MECHANICS", "OBSERVABLE"), check="expectation_values_example",
  note="期待値 = Σ a p(a) の具体例。")
k("QM_COMMUTING_OBSERVABLES_COMPATIBLE", "QM_OBSERVABLES", "THEOREM", "STANDARD_RESULT",
  "有限次元で A, B がエルミートかつ [A, B] = 0 なら、共通の正規直交固有基底が存在し同時測定可能。",
  premises=("A† = A", "B† = B", "AB = BA"), deps=("QM_HERMITIAN_REAL_SPECTRUM",),
  symbols=("commutator", "compatible_observables"), goals=("QUANTUM_MECHANICS", "OBSERVABLE"),
  note="同時対角化定理。このパックでは未検査。", reference=TEXTBOOK,
  gke=rule(["A", "B"], [("COMMUTE", "A", "B"), ("HERMITIAN", "A"), ("HERMITIAN", "B")],
           [("COMPATIBLE", "A", "B")]))
k("QM_ORTHOGONAL_STATES_DISTINGUISHABLE", "QM_OBSERVABLES", "THEOREM", "STANDARD_RESULT",
  "直交する状態 ⟨ψ|φ⟩ = 0 は射影測定 {|ψ⟩⟨ψ|, I − |ψ⟩⟨ψ|} で確実に区別でき、非直交状態は単一測定で誤りなく区別できない。",
  deps=("QM_POSTULATE_BORN_RULE",), symbols=("distinguishability", "orthogonality"),
  goals=("QUANTUM_MECHANICS", "MEASUREMENT"), note="遷移確率 |⟨ψ|φ⟩|² による。", reference=TEXTBOOK,
  gke=rule(["psi", "phi"], [("ORTHOGONAL", "psi", "phi")], [("PERFECTLY_DISTINGUISHABLE", "psi", "phi")]))

# ---------------------------------------------------------------- Stage 2: spin
k("QM_PAULI_HERMITIAN_UNITARY", "QM_SPIN", "IDENTITY", "EXACT_FINITE_CHECKED",
  "Pauli行列 σx = [[0,1],[1,0]], σy = [[0,−i],[i,0]], σz = [[1,0],[0,−1]] はエルミートかつユニタリで、σ_k² = I, tr σ_k = 0, det σ_k = −1。",
  symbols=("pauli_matrix",), goals=("QUANTUM_MECHANICS", "SPIN"), check="pauli_hermitian_unitary",
  note="Pauli行列の基本性質。固有値は ±1。")
k("QM_PAULI_PRODUCT", "QM_SPIN", "IDENTITY", "EXACT_FINITE_CHECKED",
  "σ_j σ_k = δ_jk I + i ε_jkl σ_l。特に σxσy = iσz, σyσz = iσx, σzσx = iσy。",
  deps=("QM_PAULI_HERMITIAN_UNITARY",), symbols=("pauli_matrix", "levi_civita"),
  goals=("QUANTUM_MECHANICS", "SPIN"), check="pauli_product", note="9通りすべての積を厳密に検査。")
k("QM_PAULI_COMMUTATOR", "QM_SPIN", "IDENTITY", "EXACT_FINITE_CHECKED",
  "[σ_j, σ_k] = 2i ε_jkl σ_l、{σ_j, σ_k} = 2 δ_jk I。",
  deps=("QM_PAULI_PRODUCT",), symbols=("pauli_matrix", "commutator", "anticommutator"),
  goals=("QUANTUM_MECHANICS", "SPIN", "UNCERTAINTY"), check="pauli_commutator", note="9通りすべての組を厳密に検査。")
k("QM_SPIN_HALF_ALGEBRA", "QM_SPIN", "THEOREM", "EXACT_FINITE_CHECKED",
  "S = σ/2 は [S_x, S_y] = iS_z（巡回）を満たし、S² = S_x² + S_y² + S_z² = (3/4) I = s(s+1) I（s = 1/2）。S_z の固有値は ±1/2。",
  deps=("QM_PAULI_COMMUTATOR",), symbols=("spin_half", "angular_momentum"),
  goals=("QUANTUM_MECHANICS", "SPIN"), check="spin_half_algebra", units=HBAR1,
  note="スピン1/2表現。ħ を戻すと [S_x,S_y] = iħS_z, S² = (3/4)ħ²。")
k("QM_SPIN_ONE_ALGEBRA", "QM_SPIN", "THEOREM", "EXACT_FINITE_CHECKED",
  "スピン1行列 S_x = (1/√2)[[0,1,0],[1,0,1],[0,1,0]], S_y = (1/√2)[[0,−i,0],[i,0,−i],[0,i,0]], S_z = diag(1,0,−1) は [S_x,S_y] = iS_z（巡回）、S² = 2I = s(s+1)I（s = 1）を満たしエルミート。",
  deps=("QM_ANGULAR_MOMENTUM_ALGEBRA",), symbols=("spin_one", "angular_momentum"),
  goals=("QUANTUM_MECHANICS", "SPIN"), check="spin_one_algebra", units=HBAR1,
  note="1/√2 の因子は M = √2·S を有理行列として持ち運び、積で 1/2 に戻して厳密に検査。")
k("QM_ANGULAR_MOMENTUM_ALGEBRA", "QM_SPIN", "THEOREM", "STANDARD_RESULT",
  "[J_x, J_y] = iħJ_z（巡回）を満たす有限次元既約表現は j = 0, 1/2, 1, … で次元 2j+1、J² = j(j+1)ħ²、J_z の固有値は mħ（m = −j, …, j）。",
  deps=("QM_POSTULATE_OBSERVABLE",), symbols=("angular_momentum", "casimir"),
  goals=("QUANTUM_MECHANICS", "SPIN"), note="一般論は未検査。j = 1/2, 1 は厳密検査済みの具体例あり。",
  reference=TEXTBOOK)

# ---------------------------------------------------------------- Stage 3: dynamics
k("QM_UNITARY_PRESERVES_NORM", "QM_DYNAMICS", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "U†U = I かつ φ = Uψ なら ‖φ‖ = ‖ψ‖。ユニタリ時間発展は正規化（確率の総和 1）を保存する。",
  premises=("U†U = I", "φ = U ψ", "‖ψ‖ = r"), deps=("SP_UNITARY_NORM_FORM", "QM_POSTULATE_SCHRODINGER"),
  symbols=("unitary", "time_evolution", "normalization"), goals=("QUANTUM_MECHANICS", "DYNAMICS", "NORM"),
  anchor="SP_UNITARY_NORM_FORM", note="数学的内容は SP_UNITARY_NORM_FORM（Lean検査済み）。",
  gke=rule(["U", "psi", "phi"],
           [("UNITARY", "U"), ("EVOLVES", "U", "psi", "phi"), ("NORMALIZED", "psi")],
           [("NORMALIZED", "phi")]), priority=10)
k("QM_UNITARY_DENSITY_UPDATE", "QM_DYNAMICS", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "密度行列 ρ とユニタリ U について UρU† は密度行列（半正定値・トレース1）。",
  premises=("U†U = I",), deps=("USM_DENSITY_CONJUGATION_POSITIVE", "USM_DENSITYUPDATE_TRACE"),
  symbols=("unitary", "density_operator", "time_evolution"), goals=("QUANTUM_MECHANICS", "DYNAMICS"),
  anchor="USM_DENSITYUPDATE_TRACE", note="von Neumann方程式の解 ρ(t) = U(t)ρU(t)† が状態であることの数学的部分。")
k("QM_TIME_EVOLUTION_UNITARY", "QM_DYNAMICS", "THEOREM", "STANDARD_RESULT",
  "H が自己共役で時間に依存しなければ U(t) = exp(−iHt/ħ) はユニタリで ψ(t) = U(t)ψ(0) はSchrödinger方程式を解く。U(t)U(s) = U(t+s)。",
  premises=("H† = H",), deps=("QM_POSTULATE_SCHRODINGER", "QM_HERMITIAN_REAL_SPECTRUM"),
  symbols=("time_evolution", "hamiltonian", "unitary"), goals=("QUANTUM_MECHANICS", "DYNAMICS"),
  note="有限次元では行列指数関数とスペクトル分解による。", reference=TEXTBOOK,
  gke=rule(["H", "U"], [("HAMILTONIAN", "H"), ("GENERATES", "H", "U")], [("UNITARY", "U")]))
k("QM_STATIONARY_STATE", "QM_DYNAMICS", "THEOREM", "STANDARD_RESULT",
  "Hψ = Eψ なら ψ(t) = e^{−iEt/ħ}ψ で、あらゆる測定確率は時間に依存しない（定常状態）。",
  premises=("H ψ = E ψ",), deps=("QM_TIME_EVOLUTION_UNITARY", "QM_GLOBAL_PHASE_SAME_DENSITY"),
  symbols=("stationary_state", "energy_eigenstate"), goals=("QUANTUM_MECHANICS", "DYNAMICS"),
  note="時間依存性は大域位相だけなので密度行列が不変。", reference=TEXTBOOK)
k("QM_EHRENFEST", "QM_DYNAMICS", "THEOREM", "STANDARD_RESULT",
  "時間に陽に依存しない観測量 A について d⟨A⟩/dt = (i/ħ)⟨[H, A]⟩。",
  deps=("QM_TIME_EVOLUTION_UNITARY",), symbols=("ehrenfest", "commutator", "expectation_value"),
  goals=("QUANTUM_MECHANICS", "DYNAMICS"), note="Heisenberg描像の運動方程式の期待値版。", reference=TEXTBOOK)
k("QM_CONSERVED_QUANTITY", "QM_DYNAMICS", "THEOREM", "STANDARD_RESULT",
  "[H, A] = 0 で A が時間に陽に依存しなければ ⟨A⟩ と A の測定確率分布は保存される。",
  premises=("[H, A] = 0",), deps=("QM_EHRENFEST", "QM_COMMUTING_OBSERVABLES_COMPATIBLE"),
  symbols=("conserved_quantity", "commutator", "hamiltonian"), goals=("QUANTUM_MECHANICS", "DYNAMICS"),
  note="Ehrenfestの定理の帰結。", reference=TEXTBOOK,
  gke=rule(["H", "A"], [("HAMILTONIAN", "H"), ("COMMUTE", "H", "A")], [("CONSERVED", "A")]))
k("QM_RATIONAL_UNITARY_EXAMPLE", "QM_DYNAMICS", "THEOREM", "EXACT_FINITE_CHECKED",
  "U = (1/5)[[3, 4i],[4i, 3]] は U†U = UU† = I を満たし、ψ = (3/5, 4i/5) に作用させると Uψ = (−7/25, 24i/25) で ‖Uψ‖² = 1。",
  deps=("QM_UNITARY_PRESERVES_NORM",), symbols=("unitary", "normalization"),
  goals=("QUANTUM_MECHANICS", "DYNAMICS"), check="rational_unitary_example",
  note="ユニタリ性とノルム保存の厳密な具体例。")
k("QM_HADAMARD_GATE", "QM_DYNAMICS", "IDENTITY", "EXACT_FINITE_CHECKED",
  "H = (1/√2)[[1,1],[1,−1]] は H† = H, H² = I、H|0⟩ = |+⟩、H σx H = σz、H σz H = σx。",
  deps=("QM_PAULI_HERMITIAN_UNITARY",), symbols=("hadamard", "quantum_gate"),
  goals=("QUANTUM_MECHANICS", "DYNAMICS", "QUANTUM_COMPUTING"), check="hadamard_gate",
  note="√2 因子は H' = √2 H を有理行列で持ち、H'² = 2I などで厳密に検査。USM_U06_FORWARD とも整合。")
k("QM_PHASE_GATE", "QM_DYNAMICS", "IDENTITY", "EXACT_FINITE_CHECKED",
  "位相ゲート S = diag(1, i) はユニタリで S² = σz、S σx S† = σy。",
  deps=("QM_PAULI_PRODUCT",), symbols=("phase_gate", "quantum_gate"),
  goals=("QUANTUM_MECHANICS", "DYNAMICS", "QUANTUM_COMPUTING"), check="phase_gate", note="Clifford群の生成元の1つ。")
k("QM_CNOT_GATE", "QM_DYNAMICS", "IDENTITY", "EXACT_FINITE_CHECKED",
  "CNOT = |0⟩⟨0| ⊗ I + |1⟩⟨1| ⊗ σx はエルミートかつユニタリで CNOT² = I。",
  deps=("QM_PAULI_HERMITIAN_UNITARY",), symbols=("cnot", "quantum_gate"),
  goals=("QUANTUM_MECHANICS", "DYNAMICS", "QUANTUM_COMPUTING"), check="cnot_gate", note="2量子ビットのエンタングリングゲート。")

# ---------------------------------------------------------------- Stage 3: uncertainty
k("QM_ROBERTSON_UNCERTAINTY", "QM_UNCERTAINTY", "THEOREM", "STANDARD_RESULT",
  "エルミートな A, B と正規化された ψ について σ_A σ_B ≥ |⟨[A, B]⟩| / 2（Robertsonの不確定性関係）。",
  premises=("A† = A", "B† = B", "‖ψ‖ = 1"), deps=("UNI_CAUCHY_SCHWARZ", "QM_EXPECTATION_REAL"),
  symbols=("uncertainty", "standard_deviation", "commutator"), goals=("QUANTUM_MECHANICS", "UNCERTAINTY"),
  note="Cauchy–Schwarz（UNI_CAUCHY_SCHWARZ はLean検査済み）からの標準的導出。全体はこのパックでは未検査。",
  reference=TEXTBOOK)
k("QM_ROBERTSON_PAULI_EXAMPLE", "QM_UNCERTAINTY", "THEOREM", "EXACT_FINITE_CHECKED",
  "A = σx, B = σy で、検査した各有理純粋状態について Var(σx)·Var(σy) ≥ |⟨[σx, σy]⟩|²/4 = ⟨σz⟩²。例: ψ = (3/5, 4i/5) では Var(σx) = 1, Var(σy) = 49/625 で等号成立。",
  deps=("QM_ROBERTSON_UNCERTAINTY", "QM_PAULI_COMMUTATOR"), symbols=("uncertainty", "pauli_matrix"),
  goals=("QUANTUM_MECHANICS", "UNCERTAINTY"), check="robertson_pauli_example",
  note="2乗形で有限個の状態について厳密に検査。一般証明の代用ではない。")
k("QM_HEISENBERG_UNCERTAINTY", "QM_UNCERTAINTY", "THEOREM", "STANDARD_RESULT",
  "位置と運動量について σ_x σ_p ≥ ħ/2。",
  deps=("QM_ROBERTSON_UNCERTAINTY", "QM_CANONICAL_COMMUTATION"), symbols=("uncertainty", "position", "momentum"),
  goals=("QUANTUM_MECHANICS", "UNCERTAINTY"),
  note="Robertson関係に [x̂, p̂] = iħ を代入。非有界作用素の定義域の扱いは省略している。", reference=TEXTBOOK)
k("QM_CCR_NO_FINITE_DIMENSION", "QM_UNCERTAINTY", "THEOREM", "STANDARD_RESULT",
  "n ≥ 1 の n×n 行列 X, P で XP − PX = cI（c ≠ 0）となるものは存在しない。tr(XP − PX) = 0 だが tr(cI) = cn ≠ 0。",
  premises=("n ≥ 1", "c ≠ 0"), deps=("UNI_TRACE_CYCLIC", "QM_CANONICAL_COMMUTATION"),
  symbols=("commutator", "trace", "canonical_commutation"), goals=("QUANTUM_MECHANICS", "UNCERTAINTY"),
  note="証明はトレースの巡回性（UNI_TRACE_CYCLIC はLean検査済み）の1行の帰結だが、本命題自体はLean未検査。正準交換関係には無限次元が必要。",
  reference=TEXTBOOK)

# ---------------------------------------------------------------- Stage 3: composite systems
k("QM_PRODUCT_STATE_NORMALIZED", "QM_COMPOSITE", "THEOREM", "ANCHORED_TO_KERNEL_CHECKED",
  "‖x‖ = ‖y‖ = 1 なら ‖x ⊗ y‖ = 1（一般に ‖x ⊗ y‖ = ‖x‖‖y‖）。",
  premises=("‖x‖ = 1", "‖y‖ = 1"), deps=("USM_R01", "USM_P0_05", "QM_POSTULATE_COMPOSITE"),
  symbols=("tensor_product", "product_state"), goals=("QUANTUM_MECHANICS", "COMPOSITE"),
  anchor="USM_R01", note="数学的内容は USM_R01, USM_P0_05（Lean検査済み）。")
k("QM_BELL_BASIS", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "Bell状態 Φ± = (|00⟩ ± |11⟩)/√2, Ψ± = (|01⟩ ± |10⟩)/√2 は C²⊗C² の正規直交基底をなす。",
  deps=("QM_POSTULATE_COMPOSITE",), symbols=("bell_state", "entanglement"),
  goals=("QUANTUM_MECHANICS", "COMPOSITE"), check="bell_basis",
  note="√2 倍したベクトルの内積が 2δ になることを厳密に検査。")
k("QM_BELL_CIRCUIT", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "CNOT (H ⊗ I) |00⟩ = Φ+。",
  deps=("QM_HADAMARD_GATE", "QM_CNOT_GATE", "QM_BELL_BASIS"), symbols=("bell_state", "quantum_circuit"),
  goals=("QUANTUM_MECHANICS", "COMPOSITE", "QUANTUM_COMPUTING"), check="bell_circuit", note="エンタングル状態の標準的な生成回路。")
k("QM_BELL_ENTANGLED", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "ρ = |Φ+⟩⟨Φ+| は tr ρ² = 1（全体は純粋）だが、部分トレース tr_B ρ = I/2 で tr(ρ_A²) = 1/2。係数行列の行列式は 1/2 ≠ 0 なので Φ+ は積状態 a ⊗ b ではない。",
  deps=("QM_BELL_BASIS", "QM_MAXIMALLY_MIXED_NOT_PURE"), symbols=("entanglement", "partial_trace", "purity"),
  goals=("QUANTUM_MECHANICS", "COMPOSITE"), check="bell_entangled",
  note="2量子ビット純粋状態 Σ c_ij|ij⟩ は det(c) = 0 ⇔ 積状態（Schmidt階数1）という標準的判定を使う。")
k("QM_SUPERDENSE_CODING", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "(σ ⊗ I)Φ+ は σ = I, σx, iσy, σz に対してそれぞれ Φ+, Ψ+, Ψ−, Φ−（符号は大域位相）を与え、4状態は直交する（超高密度符号化）。",
  deps=("QM_BELL_BASIS", "QM_PAULI_HERMITIAN_UNITARY", "QM_ORTHOGONAL_STATES_DISTINGUISHABLE"),
  symbols=("superdense_coding", "bell_state"), goals=("QUANTUM_MECHANICS", "COMPOSITE", "QUANTUM_COMPUTING"),
  check="superdense_coding", note="局所操作で4つの直交Bell状態を作れることの厳密な検査。")
k("QM_NO_CLONING", "QM_COMPOSITE", "THEOREM", "STANDARD_RESULT",
  "任意の未知状態 ψ に対し U(ψ ⊗ e) = ψ ⊗ ψ となるユニタリ U は存在しない（ψ が2つ以上の非直交・非同一状態を含む集合を動く場合）。",
  deps=("QM_UNITARY_PRESERVES_NORM", "QM_POSTULATE_COMPOSITE"), symbols=("no_cloning",),
  goals=("QUANTUM_MECHANICS", "COMPOSITE", "QUANTUM_COMPUTING"),
  note="内積の保存 ⟨ψ|φ⟩ = ⟨ψ|φ⟩² から ⟨ψ|φ⟩ ∈ {0, 1}。具体例は QM_NO_CLONING_CNOT_EXAMPLE。", reference=TEXTBOOK)
k("QM_NO_CLONING_CNOT_EXAMPLE", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "CNOT は |0⟩|0⟩ → |0⟩|0⟩, |1⟩|0⟩ → |1⟩|1⟩ と基底状態を複製するが、|+⟩|0⟩ は Φ+ に写り |+⟩|+⟩ にならない（密度行列が異なる）。",
  deps=("QM_NO_CLONING", "QM_BELL_CIRCUIT"), symbols=("no_cloning", "cnot"),
  goals=("QUANTUM_MECHANICS", "COMPOSITE", "QUANTUM_COMPUTING"), check="no_cloning_cnot_example",
  note="線形性により複製は基底状態でしか成り立たない具体例。")
k("QM_CHSH_LOCAL_BOUND", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "±1 の決定論的な値 a0, a1, b0, b1 について S = a0b0 + a0b1 + a1b0 − a1b1 ∈ {−2, 2}。したがって局所隠れた変数模型では |⟨S⟩| ≤ 2。",
  symbols=("chsh", "bell_inequality", "local_hidden_variable"), goals=("QUANTUM_MECHANICS", "COMPOSITE"),
  check="chsh_local_bound", note="16通りを全数検査。確率混合への拡張は凸性による。")
k("QM_CHSH_QUANTUM_VIOLATION", "QM_COMPOSITE", "THEOREM", "EXACT_FINITE_CHECKED",
  "Φ+ と A0 = σz, A1 = σx, B0 = (σz + σx)/√2, B1 = (σz − σx)/√2 で ⟨S⟩ = 2√2 > 2（⟨S⟩² = 8 を厳密に検査）。2√2 はTsirelson限界。",
  deps=("QM_CHSH_LOCAL_BOUND", "QM_BELL_ENTANGLED"), symbols=("chsh", "bell_inequality", "tsirelson_bound"),
  goals=("QUANTUM_MECHANICS", "COMPOSITE"), check="chsh_quantum_violation",
  note="量子力学は局所隠れた変数模型で再現できない相関を予言する。Tsirelson限界の最適性自体は未検査。")

# ---------------------------------------------------------------- Stage 4: models
k("QM_HARMONIC_OSCILLATOR", "QM_MODELS", "THEOREM", "STANDARD_RESULT",
  "H = p²/(2m) + mω²x²/2 の固有値は E_n = ħω(n + 1/2)（n = 0, 1, 2, …）。a = √(mω/2ħ)(x + ip/(mω)) について [a, a†] = 1, H = ħω(a†a + 1/2)。",
  deps=("QM_CANONICAL_COMMUTATION",), symbols=("harmonic_oscillator", "ladder_operator", "energy_level"),
  goals=("QUANTUM_MECHANICS", "SPECTRUM"), note="零点エネルギー ħω/2 を持つ。", reference=TEXTBOOK)
k("QM_INFINITE_SQUARE_WELL", "QM_MODELS", "THEOREM", "STANDARD_RESULT",
  "幅 L の無限井戸型ポテンシャル中の粒子は E_n = n²π²ħ²/(2mL²)、ψ_n(x) = √(2/L) sin(nπx/L)（n = 1, 2, …）。",
  deps=("QM_POSTULATE_SCHRODINGER",), symbols=("particle_in_box", "energy_level"),
  goals=("QUANTUM_MECHANICS", "SPECTRUM"), note="境界条件 ψ(0) = ψ(L) = 0 による量子化。", reference=TEXTBOOK)
k("QM_HYDROGEN_SPECTRUM", "QM_MODELS", "THEOREM", "STANDARD_RESULT",
  "非相対論的水素原子の束縛状態のエネルギーは E_n = −m_e e⁴ / (2(4πε₀)²ħ² n²) ≈ −13.6057 eV / n²、縮退度 n²（スピンを除く）。",
  deps=("QM_POSTULATE_SCHRODINGER", "QM_ANGULAR_MOMENTUM_ALGEBRA"), symbols=("hydrogen_atom", "energy_level"),
  goals=("QUANTUM_MECHANICS", "SPECTRUM"),
  note="微細構造・Lamb シフト・換算質量補正は含まない。", reference=TEXTBOOK)


def write(path, text):
    path.write_text(text, encoding="utf-8")


def main():
    ids = [u["knowledge_id"] for u in UNITS]
    assert len(ids) == len(set(ids))
    write(PACK / "catalogue/quantum_mechanics.json", json.dumps(UNITS, ensure_ascii=False, indent=1) + "\n")
    labels = {"EXACT_FINITE_CHECKED": "厳密計算で検査", "ANCHORED_TO_KERNEL_CHECKED": "Lean検査済みUSM命題に依拠",
              "POSTULATE": "物理的公理", "SI_DEFINITION": "SI定義", "STANDARD_RESULT": "標準的結果（未検査）"}
    lines = ["# 量子力学Knowledge索引 v0.1", "",
             "生成元: `tools/build_catalogue.py`。数式中の ħ を省いた行列は ħ = 1 単位。", ""]
    for sub, (stage, title) in STAGES.items():
        lines += [f"## 段階{stage}: {title}（`{sub}`）", "", "| ID | 種別 | 検証 | 内容 |", "|---|---|---|---|"]
        for u in UNITS:
            if u["subcategory"] == sub:
                m = u["activation_metadata"]
                extra = ""
                if "formal_anchor" in m:
                    extra = f"（アンカー: `{m['formal_anchor']['knowledge_id']}`）"
                if "exact_check" in m:
                    extra = f"（検査: `{m['exact_check']['check_id']}`）"
                stmt = u["conclusion"].replace("|", "\\|")
                lines.append(f"| `{u['knowledge_id']}` | {u['knowledge_type']} | {labels[m['verification_status']]}{extra} | {stmt} |")
        lines.append("")
    write(PACK / "catalogue/INDEX.ja.md", "\n".join(lines))
    files = sorted(p for p in PACK.rglob("*") if p.is_file() and p.name != "package_manifest.json")
    manifest = {"version": VERSION,
                "files": {str(p.relative_to(PACK)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
    write(PACK / "package_manifest.json", json.dumps(manifest, indent=2) + "\n")
    print(f"{len(UNITS)} units")


if __name__ == "__main__":
    main()
