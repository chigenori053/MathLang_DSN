use mathlang_mirp::calculation::{
    calculation_knowledge, compute, compute_with_knowledge, verify_answer, CalculationAnswer as A,
    CalculationRequest as R,
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use mathlang_mirp::{Object, SemanticState};
use serde_json::{json, Value};

const OPS: [&str; 6] = [
    "normalize",
    "inner_product",
    "born",
    "expectation",
    "density_matrix",
    "unitary",
];
fn q(n: i64, d: i64) -> Value {
    let (mut a, mut b) = (n.abs(), d);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let divisor = if d == 0 { 1 } else { a };
    json!({"numerator":(n/divisor).to_string(),"denominator":(d/divisor).to_string()})
}
fn c(re: i64, im: i64, d: i64) -> Value {
    json!({"real":q(re,d),"imaginary":q(im,d)})
}
fn z() -> Value {
    c(0, 0, 1)
}
fn fixture(op: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/fixtures/calculation/quantum_{op}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
fn request(v: Value) -> R {
    serde_json::from_value(v).unwrap()
}
fn outcome(v: Value) -> Value {
    let r = request(v);
    let result = compute(&r).unwrap();
    verify_answer(&r, &result.answer).unwrap();
    serde_json::to_value(result.answer).unwrap()["result"]["outcome"].clone()
}
fn operator(entries: Value) -> Value {
    json!({"basis_id":"qubit_z","entries":entries})
}
fn mixed(entries: Value) -> Value {
    json!({"kind":"MIXED","basis_id":"qubit_z","entries":entries})
}
fn diagonal_mixture() -> Value {
    mixed(json!([[c(1, 0, 4), z()], [z(), c(3, 0, 4)]]))
}
fn reject(v: Value) {
    let r = request(v);
    assert!(r
        .validate()
        .unwrap_err()
        .starts_with("CAPABILITY_INPUT_INVALID"));
    assert!(compute(&r)
        .unwrap_err()
        .starts_with("CAPABILITY_INPUT_INVALID"));
}

#[test]
fn all_six_knowledge_units_select_verify_explain_and_persist_evidence() {
    for op in OPS {
        let r = request(fixture(op));
        let id = format!("QUANTUM_{}", op.to_uppercase());
        let result = compute(&r).unwrap();
        assert_eq!(result.trace["knowledge_id"], id);
        assert_eq!(
            result
                .knowledge_decision
                .activation
                .as_ref()
                .unwrap()
                .knowledge_id,
            id
        );
        assert_eq!(result.verification["status"], "VERIFIED");
        assert_eq!(result.verification["formal_status"], "NOT_CHECKED");
        assert_eq!(
            result.verification["method"],
            "EXACT_COMPLEX_IDENTITY_CHECK"
        );
        assert!(result.explanation[0].contains(&id));
        assert!(result.explanation.iter().any(|line| line.contains("共役")));
        verify_answer(&r, &result.answer).unwrap();
        result.semantic_state.validate().unwrap();
        assert!(result.semantic_state.objects.iter().any(|o|matches!(o,Object::Evidence(e) if e.common.metadata["knowledge_runtime"]["knowledge_id"]==id&&e.source.contains("finite quantum"))));
        let restored =
            SemanticState::from_json(&serde_json::to_string(&result.semantic_state).unwrap())
                .unwrap();
        assert_eq!(
            restored.canonical().unwrap(),
            result.semantic_state.canonical().unwrap()
        );
        let mut space = calculation_knowledge();
        let mut unit = space.get(&id).unwrap().clone();
        unit.knowledge_id = format!("CUSTOM_{id}");
        unit.priority += 1;
        space.register_batch(vec![unit.clone()]).unwrap();
        let custom = compute_with_knowledge(&r, &space).unwrap();
        assert_eq!(custom.answer, result.answer);
        assert_eq!(custom.trace["knowledge_id"], unit.knowledge_id);
        assert_eq!(
            compute_with_knowledge(&r, &KnowledgeSpace::empty()).unwrap_err(),
            "NO_APPLICABLE_KNOWLEDGE"
        );
        let mut conflict = unit.clone();
        conflict.knowledge_id = format!("CONFLICT_{id}");
        conflict
            .activation_metadata
            .insert("conflicts_with".into(), json!([unit.knowledge_id]));
        space.register_batch(vec![conflict]).unwrap();
        assert_eq!(
            compute_with_knowledge(&r, &space).unwrap_err(),
            "KNOWLEDGE_CONFLICT"
        );
    }
}

#[test]
fn normalization_preserves_complex_phase_and_feeds_other_operations() {
    let out = outcome(fixture("normalize"));
    assert_eq!(out["original_norm_squared"], q(2, 1));
    assert_eq!(out["norm_squared"], q(1, 1));
    assert_eq!(out["state"]["squared_scale"], q(1, 2));
    assert_eq!(out["state"]["amplitudes"], json!([c(1, 0, 1), c(0, 1, 1)]));
    for op in ["born", "expectation", "density_matrix", "unitary"] {
        let mut v = fixture(op);
        v["state"] = out["state"].clone();
        v["state"]["kind"] = json!("PURE");
        outcome(v);
    }
    let mut v = fixture("normalize");
    v["state"]["amplitudes"] = json!([c(-3, 0, 1), c(0, 4, 1)]);
    v["state"]["squared_scale"] = q(9, 4);
    let out = outcome(v);
    assert_eq!(out["original_norm_squared"], q(225, 4));
    assert_eq!(out["state"]["squared_scale"], q(1, 25));
    assert_eq!(out["state"]["amplitudes"][0], c(-3, 0, 1));
}

#[test]
fn inner_product_conjugates_the_first_argument_and_keeps_exact_root_scales() {
    let base = fixture("inner_product");
    let forward = outcome(base.clone());
    assert_eq!(forward["value"]["coefficient"], c(1, 1, 1));
    assert_eq!(forward["value"]["radicand"], q(1, 4));
    assert_eq!(forward["overlap_probability"], q(1, 2));
    let mut v = base.clone();
    v["left"] = base["right"].clone();
    v["right"] = base["left"].clone();
    assert_eq!(outcome(v)["value"]["coefficient"], c(1, -1, 1));
    let mut v = base.clone();
    v["left"]["amplitudes"] = json!([c(1, 0, 1), z()]);
    v["left"]["squared_scale"] = q(1, 1);
    v["right"]["amplitudes"] = json!([c(3, 0, 1), c(4, 0, 1)]);
    v["right"]["squared_scale"] = q(1, 25);
    let out = outcome(v);
    assert_eq!(out["value"]["coefficient"], c(3, 0, 1));
    assert_eq!(out["value"]["radicand"], q(1, 25));
    assert_eq!(out["overlap_probability"], q(9, 25));
    let mut v = base;
    v["left"]["amplitudes"] = json!([c(1, 0, 1), c(0, 1, 1)]);
    assert_eq!(outcome(v)["overlap_probability"], q(1, 1));
}

#[test]
fn born_probabilities_cover_complex_measurement_bases_and_mixed_states() {
    let base = fixture("born");
    assert_eq!(
        outcome(base.clone())["probabilities"],
        json!([q(1, 2), q(1, 2)])
    );
    let mut v = base.clone();
    v["state"] = diagonal_mixture();
    assert_eq!(outcome(v)["probabilities"], json!([q(1, 4), q(3, 4)]));
    let mut v = base.clone();
    v["projectors"][0]["operator"]["entries"] =
        json!([[c(1, 0, 2), c(0, -1, 2)], [c(0, 1, 2), c(1, 0, 2)]]);
    v["projectors"][1]["operator"]["entries"] =
        json!([[c(1, 0, 2), c(0, 1, 2)], [c(0, -1, 2), c(1, 0, 2)]]);
    assert_eq!(outcome(v)["probabilities"], json!([q(1, 1), q(0, 1)]));
    let mut v = base;
    v["projectors"] = json!([{"label":"entire_space","operator":operator(json!([[c(1,0,1),z()],[z(),c(1,0,1)]]))}]);
    let out = outcome(v);
    assert_eq!(out["labels"], json!(["entire_space"]));
    assert_eq!(out["probabilities"], json!([q(1, 1)]));
    assert_eq!(out["total_probability"], q(1, 1));
}

#[test]
fn hermitian_expectations_are_real_for_pure_and_mixed_states() {
    let base = fixture("expectation");
    assert_eq!(outcome(base.clone())["value"], q(1, 1));
    let mut v = base.clone();
    v["observable"]["entries"] = json!([[c(1, 0, 1), c(0, 1, 1)], [c(0, -1, 1), c(2, 0, 1)]]);
    assert_eq!(outcome(v)["value"], q(1, 2));
    let mut v = base.clone();
    v["state"] = diagonal_mixture();
    v["observable"]["entries"] = json!([[c(1, 0, 1), z()], [z(), c(-1, 0, 1)]]);
    assert_eq!(outcome(v)["value"], q(-1, 2));
    let mut v = base;
    v["state"]["amplitudes"] = json!([z(), c(1, 0, 1)]);
    v["state"]["squared_scale"] = q(1, 1);
    v["observable"]["entries"] = json!([[c(1, 0, 1), z()], [z(), c(-1, 0, 1)]]);
    assert_eq!(outcome(v)["value"], q(-1, 1));
}

#[test]
fn densities_use_conjugate_outer_products_and_certify_singular_psd_matrices() {
    let base = fixture("density_matrix");
    let out = outcome(base.clone());
    assert_eq!(
        out["state"]["entries"],
        json!([[c(1, 0, 2), c(0, -1, 2)], [c(0, 1, 2), c(1, 0, 2)]])
    );
    assert_eq!(out["trace"], q(1, 1));
    assert_eq!(out["purity"], q(1, 1));
    assert_eq!(out["pure"], true);
    assert_eq!(
        out["psd_certificate"]["diagonal"],
        json!([q(1, 2), q(0, 1)])
    );
    assert_eq!(out["psd_certificate"]["lower_triangular"][1][0], c(0, 1, 1));
    let mut v = base.clone();
    v["state"] = diagonal_mixture();
    let out = outcome(v);
    assert_eq!(out["purity"], q(5, 8));
    assert_eq!(out["pure"], false);
    for entries in [
        json!([[z(), z()], [z(), c(1, 0, 1)]]),
        json!([[c(1, 0, 1), z()], [z(), z()]]),
    ] {
        let mut v = base.clone();
        v["state"] = mixed(entries.clone());
        let out = outcome(v);
        assert_eq!(out["state"]["entries"], entries);
        assert_eq!(out["pure"], true);
    }
}

#[test]
fn unitary_hadamard_and_complex_nonsymmetric_maps_preserve_states_and_purity() {
    let base = fixture("unitary");
    let out = outcome(base.clone());
    assert_eq!(out["state"]["amplitudes"], json!([c(1, 1, 1), c(1, -1, 1)]));
    assert_eq!(out["state"]["squared_scale"], q(1, 4));
    assert_eq!(out["normalization"], q(1, 1));
    assert_eq!(out["purity"], q(1, 1));
    let mut born = fixture("born");
    born["state"] = out["state"].clone();
    assert_eq!(outcome(born)["probabilities"], json!([q(1, 2), q(1, 2)]));
    let mut v = base.clone();
    v["state"] = diagonal_mixture();
    let out = outcome(v);
    assert_eq!(
        out["state"]["entries"],
        json!([[c(1, 0, 2), c(-1, 0, 4)], [c(-1, 0, 4), c(1, 0, 2)]])
    );
    assert_eq!(out["purity"], q(5, 8));
    let mut v = base.clone();
    v["unitary"]["operator"]["entries"] = json!([[z(), c(0, 1, 1)], [c(1, 0, 1), z()]]);
    v["unitary"]["squared_scale"] = q(1, 1);
    let transformed = outcome(v.clone());
    assert_eq!(
        transformed["state"]["amplitudes"],
        json!([c(-1, 0, 1), c(1, 0, 1)])
    );
    v["state"] = transformed["state"].clone();
    v["unitary"]["operator"]["entries"] = json!([[z(), c(1, 0, 1)], [c(0, -1, 1), z()]]);
    let back = outcome(v);
    assert_eq!(back["state"]["amplitudes"], json!([c(1, 0, 1), c(0, 1, 1)]));
    let mut v = base.clone();
    v["state"] = outcome({
        let mut t = base.clone();
        t["state"] = diagonal_mixture();
        t
    })["state"]
        .clone();
    assert_eq!(
        outcome(v)["state"]["entries"],
        diagonal_mixture()["entries"]
    );
}

#[test]
fn dimensions_one_through_eight_and_degenerate_measurements_are_exact() {
    for n in 1..=8 {
        let mut normalize = fixture("normalize");
        normalize["basis"]["labels"] = json!((0..n).map(|i| i.to_string()).collect::<Vec<_>>());
        normalize["state"]["amplitudes"] = json!(vec![c(1, 0, 1); n]);
        let normalized = outcome(normalize.clone());
        assert_eq!(normalized["state"]["squared_scale"], q(1, n as i64));
        let mut state = normalized["state"].clone();
        state["kind"] = json!("PURE");
        let mut rho = fixture("density_matrix");
        rho["basis"] = normalize["basis"].clone();
        rho["state"] = state.clone();
        let out = outcome(rho);
        assert_eq!(out["pure"], true);
        assert_eq!(out["state"]["entries"].as_array().unwrap().len(), n);
        let mut born = fixture("born");
        born["basis"] = normalize["basis"].clone();
        born["state"] = state;
        let mut projectors = vec![];
        for k in 0..n {
            let entries: Vec<Vec<Value>> = (0..n)
                .map(|i| {
                    (0..n)
                        .map(|j| if i == k && j == k { c(1, 0, 1) } else { z() })
                        .collect()
                })
                .collect();
            projectors.push(json!({"label":k.to_string(),"operator":operator(json!(entries))}));
        }
        born["projectors"] = json!(projectors);
        let out = outcome(born);
        assert_eq!(out["probabilities"], json!(vec![q(1, n as i64); n]));
    }
    let mut v = fixture("born");
    v["basis"]["labels"] = json!(["00", "01", "10", "11"]);
    v["state"]["amplitudes"] = json!([c(1, 0, 1), z(), z(), c(1, 0, 1)]);
    let rank_two = |indices: Vec<usize>| -> Value {
        json!((0..4)
            .map(|i| (0..4)
                .map(|j| if i == j && indices.contains(&i) {
                    c(1, 0, 1)
                } else {
                    z()
                })
                .collect::<Vec<_>>())
            .collect::<Vec<_>>())
    };
    v["projectors"][0]["operator"]["entries"] = rank_two(vec![0, 1]);
    v["projectors"][1]["operator"]["entries"] = rank_two(vec![2, 3]);
    assert_eq!(outcome(v)["probabilities"], json!([q(1, 2), q(1, 2)]));
}

#[test]
fn invalid_density_matrices_fail_beyond_diagonal_and_trace_checks() {
    for entries in [
        json!([[c(1, 0, 2), c(1, 0, 1)], [c(1, 0, 1), c(1, 0, 2)]]),
        json!([[z(), c(0, 1, 1)], [c(0, -1, 1), c(1, 0, 1)]]),
        json!([[c(1, 0, 2), c(0, 1, 2)], [c(0, 1, 2), c(1, 0, 2)]]),
        json!([[c(1, 0, 1), z()], [z(), c(1, 0, 1)]]),
        json!([[c(-1, 0, 1), z()], [z(), c(2, 0, 1)]]),
        json!([[c(1, 1, 2), z()], [z(), c(1, -1, 2)]]),
    ] {
        for op in ["born", "expectation", "density_matrix", "unitary"] {
            let mut v = fixture(op);
            v["state"] = mixed(entries.clone());
            reject(v);
        }
    }
    let mut v = fixture("density_matrix");
    v["basis"]["labels"] = json!(["0", "1", "2"]);
    v["state"] = mixed(json!([
        [c(1, 0, 3), c(-1, 0, 4), c(-1, 0, 4)],
        [c(-1, 0, 4), c(1, 0, 3), c(-1, 0, 4)],
        [c(-1, 0, 4), c(-1, 0, 4), c(1, 0, 3)]
    ]));
    reject(v);
}

#[test]
fn invalid_measurements_observables_and_unitaries_are_rejected() {
    let base = fixture("born");
    for projectors in [
        json!([]),
        json!([base["projectors"][0]]),
        {
            let mut p = base["projectors"].clone();
            p[1]["operator"] = p[0]["operator"].clone();
            p
        },
        {
            let mut p = base["projectors"].clone();
            p[1]["label"] = p[0]["label"].clone();
            p
        },
        {
            let mut p = base["projectors"].clone();
            p[0]["operator"]["entries"] = json!([[c(1, 0, 2), z()], [z(), z()]]);
            p
        },
        {
            let mut p = base["projectors"].clone();
            p[0]["operator"]["entries"] = json!([[c(1, 0, 1), c(1, 0, 1)], [z(), z()]]);
            p
        },
        {
            let mut p = base["projectors"].clone();
            p[0]["operator"]["entries"] = json!([[z(), z()], [z(), z()]]);
            p
        },
    ] {
        let mut v = base.clone();
        v["projectors"] = projectors;
        reject(v);
    }
    for entries in [
        json!([[z(), c(0, 1, 1)], [c(0, 1, 1), z()]]),
        json!([[c(0, 1, 1), z()], [z(), z()]]),
    ] {
        let mut v = fixture("expectation");
        v["observable"]["entries"] = entries;
        reject(v);
    }
    for entries in [
        json!([[c(1, 0, 1), c(1, 0, 1)], [z(), z()]]),
        json!([[c(1, 0, 1), z()], [z(), c(2, 0, 1)]]),
    ] {
        let mut v = fixture("unitary");
        v["unitary"]["operator"]["entries"] = entries;
        reject(v);
    }
    for scale in [q(0, 1), q(-1, 1), q(1, 1)] {
        let mut v = fixture("unitary");
        v["unitary"]["squared_scale"] = scale;
        reject(v);
    }
}

#[test]
fn normalization_basis_shape_and_numeric_contracts_are_enforced() {
    for op in OPS {
        let base = fixture(op);
        for basis in [
            json!({"id":"","labels":["0","1"]}),
            json!({"id":"qubit_z","labels":[]}),
            json!({"id":"qubit_z","labels":["0","0"]}),
            json!({"id":"qubit_z","labels":["0",""]}),
            json!({"id":"qubit_z","labels":(0..33).map(|i| i.to_string()).collect::<Vec<_>>()}),
        ] {
            let mut v = base.clone();
            v["basis"] = basis;
            reject(v);
        }
        let statekey = if op == "inner_product" {
            "left"
        } else {
            "state"
        };
        for scale in [
            q(0, 1),
            q(-1, 1),
            q(1, 0),
            json!({"numerator":"1".repeat(81),"denominator":"1"}),
        ] {
            let mut v = base.clone();
            v[statekey]["squared_scale"] = scale;
            reject(v);
        }
        let mut v = base.clone();
        v[statekey]["basis_id"] = json!("different_basis");
        reject(v);
        let mut v = base.clone();
        v[statekey]["amplitudes"] = json!([z(), z()]);
        reject(v);
        let mut v = base.clone();
        v[statekey]["amplitudes"] = json!([c(1, 0, 1)]);
        reject(v);
        if op != "normalize" {
            let mut v = base.clone();
            v[statekey]["squared_scale"] = q(2, 1);
            reject(v);
        }
        let mut v = base;
        v[statekey]["amplitudes"][0]["real"] = json!(0.1);
        assert!(serde_json::from_value::<R>(v).is_err());
    }
    for (op, key) in [("expectation", "observable"), ("unitary", "unitary")] {
        let mut v = fixture(op);
        let m = if op == "unitary" {
            &mut v[key]["operator"]
        } else {
            &mut v[key]
        };
        m["basis_id"] = json!("other");
        reject(v);
    }
    for entries in [
        json!([]),
        json!([[c(1, 0, 1)], [z(), c(1, 0, 1)]]),
        json!([[c(1, 0, 1), z(), z()], [z(), c(1, 0, 1), z()]]),
    ] {
        let mut v = fixture("expectation");
        v["observable"]["entries"] = entries;
        reject(v);
    }
    let mut v = fixture("normalize");
    v["state"]["amplitudes"][0]["extra"] = json!(true);
    assert!(serde_json::from_value::<R>(v).is_err());
}

#[test]
fn verification_detects_altered_numbers_phases_probabilities_and_psd_certificates() {
    for (op, pointer, replacement) in [
        ("normalize", "/state/squared_scale", q(1, 3)),
        ("normalize", "/original_norm_squared", q(99, 1)),
        ("normalize", "/norm_squared", q(2, 1)),
        ("normalize", "/state/amplitudes/1/imaginary", q(-1, 1)),
        ("inner_product", "/value/coefficient/imaginary", q(-1, 1)),
        ("inner_product", "/value/radicand", q(1, 2)),
        ("inner_product", "/overlap_probability", q(3, 4)),
        ("born", "/probabilities/0", q(1, 3)),
        ("born", "/total_probability", q(2, 1)),
        ("born", "/labels/0", json!("altered")),
        ("expectation", "/value", q(0, 1)),
        ("density_matrix", "/state/entries/0/1/imaginary", q(1, 2)),
        ("density_matrix", "/trace", q(2, 1)),
        ("density_matrix", "/purity", q(1, 2)),
        ("density_matrix", "/pure", json!(false)),
        ("density_matrix", "/psd_certificate/diagonal/0", q(-1, 2)),
        (
            "density_matrix",
            "/psd_certificate/lower_triangular/1/0/imaginary",
            q(-1, 1),
        ),
        ("unitary", "/state/amplitudes/0/imaginary", q(-1, 1)),
        ("unitary", "/state/squared_scale", q(1, 2)),
        ("unitary", "/normalization", q(0, 1)),
        ("unitary", "/purity", q(1, 2)),
    ] {
        let r = request(fixture(op));
        let mut v = serde_json::to_value(compute(&r).unwrap().answer).unwrap();
        let path = format!("/result/outcome{pointer}");
        *v.pointer_mut(&path).unwrap() = replacement;
        assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap())
            .unwrap_err()
            .starts_with("CAPABILITY_OUTPUT_INVALID"));
    }
    for op in OPS {
        let r = request(fixture(op));
        let mut v = serde_json::to_value(compute(&r).unwrap().answer).unwrap();
        v["result"]["basis"]["labels"] = json!(["1", "0"]);
        assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
    }
    let r = request(fixture("density_matrix"));
    let mut v = serde_json::to_value(compute(&r).unwrap().answer).unwrap();
    v["result"]["outcome"]["psd_certificate"]["diagonal"] = json!([]);
    assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
}

#[test]
#[ignore = "explicit 100k same-goal quantum candidate, concurrent computation and conflict gate"]
fn quantum_candidates_scale_and_concurrent_states_do_not_interfere() {
    let mut space = calculation_knowledge();
    let template = space.get("QUANTUM_BORN").unwrap().clone();
    let units = (1..100_000)
        .map(|i| {
            let mut u = template.clone();
            u.knowledge_id = format!("ZLOAD_{i:06}");
            u
        })
        .collect();
    space.register_batch(units).unwrap();
    let r = request(fixture("born"));
    let start = std::time::Instant::now();
    let result = compute_with_knowledge(&r, &space).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_secs(20));
    assert_eq!(
        result.knowledge_decision.retrieval.retrieval_trace.examined,
        100_000
    );
    assert_eq!(
        result
            .knowledge_decision
            .retrieval
            .retrieval_trace
            .truncated_count,
        0
    );
    assert_eq!(result.answer, compute(&r).unwrap().answer);
    drop(result);
    let parallel_start = std::time::Instant::now();
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|i| {
                let space = &space;
                scope.spawn(move || {
                    let mut v = fixture("born");
                    if i == 0 {
                        v["state"] = diagonal_mixture();
                    } else {
                        v["state"]["amplitudes"] = if i == 1 {
                            json!([c(1, 0, 1), z()])
                        } else if i == 2 {
                            json!([z(), c(1, 0, 1)])
                        } else {
                            json!([c(1, 0, 1), c(0, 1, 1)])
                        };
                        v["state"]["squared_scale"] = if i == 3 { q(1, 2) } else { q(1, 1) };
                    }
                    let r = request(v);
                    let expected = compute(&r).unwrap().answer;
                    let actual = compute_with_knowledge(&r, space).unwrap();
                    assert_eq!(actual.answer, expected);
                    verify_answer(&r, &actual.answer).unwrap();
                    assert_eq!(
                        actual.knowledge_decision.retrieval.retrieval_trace.examined,
                        100_000
                    );
                })
            })
            .collect::<Vec<_>>();
        for h in handles {
            h.join().unwrap();
        }
    });
    let parallel_elapsed = parallel_start.elapsed();
    assert!(parallel_elapsed < std::time::Duration::from_secs(30));
    let mut last = template;
    last.knowledge_id = "ZZZZ_QUANTUM_CONFLICT".into();
    last.activation_metadata
        .insert("conflicts_with".into(), json!(["QUANTUM_BORN"]));
    space.register_batch(vec![last]).unwrap();
    assert!(matches!(compute_with_knowledge(&r,&space),Err(e) if e=="KNOWLEDGE_CONFLICT"));
    println!("100000 quantum candidates: {} ms; four parallel states: {} ms; exact results and late conflict PASS",elapsed.as_millis(),parallel_elapsed.as_millis());
}

#[test]
fn pure_and_density_routes_agree_across_phases_measurements_and_observables() {
    let base = fixture("normalize");
    for k in -2..=2 {
        for l in -2..=2 {
            let mut input = base.clone();
            input["state"]["amplitudes"] = json!([c(1, k, 1), c(l, 1, 1)]);
            let normalized = outcome(input)["state"].clone();
            let mut pure = normalized;
            pure["kind"] = json!("PURE");
            let mut density_request = fixture("density_matrix");
            density_request["state"] = pure.clone();
            let density = outcome(density_request);
            let mut mixed = density["state"].clone();
            mixed["kind"] = json!("MIXED");
            for op in ["born", "expectation"] {
                let mut v = fixture(op);
                v["state"] = pure.clone();
                let pure_result = outcome(v.clone());
                v["state"] = mixed.clone();
                assert_eq!(pure_result, outcome(v));
            }
            let mut v = fixture("unitary");
            v["state"] = pure;
            let pure_output = outcome(v.clone())["state"].clone();
            v["state"] = mixed;
            let mixed_output = outcome(v)["state"].clone();
            let mut density_request = fixture("density_matrix");
            density_request["state"] = pure_output;
            assert_eq!(
                outcome(density_request)["state"]["entries"],
                mixed_output["entries"]
            );
        }
    }
}

#[test]
fn full_rank_complex_densities_and_maximally_mixed_boundary_are_certified() {
    let mut v = fixture("density_matrix");
    v["state"] = mixed(json!([[c(5, 0, 6), c(0, -1, 6)], [c(0, 1, 6), c(1, 0, 6)]]));
    let out = outcome(v);
    assert_eq!(out["purity"], q(7, 9));
    assert_eq!(
        out["psd_certificate"]["diagonal"],
        json!([q(5, 6), q(2, 15)])
    );
    assert_eq!(out["psd_certificate"]["lower_triangular"][1][0], c(0, 1, 5));
    let n = 8;
    let entries: Vec<Vec<Value>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { c(1, 0, n) } else { z() })
                .collect()
        })
        .collect();
    let mut v = fixture("density_matrix");
    v["basis"]["labels"] = json!((0..n).map(|i| i.to_string()).collect::<Vec<_>>());
    v["state"] = mixed(json!(entries));
    let out = outcome(v);
    assert_eq!(out["trace"], q(1, 1));
    assert_eq!(out["purity"], q(1, n));
    assert_eq!(
        out["psd_certificate"]["diagonal"],
        json!(vec![q(1, n); n as usize])
    );
}
