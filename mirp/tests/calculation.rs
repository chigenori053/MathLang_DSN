use mathlang_mirp::calculation::{
    calculation_knowledge, compute, compute_with_knowledge, verify_answer, CalculationAnswer,
    CalculationRequest, Rational,
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use mathlang_mirp::memory::MemorySpace;
use mathlang_mirp::{Object, SemanticState};
use serde_json::json;

fn checked(request: CalculationRequest) -> mathlang_mirp::calculation::CalculationResult {
    let result = compute(&request).unwrap();
    assert_eq!(result.verification["status"], "VERIFIED");
    assert_eq!(result.verification["formal_status"], "NOT_CHECKED");
    assert!(!result.explanation.is_empty());
    assert_eq!(result.trace["result"], json!(result.answer));
    assert_eq!(result.trace["verification"], result.verification);
    let activation = result.knowledge_decision.activation.as_ref().unwrap();
    assert_eq!(
        result.trace["ruo_ref"],
        *activation.ruo_ref.as_ref().unwrap()
    );
    assert_eq!(
        activation.result_state.as_ref().unwrap(),
        &mathlang_mirp::stable_id("state", &[&result.semantic_state.canonical().unwrap()])
    );
    assert!(result.semantic_state.objects.iter().any(|object| {
        matches!(object, Object::Evidence(e) if e.common.metadata["knowledge_runtime"]["ruo_ref"] == result.trace["ruo_ref"])
    }));
    result.semantic_state.validate().unwrap();
    let restored =
        SemanticState::from_json(&serde_json::to_string(&result.semantic_state).unwrap()).unwrap();
    assert_eq!(
        result.semantic_state.canonical().unwrap(),
        restored.canonical().unwrap()
    );
    result
}

#[test]
fn principal_square_roots_are_exact_and_canonical() {
    for (n, coefficient, radicand) in [
        (0, 0, 1),
        (1, 1, 1),
        (2, 1, 2),
        (4, 2, 1),
        (72, 6, 2),
        (200_000_000, 10_000, 2),
    ] {
        let request = CalculationRequest::SimplifySqrt { radicand: n };
        let result = checked(request.clone());
        assert_eq!(
            result.answer,
            CalculationAnswer::Radical {
                coefficient,
                radicand
            }
        );
        assert_eq!(result.trace["knowledge_id"], "EXACT_SQUARE_ROOT");
        assert!(result.trace["execution_metadata"]["expression"]
            .as_str()
            .unwrap()
            .starts_with("JuniorHigh::JSimplifySqrt"));
        for bad in [
            CalculationAnswer::Radical {
                coefficient: -1,
                radicand,
            },
            CalculationAnswer::Radical {
                coefficient: i64::MAX,
                radicand: i64::MAX,
            },
        ] {
            assert!(verify_answer(&request, &bad).is_err());
        }
    }
    assert!(verify_answer(
        &CalculationRequest::SimplifySqrt { radicand: 72 },
        &CalculationAnswer::Radical {
            coefficient: 3,
            radicand: 8
        }
    )
    .is_err());
}

#[test]
fn summaries_cover_singletons_unsorted_odd_even_and_duplicates() {
    for (values, expected) in [
        (
            vec![3, 1, 2],
            json!({"kind":"SUMMARY","count":3,"mean":{"numerator":2,"denominator":1},"median":{"numerator":2,"denominator":1},"lower_quartile":{"numerator":1,"denominator":1},"upper_quartile":{"numerator":3,"denominator":1},"range":2}),
        ),
        (
            vec![-7],
            json!({"kind":"SUMMARY","count":1,"mean":{"numerator":-7,"denominator":1},"median":{"numerator":-7,"denominator":1},"lower_quartile":{"numerator":-7,"denominator":1},"upper_quartile":{"numerator":-7,"denominator":1},"range":0}),
        ),
        (
            vec![4, 1, 3, 2],
            json!({"kind":"SUMMARY","count":4,"mean":{"numerator":5,"denominator":2},"median":{"numerator":5,"denominator":2},"lower_quartile":{"numerator":3,"denominator":2},"upper_quartile":{"numerator":7,"denominator":2},"range":3}),
        ),
        (
            vec![2, 2, 2],
            json!({"kind":"SUMMARY","count":3,"mean":{"numerator":2,"denominator":1},"median":{"numerator":2,"denominator":1},"lower_quartile":{"numerator":2,"denominator":1},"upper_quartile":{"numerator":2,"denominator":1},"range":0}),
        ),
    ] {
        let request = CalculationRequest::DescribeData { values };
        let result = checked(request.clone());
        assert_eq!(json!(result.answer), expected);
        assert_eq!(result.trace["knowledge_id"], "EXACT_DATA_SUMMARY");
        let mut corrupted = expected;
        corrupted["median"]["numerator"] = json!(99);
        assert!(verify_answer(&request, &serde_json::from_value(corrupted).unwrap()).is_err());
    }
}

#[test]
fn variance_distinguishes_population_sample_and_undefined() {
    for (values, mean, population, sample) in [
        (vec![1, 2, 3], (2, 1), (2, 3), Some((1, 1))),
        (vec![-1, 0, 1], (0, 1), (2, 3), Some((1, 1))),
        (
            vec![999_999, 1_000_000],
            (1_999_999, 2),
            (1, 4),
            Some((1, 2)),
        ),
        (vec![7], (7, 1), (0, 1), None),
        (vec![42; 64], (42, 1), (0, 1), Some((0, 1))),
        (
            (0..64)
                .map(|i| if i == 63 { 1_000_000 } else { 999_999 })
                .collect(),
            (63_999_937, 64),
            (63, 4096),
            Some((1, 64)),
        ),
        (
            (0..64)
                .map(|i| if i % 2 == 0 { -1_000_000 } else { 1_000_000 })
                .collect(),
            (0, 1),
            (1_000_000_000_000, 1),
            Some((64_000_000_000_000, 63)),
        ),
    ] {
        let result = checked(CalculationRequest::DataVariance {
            values: values.clone(),
        });
        let fraction = |(numerator, denominator)| Rational {
            numerator,
            denominator,
        };
        assert_eq!(
            result.answer,
            CalculationAnswer::Variance {
                count: values.len(),
                mean: fraction(mean),
                population_variance: fraction(population),
                sample_variance: sample.map(fraction)
            }
        );
        assert_eq!(result.trace["knowledge_id"], "EXACT_DATA_VARIANCE");
        let mut corrupted = json!(result.answer);
        corrupted["population_variance"]["denominator"] = json!(0);
        assert!(
            verify_answer(&result.request, &serde_json::from_value(corrupted).unwrap()).is_err()
        );
        let mut corrupted = json!(result.answer);
        corrupted["sample_variance"] = if sample.is_none() {
            json!({"numerator":0,"denominator":1})
        } else {
            json!(null)
        };
        assert!(
            verify_answer(&result.request, &serde_json::from_value(corrupted).unwrap()).is_err()
        );
    }
}

#[test]
fn invalid_inputs_are_rejected_before_execution() {
    let mut cases = vec![
        CalculationRequest::SimplifySqrt { radicand: -1 },
        CalculationRequest::SimplifySqrt {
            radicand: 200_000_001,
        },
        CalculationRequest::SimplifySqrt { radicand: i64::MIN },
    ];
    for values in [vec![], vec![0; 65], vec![1_000_001], vec![i64::MIN]] {
        cases.push(CalculationRequest::DescribeData {
            values: values.clone(),
        });
        cases.push(CalculationRequest::DataVariance { values });
    }
    for request in cases {
        assert!(compute(&request)
            .unwrap_err()
            .starts_with("CAPABILITY_INPUT_INVALID"));
    }
    for request in [
        json!({"operation":"DESCRIBE_DATA","values":[1.5]}),
        json!({"operation":"SIMPLIFY_SQRT","radicand":2,"ignored":true}),
        json!({"operation":"EVALUATE","source":"anything"}),
    ] {
        assert!(serde_json::from_value::<CalculationRequest>(request).is_err());
    }
}

#[test]
fn knowledge_selection_is_data_driven_and_survives_memory_reload() {
    let request = CalculationRequest::DescribeData {
        values: vec![3, 1, 2],
    };
    assert_eq!(
        compute_with_knowledge(&request, &KnowledgeSpace::empty()).unwrap_err(),
        "NO_APPLICABLE_KNOWLEDGE"
    );
    let mut space = calculation_knowledge();
    let mut added = space.get("EXACT_DATA_SUMMARY").unwrap().clone();
    added.knowledge_id = "CUSTOM_SUMMARY".into();
    added.priority = 20;
    space.register_batch(vec![added.clone()]).unwrap();
    let path = std::env::temp_dir().join(format!("calculation-memory-{}.json", std::process::id()));
    let memory = MemorySpace::with_knowledge_space(space).unwrap();
    memory.save(&path).unwrap();
    let mut memory = MemorySpace::load(&path).unwrap();
    let result = compute_with_knowledge(&request, memory.knowledge_space()).unwrap();
    assert_eq!(result.trace["knowledge_id"], "CUSTOM_SUMMARY");
    assert!(result.explanation[0].contains("CUSTOM_SUMMARY"));
    memory
        .store(
            "calculation-result",
            &result.semantic_state,
            &[result.trace],
        )
        .unwrap();
    memory.save(&path).unwrap();
    let restored = MemorySpace::load(&path)
        .unwrap()
        .retrieve("calculation-result")
        .unwrap();
    restored.validate().unwrap();
    // Memory retrieval adds its own evidence while preserving every saved occurrence.
    assert_eq!(
        restored.objects.len(),
        result.semantic_state.objects.len() + 1
    );
    for object in &result.semantic_state.objects {
        assert_eq!(
            json!(restored.get_occurrence(object.occurrence_id()).unwrap()),
            json!(object)
        );
    }
    std::fs::remove_file(path).unwrap();
    let mut missing = calculation_knowledge();
    added.capabilities = vec!["NOT_REGISTERED".into()];
    missing.register_batch(vec![added.clone()]).unwrap();
    assert!(compute_with_knowledge(&request, &missing)
        .unwrap_err()
        .starts_with("CAPABILITY_NOT_FOUND"));
    let mut conflicting = calculation_knowledge();
    added.capabilities = vec!["DESCRIBE_DATA".into()];
    let mut other = added.clone();
    other.knowledge_id = "CONFLICTING_SUMMARY".into();
    added
        .activation_metadata
        .insert("conflicts_with".into(), json!([other.knowledge_id]));
    conflicting.register_batch(vec![added, other]).unwrap();
    assert_eq!(
        compute_with_knowledge(&request, &conflicting).unwrap_err(),
        "KNOWLEDGE_CONFLICT"
    );
}

#[test]
fn conflict_selection_preserves_priority_applicability_and_candidate_order() {
    let request = CalculationRequest::DescribeData {
        values: vec![3, 1, 2],
    };
    for selected_declares_conflict in [false, true] {
        let mut space = calculation_knowledge();
        let template = space.get("EXACT_DATA_SUMMARY").unwrap().clone();
        let mut definition = template.clone();
        definition.knowledge_id = "AAA_DEFINITION".into();
        definition.knowledge_type = "DEFINITION".into();
        definition.capabilities.clear();
        definition.applicable_rus.clear();
        definition.priority = 100;
        let mut lower = template.clone();
        lower.knowledge_id = "LOWER_PRIORITY_CONFLICT".into();
        lower.priority -= 1;
        lower
            .activation_metadata
            .insert("conflicts_with".into(), json!([template.knowledge_id]));
        let mut related = template.clone();
        related.knowledge_id = "RELATED_CONFLICT".into();
        related.pattern.symbols = vec!["unmatched_symbol".into()];
        related
            .activation_metadata
            .insert("conflicts_with".into(), json!([template.knowledge_id]));
        space
            .register_batch(vec![definition, lower, related])
            .unwrap();
        let result = compute_with_knowledge(&request, &space).unwrap();
        assert_eq!(
            result.knowledge_decision.retrieval.candidates[0].knowledge_id,
            "AAA_DEFINITION"
        );
        assert_eq!(result.trace["knowledge_id"], "EXACT_DATA_SUMMARY");
        verify_answer(&request, &result.answer).unwrap();

        let mut selected = template.clone();
        selected.knowledge_id = "SELECTED_SUMMARY".into();
        selected.priority = 20;
        let mut last = selected.clone();
        last.knowledge_id = "ZZZZ_LAST_CONFLICT".into();
        if selected_declares_conflict {
            selected
                .activation_metadata
                .insert("conflicts_with".into(), json!([last.knowledge_id]));
        } else {
            last.activation_metadata
                .insert("conflicts_with".into(), json!([selected.knowledge_id]));
        }
        space.register_batch(vec![selected, last]).unwrap();
        assert_eq!(
            compute_with_knowledge(&request, &space).unwrap_err(),
            "KNOWLEDGE_CONFLICT"
        );
    }
}

#[test]
#[ignore = "explicit 100k same-goal calculation and late-conflict performance gate"]
fn dense_calculation_candidates_scale_and_preserve_late_conflicts() {
    use std::time::{Duration, Instant};
    const CANDIDATES: usize = 100_000;
    let mut space = calculation_knowledge();
    let template = space.get("EXACT_DATA_SUMMARY").unwrap().clone();
    let mut units = Vec::with_capacity(CANDIDATES - 1);
    for i in 1..CANDIDATES {
        let mut unit = template.clone();
        unit.knowledge_id = format!("LOAD_{i:06}");
        units.push(unit);
    }
    let start = Instant::now();
    space.register_batch(units).unwrap();
    let registration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let request = CalculationRequest::DescribeData {
        values: (1..=64).rev().collect(),
    };
    let expected = json!({"kind":"SUMMARY","count":64,"mean":{"numerator":65,"denominator":2},"median":{"numerator":65,"denominator":2},"lower_quartile":{"numerator":33,"denominator":2},"upper_quartile":{"numerator":97,"denominator":2},"range":63});
    let mut previous_state = None;
    let mut timings_ms = vec![];
    for _ in 0..3 {
        let start = Instant::now();
        let result = compute_with_knowledge(&request, &space).unwrap();
        let elapsed = start.elapsed();
        // The old repeated candidate scan took about 47 seconds on this workload.
        assert!(
            elapsed < Duration::from_secs(20),
            "dense selection regression: {elapsed:?}"
        );
        timings_ms.push(elapsed.as_secs_f64() * 1000.0);
        assert_eq!(json!(result.answer), expected);
        verify_answer(&request, &result.answer).unwrap();
        assert_eq!(result.verification["status"], "VERIFIED");
        assert_eq!(result.trace["knowledge_id"], "EXACT_DATA_SUMMARY");
        assert!(!result.explanation.is_empty());
        let retrieval = &result.knowledge_decision.retrieval;
        assert_eq!(retrieval.retrieval_trace.examined, CANDIDATES);
        assert_eq!(retrieval.retrieval_trace.truncated_count, 0);
        assert_eq!(retrieval.candidates.len(), CANDIDATES);
        result.semantic_state.validate().unwrap();
        let state = result.semantic_state.canonical().unwrap();
        if let Some(previous) = &previous_state {
            assert_eq!(&state, previous);
        }
        previous_state = Some(state);
    }
    println!(
        "{}",
        json!({"event":"dense_calculation", "total_knowledge":space.len(), "applicable_candidates":CANDIDATES, "registration_ms":registration_ms, "computation_ms":timings_ms, "verified":true, "deterministic":true})
    );

    let mut last = template;
    last.knowledge_id = "ZZZZ_LAST_CONFLICT".into();
    last.activation_metadata
        .insert("conflicts_with".into(), json!(["EXACT_DATA_SUMMARY"]));
    space.register_batch(vec![last]).unwrap();
    let start = Instant::now();
    assert_eq!(
        compute_with_knowledge(&request, &space).unwrap_err(),
        "KNOWLEDGE_CONFLICT"
    );
    println!(
        "{}",
        json!({"event":"late_conflict", "candidates":CANDIDATES+1, "elapsed_ms":start.elapsed().as_secs_f64()*1000.0, "status":"KNOWLEDGE_CONFLICT"})
    );
}
