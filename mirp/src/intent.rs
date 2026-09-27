//! Intent-aware assessment for a bounded, exact quadratic algebra contract.
//! Unsupported mathematics stays UNKNOWN; an answer is never checked against a reference path.

use crate::{MirpError, Object, Provenance, SemanticDelta, SemanticState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Polynomial {
    /// Constant, linear, and quadratic coefficients, respectively.
    pub coefficients: [i64; 3],
}

impl Polynomial {
    fn at(&self, x: i64) -> i128 {
        let [c, b, a] = self.coefficients.map(i128::from);
        let x = i128::from(x);
        a * x * x + b * x + c
    }

    fn supported(&self) -> bool {
        self.coefficients
            .iter()
            .all(|x| x.unsigned_abs() <= 1_000_000)
    }
}

/// Small algebra syntax tree; display strings are never used as proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgebraExpression {
    Constant(i64),
    Variable,
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Square(Box<Self>),
}

impl AlgebraExpression {
    fn coefficients(&self) -> Option<[i128; 3]> {
        fn add(a: [i128; 3], b: [i128; 3], sign: i128) -> Option<[i128; 3]> {
            Some([
                a[0].checked_add(sign.checked_mul(b[0])?)?,
                a[1].checked_add(sign.checked_mul(b[1])?)?,
                a[2].checked_add(sign.checked_mul(b[2])?)?,
            ])
        }
        fn multiply(a: [i128; 3], b: [i128; 3]) -> Option<[i128; 3]> {
            if a[1]
                .checked_mul(b[2])?
                .checked_add(a[2].checked_mul(b[1])?)?
                != 0
                || a[2].checked_mul(b[2])? != 0
            {
                return None;
            }
            Some([
                a[0].checked_mul(b[0])?,
                a[0].checked_mul(b[1])?
                    .checked_add(a[1].checked_mul(b[0])?)?,
                a[0].checked_mul(b[2])?
                    .checked_add(a[1].checked_mul(b[1])?)?
                    .checked_add(a[2].checked_mul(b[0])?)?,
            ])
        }
        match self {
            Self::Constant(n) => Some([i128::from(*n), 0, 0]),
            Self::Variable => Some([0, 1, 0]),
            Self::Add(a, b) => add(a.coefficients()?, b.coefficients()?, 1),
            Self::Subtract(a, b) => add(a.coefficients()?, b.coefficients()?, -1),
            Self::Multiply(a, b) => multiply(a.coefficients()?, b.coefficients()?),
            Self::Square(a) => {
                let p = a.coefficients()?;
                multiply(p, p)
            }
        }
    }
}

fn equivalent_equations(before: &Polynomial, after: [i128; 3]) -> bool {
    let a = before.coefficients.map(i128::from);
    a.iter().position(|n| *n != 0).is_some_and(|i| {
        after[i] != 0 && (0..3).all(|j| a[j].checked_mul(after[i]) == after[j].checked_mul(a[i]))
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Given {
    pub equation: Polynomial,
    pub provenance: Provenance,
    pub source_occurrence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintKind {
    GreaterThan(i64),
    GreaterOrEqual(i64),
    LessThan(i64),
    LessOrEqual(i64),
    NotEqual(i64),
    ClosedInterval(i64, i64),
}

impl ConstraintKind {
    fn accepts(&self, x: i64) -> bool {
        match self {
            Self::GreaterThan(n) => x > *n,
            Self::GreaterOrEqual(n) => x >= *n,
            Self::LessThan(n) => x < *n,
            Self::LessOrEqual(n) => x <= *n,
            Self::NotEqual(n) => x != *n,
            Self::ClosedInterval(a, b) => *a <= x && x <= *b,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub provenance: Provenance,
    pub source_occurrence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    /// MIRP semantic entity ID, never a display string or fixture ID.
    pub semantic_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationIntent {
    Solve,
    Calculate,
    Simplify,
    Expand,
    Factor,
    Prove,
    Compare,
    Determine,
    FindMaximum,
    FindMinimum,
    Differentiate,
    Integrate,
    Graph,
    Explain,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpectedOutput {
    Scalar,
    Expression,
    Equation,
    Solution,
    SolutionSet,
    Interval,
    Coordinate,
    Vector,
    Matrix,
    Function,
    GraphProperty,
    Proof,
    Boolean,
    Explanation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    Factorization,
    QuadraticFormula,
    CompletingSquare,
    Differentiation,
    Graphical,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodConstraint {
    pub required: Method,
    pub provenance: Provenance,
    pub source_occurrence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompletionCriterion {
    AllValidSolutions,
    NoExtraneousSolutions,
    OutputType(ExpectedOutput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemIntent {
    pub given: Vec<Given>,
    pub constraints: Vec<Constraint>,
    pub target: Target,
    pub operation_intent: OperationIntent,
    pub expected_output: ExpectedOutput,
    pub method_constraints: Vec<MethodConstraint>,
    pub completion_criteria: Vec<CompletionCriterion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problem {
    pub statement_provenance: Provenance,
    pub statement_occurrence_id: String,
    pub intent: ProblemIntent,
}

impl Problem {
    pub fn validate(&self, state: &SemanticState) -> Result<(), String> {
        let source = state
            .get_occurrence(&self.statement_occurrence_id)
            .ok_or("problem statement occurrence is absent")?;
        if source.common().provenance.source_id != self.statement_provenance.source_id {
            return Err("problem provenance does not match MIRP".into());
        }
        self.intent.validate(state)
    }
}

impl ProblemIntent {
    pub fn validate(&self, state: &SemanticState) -> Result<(), String> {
        if !matches!(state.get(&self.target.semantic_id), Some(Object::Entity(_))) {
            return Err("target is not a MIRP entity".into());
        }
        if self.given.is_empty() {
            return Err("problem has no given".into());
        }
        for given in &self.given {
            let source = state
                .get_occurrence(&given.source_occurrence_id)
                .ok_or("given occurrence is absent")?;
            if source.common().provenance.source_id != given.provenance.source_id
                || given.provenance.source_id.is_empty()
            {
                return Err("given provenance does not match MIRP".into());
            }
        }
        for constraint in &self.constraints {
            let source = state
                .get_occurrence(&constraint.source_occurrence_id)
                .ok_or("constraint occurrence is absent")?;
            if source.common().provenance.source_id != constraint.provenance.source_id {
                return Err("constraint provenance does not match MIRP".into());
            }
            if let ConstraintKind::ClosedInterval(a, b) = constraint.kind {
                if a > b {
                    return Err("empty interval".into());
                }
            }
        }
        for method in &self.method_constraints {
            let source = state
                .get_occurrence(&method.source_occurrence_id)
                .ok_or("method occurrence is absent")?;
            if source.common().provenance.source_id != method.provenance.source_id {
                return Err("method provenance does not match MIRP".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Claim {
    /// Both sides are structured polynomials; a display string is never trusted as proof.
    Transformation {
        before: Polynomial,
        after: Polynomial,
    },
    ExpressionTransformation {
        before: Polynomial,
        after: AlgebraExpression,
    },
    /// A solution claim may be a valid subset of the full set.
    Solutions {
        values: Vec<i64>,
        output: ExpectedOutput,
    },
    /// A true but unrelated calculation can be VALID + IRRELEVANT.
    ArithmeticEquality { left: i64, right: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub target: Target,
    pub source_occurrence_ids: Vec<String>,
    pub method: Option<Method>,
    pub claim: Claim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MathematicalValidity {
    Valid,
    Invalid,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemRelevance {
    Relevant,
    Irrelevant,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentAlignment {
    Aligned,
    Partial,
    Misaligned,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MethodCompliance {
    Satisfied,
    Violation,
    NotRequired,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GoalCompletion {
    Complete,
    Incomplete,
    Overcomplete,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EducationalError {
    CalculationError,
    AlgebraicTransformationError,
    ConstraintError,
    MissingSolution,
    ExtraneousSolution,
    WrongTarget,
    MethodViolation,
    IrrelevantReasoning,
    IncompleteReasoning,
    UnsupportedReasoning,
    UnknownReasoning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Elimination {
    pub candidate: i64,
    pub constraint: ConstraintKind,
    pub constraint_source_id: String,
    pub constraint_occurrence_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EducationalEvaluation {
    pub mathematical_validity: MathematicalValidity,
    pub problem_relevance: ProblemRelevance,
    pub intent_alignment: IntentAlignment,
    pub method_compliance: MethodCompliance,
    pub goal_completion: GoalCompletion,
    pub errors: Vec<EducationalError>,
    pub evidence_occurrence_ids: Vec<String>,
    pub required_solutions: Option<Vec<i64>>,
    pub eliminations: Vec<Elimination>,
}

pub struct ReasoningContext {
    pub semantic_state: SemanticState,
    pub problem_intent: ProblemIntent,
    pub reasoning_history: Vec<ReasoningStep>,
}

impl ReasoningContext {
    pub fn new(
        semantic_state: SemanticState,
        problem_intent: ProblemIntent,
    ) -> Result<Self, String> {
        problem_intent.validate(&semantic_state)?;
        Ok(Self {
            semantic_state,
            problem_intent,
            reasoning_history: vec![],
        })
    }

    /// Evaluates a candidate without accepting an invalid or unknown step into history.
    pub fn evaluate(&self, step: &ReasoningStep) -> EducationalEvaluation {
        let intent = &self.problem_intent;
        if intent.validate(&self.semantic_state).is_err() {
            return EducationalEvaluation {
                mathematical_validity: MathematicalValidity::Unknown,
                problem_relevance: ProblemRelevance::Unknown,
                intent_alignment: IntentAlignment::Unknown,
                method_compliance: MethodCompliance::Unknown,
                goal_completion: GoalCompletion::Unknown,
                errors: vec![EducationalError::UnknownReasoning],
                evidence_occurrence_ids: vec![],
                required_solutions: None,
                eliminations: vec![],
            };
        }
        let supported_intent = intent.operation_intent == OperationIntent::Solve
            && intent.expected_output == ExpectedOutput::SolutionSet
            && intent.given.len() == 1;
        let evidence: Vec<_> = step
            .source_occurrence_ids
            .iter()
            .filter(|id| self.semantic_state.get_occurrence(id).is_some())
            .cloned()
            .collect();
        let given_ids: BTreeSet<_> = intent
            .given
            .iter()
            .map(|g| g.source_occurrence_id.as_str())
            .collect();
        let cites_given = evidence.iter().any(|id| given_ids.contains(id.as_str()));
        let claim_related = match &step.claim {
            Claim::Transformation { before, .. }
            | Claim::ExpressionTransformation { before, .. } => {
                equation_matches_history(before, intent, &self.reasoning_history)
            }
            Claim::Solutions { .. } => true,
            Claim::ArithmeticEquality { .. } => false,
        };
        let relevant = step.target == intent.target && cites_given && claim_related;
        let relevance = if step.target != intent.target
            || (!step.source_occurrence_ids.is_empty()
                && evidence.len() == step.source_occurrence_ids.len()
                && !cites_given)
        {
            ProblemRelevance::Irrelevant
        } else if relevant {
            ProblemRelevance::Relevant
        } else {
            ProblemRelevance::Unknown
        };
        let equation = intent.given.first().map(|given| &given.equation);
        let mut eliminations = vec![];
        let required = equation
            .filter(|_| supported_intent)
            .and_then(integer_quadratic_roots)
            .map(|roots| {
                roots
                    .into_iter()
                    .filter(|x| {
                        let mut accepted = true;
                        for constraint in &intent.constraints {
                            if !constraint.kind.accepts(*x) {
                                accepted = false;
                                eliminations.push(Elimination {
                                    candidate: *x,
                                    constraint: constraint.kind.clone(),
                                    constraint_source_id: constraint.provenance.source_id.clone(),
                                    constraint_occurrence_id: constraint
                                        .source_occurrence_id
                                        .clone(),
                                    reason: "candidate violates the problem constraint".into(),
                                });
                            }
                        }
                        accepted
                    })
                    .collect::<Vec<_>>()
            });
        let validity = match &step.claim {
            Claim::Transformation { before, after } if before.supported() && after.supported() => {
                if equivalent_equations(before, after.coefficients.map(i128::from)) {
                    MathematicalValidity::Valid
                } else {
                    MathematicalValidity::Invalid
                }
            }
            Claim::Transformation { .. } => MathematicalValidity::Unknown,
            Claim::ExpressionTransformation { before, after } if before.supported() => {
                match after.coefficients() {
                    Some(coefficients) if equivalent_equations(before, coefficients) => {
                        MathematicalValidity::Valid
                    }
                    Some(_) => MathematicalValidity::Invalid,
                    None => MathematicalValidity::Unknown,
                }
            }
            Claim::ExpressionTransformation { .. } => MathematicalValidity::Unknown,
            Claim::Solutions { values, .. } if supported_intent => match equation {
                Some(poly) if poly.supported() => {
                    if values.iter().all(|x| poly.at(*x) == 0) {
                        MathematicalValidity::Valid
                    } else {
                        MathematicalValidity::Invalid
                    }
                }
                _ => MathematicalValidity::Unknown,
            },
            Claim::Solutions { .. } => MathematicalValidity::Unknown,
            Claim::ArithmeticEquality { left, right } => {
                if left == right {
                    MathematicalValidity::Valid
                } else {
                    MathematicalValidity::Invalid
                }
            }
        };
        let method = if intent.method_constraints.is_empty() {
            MethodCompliance::NotRequired
        } else if intent.method_constraints.iter().all(|c| {
            step.method.as_ref() == Some(&c.required)
                || self
                    .reasoning_history
                    .iter()
                    .any(|prior| prior.method.as_ref() == Some(&c.required))
        }) {
            MethodCompliance::Satisfied
        } else if step.method.is_some() || matches!(step.claim, Claim::Solutions { .. }) {
            MethodCompliance::Violation
        } else {
            MethodCompliance::Unknown
        };
        let completion = match (&step.claim, &required, &validity, &relevance) {
            (
                Claim::Solutions { values, output },
                Some(required),
                MathematicalValidity::Valid,
                ProblemRelevance::Relevant,
            ) => {
                let submitted: BTreeSet<_> = values.iter().copied().collect();
                let required: BTreeSet<_> = required.iter().copied().collect();
                let correct_type = output == &intent.expected_output;
                let all_criteria =
                    intent
                        .completion_criteria
                        .iter()
                        .all(|criterion| match criterion {
                            CompletionCriterion::AllValidSolutions => submitted == required,
                            CompletionCriterion::NoExtraneousSolutions => {
                                submitted.is_subset(&required)
                            }
                            CompletionCriterion::OutputType(kind) => output == kind,
                        });
                if correct_type && all_criteria && submitted == required {
                    GoalCompletion::Complete
                } else {
                    GoalCompletion::Incomplete
                }
            }
            (Claim::Solutions { .. }, None, _, _) => GoalCompletion::Unknown,
            (_, _, MathematicalValidity::Unknown, _) | (_, _, _, ProblemRelevance::Unknown) => {
                GoalCompletion::Unknown
            }
            _ => GoalCompletion::Incomplete,
        };
        let completion = if supported_intent {
            completion
        } else {
            GoalCompletion::Unknown
        };
        let alignment = match (&relevance, &validity, &completion, &step.claim, &method) {
            (ProblemRelevance::Irrelevant, _, _, _, _) => IntentAlignment::Misaligned,
            (ProblemRelevance::Unknown, _, _, _, _)
            | (_, MathematicalValidity::Unknown, _, _, _) => IntentAlignment::Unknown,
            (_, MathematicalValidity::Invalid, _, _, _) => IntentAlignment::Partial,
            (_, _, GoalCompletion::Complete, _, MethodCompliance::Violation) => {
                IntentAlignment::Partial
            }
            (_, _, GoalCompletion::Complete, _, _) => IntentAlignment::Aligned,
            (_, _, GoalCompletion::Incomplete, Claim::Solutions { .. }, _) => {
                IntentAlignment::Partial
            }
            (_, _, _, Claim::Transformation { .. } | Claim::ExpressionTransformation { .. }, _) => {
                IntentAlignment::Aligned
            }
            _ => IntentAlignment::Unknown,
        };
        let alignment = if supported_intent {
            alignment
        } else {
            IntentAlignment::Unknown
        };
        let mut errors = vec![];
        if step.target != intent.target {
            errors.push(EducationalError::WrongTarget);
        }
        if relevance == ProblemRelevance::Irrelevant {
            errors.push(EducationalError::IrrelevantReasoning);
        }
        if validity == MathematicalValidity::Invalid {
            errors.push(match step.claim {
                Claim::Transformation { .. } | Claim::ExpressionTransformation { .. } => {
                    EducationalError::AlgebraicTransformationError
                }
                _ => EducationalError::CalculationError,
            });
        }
        if method == MethodCompliance::Violation {
            errors.push(EducationalError::MethodViolation);
        }
        if let (Claim::Solutions { values, .. }, Some(needed)) = (&step.claim, &required) {
            if values.iter().any(|x| !needed.contains(x)) {
                errors.push(
                    if values
                        .iter()
                        .any(|x| equation.is_some_and(|p| p.at(*x) == 0))
                    {
                        EducationalError::ConstraintError
                    } else {
                        EducationalError::ExtraneousSolution
                    },
                );
            }
            if validity == MathematicalValidity::Valid && needed.iter().any(|x| !values.contains(x))
            {
                errors.push(EducationalError::MissingSolution);
            }
        }
        if completion == GoalCompletion::Incomplete && errors.is_empty() {
            errors.push(EducationalError::IncompleteReasoning);
        }
        if validity == MathematicalValidity::Unknown {
            errors.push(EducationalError::UnknownReasoning);
        }
        EducationalEvaluation {
            mathematical_validity: validity,
            problem_relevance: relevance,
            intent_alignment: alignment,
            method_compliance: method,
            goal_completion: completion,
            errors,
            evidence_occurrence_ids: evidence,
            required_solutions: required,
            eliminations,
        }
    }

    pub fn accept(&mut self, step: ReasoningStep) -> EducationalEvaluation {
        let evaluation = self.evaluate(&step);
        if evaluation.mathematical_validity == MathematicalValidity::Valid
            && evaluation.problem_relevance == ProblemRelevance::Relevant
        {
            self.reasoning_history.push(step);
        }
        evaluation
    }

    /// Apply a MIRP delta transactionally, then assess the resulting candidate state.
    /// The intent is held constant throughout the transition.
    pub fn accept_transition(
        &mut self,
        delta: SemanticDelta,
        step: ReasoningStep,
    ) -> Result<EducationalEvaluation, MirpError> {
        let mut candidate = self.semantic_state.clone();
        candidate.apply(delta)?;
        let evaluation = Self {
            semantic_state: candidate.clone(),
            problem_intent: self.problem_intent.clone(),
            reasoning_history: self.reasoning_history.clone(),
        }
        .evaluate(&step);
        if evaluation.mathematical_validity == MathematicalValidity::Valid
            && evaluation.problem_relevance == ProblemRelevance::Relevant
        {
            self.semantic_state = candidate;
            self.reasoning_history.push(step);
        }
        Ok(evaluation)
    }
}

fn equation_matches_history(
    before: &Polynomial,
    intent: &ProblemIntent,
    history: &[ReasoningStep],
) -> bool {
    intent.given.iter().any(|given| given.equation == *before)
        || history.iter().any(|step| match &step.claim {
            Claim::Transformation { after, .. } => after == before,
            Claim::ExpressionTransformation { after, .. } => after
                .coefficients()
                .is_some_and(|coefficients| equivalent_equations(before, coefficients)),
            _ => false,
        })
}

/// Exact integer roots only. A non-integral or irrational root preserves UNKNOWN completion.
fn integer_quadratic_roots(poly: &Polynomial) -> Option<Vec<i64>> {
    if !poly.supported() {
        return None;
    }
    let [c, b, a] = poly.coefficients.map(i128::from);
    if a == 0 {
        return None;
    }
    let discriminant = b * b - 4 * a * c;
    if discriminant < 0 {
        return Some(vec![]);
    }
    let root = (discriminant as f64).sqrt() as i128;
    let square = [root.saturating_sub(1), root, root + 1]
        .into_iter()
        .find(|r| r * r == discriminant)?;
    let denominator = 2 * a;
    let mut roots = vec![];
    for numerator in [-b - square, -b + square] {
        if numerator % denominator != 0 {
            return None;
        }
        roots.push(i64::try_from(numerator / denominator).ok()?);
    }
    roots.sort();
    roots.dedup();
    Some(roots)
}
