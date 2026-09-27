//! Domain independent MIRP state. Reasoning is performed by RU/RUS/RUO, not here.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

pub mod memory;
pub mod session;

pub const VERSION: &str = "mirp/1.0";

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
    pub parent_ids: Vec<String>,
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
            parent_ids: vec![],
            transformation: vec![],
            memory_id: None,
        }
    }
    pub fn derived(&self, parents: Vec<String>, operation: &str) -> Self {
        let mut next = self.clone();
        next.parent_ids = parents;
        next.transformation.push(operation.into());
        next
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Common {
    pub id: String,
    pub status: Status,
    pub provenance: Provenance,
    pub confidence: f64,
    #[serde(default)]
    pub metadata: BTreeMap<String, Json>,
}

impl Common {
    pub fn new(id: String, status: Status, provenance: Provenance) -> Self {
        Self {
            id,
            status,
            provenance,
            confidence: 1.0,
            metadata: BTreeMap::new(),
        }
    }
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
        self.objects.iter().find(|item| item.id() == id)
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
        let id = stable_id("entity", &[namespace, scope, name]);
        let entity = Entity {
            common: Common::new(id.clone(), Status::Known, provenance),
            entity_type: kind.into(),
            label: name.into(),
            namespace: namespace.into(),
            scope: scope.into(),
            aliases: vec![],
        };
        self.apply(SemanticDelta {
            added: vec![Object::Entity(entity)],
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
        let encoded = serde_json::to_string(&value).map_err(|exc| {
            error(
                "MirpTypeError",
                exc.to_string(),
                Some(&provenance.source_id),
            )
        })?;
        let id = stable_id(
            "attribute",
            &[
                subject,
                "VALUE",
                &encoded,
                &provenance.source_id,
                &provenance.parent_ids.join("|"),
            ],
        );
        if self.get(&id).is_some() {
            return Ok(id);
        }
        let attribute = Attribute {
            common: Common::new(id.clone(), status, provenance.clone()),
            subject: subject.into(),
            key: "VALUE".into(),
            value,
            polarity: Polarity::Positive,
        };
        let dependencies = BTreeMap::from([(id.clone(), provenance.parent_ids)]);
        self.apply(SemanticDelta {
            added: vec![Object::Attribute(attribute)],
            dependencies,
            ..Default::default()
        })?;
        Ok(id)
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
        if values.iter().any(|value| {
            values
                .first()
                .map(|first| first.value != value.value)
                .unwrap_or(false)
        }) {
            return Err(error("MirpConflictError", subject, None));
        }
        Ok(values
            .first()
            .map(|value| (&value.value, value.common.id.as_str())))
    }

    pub fn conflicts(&self) -> Vec<[String; 2]> {
        let mut seen: BTreeMap<(&str, &str), &Attribute> = BTreeMap::new();
        let mut conflicts = vec![];
        for item in &self.objects {
            if let Object::Attribute(attribute) = item {
                let key = (attribute.subject.as_str(), attribute.key.as_str());
                if let Some(previous) = seen.get(&key) {
                    if previous.value != attribute.value
                        && previous.common.status != Status::Unknown
                        && attribute.common.status != Status::Unknown
                    {
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
                .position(|item| item.id() == id)
                .ok_or_else(|| error("MirpReferenceError", &id, None))?;
            next.objects.remove(index);
            next.dependencies.remove(&id);
        }
        for item in delta.updated {
            let index = next
                .objects
                .iter()
                .position(|old| old.id() == item.id())
                .ok_or_else(|| {
                    error(
                        "MirpReferenceError",
                        item.id(),
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
            next.objects[index] = item;
        }
        for item in delta.added {
            if next.get(item.id()).is_some() {
                return Err(error(
                    "MirpConflictError",
                    item.id(),
                    Some(&item.common().provenance.source_id),
                ));
            }
            next.objects.push(item);
        }
        next.dependencies.extend(delta.dependencies);
        next.objects
            .sort_by(|left, right| left.id().cmp(right.id()));
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
        if ids.len() != self.objects.len() {
            return Err(error("MirpConflictError", "duplicate ID", None));
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
            for parent in &common.provenance.parent_ids {
                require(&ids, parent, source)?;
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
                    if !matches!(self.get(&x.call_id), Some(Object::Call(_))) {
                        return Err(error("MirpReferenceError", &x.call_id, source));
                    }
                    if !x.value.validate() {
                        return Err(error("MirpTypeError", "invalid call result", source));
                    }
                    if !self
                        .dependencies
                        .get(&x.common.id)
                        .is_some_and(|parents| parents.contains(&x.call_id))
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
            require(&ids, child, None)?;
            for parent in parents {
                require(&ids, parent, None)?;
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
        normalized.objects.sort_by(|a, b| a.id().cmp(b.id()));
        normalized.goals.sort();
        for item in &mut normalized.objects {
            if let Object::Entity(entity) = item {
                entity.aliases.sort();
                entity.aliases.dedup();
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
