use mathlang_mirp::calculation::{
    calculation_knowledge, compute, compute_with_knowledge, verify_answer, CalculationAnswer as A,
    CalculationRequest as R,
};
use mathlang_mirp::exact::{ExactExpression, ExactOrdering, ExactRational as Q, RadicalTerm};
use mathlang_mirp::knowledge::{Applicability, KnowledgeSpace};
use mathlang_mirp::{Object, SemanticState};
use serde_json::json;

fn q(n: i64, d: i64) -> Q {
    Q::new(n, d)
}
fn qs(values: &[i64]) -> Vec<Q> {
    values.iter().map(|&n| q(n, 1)).collect()
}
fn expr(values: &[(i64, i64, i64)]) -> ExactExpression {
    ExactExpression {
        terms: values
            .iter()
            .map(|&(n, d, r)| RadicalTerm {
                coefficient: q(n, d),
                radicand: r,
            })
            .collect(),
    }
}
fn checked(request: R, id: &str) -> A {
    let r = compute(&request).unwrap();
    assert_eq!(r.verification["status"], "VERIFIED");
    assert_eq!(r.verification["formal_status"], "NOT_CHECKED");
    assert_eq!(r.trace["knowledge_id"], id);
    assert!(r
        .knowledge_decision
        .applicability
        .contains(&(id.into(), Applicability::Applicable)));
    assert!(r.explanation[0].contains(id));
    assert!(r.explanation.len() >= 4);
    assert!(r.semantic_state.objects.iter().any(|o| matches!(o,Object::Evidence(e) if e.common.metadata["knowledge_runtime"]["knowledge_id"]==id)));
    r.semantic_state.validate().unwrap();
    let restored =
        SemanticState::from_json(&serde_json::to_string(&r.semantic_state).unwrap()).unwrap();
    assert_eq!(
        restored.canonical().unwrap(),
        r.semantic_state.canonical().unwrap()
    );
    verify_answer(&request, &r.answer).unwrap();
    r.answer
}

#[test]
fn covariance_is_exact_and_distinguishes_the_two_denominators() {
    for (x, y, expected) in [
        (
            qs(&[1, 2, 3]),
            qs(&[2, 4, 6]),
            A::Covariance {
                count: 3,
                mean_x: q(2, 1),
                mean_y: q(4, 1),
                population_covariance: q(4, 3),
                sample_covariance: Some(q(2, 1)),
            },
        ),
        (
            qs(&[1, 2, 3]),
            qs(&[3, 2, 1]),
            A::Covariance {
                count: 3,
                mean_x: q(2, 1),
                mean_y: q(2, 1),
                population_covariance: q(-2, 3),
                sample_covariance: Some(q(-1, 1)),
            },
        ),
        (
            vec![q(1, 2), q(3, 2)],
            qs(&[2, 4]),
            A::Covariance {
                count: 2,
                mean_x: q(1, 1),
                mean_y: q(3, 1),
                population_covariance: q(1, 2),
                sample_covariance: Some(q(1, 1)),
            },
        ),
        (
            qs(&[-7]),
            qs(&[9]),
            A::Covariance {
                count: 1,
                mean_x: q(-7, 1),
                mean_y: q(9, 1),
                population_covariance: q(0, 1),
                sample_covariance: None,
            },
        ),
    ] {
        let request = R::DataCovariance { x, y };
        let answer = checked(request.clone(), "EXACT_DATA_COVARIANCE");
        assert_eq!(answer, expected);
        if let A::Covariance {
            mut population_covariance,
            count,
            mean_x,
            mean_y,
            sample_covariance,
        } = answer
        {
            population_covariance.numerator = "999".into();
            assert!(verify_answer(
                &request,
                &A::Covariance {
                    count,
                    mean_x,
                    mean_y,
                    population_covariance,
                    sample_covariance
                }
            )
            .is_err());
        }
    }
}

#[test]
fn normalized_weights_are_nonnegative_exact_probabilities() {
    for (weights, total, probabilities) in [
        (qs(&[2, 1, 1]), q(4, 1), vec![q(1, 2), q(1, 4), q(1, 4)]),
        (
            vec![q(1, 2), q(1, 3), q(1, 6)],
            q(1, 1),
            vec![q(1, 2), q(1, 3), q(1, 6)],
        ),
        (qs(&[0, 3]), q(3, 1), vec![q(0, 1), q(1, 1)]),
    ] {
        let request = R::NormalizeWeights { weights };
        let result = checked(request.clone(), "EXACT_WEIGHT_NORMALIZATION");
        assert_eq!(
            result,
            A::Weights {
                total_weight: total,
                probabilities
            }
        );
        let mut bad = json!(result);
        bad["probabilities"][0]["numerator"] = json!("99");
        assert!(verify_answer(&request, &serde_json::from_value(bad).unwrap()).is_err());
    }
    let large = Q {
        numerator:
            "99999999999999999999999999999999999999999999999999999999999999999999999999999999"
                .into(),
        denominator: "1".into(),
    };
    let result = checked(
        R::NormalizeWeights {
            weights: vec![large.clone(), large],
        },
        "EXACT_WEIGHT_NORMALIZATION",
    );
    if let A::Weights { probabilities, .. } = result {
        assert_eq!(probabilities, vec![q(1, 2), q(1, 2)]);
    } else {
        panic!();
    }
}

#[test]
fn discrete_expectation_requires_explicit_normalization() {
    for (values, weights, normalize_weights, value, total, probabilities) in [
        (
            qs(&[1, 2, 3]),
            vec![q(1, 2), q(1, 3), q(1, 6)],
            false,
            q(5, 3),
            q(1, 1),
            vec![q(1, 2), q(1, 3), q(1, 6)],
        ),
        (
            qs(&[1, 2, 3]),
            qs(&[2, 1, 1]),
            true,
            q(7, 4),
            q(4, 1),
            vec![q(1, 2), q(1, 4), q(1, 4)],
        ),
        (
            vec![q(1, 2), q(3, 2)],
            qs(&[1, 3]),
            true,
            q(5, 4),
            q(4, 1),
            vec![q(1, 4), q(3, 4)],
        ),
        (qs(&[-3]), qs(&[1]), false, q(-3, 1), q(1, 1), vec![q(1, 1)]),
    ] {
        let request = R::DiscreteExpectation {
            values,
            weights,
            normalize_weights,
        };
        let answer = checked(request.clone(), "EXACT_DISCRETE_EXPECTATION");
        assert_eq!(
            answer,
            A::Expectation {
                value,
                total_weight: total,
                probabilities,
                normalized: normalize_weights
            }
        );
        let mut bad = json!(answer);
        bad["value"]["numerator"] = json!("123");
        assert!(verify_answer(&request, &serde_json::from_value(bad).unwrap()).is_err());
    }
}

#[test]
fn finite_radical_sums_have_a_canonical_exact_form() {
    for (input, expected) in [
        (expr(&[(1, 1, 8), (3, 1, 2), (-5, 1, 2)]), expr(&[])),
        (expr(&[(1, 2, 5), (1, 2, 1)]), expr(&[(1, 2, 1), (1, 2, 5)])),
        (expr(&[(2, 4, 72)]), expr(&[(3, 1, 2)])),
        (expr(&[(1, 2, 4), (-1, 1, 1), (9, 1, 0)]), expr(&[])),
        (
            expr(&[(-3, 2, 12), (2, 3, 2)]),
            expr(&[(2, 3, 2), (-3, 1, 3)]),
        ),
    ] {
        let request = R::NormalizeExact { expression: input };
        let answer = checked(request.clone(), "EXACT_RADICAL_NORMAL_FORM");
        assert_eq!(
            answer,
            A::ExactExpression {
                expression: expected
            }
        );
        assert!(verify_answer(
            &request,
            &A::ExactExpression {
                expression: expr(&[(99, 1, 2)])
            }
        )
        .is_err());
    }
}

#[test]
fn equality_and_order_use_exact_sign_certificates() {
    for (left, right, order) in [
        (expr(&[(1, 1, 8)]), expr(&[(2, 1, 2)]), ExactOrdering::Equal),
        (
            expr(&[(1, 2, 1), (1, 2, 5)]),
            expr(&[(1, 2, 5), (1, 2, 1)]),
            ExactOrdering::Equal,
        ),
        (
            expr(&[(1, 1, 2), (1, 1, 3)]),
            expr(&[(3, 1, 1)]),
            ExactOrdering::Greater,
        ),
        (
            expr(&[(1, 1, 2), (1, 1, 3)]),
            expr(&[(1, 1, 10)]),
            ExactOrdering::Less,
        ),
        (
            expr(&[(-1, 1, 2)]),
            expr(&[(-1, 1, 1)]),
            ExactOrdering::Less,
        ),
        (
            expr(&[(1, 1, 2)]),
            expr(&[(1414213562373095, 1000000000000000, 1)]),
            ExactOrdering::Greater,
        ),
        (
            expr(&[(1, 1, 2)]),
            expr(&[(14142135623730951, 10000000000000000, 1)]),
            ExactOrdering::Less,
        ),
        (
            expr(&[(1, 1, 2), (-1, 1, 2), (1, 1, 1)]),
            expr(&[]),
            ExactOrdering::Greater,
        ),
    ] {
        for equal_operation in [false, true] {
            let request = if equal_operation {
                R::ExactEqual {
                    left: left.clone(),
                    right: right.clone(),
                }
            } else {
                R::ExactCompare {
                    left: left.clone(),
                    right: right.clone(),
                }
            };
            let answer = checked(
                request.clone(),
                if equal_operation {
                    "EXACT_RADICAL_EQUALITY"
                } else {
                    "EXACT_RADICAL_COMPARISON"
                },
            );
            let A::ExactComparison { comparison } = answer else {
                panic!()
            };
            assert_eq!(comparison.ordering, order);
            assert_eq!(comparison.equal, order == ExactOrdering::Equal);
            let mut corrupt = comparison.clone();
            corrupt.equal = !comparison.equal;
            assert!(verify_answer(
                &request,
                &A::ExactComparison {
                    comparison: corrupt
                }
            )
            .is_err());
            if !comparison.certificate.roots.is_empty() {
                let mut corrupt = comparison.clone();
                corrupt.certificate.roots[0].floor = "0".into();
                assert!(verify_answer(
                    &request,
                    &A::ExactComparison {
                        comparison: corrupt
                    }
                )
                .is_err());
                let mut corrupt = comparison.clone();
                corrupt.certificate.precision_bits = u32::MAX;
                assert!(verify_answer(
                    &request,
                    &A::ExactComparison {
                        comparison: corrupt
                    }
                )
                .is_err());
            }
        }
    }
}

#[test]
fn rational_statistics_support_the_full_64_element_bound() {
    let values: Vec<_> = (1..=64).map(|n| q(n, 1)).collect();
    let covariance = checked(
        R::DataCovariance {
            x: values.clone(),
            y: (1..=64).map(|n| q(2 * n, 1)).collect(),
        },
        "EXACT_DATA_COVARIANCE",
    );
    assert_eq!(
        covariance,
        A::Covariance {
            count: 64,
            mean_x: q(65, 2),
            mean_y: q(65, 1),
            population_covariance: q(1365, 2),
            sample_covariance: Some(q(2080, 3))
        }
    );
    let weights = vec![q(1, 64); 64];
    let normalized = checked(
        R::NormalizeWeights {
            weights: weights.clone(),
        },
        "EXACT_WEIGHT_NORMALIZATION",
    );
    assert_eq!(
        normalized,
        A::Weights {
            total_weight: q(1, 1),
            probabilities: weights.clone()
        }
    );
    let expected = checked(
        R::DiscreteExpectation {
            values,
            weights: weights.clone(),
            normalize_weights: false,
        },
        "EXACT_DISCRETE_EXPECTATION",
    );
    assert_eq!(
        expected,
        A::Expectation {
            value: q(65, 2),
            total_weight: q(1, 1),
            probabilities: weights,
            normalized: false
        }
    );
}

#[test]
fn domain_errors_are_rejected_before_execution() {
    let mut requests = vec![
        R::DataCovariance {
            x: vec![],
            y: vec![],
        },
        R::DataCovariance {
            x: qs(&[1, 2]),
            y: qs(&[1]),
        },
        R::DataCovariance {
            x: qs(&[1; 65]),
            y: qs(&[1; 65]),
        },
        R::NormalizeWeights {
            weights: qs(&[0, 0]),
        },
        R::NormalizeWeights {
            weights: qs(&[-1, 2]),
        },
        R::NormalizeWeights {
            weights: vec![q(1, 0)],
        },
        R::NormalizeWeights {
            weights: vec![q(1, -1)],
        },
        R::DiscreteExpectation {
            values: qs(&[1]),
            weights: qs(&[2]),
            normalize_weights: false,
        },
        R::DiscreteExpectation {
            values: qs(&[1, 2]),
            weights: qs(&[1]),
            normalize_weights: true,
        },
        R::NormalizeExact {
            expression: expr(&[(0, 1, -1)]),
        },
        R::NormalizeExact {
            expression: expr(&[(1, 1, 200000001)]),
        },
        R::NormalizeExact {
            expression: expr(&[(1, 1, 2); 33]),
        },
    ];
    requests.push(R::NormalizeWeights {
        weights: vec![Q {
            numerator: "1".repeat(81),
            denominator: "1".into(),
        }],
    });
    requests.push(R::NormalizeWeights {
        weights: vec![Q {
            numerator: "1e2".into(),
            denominator: "1".into(),
        }],
    });
    for request in requests {
        assert!(compute(&request)
            .unwrap_err()
            .starts_with("CAPABILITY_INPUT_INVALID"));
    }
    for value in [
        json!({"operation":"NORMALIZE_WEIGHTS","weights":[{"numerator":0.5,"denominator":1}]}),
        json!({"operation":"NORMALIZE_EXACT","expression":{"terms":[],"ignored":true}}),
    ] {
        assert!(serde_json::from_value::<R>(value).is_err());
    }
}

#[test]
fn all_six_new_knowledge_units_drive_execution_and_can_be_replaced() {
    let cases = [
        (
            "EXACT_DATA_COVARIANCE",
            R::DataCovariance {
                x: qs(&[1, 2]),
                y: qs(&[2, 4]),
            },
        ),
        (
            "EXACT_WEIGHT_NORMALIZATION",
            R::NormalizeWeights {
                weights: qs(&[1, 2]),
            },
        ),
        (
            "EXACT_DISCRETE_EXPECTATION",
            R::DiscreteExpectation {
                values: qs(&[1, 2]),
                weights: qs(&[1, 2]),
                normalize_weights: true,
            },
        ),
        (
            "EXACT_RADICAL_NORMAL_FORM",
            R::NormalizeExact {
                expression: expr(&[(1, 1, 72)]),
            },
        ),
        (
            "EXACT_RADICAL_EQUALITY",
            R::ExactEqual {
                left: expr(&[(1, 1, 8)]),
                right: expr(&[(2, 1, 2)]),
            },
        ),
        (
            "EXACT_RADICAL_COMPARISON",
            R::ExactCompare {
                left: expr(&[(1, 1, 2)]),
                right: expr(&[(1, 1, 1)]),
            },
        ),
    ];
    for (id, request) in cases {
        let original = checked(request.clone(), id);
        let mut space = calculation_knowledge();
        let mut unit = space.get(id).unwrap().clone();
        unit.knowledge_id = format!("CUSTOM_{id}");
        unit.priority += 1;
        space.register_batch(vec![unit.clone()]).unwrap();
        let replacement = compute_with_knowledge(&request, &space).unwrap();
        assert_eq!(replacement.answer, original);
        assert_eq!(replacement.trace["knowledge_id"], unit.knowledge_id);
        assert_eq!(
            compute_with_knowledge(&request, &KnowledgeSpace::empty()).unwrap_err(),
            "NO_APPLICABLE_KNOWLEDGE"
        );
        let mut conflict = unit.clone();
        conflict.knowledge_id = format!("CONFLICT_{id}");
        conflict
            .activation_metadata
            .insert("conflicts_with".into(), json!([unit.knowledge_id]));
        space.register_batch(vec![conflict]).unwrap();
        assert_eq!(
            compute_with_knowledge(&request, &space).unwrap_err(),
            "KNOWLEDGE_CONFLICT"
        );
    }
}

#[test]
fn insufficient_comparison_precision_never_becomes_false_equality() {
    // CRT decomposition represents a rational within about 10^-2449 of sqrt(2)
    // using the permitted 32 terms and 80-digit input integers.
    let request: R = serde_json::from_str(include_str!(
        "../fixtures/calculation/comparison_limit.json"
    ))
    .unwrap();
    request.validate().unwrap();
    assert!(compute(&request).unwrap_err().starts_with("RESOURCE_LIMIT"));
    if let R::ExactCompare { left, right } = request {
        assert!(compute(&R::ExactEqual { left, right })
            .unwrap_err()
            .starts_with("RESOURCE_LIMIT"));
    }
}
