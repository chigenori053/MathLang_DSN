use super::schema::*;
use crate::intent::{
    AlgebraExpression as Expr, Constraint, ConstraintKind, ExpectedOutput, Given, Method,
    OperationIntent, Polynomial,
};
use crate::{stable_id, Provenance, SemanticState};

pub struct ParseOutcome {
    pub state: SemanticState,
    pub definition: ProblemDefinition,
}

/// Bounded English/Japanese instruction adapter. It does not guess unsupported syntax.
pub fn parse_problem(text: &str) -> Result<ParseOutcome, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty problem".into());
    }
    let provenance = Provenance::input("problem_statement", text);
    let mut state = SemanticState::default();
    let problem_ref = state
        .bind("problem", "pif", "global", "PROBLEM", provenance.clone())
        .map_err(|e| e.to_string())?;
    let occurrence = state
        .get(&problem_ref)
        .ok_or("problem occurrence absent")?
        .occurrence_id()
        .to_owned();
    let x = state
        .bind("x", "pif", "global", "VARIABLE", provenance.clone())
        .map_err(|e| e.to_string())?;
    let clean = text.trim_end_matches(['.', '。']).trim();
    let ambiguous = clean.starts_with("Solve or factor ") || clean.contains("または因数分解");
    let (kind, body, qualifier) = if ambiguous {
        (None, "", vec![])
    } else if let Some(body) = clean.strip_prefix("Find all solutions of ") {
        (
            Some(OperationIntent::Solve),
            body,
            vec![IntentQualifier::AllSolutions],
        )
    } else if let Some(body) = clean.strip_prefix("Solve ") {
        (
            Some(OperationIntent::Solve),
            body,
            vec![IntentQualifier::AllSolutions],
        )
    } else if let Some(body) = clean.strip_prefix("Factor ") {
        (Some(OperationIntent::Factor), body, vec![])
    } else if let Some(body) = clean.strip_prefix("Expand ") {
        (Some(OperationIntent::Expand), body, vec![])
    } else if let Some(body) = clean.strip_prefix("Verify that ") {
        (Some(OperationIntent::Verify), body, vec![])
    } else if let Some(body) = clean.strip_suffix(" のすべての解を求めよ") {
        (
            Some(OperationIntent::Solve),
            body,
            vec![IntentQualifier::AllSolutions],
        )
    } else if let Some(body) = clean.strip_suffix(" を解け") {
        (
            Some(OperationIntent::Solve),
            body,
            vec![IntentQualifier::AllSolutions],
        )
    } else if let Some(body) = clean.strip_suffix(" を因数分解せよ") {
        (Some(OperationIntent::Factor), body, vec![])
    } else if let Some(body) = clean.strip_suffix(" を展開せよ") {
        (Some(OperationIntent::Expand), body, vec![])
    } else if let Some(body) = clean.strip_suffix(" の解であることを確認せよ") {
        (Some(OperationIntent::Verify), body, vec![])
    } else {
        (None, "", vec![])
    };
    let status = if ambiguous {
        IntentStatus::Ambiguous
    } else if kind.is_none() {
        IntentStatus::Unknown
    } else {
        IntentStatus::Known
    };
    let mut intent = ProblemIntent {
        schema_id: INTENT_SCHEMA.into(),
        problem_id: String::new(),
        givens: vec![],
        constraints: vec![],
        primary_goal: None,
        subgoals: vec![],
        operation_intent: OperationSpec {
            kind: kind.clone(),
            qualifiers: qualifier,
            source_occurrence_id: occurrence.clone(),
            provenance: provenance.clone(),
        },
        expected_output: None,
        method_constraints: vec![],
        completion_criteria: vec![],
        intent_status: status,
        source_occurrence_id: occurrence.clone(),
        provenance: provenance.clone(),
    };
    if let Some(kind) = kind {
        let (body, constraint) = if let Some((math, condition)) = body.split_once(" where ") {
            (math.trim(), Some(condition.trim()))
        } else if let Some((math, condition)) = body.split_once(" ただし ") {
            (math.trim(), Some(condition.trim()))
        } else {
            (body.trim(), None)
        };
        let (body, method) = if let Some(math) = body.strip_suffix(" using factorization") {
            (math.trim(), Some(Method::Factorization))
        } else if let Some(math) = body.strip_suffix(" using quadratic formula") {
            (math.trim(), Some(Method::QuadraticFormula))
        } else if let Some(math) = body.strip_suffix(" 因数分解を用いて") {
            (math.trim(), Some(Method::Factorization))
        } else {
            (body, None)
        };
        let (math, relation) = if kind == OperationIntent::Verify {
            if let Some((candidate, equation)) = body.split_once(" is a solution of ") {
                (equation, Some(candidate.trim().to_owned()))
            } else if let Some((candidate, equation)) = body.split_once(" が ") {
                (equation, Some(candidate.trim().to_owned()))
            } else {
                return Err("unsupported verification form".into());
            }
        } else {
            (body, None)
        };
        let equation = match kind {
            OperationIntent::Solve | OperationIntent::Verify => parse_equation(math)?,
            OperationIntent::Factor | OperationIntent::Expand => parse_polynomial(math)?,
            _ => return Err("unsupported operation".into()),
        };
        let output_kind = match kind {
            OperationIntent::Solve => ExpectedOutput::SolutionSet,
            OperationIntent::Verify => ExpectedOutput::Boolean,
            _ => ExpectedOutput::Expression,
        };
        let cardinality = if kind == OperationIntent::Solve {
            Cardinality::ExactSet
        } else {
            Cardinality::One
        };
        let policy = if kind == OperationIntent::Solve {
            EquivalencePolicy::SetEquivalence
        } else if kind == OperationIntent::Verify {
            EquivalencePolicy::LogicalEquivalence
        } else {
            EquivalencePolicy::AlgebraicEquivalence
        };
        let problem_id = stable_id(
            "problem-intent",
            &[
                &format!("{:?}", kind),
                &format!("{:?}", equation.coefficients),
                &format!("{:?}", constraint),
                &format!("{:?}", method),
            ],
        );
        let output_id = stable_id("output", &[&problem_id, &format!("{:?}", output_kind)]);
        let goal_id = stable_id("goal", &[&problem_id, "primary"]);
        let goal_kind = match kind {
            OperationIntent::Solve => GoalKind::Solve,
            OperationIntent::Factor => GoalKind::Factor,
            OperationIntent::Expand => GoalKind::Expand,
            OperationIntent::Verify => GoalKind::Verify,
            _ => unreachable!(),
        };
        let kinds = if kind == OperationIntent::Solve {
            vec![
                CriterionKind::AllSolutionsFound,
                CriterionKind::NoExtraneousSolutions,
                CriterionKind::OutputTypeMatched,
                CriterionKind::ConstraintsSatisfied,
            ]
        } else {
            vec![
                CriterionKind::TargetDerived,
                CriterionKind::OutputTypeMatched,
            ]
        };
        intent.problem_id = problem_id.clone();
        intent.givens.push(Given {
            equation,
            provenance: provenance.clone(),
            source_occurrence_id: occurrence.clone(),
        });
        intent.expected_output = Some(OutputSpec {
            id: output_id.clone(),
            kind: output_kind,
            cardinality,
            domain_ref: Some(x.clone()),
            equivalence_policy: policy,
            source_occurrence_id: occurrence.clone(),
            provenance: provenance.clone(),
        });
        intent.primary_goal = Some(Goal {
            id: goal_id.clone(),
            kind: goal_kind,
            target_refs: vec![x.clone()],
            required_relation: relation,
            expected_output_ref: output_id,
            parent_goal: None,
            dependencies: vec![],
            source_occurrence_id: occurrence.clone(),
            provenance: provenance.clone(),
        });
        for criterion in kinds {
            intent.completion_criteria.push(CompletionCriterion {
                id: stable_id("criterion", &[&goal_id, &format!("{:?}", criterion)]),
                kind: criterion,
                target_refs: vec![x.clone()],
                quantifier: Quantifier::All,
                required: true,
                source_occurrence_id: occurrence.clone(),
                provenance: provenance.clone(),
            });
        }
        if let Some(method) = method {
            intent.method_constraints.push(MethodRequirement {
                id: stable_id("method-requirement", &[&problem_id]),
                required: method,
                source_occurrence_id: occurrence.clone(),
                provenance: provenance.clone(),
            });
            intent.completion_criteria.push(CompletionCriterion {
                id: stable_id("criterion", &[&goal_id, "method"]),
                kind: CriterionKind::MethodUsed,
                target_refs: vec![x.clone()],
                quantifier: Quantifier::All,
                required: true,
                source_occurrence_id: occurrence.clone(),
                provenance: provenance.clone(),
            });
        }
        if let Some(condition) = constraint {
            let kind = parse_constraint(condition)?;
            intent.constraints.push(Constraint {
                kind,
                source_occurrence_id: occurrence.clone(),
                provenance: provenance.clone(),
            });
        }
    } else {
        intent.problem_id = stable_id("problem-intent-uncertain", &[text]);
    }
    let definition = ProblemDefinition::new(intent, &state)?;
    Ok(ParseOutcome { state, definition })
}

fn parse_constraint(text: &str) -> Result<ConstraintKind, String> {
    let t = text.trim();
    if let Some(interval) = t.strip_prefix("x in [").and_then(|s| s.strip_suffix(']')) {
        let (left, right) = interval.split_once(',').ok_or("invalid interval")?;
        let left = left
            .trim()
            .parse::<i64>()
            .map_err(|_| "invalid interval lower bound")?;
        let right = right
            .trim()
            .parse::<i64>()
            .map_err(|_| "invalid interval upper bound")?;
        if left > right {
            return Err("empty interval".into());
        }
        return Ok(ConstraintKind::ClosedInterval(left, right));
    }
    for (symbol, constructor) in [
        (
            ">=",
            ConstraintKind::GreaterOrEqual as fn(i64) -> ConstraintKind,
        ),
        ("<=", ConstraintKind::LessOrEqual),
        (">", ConstraintKind::GreaterThan),
        ("<", ConstraintKind::LessThan),
        ("!=", ConstraintKind::NotEqual),
    ] {
        if let Some(number) = t.strip_prefix(&format!("x {symbol} ")) {
            return number
                .parse::<i64>()
                .map(constructor)
                .map_err(|_| "invalid constraint number".into());
        }
    }
    Err("unsupported constraint".into())
}

fn parse_equation(text: &str) -> Result<Polynomial, String> {
    let (left, right) = text.split_once('=').ok_or("equation required")?;
    let a = parse_polynomial(left)?;
    let b = parse_polynomial(right)?;
    let mut coefficients = [0; 3];
    for (i, slot) in coefficients.iter_mut().enumerate() {
        *slot = a.coefficients[i]
            .checked_sub(b.coefficients[i])
            .ok_or("coefficient overflow")?;
    }
    Ok(Polynomial { coefficients })
}

fn parse_polynomial(text: &str) -> Result<Polynomial, String> {
    let tokens = tokenize(text)?;
    let mut cursor = 0;
    let expression = parse_sum(&tokens, &mut cursor)?;
    if cursor != tokens.len() {
        return Err("trailing mathematical tokens".into());
    }
    let coefficients = expression
        .coefficients()
        .ok_or("expression exceeds quadratic scope")?;
    let mut checked = [0; 3];
    for (slot, value) in checked.iter_mut().zip(coefficients) {
        *slot = i64::try_from(value).map_err(|_| "coefficient overflow")?;
        if slot.unsigned_abs() > 1_000_000 {
            return Err("coefficient resource limit".into());
        }
    }
    Ok(Polynomial {
        coefficients: checked,
    })
}

fn tokenize(text: &str) -> Result<Vec<String>, String> {
    let mut tokens = vec![];
    let chars: Vec<_> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(chars[start..i].iter().collect());
        } else if chars[i] == '²' {
            tokens.push("^".into());
            tokens.push("2".into());
            i += 1;
        } else if "x+-*()^".contains(chars[i]) {
            if chars[i] == 'x'
                && tokens
                    .last()
                    .is_some_and(|previous: &String| previous.parse::<i64>().is_ok())
            {
                tokens.push("*".into());
            }
            tokens.push(chars[i].to_string());
            i += 1;
        } else {
            return Err(format!("unsupported mathematical token: {}", chars[i]));
        }
    }
    Ok(tokens)
}

fn parse_sum(t: &[String], p: &mut usize) -> Result<Expr, String> {
    let mut left = parse_product(t, p)?;
    while matches!(t.get(*p).map(String::as_str), Some("+" | "-")) {
        let op = t[*p].as_str();
        *p += 1;
        let right = parse_product(t, p)?;
        left = if op == "+" {
            Expr::Add(Box::new(left), Box::new(right))
        } else {
            Expr::Subtract(Box::new(left), Box::new(right))
        };
    }
    Ok(left)
}

fn parse_product(t: &[String], p: &mut usize) -> Result<Expr, String> {
    let mut left = parse_power(t, p)?;
    while t.get(*p).map(String::as_str) == Some("*") {
        *p += 1;
        left = Expr::Multiply(Box::new(left), Box::new(parse_power(t, p)?));
    }
    Ok(left)
}

fn parse_power(t: &[String], p: &mut usize) -> Result<Expr, String> {
    let mut left = parse_atom(t, p)?;
    if t.get(*p).map(String::as_str) == Some("^") {
        *p += 1;
        if t.get(*p).map(String::as_str) != Some("2") {
            return Err("only square is supported".into());
        }
        *p += 1;
        left = Expr::Square(Box::new(left));
    }
    Ok(left)
}

fn parse_atom(t: &[String], p: &mut usize) -> Result<Expr, String> {
    let token = t.get(*p).ok_or("expected mathematical atom")?.as_str();
    *p += 1;
    match token {
        "x" => Ok(Expr::Variable),
        "-" => Ok(Expr::Subtract(
            Box::new(Expr::Constant(0)),
            Box::new(parse_atom(t, p)?),
        )),
        "+" => parse_atom(t, p),
        "(" => {
            let inner = parse_sum(t, p)?;
            if t.get(*p).map(String::as_str) != Some(")") {
                return Err("unclosed parenthesis".into());
            }
            *p += 1;
            Ok(inner)
        }
        number => number
            .parse::<i64>()
            .map(Expr::Constant)
            .map_err(|_| "invalid number".into()),
    }
}
