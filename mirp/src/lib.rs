//! Domain independent MIRP state. Reasoning is performed by RU/RUS/RUO, not here.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

pub mod intent;
pub mod memory;
pub mod pif;
pub mod session;

pub const VERSION: &str = "mirp/0.1-si2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirpError {
    pub category: &'static str,
    pub message: String,
    pub source_id: Option<String>,
}

impl fmt::Display for MirpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.category, self.message)
    }
}
impl std::error::Error for MirpError {}

fn error(category: &'static str, message: impl Into<String>, source_id: Option<&str>) -> MirpError {
    MirpError {
        category,
        message: message.into(),
        source_id: source_id.map(str::to_owned),
    }
}

pub fn stable_id(kind: &str, parts: &[&str]) -> String {
    let bytes = serde_json::to_vec(&(kind, parts)).expect("strings serialize");
    let digest = Sha256::digest(bytes);
    format!(
        "{kind}:{}",
        digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Known,
    Derived,
    Assumed,
    Unknown,
    Conflict,
    Unsupported,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Truth {
    True,
    False,
    Unknown,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Polarity {
    Positive,
    Negative,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Modality {
    Asserted,
    Possible,
    Necessary,
    Hypothetical,
    Conditional,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Symbol(String),
    Expression(String),
    Vector(Vec<Value>),
    Matrix(Vec<Vec<Value>>),
    Set(Vec<Value>),
    Interval(Box<[Value; 2]>),
    Unknown,
}

impl Value {
    fn normalized(self) -> Self {
        match self {
            Self::Set(items) => {
                let mut items: Vec<Self> = items.into_iter().map(Self::normalized).collect();
                items.sort_by_key(|item| serde_json::to_string(item).unwrap());
                items.dedup();
                Self::Set(items)
            }
            Self::Vector(items) => Self::Vector(items.into_iter().map(Self::normalized).collect()),
            Self::Matrix(rows) => Self::Matrix(
                rows.into_iter()
                    .map(|row| row.into_iter().map(Self::normalized).collect())
                    .collect(),
            ),
            Self::Interval(pair) => {
                let [left, right] = *pair;
                Self::Interval(Box::new([left.normalized(), right.normalized()]))
            }
            other => other,
        }
    }

    fn validate(&self) -> bool {
        match self {
            Self::Real(number) => number.is_finite(),
            Self::Vector(items) | Self::Set(items) => items.iter().all(Self::validate),
            Self::Matrix(rows) => rows.iter().all(|row| row.iter().all(Self::validate)),
            Self::Interval(pair) => pair.iter().all(Self::validate),
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub source_type: String,
    pub source_id: String,
    #[serde(default)]
    pub source_span: Option<[usize; 2]>,
    #[serde(default)]
    pub parent_occurrence_ids: Vec<String>,
    #[serde(default)]
    pub transformation: Vec<String>,
    #[serde(default)]
    pub memory_id: Option<String>,
}

impl Provenance {
    pub fn input(domain: &str, source: &str) -> Self {
        Self {
            source_type: domain.into(),
            source_id: stable_id("source", &[domain, source]),
            source_span: None,
            parent_occurrence_ids: vec![],
            transformation: vec![],
            memory_id: None,
        }
    }
    pub fn derived(&self, parents: Vec<String>, operation: &str) -> Self {
        let mut next = self.clone();
        let mut parents = parents;
        parents.sort();
        parents.dedup();
        next.parent_occurrence_ids = parents;
        next.transformation.push(operation.into());
        next
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Common {
    /// Canonical semantic meaning, independent of source and state version.
    pub id: String,
    /// One observation or derivation of this meaning.
    pub occurrence_id: String,
    /// Persisted local discriminator used to recompute `occurrence_id`.
    pub occurrence_key: String,
    pub status: Status,
    pub provenance: Provenance,
    pub confidence: f64,
    #[serde(default)]
    pub metadata: BTreeMap<String, Json>,
}

impl Common {
    pub fn new(id: String, occurrence_key: String, status: Status, provenance: Provenance) -> Self {
        let mut provenance = provenance;
        provenance.parent_occurrence_ids = canonical_parents(&provenance.parent_occurrence_ids);
        let occurrence_id = occurrence_id(&id, &provenance, &occurrence_key);
        Self {
            id,
            occurrence_id,
            occurrence_key,
            status,
            provenance,
            confidence: 1.0,
            metadata: BTreeMap::new(),
        }
    }

    pub fn unidentified(status: Status, provenance: Provenance) -> Self {
        Self {
            id: String::new(),
            occurrence_id: String::new(),
            occurrence_key: String::new(),
            status,
            provenance,
            confidence: 1.0,
            metadata: BTreeMap::new(),
        }
    }
}

fn canonical_parents(parents: &[String]) -> Vec<String> {
    let mut canonical = parents.to_vec();
    canonical.sort();
    canonical.dedup();
    canonical
}

fn occurrence_id(semantic_id: &str, provenance: &Provenance, occurrence_key: &str) -> String {
    let span = serde_json::to_string(&provenance.source_span).expect("span serializes");
    let parents = canonical_parents(&provenance.parent_occurrence_ids);
    stable_id(
        "occurrence",
        &[
            semantic_id,
            &provenance.source_type,
            &provenance.source_id,
            &span,
            &serde_json::to_string(&parents).expect("parents serialize"),
            occurrence_key,
        ],
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    #[serde(flatten)]
    pub common: Common,
    pub entity_type: String,
    pub label: String,
    pub namespace: String,
    pub scope: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    #[serde(flatten)]
    pub common: Common,
    pub subject: String,
    pub key: String,
    pub value: Value,
    pub polarity: Polarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    #[serde(flatten)]
    pub common: Common,
    pub relation_type: String,
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub polarity: Polarity,
    pub modality: Modality,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Expression {
    #[serde(flatten)]
    pub common: Common,
    pub operator: String,
    pub operands: Vec<String>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    #[serde(flatten)]
    pub common: Common,
    pub expression: String,
    pub expected_truth: Truth,
    pub truth: Truth,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(flatten)]
    pub common: Common,
    pub event_type: String,
    pub actor: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub effects: Vec<String>,
}

/// A semantic operation request. Execution is reserved for a later RU family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Call {
    #[serde(flatten)]
    pub common: Common,
    pub target: String,
    pub operation: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub call_id: String,
}

/// An observed result of a Call. Its status is in `common.status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallResult {
    #[serde(flatten)]
    pub common: Common,
    pub call_id: String,
    pub call_occurrence_id: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    #[serde(flatten)]
    pub common: Common,
    pub evidence_type: String,
    pub source: String,
    #[serde(default)]
    pub supports: Vec<String>,
    #[serde(default)]
    pub contradicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hypothesis {
    #[serde(flatten)]
    pub common: Common,
    pub proposition: String,
    #[serde(default)]
    pub evidence_for: Vec<String>,
    #[serde(default)]
    pub evidence_against: Vec<String>,
    pub hypothesis_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Object {
    Entity(Entity),
    Attribute(Attribute),
    Relation(Relation),
    Expression(Expression),
    Condition(Condition),
    Event(Event),
    Call(Call),
    CallResult(CallResult),
    Evidence(Evidence),
    Hypothesis(Hypothesis),
}

impl Object {
    pub fn common(&self) -> &Common {
        match self {
            Self::Entity(x) => &x.common,
            Self::Attribute(x) => &x.common,
            Self::Relation(x) => &x.common,
            Self::Expression(x) => &x.common,
            Self::Condition(x) => &x.common,
            Self::Event(x) => &x.common,
            Self::Call(x) => &x.common,
            Self::CallResult(x) => &x.common,
            Self::Evidence(x) => &x.common,
            Self::Hypothesis(x) => &x.common,
        }
    }

    pub fn common_mut(&mut self) -> &mut Common {
        match self {
            Self::Entity(x) => &mut x.common,
            Self::Attribute(x) => &mut x.common,
            Self::Relation(x) => &mut x.common,
            Self::Expression(x) => &mut x.common,
            Self::Condition(x) => &mut x.common,
            Self::Event(x) => &mut x.common,
            Self::Call(x) => &mut x.common,
            Self::CallResult(x) => &mut x.common,
            Self::Evidence(x) => &mut x.common,
            Self::Hypothesis(x) => &mut x.common,
        }
    }
    pub fn id(&self) -> &str {
        &self.common().id
    }

    pub fn occurrence_id(&self) -> &str {
        &self.common().occurrence_id
    }

    pub fn semantic_id(&self) -> String {
        let (kind, content) = match self {
            Self::Entity(x) => ("entity", serde_json::json!([x.namespace, x.scope, x.label])),
            Self::Attribute(x) => (
                "attribute",
                serde_json::json!([x.subject, x.key, x.value.clone().normalized(), x.polarity]),
            ),
            Self::Relation(x) => {
                let mut pair = [x.source.clone(), x.target.clone()];
                if x.relation_type == "COMPARISON.EQUAL" {
                    pair.sort();
                }
                (
                    "relation",
                    serde_json::json!([x.relation_type, pair, x.arguments, x.polarity, x.modality]),
                )
            }
            Self::Expression(x) => {
                let mut operands = x.operands.clone();
                if ["ADD", "MULTIPLY", "AND", "OR"].contains(&x.operator.as_str()) {
                    operands.sort();
                }
                ("expression", serde_json::json!([x.operator, operands]))
            }
            Self::Condition(x) => (
                "condition",
                serde_json::json!([x.expression, x.expected_truth]),
            ),
            Self::Event(x) => (
                "event",
                serde_json::json!([
                    x.event_type,
                    x.actor,
                    x.target,
                    x.inputs,
                    x.outputs,
                    x.preconditions,
                    x.effects
                ]),
            ),
            Self::Call(x) => (
                "call",
                serde_json::json!([x.target, x.operation, x.arguments]),
            ),
            Self::CallResult(x) => (
                "call-result",
                serde_json::json!([x.call_id, x.value.clone().normalized()]),
            ),
            Self::Evidence(x) => {
                let mut supports = x.supports.clone();
                let mut contradicts = x.contradicts.clone();
                supports.sort();
                contradicts.sort();
                (
                    "evidence",
                    serde_json::json!([x.evidence_type, supports, contradicts]),
                )
            }
            Self::Hypothesis(x) => ("hypothesis", serde_json::json!([x.proposition])),
        };
        stable_id(kind, &[&content.to_string()])
    }

    pub fn normalize_identity(&mut self, occurrence_key: &str) {
        let semantic_id = self.semantic_id();
        let parents = canonical_parents(&self.common().provenance.parent_occurrence_ids);
        let common = self.common_mut();
        common.provenance.parent_occurrence_ids = parents;
        common.id = semantic_id;
        common.occurrence_key = occurrence_key.into();
        common.occurrence_id = occurrence_id(&common.id, &common.provenance, occurrence_key);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SemanticDelta {
    #[serde(default)]
    pub added: Vec<Object>,
    #[serde(default)]
    pub updated: Vec<Object>,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticState {
    pub mirp_version: String,
    pub version: u64,
    pub status: Status,
    #[serde(default)]
    pub objects: Vec<Object>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Vec<String>>,
}

impl Default for SemanticState {
    fn default() -> Self {
        Self {
            mirp_version: VERSION.into(),
            version: 0,
            status: Status::Unknown,
            objects: vec![],
            goals: vec![],
            dependencies: BTreeMap::new(),
        }
    }
}

impl SemanticState {
    pub fn get(&self, id: &str) -> Option<&Object> {
        self.get_occurrence(id)
            .or_else(|| self.objects.iter().find(|item| item.id() == id))
    }

    pub fn get_occurrence(&self, id: &str) -> Option<&Object> {
        self.objects.iter().find(|item| item.occurrence_id() == id)
    }

    pub fn resolve(&self, name: &str, namespace: &str, scope: &str) -> Option<&Entity> {
        self.objects.iter().find_map(|item| match item {
            Object::Entity(entity)
                if entity.namespace == namespace
                    && entity.scope == scope
                    && (entity.label == name
                        || entity.aliases.iter().any(|alias| alias == name)) =>
            {
                Some(entity)
            }
            _ => None,
        })
    }

    pub fn bind(
        &mut self,
        name: &str,
        namespace: &str,
        scope: &str,
        kind: &str,
        provenance: Provenance,
    ) -> Result<String, MirpError> {
        if let Some(entity) = self.resolve(name, namespace, scope) {
            return Ok(entity.common.id.clone());
        }
        let mut entity = Object::Entity(Entity {
            common: Common::unidentified(Status::Known, provenance),
            entity_type: kind.into(),
            label: name.into(),
            namespace: namespace.into(),
            scope: scope.into(),
            aliases: vec![],
        });
        entity.normalize_identity("entity");
        let id = entity.id().to_owned();
        self.apply(SemanticDelta {
            added: vec![entity],
            ..Default::default()
        })?;
        Ok(id)
    }

    pub fn add_alias(&mut self, entity_id: &str, alias: &str) -> Result<(), MirpError> {
        let entity = match self.get(entity_id) {
            Some(Object::Entity(entity)) => entity.clone(),
            _ => return Err(error("MirpReferenceError", entity_id, None)),
        };
        if !alias.is_empty() {
            if let Some(other) = self.resolve(alias, &entity.namespace, &entity.scope) {
                if other.common.id != entity_id {
                    return Err(error("MirpConflictError", "alias collision", None));
                }
            }
            let mut updated = entity;
            updated.aliases.push(alias.into());
            updated.aliases.sort();
            updated.aliases.dedup();
            self.apply(SemanticDelta {
                updated: vec![Object::Entity(updated)],
                ..Default::default()
            })?;
        }
        Ok(())
    }

    pub fn assert_value(
        &mut self,
        subject: &str,
        value: Value,
        provenance: Provenance,
        status: Status,
    ) -> Result<String, MirpError> {
        let value = value.normalized();
        let previous_assertions = self
            .objects
            .iter()
            .filter(|object| {
                matches!(object, Object::Attribute(attribute)
                if attribute.subject == subject
                    && attribute.key == "VALUE")
            })
            .count();
        let mut attribute = Object::Attribute(Attribute {
            common: Common::unidentified(status, provenance.clone()),
            subject: subject.into(),
            key: "VALUE".into(),
            value,
            polarity: Polarity::Positive,
        });
        attribute.normalize_identity(&format!("assertion:{previous_assertions}"));
        let occurrence = attribute.occurrence_id().to_owned();
        let dependencies = BTreeMap::from([(occurrence.clone(), provenance.parent_occurrence_ids)]);
        self.apply(SemanticDelta {
            added: vec![attribute],
            dependencies,
            ..Default::default()
        })?;
        Ok(occurrence)
    }

    pub fn value_of(&self, subject: &str) -> Result<Option<(&Value, &str)>, MirpError> {
        let values: Vec<&Attribute> = self
            .objects
            .iter()
            .filter_map(|item| match item {
                Object::Attribute(value) if value.subject == subject && value.key == "VALUE" => {
                    Some(value)
                }
                _ => None,
            })
            .collect();
        let known: Vec<_> = values
            .iter()
            .copied()
            .filter(|attribute| {
                attribute.common.status != Status::Unknown && attribute.value != Value::Unknown
            })
            .collect();
        if known.iter().any(|value| {
            known
                .first()
                .is_some_and(|first| first.value != value.value)
        }) {
            return Err(error("MirpConflictError", subject, None));
        }
        Ok(known
            .first()
            .or_else(|| values.first())
            .map(|value| (&value.value, value.common.occurrence_id.as_str())))
    }

    pub fn conflicts(&self) -> Vec<[String; 2]> {
        let mut seen: BTreeMap<(&str, &str), &Attribute> = BTreeMap::new();
        let mut conflicts = vec![];
        for item in &self.objects {
            if let Object::Attribute(attribute) = item {
                if attribute.common.status == Status::Unknown || attribute.value == Value::Unknown {
                    continue;
                }
                let key = (attribute.subject.as_str(), attribute.key.as_str());
                if let Some(previous) = seen.get(&key) {
                    if previous.value != attribute.value {
                        conflicts.push([previous.common.id.clone(), attribute.common.id.clone()]);
                    }
                } else {
                    seen.insert(key, attribute);
                }
            }
        }
        conflicts
    }

    pub fn apply(&mut self, delta: SemanticDelta) -> Result<(), MirpError> {
        let mut next = self.clone();
        for id in delta.removed {
            let index = next
                .objects
                .iter()
                .position(|item| item.occurrence_id() == id)
                .ok_or_else(|| error("MirpReferenceError", &id, None))?;
            next.objects.remove(index);
            next.dependencies.remove(&id);
        }
        for item in delta.updated {
            let index = next
                .objects
                .iter()
                .position(|old| old.occurrence_id() == item.occurrence_id())
                .ok_or_else(|| {
                    error(
                        "MirpValidationError",
                        "updated occurrence not found",
                        Some(&item.common().provenance.source_id),
                    )
                })?;
            if std::mem::discriminant(&next.objects[index]) != std::mem::discriminant(&item) {
                return Err(error(
                    "MirpTypeError",
                    "object type changed",
                    Some(&item.common().provenance.source_id),
                ));
            }
            let old = next.objects[index].common();
            let new = item.common();
            if old.id != new.id
                || old.occurrence_key != new.occurrence_key
                || old.provenance.source_type != new.provenance.source_type
                || old.provenance.source_id != new.provenance.source_id
                || old.provenance.source_span != new.provenance.source_span
                || canonical_parents(&old.provenance.parent_occurrence_ids)
                    != canonical_parents(&new.provenance.parent_occurrence_ids)
            {
                return Err(error(
                    "MirpValidationError",
                    "occurrence meaning or origin changed",
                    Some(&new.provenance.source_id),
                ));
            }
            next.objects[index] = item;
        }
        for item in delta.added {
            if next.get_occurrence(item.occurrence_id()).is_some() {
                return Err(error(
                    "MirpConflictError",
                    item.occurrence_id(),
                    Some(&item.common().provenance.source_id),
                ));
            }
            next.objects.push(item);
        }
        next.dependencies.extend(delta.dependencies);
        for parents in next.dependencies.values_mut() {
            parents.sort();
            parents.dedup();
        }
        next.objects.sort_by(|left, right| {
            left.id()
                .cmp(right.id())
                .then(left.occurrence_id().cmp(right.occurrence_id()))
        });
        next.version += 1;
        next.status = if !next.conflicts().is_empty() {
            Status::Conflict
        } else if next.objects.is_empty() {
            Status::Unknown
        } else {
            Status::Known
        };
        next.validate()?;
        *self = next;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), MirpError> {
        if self.mirp_version != VERSION {
            return Err(error("MirpUnsupportedError", "MIRP version", None));
        }
        let ids: BTreeSet<&str> = self.objects.iter().map(Object::id).collect();
        let occurrences: BTreeSet<&str> = self.objects.iter().map(Object::occurrence_id).collect();
        if occurrences.len() != self.objects.len() {
            return Err(error("MirpConflictError", "duplicate occurrence ID", None));
        }
        let expected_status = if !self.conflicts().is_empty() {
            Status::Conflict
        } else if self.objects.is_empty() {
            Status::Unknown
        } else {
            Status::Known
        };
        if self.status != expected_status {
            return Err(error("MirpValidationError", "state status", None));
        }
        let mut bindings = BTreeMap::new();
        for item in &self.objects {
            if let Object::Entity(entity) = item {
                for name in std::iter::once(&entity.label).chain(entity.aliases.iter()) {
                    let key = (&entity.namespace, &entity.scope, name);
                    if let Some(existing) = bindings.insert(key, &entity.common.id) {
                        if existing != &entity.common.id {
                            return Err(error(
                                "MirpConflictError",
                                "binding collision",
                                Some(&entity.common.provenance.source_id),
                            ));
                        }
                    }
                }
            }
        }
        for item in &self.objects {
            let common = item.common();
            let source = Some(common.provenance.source_id.as_str());
            if common.id.is_empty()
                || common.occurrence_id.is_empty()
                || common.occurrence_key.is_empty()
                || common.id != item.semantic_id()
                || common.provenance.source_type.is_empty()
                || common.provenance.source_id.is_empty()
                || common
                    .provenance
                    .source_span
                    .is_some_and(|[start, end]| start > end)
                || !common.confidence.is_finite()
                || !(0.0..=1.0).contains(&common.confidence)
            {
                return Err(error("MirpValidationError", common.id.clone(), source));
            }
            if common.provenance.parent_occurrence_ids
                != canonical_parents(&common.provenance.parent_occurrence_ids)
                || common.occurrence_id
                    != occurrence_id(&common.id, &common.provenance, &common.occurrence_key)
            {
                return Err(error(
                    "MirpValidationError",
                    "occurrence identity mismatch",
                    source,
                ));
            }
            let dependency_parents = self
                .dependencies
                .get(&common.occurrence_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            if dependency_parents != canonical_parents(dependency_parents)
                || dependency_parents != common.provenance.parent_occurrence_ids
            {
                return Err(error(
                    "MirpValidationError",
                    "dependency and provenance disagree",
                    source,
                ));
            }
            for parent in &common.provenance.parent_occurrence_ids {
                require(&occurrences, parent, source)?;
            }
            match item {
                Object::Entity(x)
                    if x.entity_type.is_empty()
                        || x.label.is_empty()
                        || x.namespace.is_empty()
                        || x.scope.is_empty() =>
                {
                    return Err(error("MirpTypeError", &common.id, source))
                }
                Object::Attribute(x) => {
                    require(&ids, &x.subject, source)?;
                    if !x.value.validate() {
                        return Err(error("MirpTypeError", "nonfinite value", source));
                    }
                }
                Object::Relation(x) => {
                    let prefix = x.relation_type.split('.').next().unwrap_or("");
                    if ![
                        "SEMANTIC",
                        "LOGICAL",
                        "COMPARISON",
                        "MATHEMATICAL",
                        "CAUSAL",
                        "TEMPORAL",
                        "SPATIAL",
                        "STRUCTURAL",
                        "PROGRAM",
                        "REFERENCE",
                    ]
                    .contains(&prefix)
                        || !x.relation_type.contains('.')
                    {
                        return Err(error("MirpRelationError", &x.relation_type, source));
                    }
                    for reference in [&x.source, &x.target].into_iter().chain(x.arguments.iter()) {
                        require(&ids, reference, source)?;
                    }
                }
                Object::Expression(x) => {
                    for reference in &x.operands {
                        require(&ids, reference, source)?;
                    }
                    if let Some(result) = &x.result {
                        require(&ids, result, source)?;
                    }
                }
                Object::Condition(x) => {
                    require(&ids, &x.expression, source)?;
                    for reference in &x.dependencies {
                        require(&ids, reference, source)?;
                    }
                }
                Object::Event(x) => {
                    for reference in x
                        .actor
                        .iter()
                        .chain(x.target.iter())
                        .chain(x.inputs.iter())
                        .chain(x.outputs.iter())
                        .chain(x.preconditions.iter())
                        .chain(x.effects.iter())
                    {
                        require(&ids, reference, source)?;
                    }
                }
                Object::Call(x) => {
                    if x.call_id != x.common.id || x.operation.is_empty() {
                        return Err(error("MirpValidationError", "invalid call", source));
                    }
                    for reference in std::iter::once(&x.target).chain(x.arguments.iter()) {
                        require(&ids, reference, source)?;
                    }
                }
                Object::CallResult(x) => {
                    if !matches!(self.get_occurrence(&x.call_occurrence_id), Some(Object::Call(call))
                        if call.call_id == x.call_id)
                    {
                        return Err(error("MirpReferenceError", &x.call_id, source));
                    }
                    if !x.value.validate() {
                        return Err(error("MirpTypeError", "invalid call result", source));
                    }
                    if !self
                        .dependencies
                        .get(&x.common.occurrence_id)
                        .is_some_and(|parents| parents.contains(&x.call_occurrence_id))
                    {
                        return Err(error(
                            "MirpValidationError",
                            "call result dependency",
                            source,
                        ));
                    }
                }
                Object::Evidence(x) => {
                    for reference in x.supports.iter().chain(x.contradicts.iter()) {
                        require(&ids, reference, source)?;
                    }
                }
                Object::Hypothesis(x) => {
                    for reference in std::iter::once(&x.proposition)
                        .chain(x.evidence_for.iter())
                        .chain(x.evidence_against.iter())
                    {
                        require(&ids, reference, source)?;
                    }
                }
                _ => {}
            }
        }
        for (child, parents) in &self.dependencies {
            require(&occurrences, child, None)?;
            for parent in parents {
                require(&occurrences, parent, None)?;
            }
        }
        fn visit<'a>(
            node: &'a str,
            edges: &'a BTreeMap<String, Vec<String>>,
            path: &mut BTreeSet<&'a str>,
            done: &mut BTreeSet<&'a str>,
        ) -> Result<(), MirpError> {
            if done.contains(node) {
                return Ok(());
            }
            if !path.insert(node) {
                return Err(error("MirpValidationError", "dependency cycle", None));
            }
            if let Some(parents) = edges.get(node) {
                for parent in parents {
                    visit(parent, edges, path, done)?;
                }
            }
            path.remove(node);
            done.insert(node);
            Ok(())
        }
        let mut done = BTreeSet::new();
        for node in self.dependencies.keys() {
            visit(node, &self.dependencies, &mut BTreeSet::new(), &mut done)?;
        }
        Ok(())
    }

    pub fn canonical(&self) -> Result<String, MirpError> {
        self.validate()?;
        let mut normalized = self.clone();
        normalized.objects.sort_by(|a, b| {
            a.id()
                .cmp(b.id())
                .then(a.occurrence_id().cmp(b.occurrence_id()))
        });
        normalized.goals.sort();
        for item in &mut normalized.objects {
            if let Object::Entity(entity) = item {
                entity.aliases.sort();
                entity.aliases.dedup();
            }
            if let Object::Attribute(attribute) = item {
                attribute.value = attribute.value.clone().normalized();
            }
            if let Object::CallResult(result) = item {
                result.value = result.value.clone().normalized();
            }
            if let Object::Expression(expression) = item {
                if ["ADD", "MULTIPLY", "AND", "OR"].contains(&expression.operator.as_str()) {
                    expression.operands.sort();
                }
            }
            if let Object::Relation(relation) = item {
                if relation.relation_type == "COMPARISON.EQUAL" && relation.source > relation.target
                {
                    std::mem::swap(&mut relation.source, &mut relation.target);
                }
            }
        }
        serde_json::to_string(&normalized)
            .map_err(|exc| error("MirpTypeError", exc.to_string(), None))
    }

    pub fn from_json(text: &str) -> Result<Self, MirpError> {
        let state: Self = serde_json::from_str(text)
            .map_err(|exc| error("MirpParseError", exc.to_string(), None))?;
        state.validate()?;
        Ok(state)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), MirpError> {
        let path = path.as_ref();
        let temporary = path.with_extension("mirp.tmp");
        fs::write(&temporary, format!("{}\n", self.canonical()?))
            .and_then(|_| fs::rename(&temporary, path))
            .map_err(|exc| error("MirpValidationError", exc.to_string(), None))
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, MirpError> {
        let text = fs::read_to_string(path)
            .map_err(|exc| error("MirpParseError", exc.to_string(), None))?;
        Self::from_json(&text)
    }
}

fn require(ids: &BTreeSet<&str>, reference: &str, source: Option<&str>) -> Result<(), MirpError> {
    if ids.contains(reference) {
        Ok(())
    } else {
        Err(error("MirpReferenceError", reference, source))
    }
}
