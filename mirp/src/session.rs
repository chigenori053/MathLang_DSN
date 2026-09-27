//! Bounded domain adapters and the MIRP -> ReasonScript -> SemanticDelta bridge.

use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum Domain {
    NaturalLanguage,
    Japanese,
    Mathematics,
    Code,
}

impl Domain {
    fn name(self) -> &'static str {
        match self {
            Self::NaturalLanguage => "natural_language",
            Self::Japanese => "nlsrv_japanese",
            Self::Mathematics => "mathematics",
            Self::Code => "code",
        }
    }
}

#[derive(Debug, Clone)]
enum Parsed {
    Number(i64),
    Name(String),
    Binary(String, Box<Parsed>, Box<Parsed>),
}

#[derive(Debug, Clone)]
struct Statement {
    target: String,
    expression: Parsed,
    condition: Option<Parsed>,
}

#[derive(Debug, Clone)]
struct Token {
    text: String,
}

fn tokenize(text: &str) -> Result<Vec<Token>, MirpError> {
    let mut tokens = vec![];
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if chars[i].is_ascii_digit() || chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
        } else if "+-*/()<>!=".contains(chars[i]) {
            i += 1;
            if i < chars.len() && chars[i] == '=' && "<>!=".contains(chars[start]) {
                i += 1;
            }
        } else {
            return Err(error(
                "MirpParseError",
                format!("invalid character: {}", chars[i]),
                None,
            ));
        }
        tokens.push(Token {
            text: chars[start..i].iter().collect(),
        });
    }
    Ok(tokens)
}

fn precedence(operator: &str) -> u8 {
    match operator {
        "==" | "!=" | ">" | "<" | ">=" | "<=" => 1,
        "+" | "-" => 2,
        "*" | "/" => 3,
        _ => 0,
    }
}

fn parse_expression(text: &str) -> Result<Parsed, MirpError> {
    let tokens = tokenize(text)?;
    fn parse(tokens: &[Token], offset: &mut usize, minimum: u8) -> Result<Parsed, MirpError> {
        let token = tokens
            .get(*offset)
            .ok_or_else(|| error("MirpParseError", "expected operand", None))?
            .text
            .as_str();
        let mut left = if token == "(" {
            *offset += 1;
            let inner = parse(tokens, offset, 1)?;
            if tokens.get(*offset).map(|token| token.text.as_str()) != Some(")") {
                return Err(error("MirpParseError", "unclosed parenthesis", None));
            }
            *offset += 1;
            inner
        } else if token == "-" {
            *offset += 1;
            Parsed::Binary(
                "SUBTRACT".into(),
                Box::new(Parsed::Number(0)),
                Box::new(parse(tokens, offset, 4)?),
            )
        } else if token.chars().all(|c| c.is_ascii_digit()) {
            *offset += 1;
            Parsed::Number(
                token
                    .parse()
                    .map_err(|_| error("MirpParseError", "integer overflow", None))?,
            )
        } else if token
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            *offset += 1;
            Parsed::Name(token.into())
        } else {
            return Err(error("MirpParseError", "expected operand", None));
        };
        while let Some(operator) = tokens.get(*offset).map(|token| token.text.as_str()) {
            let power = precedence(operator);
            if power < minimum || power == 0 {
                break;
            }
            *offset += 1;
            let right = parse(tokens, offset, power + 1)?;
            let name = match operator {
                "+" => "ADD",
                "-" => "SUBTRACT",
                "*" => "MULTIPLY",
                "/" => "DIVIDE",
                "==" => "COMPARISON.EQUAL",
                "!=" => "COMPARISON.NOT_EQUAL",
                ">" => "COMPARISON.GREATER_THAN",
                "<" => "COMPARISON.LESS_THAN",
                ">=" => "COMPARISON.GREATER_EQUAL",
                "<=" => "COMPARISON.LESS_EQUAL",
                _ => unreachable!(),
            };
            left = Parsed::Binary(name.into(), Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    let mut offset = 0;
    let result = parse(&tokens, &mut offset, 1)?;
    if offset != tokens.len() {
        return Err(error("MirpParseError", "unexpected token", None));
    }
    Ok(result)
}

fn identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn assignment(line: &str) -> Result<(String, Parsed), MirpError> {
    let (target, expression) = line
        .split_once('=')
        .ok_or_else(|| error("MirpUnsupportedError", "assignment required", None))?;
    let target = target.trim();
    if !identifier(target) {
        return Err(error("MirpParseError", "invalid target", None));
    }
    Ok((target.into(), parse_expression(expression.trim())?))
}

fn parse_input(domain: Domain, source: &str) -> Result<Vec<Statement>, MirpError> {
    match domain {
        Domain::NaturalLanguage => {
            let line = source.trim().trim_end_matches('.');
            let (name, word) = line.split_once(" is ").ok_or_else(|| {
                error(
                    "MirpUnsupportedError",
                    "expected '<name> is <number>'",
                    None,
                )
            })?;
            if !identifier(name) {
                return Err(error("MirpParseError", "invalid entity", None));
            }
            let number = match word.trim().to_ascii_lowercase().as_str() {
                "zero" => 0,
                "one" => 1,
                "two" => 2,
                "three" => 3,
                "four" => 4,
                "five" => 5,
                "six" => 6,
                "seven" => 7,
                "eight" => 8,
                "nine" => 9,
                "ten" => 10,
                other => other.parse().map_err(|_| {
                    error(
                        "MirpUnsupportedError",
                        "unsupported natural language fact",
                        None,
                    )
                })?,
            };
            Ok(vec![Statement {
                target: name.into(),
                expression: Parsed::Number(number),
                condition: None,
            }])
        }
        Domain::Japanese => Err(error(
            "MirpUnsupportedError",
            "Japanese uses the NLSRV adapter",
            None,
        )),
        Domain::Mathematics => {
            let (target, expression) = assignment(source.trim())?;
            Ok(vec![Statement {
                target,
                expression,
                condition: None,
            }])
        }
        Domain::Code => {
            let lines: Vec<&str> = source
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect();
            if lines.is_empty() {
                return Err(error("MirpParseError", "empty code", None));
            }
            let mut statements = vec![];
            let mut condition = None;
            for line in lines {
                if let Some(test) = line
                    .strip_prefix("if ")
                    .and_then(|line| line.strip_suffix(':'))
                {
                    condition = Some(parse_expression(test)?);
                    continue;
                }
                let indented = line.starts_with(' ') || line.starts_with('\t');
                if condition.is_some() != indented {
                    return Err(error(
                        "MirpUnsupportedError",
                        "bounded code adapter requires one-level if block",
                        None,
                    ));
                }
                let (target, expression) = assignment(line.trim())?;
                statements.push(Statement {
                    target,
                    expression,
                    condition: condition.clone(),
                });
            }
            Ok(statements)
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RunResult {
    pub status: Status,
    pub result: Option<Value>,
    pub result_id: Option<String>,
    pub ruos: Vec<Json>,
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Metrics {
    pub mirp_objects: usize,
    pub ru_count: usize,
    pub rus_count: usize,
    pub ruo_count: usize,
    pub state_transitions: u64,
    pub candidate_count: usize,
    pub execution_time_ns: u128,
    pub peak_state_size: usize,
}

pub struct Session {
    pub state: SemanticState,
    pub namespace: String,
    pub scope: String,
}

impl Session {
    pub fn new(namespace: &str, scope: &str) -> Self {
        Self {
            state: SemanticState::default(),
            namespace: namespace.into(),
            scope: scope.into(),
        }
    }
    pub fn restore(state: SemanticState, namespace: &str, scope: &str) -> Result<Self, MirpError> {
        state.validate()?;
        Ok(Self {
            state,
            namespace: namespace.into(),
            scope: scope.into(),
        })
    }

    fn input_context(&self, domain: &str, source: &str) -> (Provenance, String) {
        let provenance = Provenance::input(domain, source);
        let next = self
            .state
            .objects
            .iter()
            .filter(|object| object.common().provenance.source_id == provenance.source_id)
            .filter_map(|object| {
                object
                    .common()
                    .occurrence_key
                    .strip_prefix("input:")?
                    .split(':')
                    .next()?
                    .parse::<usize>()
                    .ok()
            })
            .max()
            .map_or(0, |index| index + 1);
        (provenance, format!("input:{next}:"))
    }

    pub fn apply_input(
        &mut self,
        domain: Domain,
        source: &str,
    ) -> Result<Vec<RunResult>, MirpError> {
        let started = Instant::now();
        let before_version = self.state.version;
        let mut staged = Self {
            state: self.state.clone(),
            namespace: self.namespace.clone(),
            scope: self.scope.clone(),
        };
        let mut result = staged.apply_input_inner(domain, source)?;
        for outcome in &mut result {
            let rules: BTreeSet<String> = outcome
                .ruos
                .iter()
                .filter_map(|ruo| ruo.get("source_ru").map(Json::to_string))
                .collect();
            let sequences: BTreeSet<&str> = outcome
                .ruos
                .iter()
                .filter_map(|ruo| ruo.get("source_rus").and_then(Json::as_str))
                .collect();
            outcome.metrics = Metrics {
                mirp_objects: staged.state.objects.len(),
                ru_count: rules.len(),
                rus_count: sequences.len(),
                ruo_count: outcome.ruos.len(),
                state_transitions: staged.state.version - before_version,
                candidate_count: rules.len(),
                execution_time_ns: started.elapsed().as_nanos(),
                peak_state_size: staged.state.objects.len(),
            };
        }
        self.state = staged.state;
        Ok(result)
    }

    fn apply_input_inner(
        &mut self,
        domain: Domain,
        source: &str,
    ) -> Result<Vec<RunResult>, MirpError> {
        if matches!(domain, Domain::Japanese) {
            return Ok(vec![self.apply_japanese(source)?]);
        }
        if matches!(domain, Domain::NaturalLanguage) && parse_input(domain, source).is_err() {
            return Ok(vec![self.apply_nl_relation(source)?]);
        }
        let statements = match parse_input(domain, source) {
            Ok(statements) => statements,
            Err(_original) if matches!(domain, Domain::Mathematics | Domain::Code) => {
                let document = stage2a_document(domain, source)?;
                let statements = stage2a_statements(&document)?;
                if statements.is_empty() {
                    return Ok(vec![self.apply_stage2a_relation(domain, source, &document)?]);
                }
                statements
            }
            Err(original) => return Err(original),
        };
        let (provenance, occurrence_prefix) = self.input_context(domain.name(), source);
        let mut results = vec![];
        for (index, statement) in statements.iter().enumerate() {
            let target = self.state.bind(
                &statement.target,
                &self.namespace,
                &self.scope,
                "VARIABLE",
                provenance.clone(),
            )?;
            let expression = self.build(
                &statement.expression,
                &provenance,
                &format!("{occurrence_prefix}{index}:value"),
            )?;
            let condition = if let Some(condition) = &statement.condition {
                let reference = self.build(
                    condition,
                    &provenance,
                    &format!("{occurrence_prefix}{index}:condition"),
                )?;
                let mut object = Object::Condition(Condition {
                    common: Common::unidentified(Status::Unknown, provenance.clone()),
                    expression: reference,
                    expected_truth: Truth::True,
                    truth: Truth::Unknown,
                    dependencies: vec![],
                });
                object.normalize_identity(&format!("{occurrence_prefix}{index}:condition"));
                let id = object.id().to_owned();
                self.state.apply(SemanticDelta {
                    added: vec![object],
                    ..Default::default()
                })?;
                Some(id)
            } else {
                None
            };
            let mut event = Object::Event(Event {
                common: Common::unidentified(Status::Known, provenance.clone()),
                event_type: "PROGRAM.ASSIGNS".into(),
                actor: None,
                target: Some(target),
                inputs: vec![expression],
                outputs: vec![],
                preconditions: condition.into_iter().collect(),
                effects: vec![],
            });
            event.normalize_identity(&format!("{occurrence_prefix}{index}:event"));
            let id = event.occurrence_id().to_owned();
            self.state.apply(SemanticDelta {
                added: vec![event],
                ..Default::default()
            })?;
            results.push(self.run_event(&id)?);
        }
        Ok(results)
    }

    fn apply_stage2a_relation(
        &mut self,
        domain: Domain,
        source: &str,
        document: &Json,
    ) -> Result<RunResult, MirpError> {
        let relations = document["relations"]
            .as_array()
            .ok_or_else(|| error("MirpParseError", "Stage2A relations", None))?;
        if relations.len() != 1 {
            return Err(error("MirpUnsupportedError", "Stage2A relation form", None));
        }
        let relation = &relations[0];
        let (provenance, occurrence_prefix) = self.input_context(domain.name(), source);
        let left = self.build(
            &parse_expression(required_str(relation, "lhs")?)?,
            &provenance,
            &format!("{occurrence_prefix}relation.0"),
        )?;
        let right = self.build(
            &parse_expression(required_str(relation, "rhs")?)?,
            &provenance,
            &format!("{occurrence_prefix}relation.1"),
        )?;
        let mut object = Object::Relation(Relation {
            common: Common::unidentified(Status::Known, provenance),
            relation_type: comparison_name(required_str(relation, "type")?)?.into(),
            source: left,
            target: right,
            arguments: vec![],
            polarity: Polarity::Positive,
            modality: Modality::Asserted,
        });
        object.normalize_identity(&format!("{occurrence_prefix}relation"));
        let id = object.occurrence_id().to_owned();
        self.state.apply(SemanticDelta {
            added: vec![object],
            ..Default::default()
        })?;
        Ok(RunResult {
            status: Status::Known,
            result: None,
            result_id: Some(id),
            ruos: vec![],
            metrics: Metrics::default(),
        })
    }

    fn apply_japanese(&mut self, source: &str) -> Result<RunResult, MirpError> {
        let literal = serde_json::to_string(source)
            .map_err(|exc| error("MirpParseError", exc.to_string(), None))?;
        let doc = run_reason_source(
            &format!("Japanese_semantic_adapter::AdaptJapaneseSemantic({literal})"),
            &[
                ("cr_surface", "Cr_surface"),
                ("semantic_representation", "Semantic_representation"),
                ("japanese_semantic_adapter", "Japanese_semantic_adapter"),
            ],
        )?;
        let classification = doc["status"].as_str().unwrap_or("INVALID");
        if classification != "RESOLVED" {
            return Err(error(
                if classification == "INVALID" {
                    "MirpParseError"
                } else {
                    "MirpUnsupportedError"
                },
                format!("NLSRV: {classification}"),
                None,
            ));
        }
        let primitives = doc["primitives"]
            .as_array()
            .ok_or_else(|| error("MirpParseError", "NLSRV primitives", None))?;
        let (provenance, occurrence_prefix) = self.input_context("nlsrv_japanese", source);
        let mut bindings = BTreeMap::new();
        for primitive in primitives {
            if primitive["kind"] == "ENTITY" {
                let label = required_str(primitive, "label")?;
                let kind = required_str(primitive, "type")?;
                let id = self.state.bind(
                    label,
                    &self.namespace,
                    &self.scope,
                    kind,
                    provenance.clone(),
                )?;
                bindings.insert(required_str(primitive, "id")?.to_string(), id);
            }
        }
        let mut last = None;
        for primitive in primitives {
            let kind = required_str(primitive, "kind")?;
            if kind == "ENTITY" {
                continue;
            }
            let original = required_str(primitive, "id")?;
            let subject = bindings
                .get(required_str(primitive, "subject")?)
                .ok_or_else(|| error("MirpReferenceError", original, Some(&provenance.source_id)))?
                .clone();
            let mut common = Common::unidentified(Status::Known, provenance.clone());
            common
                .metadata
                .insert("source_span".into(), primitive["source_span"].clone());
            common.metadata.insert(
                "temporal_context".into(),
                primitive["temporal_context"].clone(),
            );
            for field in ["source_sentence", "source_index", "value"] {
                common
                    .metadata
                    .insert(field.into(), primitive[field].clone());
            }
            let polarity = match primitive["polarity"].as_str() {
                Some("POSITIVE") => Polarity::Positive,
                Some("NEGATIVE") => Polarity::Negative,
                _ => Polarity::Unknown,
            };
            let modality = match primitive["modality"].as_str() {
                Some("ASSERTED") => Modality::Asserted,
                Some("POSSIBLE") => Modality::Possible,
                Some("REQUIRED" | "NECESSARY") => Modality::Necessary,
                Some("HYPOTHETICAL") => Modality::Hypothetical,
                _ => Modality::Unknown,
            };
            if kind == "EVENT" {
                common
                    .metadata
                    .insert("polarity".into(), primitive["polarity"].clone());
                common
                    .metadata
                    .insert("modality".into(), primitive["modality"].clone());
            }
            let mut object = match kind {
                "STATE" => {
                    let property = required_str(primitive, "type")?;
                    let target = self.state.bind(
                        property,
                        "concept",
                        "global",
                        "ABSTRACT_OBJECT",
                        provenance.clone(),
                    )?;
                    Object::Relation(Relation {
                        common,
                        relation_type: "SEMANTIC.IS_A".into(),
                        source: subject,
                        target,
                        arguments: vec![],
                        polarity,
                        modality,
                    })
                }
                "RELATION" => {
                    let target_name = required_str(primitive, "object")?;
                    let target = bindings
                        .get(target_name)
                        .ok_or_else(|| {
                            error(
                                "MirpReferenceError",
                                target_name,
                                Some(&provenance.source_id),
                            )
                        })?
                        .clone();
                    Object::Relation(Relation {
                        common,
                        relation_type: required_str(primitive, "type")?.into(),
                        source: subject,
                        target,
                        arguments: vec![],
                        polarity,
                        modality,
                    })
                }
                "QUANTITY" => {
                    let number =
                        required_str(primitive, "value")?
                            .parse::<i64>()
                            .map_err(|_| {
                                error(
                                    "MirpTypeError",
                                    "NLSRV quantity",
                                    Some(&provenance.source_id),
                                )
                            })?;
                    Object::Attribute(Attribute {
                        common,
                        subject,
                        key: "COUNT".into(),
                        value: Value::Integer(number),
                        polarity,
                    })
                }
                "EVENT" => {
                    let actor = primitive["agent"]
                        .as_str()
                        .and_then(|id| bindings.get(id))
                        .cloned()
                        .or(Some(subject));
                    let target = primitive["target"]
                        .as_str()
                        .and_then(|id| bindings.get(id))
                        .cloned();
                    let inputs = primitive["object"]
                        .as_str()
                        .and_then(|id| bindings.get(id))
                        .cloned()
                        .into_iter()
                        .collect();
                    Object::Event(Event {
                        common,
                        event_type: format!("SEMANTIC.{}", required_str(primitive, "type")?),
                        actor,
                        target,
                        inputs,
                        outputs: vec![],
                        preconditions: vec![],
                        effects: vec![],
                    })
                }
                _ => {
                    return Err(error(
                        "MirpUnsupportedError",
                        format!("NLSRV primitive: {kind}"),
                        Some(&provenance.source_id),
                    ))
                }
            };
            object.normalize_identity(&format!("{occurrence_prefix}nlsrv:{original}"));
            last = Some(object.occurrence_id().to_string());
            self.state.apply(SemanticDelta {
                added: vec![object],
                ..Default::default()
            })?;
        }
        if let Some(reference_bindings) = doc["bindings"].as_array() {
            for (index, binding) in reference_bindings.iter().enumerate() {
                let target = bindings
                    .get(required_str(binding, "target_entity_id")?)
                    .ok_or_else(|| {
                        error(
                            "MirpReferenceError",
                            "NLSRV binding",
                            Some(&provenance.source_id),
                        )
                    })?;
                let mut common = Common::unidentified(Status::Known, provenance.clone());
                common
                    .metadata
                    .insert("surface".into(), binding["surface"].clone());
                common
                    .metadata
                    .insert("confidence".into(), binding["confidence"].clone());
                common
                    .metadata
                    .insert("status".into(), binding["status"].clone());
                common
                    .metadata
                    .insert("source_sentence".into(), binding["source_sentence"].clone());
                let mut evidence = Object::Evidence(Evidence {
                    common,
                    evidence_type: "REFERENCE_BINDING".into(),
                    source: "NLSRV".into(),
                    supports: vec![target.clone()],
                    contradicts: vec![],
                });
                evidence.normalize_identity(&format!("{occurrence_prefix}nlsrv-binding:{index}"));
                self.state.apply(SemanticDelta {
                    added: vec![evidence],
                    ..Default::default()
                })?;
            }
        }
        Ok(RunResult {
            status: Status::Known,
            result: None,
            result_id: last,
            ruos: vec![],
            metrics: Metrics::default(),
        })
    }

    fn apply_nl_relation(&mut self, source: &str) -> Result<RunResult, MirpError> {
        let mut text = source.trim().trim_end_matches('.');
        let mut conditional = None;
        if let Some(rest) = text.strip_prefix("If ") {
            let (condition, consequent) = rest
                .split_once(',')
                .ok_or_else(|| error("MirpUnsupportedError", "conditional clause", None))?;
            if !identifier(condition.trim()) {
                return Err(error("MirpUnsupportedError", "conditional clause", None));
            }
            conditional = Some(condition.trim());
            text = consequent.trim();
        }
        let (left, right, polarity, modality) = if let Some((a, b)) = text.split_once(" is not ") {
            (a, b, Polarity::Negative, Modality::Asserted)
        } else if let Some((a, b)) = text.split_once(" may be ") {
            (a, b, Polarity::Positive, Modality::Possible)
        } else if let Some((a, b)) = text.split_once(" must be ") {
            (a, b, Polarity::Positive, Modality::Necessary)
        } else if let Some((a, b)) = text.split_once(" is ") {
            (a, b, Polarity::Positive, Modality::Asserted)
        } else {
            return Err(error(
                "MirpUnsupportedError",
                "unsupported natural-language statement",
                None,
            ));
        };
        if !identifier(left) || !identifier(right) {
            return Err(error(
                "MirpUnsupportedError",
                "relation requires named entities",
                None,
            ));
        }
        let (provenance, occurrence_prefix) = self.input_context("natural_language", source);
        let a = self.state.bind(
            left,
            &self.namespace,
            &self.scope,
            "OBJECT",
            provenance.clone(),
        )?;
        let b = self.state.bind(
            right,
            &self.namespace,
            &self.scope,
            "OBJECT",
            provenance.clone(),
        )?;
        let mut arguments = vec![];
        if let Some(name) = conditional {
            let entity = self.state.bind(
                name,
                &self.namespace,
                &self.scope,
                "ABSTRACT_OBJECT",
                provenance.clone(),
            )?;
            let mut condition = Object::Condition(Condition {
                common: Common::unidentified(Status::Unknown, provenance.clone()),
                expression: entity,
                expected_truth: Truth::True,
                truth: Truth::Unknown,
                dependencies: vec![],
            });
            condition.normalize_identity(&format!("{occurrence_prefix}condition"));
            let condition_id = condition.id().to_owned();
            self.state.apply(SemanticDelta {
                added: vec![condition],
                ..Default::default()
            })?;
            arguments.push(condition_id);
        }
        let modality = if conditional.is_some() {
            Modality::Conditional
        } else {
            modality
        };
        let mut relation = Object::Relation(Relation {
            common: Common::unidentified(Status::Known, provenance),
            relation_type: "SEMANTIC.IS_A".into(),
            source: a,
            target: b,
            arguments,
            polarity,
            modality,
        });
        relation.normalize_identity(&format!("{occurrence_prefix}relation"));
        let id = relation.occurrence_id().to_owned();
        self.state.apply(SemanticDelta {
            added: vec![relation],
            ..Default::default()
        })?;
        Ok(RunResult {
            status: Status::Known,
            result: None,
            result_id: Some(id),
            ruos: vec![],
            metrics: Metrics::default(),
        })
    }

    fn build(
        &mut self,
        parsed: &Parsed,
        provenance: &Provenance,
        path: &str,
    ) -> Result<String, MirpError> {
        match parsed {
            Parsed::Name(name) => self.state.bind(
                name,
                &self.namespace,
                &self.scope,
                "VARIABLE",
                provenance.clone(),
            ),
            Parsed::Number(number) => {
                if number.unsigned_abs() > 1_000_000 {
                    return Err(error(
                        "MirpUnsupportedError",
                        "integer exceeds ReasonScript model limit",
                        Some(&provenance.source_id),
                    ));
                }
                let label = number.to_string();
                let id =
                    self.state
                        .bind(&label, "literal", "global", "NUMBER", provenance.clone())?;
                self.state.assert_value(
                    &id,
                    Value::Integer(*number),
                    provenance.clone(),
                    Status::Known,
                )?;
                Ok(id)
            }
            Parsed::Binary(operator, left, right) => {
                let left = self.build(left, provenance, &format!("{path}.0"))?;
                let right = self.build(right, provenance, &format!("{path}.1"))?;
                let mut expression = Object::Expression(Expression {
                    common: Common::unidentified(Status::Known, provenance.clone()),
                    operator: operator.clone(),
                    operands: vec![left, right],
                    result: None,
                });
                expression.normalize_identity(path);
                let id = expression.id().to_owned();
                self.state.apply(SemanticDelta {
                    added: vec![expression],
                    ..Default::default()
                })?;
                Ok(id)
            }
        }
    }

    fn run_event(&mut self, event_id: &str) -> Result<RunResult, MirpError> {
        let event = match self.state.get_occurrence(event_id) {
            Some(Object::Event(event)) => event.clone(),
            _ => return Err(error("MirpReferenceError", event_id, None)),
        };
        let input_prefix = event
            .common
            .occurrence_key
            .strip_prefix("input:")
            .and_then(|key| key.split_once(':'))
            .map(|(index, _)| format!("input:{index}:"))
            .ok_or_else(|| error("MirpValidationError", "event occurrence key", None))?;
        let mut ruos = vec![];
        let mut parents = vec![event_id.to_owned()];
        for condition_id in &event.preconditions {
            let condition = match self.state.objects.iter().find(|object| {
                object.id() == condition_id
                    && object.common().provenance.source_id == event.common.provenance.source_id
                    && object.common().occurrence_key.starts_with(&input_prefix)
            }) {
                Some(Object::Condition(condition)) => condition.clone(),
                _ => unreachable!(),
            };
            let tested = self.evaluate(&condition.expression, &mut ruos)?;
            let truth = match tested.value {
                Some(Value::Boolean(true)) => Truth::True,
                Some(Value::Boolean(false)) => Truth::False,
                _ => Truth::Unknown,
            };
            let condition_occurrence = condition.common.occurrence_id.clone();
            let mut updated = condition;
            updated.truth = truth.clone();
            updated.common.status = if truth == Truth::Unknown {
                Status::Unknown
            } else {
                Status::Derived
            };
            self.state.apply(SemanticDelta {
                updated: vec![Object::Condition(updated)],
                ..Default::default()
            })?;
            if truth != Truth::True {
                return Ok(RunResult {
                    status: if truth == Truth::Unknown {
                        Status::Unknown
                    } else {
                        Status::Known
                    },
                    result: if truth == Truth::Unknown {
                        Some(Value::Unknown)
                    } else {
                        Some(Value::Boolean(false))
                    },
                    result_id: None,
                    ruos,
                    metrics: Metrics::default(),
                });
            }
            parents.push(condition_occurrence);
            parents.extend(tested.parents);
        }
        let evaluated = self.evaluate(&event.inputs[0], &mut ruos)?;
        if let Some(expression) = self.state.objects.iter().find(|object| {
            object.id() == event.inputs[0]
                && object.common().provenance.source_id == event.common.provenance.source_id
                && object.common().occurrence_key.starts_with(&input_prefix)
        }) {
            parents.push(expression.occurrence_id().to_owned());
        }
        parents.extend(evaluated.parents);
        parents.sort();
        parents.dedup();
        let value = evaluated.value.unwrap_or(Value::Unknown);
        let status = if value == Value::Unknown {
            Status::Unknown
        } else if event.common.provenance.source_type == "natural_language" {
            Status::Known
        } else {
            Status::Derived
        };
        let origin = event.common.provenance.derived(parents, "RU/RUS/RUO");
        let target = event.target.as_ref().unwrap();
        let result_id = self
            .state
            .assert_value(target, value.clone(), origin, status.clone())?;
        if !ruos.is_empty() {
            let result_semantic_id = self
                .state
                .get_occurrence(&result_id)
                .unwrap()
                .id()
                .to_owned();
            let mut evidence = Object::Evidence(Evidence {
                common: Common::unidentified(
                    Status::Derived,
                    event
                        .common
                        .provenance
                        .derived(vec![result_id.clone()], "RUO trace"),
                ),
                evidence_type: "RUNTIME_RESULT".into(),
                source: "ReasonScript".into(),
                supports: vec![result_semantic_id],
                contradicts: vec![],
            });
            evidence
                .common_mut()
                .metadata
                .insert("ruos".into(), Json::Array(ruos.clone()));
            evidence.normalize_identity(&format!("runtime-evidence:{event_id}"));
            let dependencies =
                BTreeMap::from([(evidence.occurrence_id().to_owned(), vec![result_id.clone()])]);
            self.state.apply(SemanticDelta {
                added: vec![evidence],
                dependencies,
                ..Default::default()
            })?;
        }
        Ok(RunResult {
            status,
            result: Some(value),
            result_id: Some(result_id),
            ruos,
            metrics: Metrics::default(),
        })
    }

    fn evaluate(&self, id: &str, ruos: &mut Vec<Json>) -> Result<Evaluated, MirpError> {
        match self.state.get(id) {
            Some(Object::Entity(_)) => match self.state.value_of(id)? {
                Some((value, attribute)) => Ok(Evaluated {
                    value: Some(value.clone()),
                    parents: vec![attribute.into()],
                }),
                None => Ok(Evaluated {
                    value: None,
                    parents: vec![],
                }),
            },
            Some(Object::Expression(expression)) => {
                let left = self.evaluate(&expression.operands[0], ruos)?;
                let right = self.evaluate(&expression.operands[1], ruos)?;
                let mut parents = left.parents;
                parents.extend(right.parents);
                let (Some(Value::Integer(a)), Some(Value::Integer(b))) = (left.value, right.value)
                else {
                    return Ok(Evaluated {
                        value: None,
                        parents,
                    });
                };
                let op = expression.operator.as_str();
                let result = reason(op, a, b)?;
                if let Some(ruo) = result.get("ruo") {
                    ruos.push(ruo.clone());
                }
                if let Some(traces) = result.get("ruos").and_then(Json::as_array) {
                    ruos.extend(traces.iter().cloned());
                }
                if op.starts_with("COMPARISON.") {
                    if result["status"] != "APPLIED" {
                        return Ok(Evaluated {
                            value: None,
                            parents,
                        });
                    }
                    Ok(Evaluated {
                        value: Some(Value::Boolean(result["value"].as_bool().unwrap())),
                        parents,
                    })
                } else if result["status"] == "CALCULATED" && result["denominator"] == 1 {
                    Ok(Evaluated {
                        value: Some(Value::Integer(result["numerator"].as_i64().unwrap())),
                        parents,
                    })
                } else {
                    Ok(Evaluated {
                        value: None,
                        parents,
                    })
                }
            }
            _ => Err(error("MirpReferenceError", id, None)),
        }
    }
}

struct Evaluated {
    value: Option<Value>,
    parents: Vec<String>,
}

static WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reason(operator: &str, left: i64, right: i64) -> Result<Json, MirpError> {
    let source = match operator {
        "COMPARISON.EQUAL" => format!("Model::CompareIntegers({left}, {right}, 0)"),
        "COMPARISON.NOT_EQUAL" => format!("Model::CompareIntegers({left}, {right}, 1)"),
        "COMPARISON.GREATER_THAN" => format!("Model::CompareIntegers({left}, {right}, 2)"),
        "COMPARISON.LESS_THAN" => format!("Model::CompareIntegers({left}, {right}, 3)"),
        "COMPARISON.GREATER_EQUAL" => format!("Model::CompareIntegers({left}, {right}, 4)"),
        "COMPARISON.LESS_EQUAL" => format!("Model::CompareIntegers({left}, {right}, 5)"),
        operation => {
            let kind = match operation {
                "ADD" => 2,
                "SUBTRACT" => 3,
                "MULTIPLY" => 4,
                "DIVIDE" => 5,
                _ => return Err(error("MirpUnsupportedError", operator, None)),
            };
            format!("Polynomial::PolyEvaluate([0, 0, {kind}], [{left}, {right}, 0], [-1, -1, 0], [-1, -1, 1])")
        }
    };
    let mut result = run_reason_source(&source, &[])?;
    if !operator.starts_with("COMPARISON.") {
        result["status"] = Json::String(
            if result["valid"] == true {
                "CALCULATED"
            } else {
                "UNSUPPORTED"
            }
            .into(),
        );
        result["numerator"] = result["coefficients"][0].clone();
    }
    Ok(result)
}

pub(crate) fn run_reason_source(source: &str, extra: &[(&str, &str)]) -> Result<Json, MirpError> {
    let sequence = WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let directory =
        std::env::temp_dir().join(format!("mathlang-mirp-{}-{sequence}", std::process::id()));
    fs::create_dir(&directory)
        .map_err(|exc| error("MirpValidationError", exc.to_string(), None))?;
    let workspace = Workspace(directory);
    fs::create_dir(workspace.0.join("src"))
        .map_err(|exc| error("MirpValidationError", exc.to_string(), None))?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    fs::copy(
        root.join("mathlang_dsn/reason.toml"),
        workspace.0.join("reason.toml"),
    )
    .map_err(|exc| error("MirpValidationError", exc.to_string(), None))?;
    let modules = [
        ("model", "Model"),
        ("polynomial", "Polynomial"),
        ("knowledge", "Knowledge"),
        ("junior_high", "JuniorHigh"),
    ];
    for (module, _) in modules.iter().chain(extra.iter()) {
        fs::copy(
            root.join(format!("mathlang_dsn/{module}.rsn")),
            workspace.0.join(format!("src/{module}.rsn")),
        )
        .map_err(|exc| error("MirpValidationError", exc.to_string(), None))?;
    }
    let imports: String = modules
        .iter()
        .chain(extra.iter())
        .map(|(_, name)| format!("  import mathlang_dsn.{name}\n"))
        .collect();
    let program = format!("package mathlang_dsn\nmodule main {{\n{imports}  calculation Request {{\n    result = {source}\n  }}\n}}\n");
    fs::write(workspace.0.join("src/main.rsn"), program)
        .map_err(|exc| error("MirpValidationError", exc.to_string(), None))?;
    for args in [
        &["build"][..],
        &["run", "--entry", "Request", "--json", "--trace=off"][..],
    ] {
        let output = Command::new("reason")
            .args(args)
            .current_dir(&workspace.0)
            .output()
            .map_err(|exc| error("MirpUnsupportedError", exc.to_string(), None))?;
        if !output.status.success() {
            return Err(error(
                "MirpUnsupportedError",
                String::from_utf8_lossy(&output.stderr),
                None,
            ));
        }
        if args[0] == "run" {
            let payload: Json = serde_json::from_slice(&output.stdout)
                .map_err(|exc| error("MirpParseError", exc.to_string(), None))?;
            if payload["status"] != "success" || payload["runtime_result"]["status"] != "success" {
                return Err(error(
                    "MirpUnsupportedError",
                    "ReasonScript execution failed",
                    None,
                ));
            }
            return Ok(unwrap_result(payload["runtime_result"]["result"].clone()));
        }
    }
    unreachable!()
}

fn unwrap_result(value: Json) -> Json {
    match value {
        Json::Object(mut object)
            if object.contains_key("type_name") && object.contains_key("fields") =>
        {
            unwrap_result(object.remove("fields").unwrap())
        }
        Json::Object(object) => Json::Object(
            object
                .into_iter()
                .map(|(key, value)| (key, unwrap_result(value)))
                .collect(),
        ),
        Json::Array(items) => Json::Array(items.into_iter().map(unwrap_result).collect()),
        other => other,
    }
}

fn stage2a_document(domain: Domain, source: &str) -> Result<Json, MirpError> {
    let literal = serde_json::to_string(source)
        .map_err(|exc| error("MirpParseError", exc.to_string(), None))?;
    let adapter = if matches!(domain, Domain::Mathematics) {
        "Math_adapter_stage2a::AdaptMath"
    } else {
        "Code_adapter_stage2a::AdaptCode"
    };
    let document = run_reason_source(
        &format!("{adapter}({literal})"),
        &[
            ("cr_surface", "Cr_surface"),
            ("common_representation_cr", "Common_representation_cr"),
            ("math_adapter_stage2a", "Math_adapter_stage2a"),
            ("code_adapter_stage2a", "Code_adapter_stage2a"),
        ],
    )?;
    if document["metadata"] == "PARSE_FAILURE" {
        return Err(error(
            "MirpUnsupportedError",
            "Stage2A adapter could not parse input",
            None,
        ));
    }
    Ok(document)
}

fn comparison_name(kind: &str) -> Result<&'static str, MirpError> {
    match kind {
        "EQUAL" => Ok("COMPARISON.EQUAL"),
        "NOT_EQUAL" => Ok("COMPARISON.NOT_EQUAL"),
        "GREATER_THAN" => Ok("COMPARISON.GREATER_THAN"),
        "LESS_THAN" => Ok("COMPARISON.LESS_THAN"),
        "GREATER_EQUAL" => Ok("COMPARISON.GREATER_EQUAL"),
        "LESS_EQUAL" => Ok("COMPARISON.LESS_EQUAL"),
        _ => Err(error(
            "MirpUnsupportedError",
            format!("Stage2A relation: {kind}"),
            None,
        )),
    }
}

fn stage2a_statements(document: &Json) -> Result<Vec<Statement>, MirpError> {
    if document["calls"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        return Err(error(
            "MirpUnsupportedError",
            "Stage2A calls need a registered RU",
            None,
        ));
    }
    let mut statements = vec![];
    for field in ["assignments", "state"] {
        let items = document[field]
            .as_array()
            .ok_or_else(|| error("MirpParseError", field, None))?;
        for item in items {
            let target = required_str(item, "target")?;
            if !identifier(target) {
                return Err(error("MirpParseError", "Stage2A target", None));
            }
            statements.push(Statement {
                target: target.into(),
                expression: parse_expression(required_str(item, "value")?)?,
                condition: None,
            });
        }
    }
    let conditions = document["conditions"]
        .as_array()
        .ok_or_else(|| error("MirpParseError", "Stage2A conditions", None))?;
    for condition in conditions {
        let predicate = &condition["predicate"];
        let comparison = comparison_name(required_str(predicate, "type")?)?;
        let test = Parsed::Binary(
            comparison.into(),
            Box::new(parse_expression(required_str(predicate, "lhs")?)?),
            Box::new(parse_expression(required_str(predicate, "rhs")?)?),
        );
        let consequence = &condition["consequence"];
        let target = required_str(consequence, "target")?;
        if !identifier(target) {
            return Err(error("MirpParseError", "Stage2A target", None));
        }
        statements.push(Statement {
            target: target.into(),
            expression: parse_expression(required_str(consequence, "value")?)?,
            condition: Some(test),
        });
    }
    let operations = document["operations"]
        .as_array()
        .ok_or_else(|| error("MirpParseError", "Stage2A operations", None))?;
    if !operations.is_empty() {
        let relations = document["relations"]
            .as_array()
            .ok_or_else(|| error("MirpParseError", "Stage2A relations", None))?;
        if operations.len() != 1
            || relations.len() != 1
            || relations[0]["type"] != "EQUAL"
            || relations[0]["lhs"] != "_tmp"
        {
            return Err(error(
                "MirpUnsupportedError",
                "Stage2A operation graph",
                None,
            ));
        }
        let target = required_str(&relations[0], "rhs")?;
        if !identifier(target) {
            return Err(error("MirpParseError", "Stage2A target", None));
        }
        let operation = &operations[0];
        let kind = required_str(operation, "type")?;
        if !["ADD", "SUBTRACT", "MULTIPLY", "DIVIDE"].contains(&kind) {
            return Err(error("MirpUnsupportedError", "Stage2A arithmetic", None));
        }
        let expression = Parsed::Binary(
            kind.into(),
            Box::new(parse_expression(required_str(operation, "lhs")?)?),
            Box::new(parse_expression(required_str(operation, "rhs")?)?),
        );
        statements.push(Statement {
            target: target.into(),
            expression,
            condition: None,
        });
    }
    Ok(statements)
}

fn required_str<'a>(value: &'a Json, field: &str) -> Result<&'a str, MirpError> {
    value[field].as_str().ok_or_else(|| {
        error(
            "MirpParseError",
            format!("missing NLSRV field: {field}"),
            None,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence_and_rejection() {
        assert!(matches!(parse_expression("x+2*3"), Ok(Parsed::Binary(op, _, _)) if op == "ADD"));
        assert!(parse_expression("x+?").is_err());
        assert!(parse_input(Domain::Code, "import os").is_err());
    }
}
