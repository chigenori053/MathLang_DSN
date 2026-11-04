use mathlang_mirp::calculation::{
    calculation_knowledge, compute, compute_with_knowledge, verify_answer, CalculationAnswer as A,
    CalculationRequest as R,
};
use mathlang_mirp::exact::ExactRational as Q;
use mathlang_mirp::geometry::{
    Circle2, ExactPoint2, IntersectionKind as K, Line2, QuadraticSurd, TriangleDegeneracy as D,
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use mathlang_mirp::{Object, SemanticState};
use serde_json::json;

fn q(n: i64) -> Q {
    Q::new(n, 1)
}
fn p(x: i64, y: i64) -> Vec<Q> {
    vec![q(x), q(y)]
}
fn l(a: i64, b: i64, c: i64) -> Line2 {
    Line2 {
        a: q(a),
        b: q(b),
        c: q(c),
    }
}
fn c(x: i64, y: i64, r: i64) -> Circle2 {
    Circle2 {
        center: p(x, y),
        radius: q(r),
    }
}
fn rational(value: Q) -> QuadraticSurd {
    QuadraticSurd {
        rational: value,
        coefficient: q(0),
        radicand: q(0),
    }
}
fn surd(n: i64, d: i64, r: i64) -> QuadraticSurd {
    QuadraticSurd {
        rational: q(0),
        coefficient: Q::new(n, d),
        radicand: q(r),
    }
}
fn ep(x: i64, y: i64) -> ExactPoint2 {
    ExactPoint2 {
        coordinates: vec![rational(q(x)), rational(q(y))],
    }
}
fn checked(request: R, id: &str) -> A {
    let r = compute(&request).unwrap();
    assert_eq!(r.trace["knowledge_id"], id);
    assert_eq!(
        r.knowledge_decision
            .activation
            .as_ref()
            .unwrap()
            .knowledge_id,
        id
    );
    assert_eq!(r.verification["status"], "VERIFIED");
    assert_eq!(r.verification["formal_status"], "NOT_CHECKED");
    assert!(r.explanation[0].contains(id));
    assert!(r.semantic_state.objects.iter().any(|o|matches!(o,Object::Evidence(e) if e.common.metadata["knowledge_runtime"]["knowledge_id"]==id)));
    verify_answer(&request, &r.answer).unwrap();
    r.semantic_state.validate().unwrap();
    let restored =
        SemanticState::from_json(&serde_json::to_string(&r.semantic_state).unwrap()).unwrap();
    assert_eq!(
        restored.canonical().unwrap(),
        r.semantic_state.canonical().unwrap()
    );
    r.answer
}
fn intersections(request: R, id: &str, kind: K, points: Vec<ExactPoint2>) {
    let answer = checked(request, id);
    let A::GeometryIntersections { intersection } = answer else {
        panic!()
    };
    assert_eq!(intersection.infinite, kind == K::Coincident);
    assert_eq!(intersection.relation, kind);
    assert_eq!(intersection.points, points);
}

#[test]
fn distances_are_exact_nonnegative_and_distinguish_coincident_points() {
    for (a, b, squared, distance, coincident) in [
        (p(0, 0), p(3, 4), q(25), rational(q(5)), false),
        (p(-1, -1), p(1, 1), q(8), surd(2, 1, 2), false),
        (p(4, -7), p(4, -7), q(0), rational(q(0)), true),
        (
            vec![Q::new(1, 2), q(0)],
            p(2, 0),
            Q::new(9, 4),
            rational(Q::new(3, 2)),
            false,
        ),
    ] {
        assert_eq!(
            checked(R::PlaneDistance { a, b }, "GEOMETRY_PLANE_DISTANCE"),
            A::PlaneDistance {
                squared_distance: squared,
                distance,
                coincident
            }
        );
    }
}

#[test]
fn inner_products_and_zero_vector_orthogonality_are_explicit() {
    assert_eq!(
        checked(
            R::PlaneDotProduct {
                u: p(1, 2),
                v: p(3, 4)
            },
            "GEOMETRY_PLANE_DOT_PRODUCT"
        ),
        A::PlaneDotProduct { value: q(11) }
    );
    for (u, v, dot, orthogonal, zero) in [
        (p(1, 2), p(-2, 1), 0, true, false),
        (p(1, 2), p(3, 4), 11, false, false),
        (p(0, 0), p(7, -3), 0, true, true),
    ] {
        assert_eq!(
            checked(R::PlaneOrthogonal { u, v }, "GEOMETRY_PLANE_ORTHOGONAL"),
            A::PlaneOrthogonality {
                dot_product: q(dot),
                orthogonal,
                zero_vector_present: zero,
                angle_defined: !zero
            }
        );
    }
}

#[test]
fn triangle_area_preserves_orientation_and_classifies_degeneracy() {
    for (a, b, c, area, signed, degeneracy) in [
        (p(0, 0), p(4, 0), p(0, 3), q(6), q(12), D::None),
        (p(0, 0), p(0, 3), p(4, 0), q(6), q(-12), D::None),
        (p(0, 0), p(0, 0), p(4, 3), q(0), q(0), D::RepeatedVertex),
        (p(0, 0), p(1, 1), p(2, 2), q(0), q(0), D::Collinear),
        (
            p(0, 0),
            vec![Q::new(1, 2), q(0)],
            vec![q(0), Q::new(1, 3)],
            Q::new(1, 12),
            Q::new(1, 6),
            D::None,
        ),
    ] {
        assert_eq!(
            checked(R::TriangleArea { a, b, c }, "GEOMETRY_TRIANGLE_AREA"),
            A::TriangleArea {
                area,
                signed_double_area: signed,
                degeneracy
            }
        );
    }
}

#[test]
fn all_line_line_cases_include_vertical_parallel_and_infinite_sets() {
    for (first, second, kind, points) in [
        (l(1, 0, -2), l(0, 1, -3), K::Crossing, vec![ep(2, 3)]),
        (l(2, 3, -7), l(1, -1, -1), K::Crossing, vec![ep(2, 1)]),
        (l(0, 1, -1), l(0, 1, -2), K::Parallel, vec![]),
        (l(1, 1, -2), l(-3, -3, 6), K::Coincident, vec![]),
    ] {
        intersections(
            R::LineLineIntersections { first, second },
            "GEOMETRY_LINE_LINE_INTERSECTIONS",
            kind,
            points,
        );
    }
}

#[test]
fn line_circle_enumerates_secants_tangents_and_point_circles() {
    for (line, circle, kind, points) in [
        (l(0, 1, 0), c(0, 0, 5), K::Secant, vec![ep(-5, 0), ep(5, 0)]),
        (l(1, 0, 0), c(0, 0, 5), K::Secant, vec![ep(0, -5), ep(0, 5)]),
        (l(0, 1, -5), c(0, 0, 5), K::Tangent, vec![ep(0, 5)]),
        (l(0, 1, -6), c(0, 0, 5), K::Disjoint, vec![]),
        (
            l(1, -1, 0),
            c(0, 0, 1),
            K::Secant,
            vec![
                ExactPoint2 {
                    coordinates: vec![surd(-1, 2, 2), surd(-1, 2, 2)],
                },
                ExactPoint2 {
                    coordinates: vec![surd(1, 2, 2), surd(1, 2, 2)],
                },
            ],
        ),
        (
            l(0, 1, -2),
            c(1, 2, 3),
            K::Secant,
            vec![ep(-2, 2), ep(4, 2)],
        ),
        (
            l(0, 1, -2),
            c(1, 2, 0),
            K::PointIntersection,
            vec![ep(1, 2)],
        ),
        (l(0, 1, -3), c(1, 2, 0), K::Disjoint, vec![]),
    ] {
        intersections(
            R::LineCircleIntersections { line, circle },
            "GEOMETRY_LINE_CIRCLE_INTERSECTIONS",
            kind,
            points,
        );
    }
}

#[test]
fn all_circle_circle_relations_and_zero_radius_cases_are_complete() {
    for (first, second, kind, points) in [
        (c(0, 0, 5), c(8, 0, 5), K::Secant, vec![ep(4, -3), ep(4, 3)]),
        (
            c(0, 0, 1),
            c(1, 0, 1),
            K::Secant,
            vec![
                ExactPoint2 {
                    coordinates: vec![rational(Q::new(1, 2)), surd(-1, 2, 3)],
                },
                ExactPoint2 {
                    coordinates: vec![rational(Q::new(1, 2)), surd(1, 2, 3)],
                },
            ],
        ),
        (c(0, 0, 1), c(2, 0, 1), K::Tangent, vec![ep(1, 0)]),
        (c(0, 0, 3), c(2, 0, 1), K::Tangent, vec![ep(3, 0)]),
        (c(0, 0, 1), c(3, 0, 1), K::Disjoint, vec![]),
        (c(0, 0, 4), c(1, 0, 1), K::Disjoint, vec![]),
        (c(0, 0, 1), c(0, 0, 2), K::Disjoint, vec![]),
        (c(1, -2, 3), c(1, -2, 3), K::Coincident, vec![]),
        (c(1, 0, 0), c(0, 0, 1), K::PointIntersection, vec![ep(1, 0)]),
        (c(0, 0, 1), c(1, 0, 0), K::PointIntersection, vec![ep(1, 0)]),
        (c(0, 0, 0), c(0, 0, 1), K::Disjoint, vec![]),
        (
            c(1, -2, 0),
            c(1, -2, 0),
            K::PointIntersection,
            vec![ep(1, -2)],
        ),
        (c(0, 0, 0), c(1, 0, 0), K::Disjoint, vec![]),
    ] {
        intersections(
            R::CircleCircleIntersections { first, second },
            "GEOMETRY_CIRCLE_CIRCLE_INTERSECTIONS",
            kind,
            points,
        );
    }
    let answer = compute(&R::CircleCircleIntersections {
        first: c(1, -2, 0),
        second: c(1, -2, 0),
    })
    .unwrap()
    .answer;
    let A::GeometryIntersections { intersection } = answer else {
        panic!()
    };
    assert!(!intersection.infinite);
    assert_eq!(
        intersection.input_degeneracies,
        vec![
            "first_circle:ZERO_RADIUS_POINT",
            "second_circle:ZERO_RADIUS_POINT"
        ]
    );
}

#[test]
fn exact_verification_rejects_missing_duplicate_wrong_and_reordered_intersections() {
    let request = R::CircleCircleIntersections {
        first: c(0, 0, 1),
        second: c(1, 0, 1),
    };
    let answer = compute(&request).unwrap().answer;
    let A::GeometryIntersections { intersection } = answer else {
        panic!()
    };
    let mut cases = vec![];
    let mut missing = intersection.clone();
    missing.points.pop();
    cases.push(missing);
    let mut duplicate = intersection.clone();
    duplicate.points[1] = duplicate.points[0].clone();
    cases.push(duplicate);
    let mut reversed = intersection.clone();
    reversed.points.swap(0, 1);
    cases.push(reversed);
    let mut wrong = intersection.clone();
    wrong.points[0].coordinates[0] = rational(q(99));
    cases.push(wrong);
    let mut aliased_duplicate = intersection.clone();
    aliased_duplicate.points[1] = aliased_duplicate.points[0].clone();
    aliased_duplicate.points[1].coordinates[1] = surd(-1, 4, 12);
    cases.push(aliased_duplicate);
    let mut false_infinity = intersection.clone();
    false_infinity.infinite = true;
    cases.push(false_infinity);
    let mut false_tangent = intersection.clone();
    false_tangent.relation = K::Tangent;
    cases.push(false_tangent);
    for intersection in cases {
        assert!(verify_answer(&request, &A::GeometryIntersections { intersection }).is_err());
    }
    let request = R::LineLineIntersections {
        first: l(1, 1, -2),
        second: l(2, 2, -4),
    };
    let mut bad = compute(&request).unwrap().answer;
    if let A::GeometryIntersections { intersection } = &mut bad {
        intersection.points.push(ep(1, 1));
    }
    assert!(verify_answer(&request, &bad).is_err());
}

#[test]
fn equivalent_line_scaling_and_circle_permutation_preserve_sorted_points() {
    let original = compute(&R::LineCircleIntersections {
        line: l(1, -1, 0),
        circle: c(0, 0, 1),
    })
    .unwrap()
    .answer;
    for scale in [-11, -1, 2, 7] {
        assert_eq!(
            compute(&R::LineCircleIntersections {
                line: l(scale, -scale, 0),
                circle: c(0, 0, 1)
            })
            .unwrap()
            .answer,
            original
        );
    }
    for (first, second) in [
        (c(0, 0, 1), c(1, 0, 1)),
        (c(-2, 3, 5), c(4, 1, 3)),
        (c(0, 0, 3), c(2, 0, 1)),
    ] {
        let forward = compute(&R::CircleCircleIntersections {
            first: first.clone(),
            second: second.clone(),
        })
        .unwrap()
        .answer;
        let reverse = compute(&R::CircleCircleIntersections {
            first: second,
            second: first,
        })
        .unwrap()
        .answer;
        assert_eq!(forward, reverse);
    }
}

#[test]
fn circle_grid_is_symmetric_and_translation_invariant() {
    let space = calculation_knowledge();
    for x in -1..=1 {
        for y in -1..=1 {
            for first_radius in 0..=3 {
                for second_radius in 0..=3 {
                    let first = c(0, 0, first_radius);
                    let second = c(x, y, second_radius);
                    let forward = compute_with_knowledge(
                        &R::CircleCircleIntersections {
                            first: first.clone(),
                            second: second.clone(),
                        },
                        &space,
                    )
                    .unwrap();
                    let reverse = compute_with_knowledge(
                        &R::CircleCircleIntersections {
                            first: second,
                            second: first,
                        },
                        &space,
                    )
                    .unwrap();
                    let A::GeometryIntersections {
                        intersection: original,
                    } = forward.answer
                    else {
                        panic!()
                    };
                    let A::GeometryIntersections {
                        intersection: backward,
                    } = reverse.answer
                    else {
                        panic!()
                    };
                    assert_eq!(original.points, backward.points);
                    assert_eq!(original.relation, backward.relation);
                    assert_eq!(original.infinite, backward.infinite);
                    let shifted = compute_with_knowledge(
                        &R::CircleCircleIntersections {
                            first: c(7, -5, first_radius),
                            second: c(x + 7, y - 5, second_radius),
                        },
                        &space,
                    )
                    .unwrap();
                    let A::GeometryIntersections {
                        intersection: shifted,
                    } = shifted.answer
                    else {
                        panic!()
                    };
                    assert_eq!(original.relation, shifted.relation);
                    assert_eq!(original.infinite, shifted.infinite);
                    let mut expected = original.points;
                    for point in &mut expected {
                        for (coordinate, offset) in point.coordinates.iter_mut().zip([7, -5]) {
                            let numerator = coordinate.rational.numerator.parse::<i64>().unwrap();
                            let denominator =
                                coordinate.rational.denominator.parse::<i64>().unwrap();
                            coordinate.rational =
                                Q::new(numerator + offset * denominator, denominator);
                        }
                    }
                    assert_eq!(shifted.points, expected);
                }
            }
        }
    }
}

#[test]
fn near_tangency_and_large_translations_do_not_use_floating_point() {
    let denominator = "10000000000000000000000000000000000000000";
    for (numerator, kind, count) in [
        ("10000000000000000000000000000000000000001", K::Disjoint, 0),
        ("9999999999999999999999999999999999999999", K::Secant, 2),
    ] {
        let y = Q {
            numerator: format!("-{numerator}"),
            denominator: denominator.into(),
        };
        let result = checked(
            R::LineCircleIntersections {
                line: Line2 {
                    a: q(0),
                    b: q(1),
                    c: y,
                },
                circle: c(0, 0, 1),
            },
            "GEOMETRY_LINE_CIRCLE_INTERSECTIONS",
        );
        let A::GeometryIntersections { intersection } = result else {
            panic!()
        };
        assert_eq!(intersection.relation, kind);
        assert_eq!(intersection.points.len(), count);
    }
    let start = Q {
        numerator: format!("1{}", "0".repeat(70)),
        denominator: "1".into(),
    };
    let end = Q {
        numerator: format!("1{}3", "0".repeat(69)),
        denominator: "1".into(),
    };
    assert_eq!(
        checked(
            R::PlaneDistance {
                a: vec![start, q(0)],
                b: vec![end, q(4)]
            },
            "GEOMETRY_PLANE_DISTANCE"
        ),
        A::PlaneDistance {
            squared_distance: q(25),
            distance: rational(q(5)),
            coincident: false
        }
    );
}

#[test]
fn malformed_coordinates_and_degenerate_lines_are_rejected() {
    for request in [
        R::PlaneDistance {
            a: vec![],
            b: p(0, 0),
        },
        R::PlaneDistance {
            a: vec![q(0)],
            b: p(0, 0),
        },
        R::PlaneDotProduct {
            u: vec![q(1), q(2), q(3)],
            v: p(1, 2),
        },
        R::PlaneOrthogonal {
            u: p(1, 2),
            v: vec![],
        },
        R::TriangleArea {
            a: p(0, 0),
            b: p(1, 0),
            c: vec![q(1)],
        },
        R::LineLineIntersections {
            first: l(0, 0, 0),
            second: l(1, 1, 0),
        },
        R::LineCircleIntersections {
            line: l(0, 0, 1),
            circle: c(0, 0, 1),
        },
        R::LineCircleIntersections {
            line: l(1, 0, 0),
            circle: c(0, 0, -1),
        },
        R::CircleCircleIntersections {
            first: Circle2 {
                center: vec![q(1)],
                radius: q(1),
            },
            second: c(0, 0, 1),
        },
        R::PlaneDistance {
            a: vec![Q::new(1, 0), q(0)],
            b: p(0, 0),
        },
    ] {
        assert!(compute(&request)
            .unwrap_err()
            .starts_with("CAPABILITY_INPUT_INVALID"));
    }
    assert!(serde_json::from_value::<R>(
        json!({"operation":"PLANE_DISTANCE","a":[0.1,0],"b":[0,0]})
    )
    .is_err());
    assert!(serde_json::from_value::<R>(json!({"operation":"LINE_LINE_INTERSECTIONS","first":{"a":q(1),"b":q(0),"c":q(0),"ignored":true},"second":l(0,1,0)})).is_err());
}

#[test]
fn all_seven_geometry_knowledge_units_are_selected_and_replaceable() {
    for (id, request) in [
        (
            "GEOMETRY_PLANE_DISTANCE",
            R::PlaneDistance {
                a: p(0, 0),
                b: p(3, 4),
            },
        ),
        (
            "GEOMETRY_PLANE_DOT_PRODUCT",
            R::PlaneDotProduct {
                u: p(1, 2),
                v: p(3, 4),
            },
        ),
        (
            "GEOMETRY_PLANE_ORTHOGONAL",
            R::PlaneOrthogonal {
                u: p(1, 0),
                v: p(0, 1),
            },
        ),
        (
            "GEOMETRY_TRIANGLE_AREA",
            R::TriangleArea {
                a: p(0, 0),
                b: p(4, 0),
                c: p(0, 3),
            },
        ),
        (
            "GEOMETRY_LINE_LINE_INTERSECTIONS",
            R::LineLineIntersections {
                first: l(1, 0, 0),
                second: l(0, 1, 0),
            },
        ),
        (
            "GEOMETRY_LINE_CIRCLE_INTERSECTIONS",
            R::LineCircleIntersections {
                line: l(1, -1, 0),
                circle: c(0, 0, 1),
            },
        ),
        (
            "GEOMETRY_CIRCLE_CIRCLE_INTERSECTIONS",
            R::CircleCircleIntersections {
                first: c(0, 0, 1),
                second: c(1, 0, 1),
            },
        ),
    ] {
        let expected = checked(request.clone(), id);
        let mut space = calculation_knowledge();
        let mut unit = space.get(id).unwrap().clone();
        unit.knowledge_id = format!("CUSTOM_{id}");
        unit.priority += 1;
        space.register_batch(vec![unit.clone()]).unwrap();
        let result = compute_with_knowledge(&request, &space).unwrap();
        assert_eq!(result.answer, expected);
        assert_eq!(result.trace["knowledge_id"], unit.knowledge_id);
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
#[ignore = "explicit 100k same-goal plane geometry performance and conflict gate"]
fn geometry_candidates_scale_and_keep_the_last_conflict() {
    let mut space = calculation_knowledge();
    let template = space.get("GEOMETRY_PLANE_DISTANCE").unwrap().clone();
    let mut units = vec![];
    for i in 1..100_000 {
        let mut u = template.clone();
        u.knowledge_id = format!("LOAD_{i:06}");
        units.push(u);
    }
    space.register_batch(units).unwrap();
    let request = R::PlaneDistance {
        a: p(0, 0),
        b: p(3, 4),
    };
    let start = std::time::Instant::now();
    let result = compute_with_knowledge(&request, &space).unwrap();
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
    assert_eq!(
        result.answer,
        A::PlaneDistance {
            squared_distance: q(25),
            distance: rational(q(5)),
            coincident: false
        }
    );
    let mut last = template;
    last.knowledge_id = "ZZZZ_GEOMETRY_CONFLICT".into();
    last.activation_metadata
        .insert("conflicts_with".into(), json!(["GEOMETRY_PLANE_DISTANCE"]));
    space.register_batch(vec![last]).unwrap();
    assert_eq!(
        compute_with_knowledge(&request, &space).unwrap_err(),
        "KNOWLEDGE_CONFLICT"
    );
    println!(
        "100000 geometry candidates: {} ms; exact answer and final conflict PASS",
        elapsed.as_millis()
    );
}
