use mathlang_mirp::calculation::{
    calculation_knowledge, compute, compute_with_knowledge, verify_answer, CalculationAnswer as A,
    CalculationRequest as R,
};
use mathlang_mirp::knowledge::KnowledgeSpace;
use mathlang_mirp::{Object, SemanticState};
use serde_json::{json, Value};

const OPS: [&str; 6] = [
    "constant_acceleration",
    "newton_force",
    "newton_acceleration",
    "constant_force_work",
    "kinetic_energy",
    "work_energy",
];
fn fixture(op: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/fixtures/calculation/physics_{op}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
fn request(mut v: Value) -> R {
    // Keep integer test mutations concise; the wire format uses rational objects.
    for quantity in v.as_object_mut().unwrap().values_mut() {
        if let Some(n) = quantity.get("value").and_then(Value::as_i64) {
            quantity["value"] = rat(n);
        }
        if let Some(values) = quantity.get_mut("values").and_then(Value::as_array_mut) {
            for value in values {
                if let Some(n) = value.as_i64() {
                    *value = rat(n);
                }
            }
        }
    }
    serde_json::from_value(v).unwrap()
}
fn rat(n: i64) -> Value {
    json!({"numerator":n.to_string(),"denominator":"1"})
}
fn outcome(v: Value) -> Value {
    let r = request(v);
    let result = compute(&r).unwrap();
    verify_answer(&r, &result.answer).unwrap();
    serde_json::to_value(result.answer).unwrap()["result"]["outcome"].clone()
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
fn six_knowledge_units_select_execute_verify_explain_and_record_evidence() {
    for op in OPS {
        let r = request(fixture(op));
        let id = format!("PHYSICS_{}", op.to_uppercase());
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
        assert!(result.explanation[0].contains(&id));
        assert!(result
            .explanation
            .iter()
            .any(|step| step.contains("条件付き")));
        verify_answer(&r, &result.answer).unwrap();
        assert!(result.semantic_state.objects.iter().any(|o|matches!(o,Object::Evidence(e) if e.common.metadata["knowledge_runtime"]["knowledge_id"]==id)));
        result.semantic_state.validate().unwrap();
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
fn converted_motion_force_work_and_energy_form_a_consistent_chain() {
    let motion = outcome(fixture("constant_acceleration"));
    assert_eq!(motion["position"]["values"], json!([rat(25)]));
    assert_eq!(motion["displacement"]["values"], json!([rat(24)]));
    assert_eq!(motion["velocity"]["values"], json!([rat(14)]));
    assert_eq!(motion["elapsed"]["value"], rat(2));
    assert_eq!(
        outcome(fixture("newton_force"))["net_force"]["values"],
        json!([rat(3)])
    );
    assert_eq!(
        outcome(fixture("newton_acceleration"))["acceleration"]["values"],
        json!([rat(2)])
    );
    assert_eq!(
        outcome(fixture("constant_force_work"))["work"]["value"],
        rat(72)
    );
    assert_eq!(
        outcome(fixture("kinetic_energy"))["kinetic_energy"]["value"],
        rat(147)
    );
    let balance = outcome(fixture("work_energy"));
    assert_eq!(balance["initial_kinetic_energy"]["value"], rat(75));
    assert_eq!(balance["final_kinetic_energy"]["value"], rat(147));
    assert_eq!(balance["kinetic_energy_change"]["value"], rat(72));
    assert_eq!(balance["residual"]["value"], rat(0));
    assert_eq!(balance["consistent"], true);
}

#[test]
fn multidimensional_signed_and_zero_cases_keep_exact_values() {
    for dimension in 1..=3 {
        let mut v = fixture("constant_acceleration");
        v["context"]["frame"]["dimension"] = json!(dimension);
        v["context"]["frame"]["axes"] = json!(["x", "y", "z"][..dimension]);
        for (key, values) in [
            ("initial_position", vec![0; dimension]),
            ("initial_velocity", vec![-3; dimension]),
            ("acceleration", vec![-2; dimension]),
        ] {
            v[key]["values"] = json!(values);
        }
        v["initial_position"]["unit"] = json!("m");
        v["initial_velocity"]["unit"] = json!("m/s");
        v["acceleration"]["unit"] = json!("m/s^2");
        v["duration"] = json!({"value":2,"unit":"s"});
        let out = outcome(v.clone());
        assert_eq!(out["position"]["values"], json!(vec![rat(-10); dimension]));
        assert_eq!(out["velocity"]["values"], json!(vec![rat(-7); dimension]));
        v["duration"]["value"] = json!(0);
        let out = outcome(v);
        assert_eq!(out["position"]["values"], json!(vec![rat(0); dimension]));
        assert_eq!(out["velocity"]["values"], json!(vec![rat(-3); dimension]));
    }
    let mut v = fixture("constant_force_work");
    v["context"]["frame"]["dimension"] = json!(2);
    v["context"]["frame"]["axes"] = json!(["x", "y"]);
    v["force"]["values"] = json!([3, 4]);
    v["force"]["unit"] = json!("N");
    v["displacement"]["unit"] = json!("m");
    for (displacement, work) in [
        (json!([4, -3]), 0),
        (json!([-3, -4]), -25),
        (json!([0, 0]), 0),
    ] {
        v["displacement"]["values"] = displacement;
        assert_eq!(outcome(v.clone())["work"]["value"], rat(work));
    }
    let mut v = fixture("kinetic_energy");
    v["velocity"]["values"] = json!([-14]);
    assert_eq!(outcome(v.clone())["kinetic_energy"]["value"], rat(147));
    v["velocity"]["values"] = json!([0]);
    assert_eq!(outcome(v)["kinetic_energy"]["value"], rat(0));
}

#[test]
fn inconsistent_work_is_a_verified_assessment_and_losses_are_signed() {
    let mut v = fixture("work_energy");
    v["net_work"]["value"] = json!(71);
    let r = request(v.clone());
    let result = compute(&r).unwrap();
    assert_eq!(result.verification["status"], "VERIFIED");
    let out = outcome(v.clone());
    assert_eq!(out["consistent"], false);
    assert_eq!(out["residual"]["value"], rat(-1));
    v["initial_velocity"]["values"] = json!([14]);
    v["final_velocity"]["values"] = json!([10]);
    v["net_work"]["value"] = json!(-72);
    let out = outcome(v);
    assert_eq!(out["consistent"], true);
    assert_eq!(out["kinetic_energy_change"]["value"], rat(-72));
}

#[test]
fn units_are_case_sensitive_dimension_checked_and_bounded() {
    for unit in [
        "kg",
        "m/s",
        "n",
        "unknown",
        "N//m",
        "N/",
        "/N",
        "N^5",
        "N^2147483647",
        "N^x",
        "N**1",
        "Ｎ",
        "N^1^1",
        "N*1*1*1*1*1*1*1*1",
    ] {
        let mut v = fixture("newton_acceleration");
        v["net_force"]["unit"] = json!(unit);
        reject(v);
    }
    for (op, key, unit) in [
        ("constant_acceleration", "duration", "m"),
        ("constant_acceleration", "initial_position", "s"),
        ("constant_acceleration", "initial_velocity", "m/s^2"),
        ("constant_acceleration", "acceleration", "m/s"),
        ("newton_force", "mass", "N"),
        ("constant_force_work", "displacement", "m^2"),
        ("kinetic_energy", "mass", "s"),
        ("work_energy", "net_work", "N"),
    ] {
        let mut v = fixture(op);
        v[key]["unit"] = json!(unit);
        reject(v);
    }
    for (unit, value) in [
        ("N", 3),
        ("mN", 3000),
        ("kg*m/s^2", 3),
        ("kg*m*s^-2", 3),
        ("kg*m/s/s", 3),
    ] {
        let mut v = fixture("newton_acceleration");
        v["net_force"]["unit"] = json!(unit);
        v["net_force"]["values"] = json!([value]);
        assert_eq!(outcome(v)["acceleration"]["values"], json!([rat(2)]));
    }
    let mut v = fixture("work_energy");
    v["net_work"]["unit"] = json!("kJ");
    v["net_work"]["value"] = json!({"numerator":9,"denominator":125});
    assert_eq!(outcome(v)["consistent"], true);
    let mut v = fixture("kinetic_energy");
    v["mass"]["value"] = json!({"numerator":1,"denominator":0});
    reject(v);
    let mut v = fixture("kinetic_energy");
    v["mass"]["value"] = json!({"numerator":"1".repeat(81),"denominator":1});
    reject(v);
}

#[test]
fn frames_models_and_assumptions_are_required_and_not_inferred() {
    let base = fixture("constant_acceleration");
    for (key, value) in [
        ("dimension", json!(0)),
        ("dimension", json!(4)),
        ("dimension", json!(2)),
        ("axes", json!([])),
        ("id", json!("")),
        ("origin", json!(" ")),
        ("kind", json!("ROTATING")),
        ("kind", json!("ACCELERATING")),
        ("basis", json!("OBLIQUE")),
    ] {
        let mut v = base.clone();
        v["context"]["frame"][key] = value;
        reject(v);
    }
    let mut v = base.clone();
    v["context"]["frame"]["dimension"] = json!(2);
    v["context"]["frame"]["axes"] = json!(["x", "x"]);
    reject(v);
    for model in ["RIGID_BODY", "RELATIVISTIC_PARTICLE"] {
        let mut v = base.clone();
        v["context"]["model"] = json!(model);
        reject(v);
    }
    for key in [
        "initial_position",
        "initial_velocity",
        "acceleration",
        "duration",
    ] {
        let mut v = base.clone();
        v[key]["frame_id"] = json!("other");
        reject(v);
    }
    let mut v = base.clone();
    v["initial_position"]["values"] = json!([1, 2]);
    reject(v);
    let mut v = fixture("work_energy");
    v["net_work"].as_object_mut().unwrap().remove("frame_id");
    reject(v);
    for op in OPS {
        let base = fixture(op);
        let assumptions = base["context"]["assumptions"].as_array().unwrap();
        for i in 0..assumptions.len() {
            let mut v = base.clone();
            v["context"]["assumptions"]
                .as_array_mut()
                .unwrap()
                .remove(i);
            reject(v);
        }
        let mut v = base;
        let list = v["context"]["assumptions"].as_array_mut().unwrap();
        list.push(list[0].clone());
        reject(v);
    }
    for key in ["model", "frame", "assumptions"] {
        let mut v = base.clone();
        v["context"].as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<R>(v).is_err());
    }
    let mut v = base;
    v["context"]["unexamined"] = json!(true);
    assert!(serde_json::from_value::<R>(v).is_err());
}

#[test]
fn mass_duration_and_speed_domains_are_enforced() {
    for op in [
        "newton_force",
        "newton_acceleration",
        "kinetic_energy",
        "work_energy",
    ] {
        for mass in [0, -1] {
            let mut v = fixture(op);
            v["mass"]["value"] = json!(mass);
            reject(v);
        }
    }
    let mut v = fixture("constant_acceleration");
    v["duration"]["value"] = json!(-1);
    reject(v);
    for (op, key) in [
        ("kinetic_energy", "velocity"),
        ("work_energy", "initial_velocity"),
        ("work_energy", "final_velocity"),
        ("constant_acceleration", "initial_velocity"),
    ] {
        let mut v = fixture(op);
        v[key]["unit"] = json!("m/s");
        v[key]["values"] = json!([299792458]);
        reject(v);
    }
    let mut v = fixture("constant_acceleration");
    v["acceleration"]["unit"] = json!("m/s^2");
    v["acceleration"]["values"] = json!([299792458]);
    reject(v);
    let mut v = fixture("kinetic_energy");
    v["context"]["frame"]["dimension"] = json!(2);
    v["context"]["frame"]["axes"] = json!(["x", "y"]);
    v["velocity"]["values"] = json!([220000000, 220000000]);
    reject(v);
}

#[test]
fn verification_rejects_altered_values_units_frames_context_and_assessment() {
    for (op, path) in [
        ("constant_acceleration", vec!["position", "values", "0"]),
        ("constant_acceleration", vec!["velocity", "values", "0"]),
        ("constant_acceleration", vec!["displacement", "values", "0"]),
        ("constant_acceleration", vec!["elapsed", "value"]),
        ("newton_force", vec!["net_force", "values", "0"]),
        ("newton_acceleration", vec!["acceleration", "values", "0"]),
        ("constant_force_work", vec!["work", "value"]),
        ("kinetic_energy", vec!["kinetic_energy", "value"]),
        ("work_energy", vec!["initial_kinetic_energy", "value"]),
        ("work_energy", vec!["final_kinetic_energy", "value"]),
        ("work_energy", vec!["kinetic_energy_change", "value"]),
        ("work_energy", vec!["net_work", "value"]),
        ("work_energy", vec!["residual", "value"]),
    ] {
        let r = request(fixture(op));
        let original = serde_json::to_value(compute(&r).unwrap().answer).unwrap();
        let mut v = original.clone();
        let mut node = &mut v["result"]["outcome"];
        for key in &path {
            node = if *key == "0" {
                &mut node[0]
            } else {
                &mut node[*key]
            };
        }
        *node = rat(999);
        assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
        let quantity = path[0];
        for (key, value) in [("unit", json!("kJ")), ("frame_id", json!("other"))] {
            let mut v = original.clone();
            v["result"]["outcome"][quantity][key] = value;
            assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
        }
        let mut v = original.clone();
        v["result"]["context"]["frame"]["origin"] = json!("other origin");
        assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
        let mut v = original;
        v["result"]["required_assumptions"] = json!([]);
        assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
    }
    let r = request(fixture("work_energy"));
    let mut v = serde_json::to_value(compute(&r).unwrap().answer).unwrap();
    v["result"]["outcome"]["consistent"] = json!(false);
    assert!(verify_answer(&r, &serde_json::from_value::<A>(v).unwrap()).is_err());
}

#[test]
#[ignore = "explicit 100k same-goal classical physics performance and conflict gate"]
fn physics_candidates_scale_without_hiding_conflicts() {
    let mut space = calculation_knowledge();
    let template = space.get("PHYSICS_NEWTON_FORCE").unwrap().clone();
    let units = (1..100_000)
        .map(|i| {
            let mut u = template.clone();
            u.knowledge_id = format!("ZLOAD_{i:06}");
            u
        })
        .collect();
    space.register_batch(units).unwrap();
    let r = request(fixture("newton_force"));
    let expected = compute(&r).unwrap().answer;
    let start = std::time::Instant::now();
    let result = compute_with_knowledge(&r, &space).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_secs(20));
    assert_eq!(result.answer, expected);
    verify_answer(&r, &result.answer).unwrap();
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
    let mut last = template;
    last.knowledge_id = "ZZZZ_PHYSICS_CONFLICT".into();
    last.activation_metadata
        .insert("conflicts_with".into(), json!(["PHYSICS_NEWTON_FORCE"]));
    space.register_batch(vec![last]).unwrap();
    assert_eq!(
        compute_with_knowledge(&r, &space).unwrap_err(),
        "KNOWLEDGE_CONFLICT"
    );
    println!(
        "100000 physics candidates: {} ms; exact answer and final conflict PASS",
        elapsed.as_millis()
    );
}
