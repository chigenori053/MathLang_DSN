//! Persistent, deterministic knowledge retrieval. This module never executes mathematics.
use crate::{stable_id, Object, Provenance, SemanticState, Status, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const CATEGORIES: &[&str] = &[
    "FOUNDATION",
    "ARITHMETIC",
    "NUMBER_SYSTEM",
    "ALGEBRA",
    "EQUATION",
    "INEQUALITY",
    "FUNCTION",
    "GEOMETRY",
    "COORDINATE_GEOMETRY",
    "TRIGONOMETRY",
    "CALCULUS",
    "LINEAR_ALGEBRA",
    "PROBABILITY",
    "STATISTICS",
    "LOGIC",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct KnowledgePattern {
    pub symbols: Vec<String>,
    pub relations: Vec<String>,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeUnit {
    pub knowledge_id: String,
    pub category: String,
    pub subcategory: String,
    pub knowledge_type: String,
    pub pattern: KnowledgePattern,
    pub premises: Vec<String>,
    pub conclusion: String,
    pub constraints: Vec<String>,
    pub dependencies: Vec<String>,
    pub applicable_goals: Vec<String>,
    pub applicable_rus: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub priority: i32,
    pub provenance: Provenance,
    pub curriculum_level: String,
    pub activation_metadata: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub runtime_rule_id: Option<i64>,
}

impl KnowledgeUnit {
    pub fn validate(&self) -> Result<(), String> {
        if self.knowledge_id.is_empty()
            || !self
                .knowledge_id
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            || !CATEGORIES.contains(&self.category.as_str())
            || self.subcategory.is_empty()
            || ![
                "AXIOM",
                "DEFINITION",
                "THEOREM",
                "IDENTITY",
                "PROPERTY",
                "RULE",
                "LEMMA",
            ]
            .contains(&self.knowledge_type.as_str())
            || self.conclusion.is_empty()
            || (self.applicable_rus.is_empty() && !self.capabilities.is_empty())
            || self.provenance.source_id.is_empty()
            || self.provenance.source_type.is_empty()
            || self.curriculum_level.is_empty()
            || self.dependencies.iter().any(|id| id == &self.knowledge_id)
            || self.runtime_rule_id.is_some_and(|id| id <= 0)
        {
            return Err(format!("malformed KnowledgeUnit: {}", self.knowledge_id));
        }
        for values in [
            &self.pattern.symbols,
            &self.pattern.relations,
            &self.pattern.properties,
            &self.premises,
            &self.constraints,
            &self.dependencies,
            &self.applicable_goals,
            &self.applicable_rus,
            &self.capabilities,
        ] {
            if values.iter().any(String::is_empty)
                || values.iter().collect::<BTreeSet<_>>().len() != values.len()
            {
                return Err(format!(
                    "invalid or duplicate feature: {}",
                    self.knowledge_id
                ));
            }
        }
        if self.capabilities.iter().any(|id| !valid_id(id)) {
            return Err(format!("invalid capability ID: {}", self.knowledge_id));
        }
        if let Some(conflicts) = self.activation_metadata.get("conflicts_with") {
            let Some(ids) = conflicts.as_array() else {
                return Err("invalid conflicts_with".into());
            };
            let ids: Option<Vec<&str>> = ids.iter().map(serde_json::Value::as_str).collect();
            let Some(ids) = ids else {
                return Err("invalid conflicts_with".into());
            };
            if ids
                .iter()
                .any(|id| !valid_id(id) || *id == self.knowledge_id)
                || ids.iter().collect::<BTreeSet<_>>().len() != ids.len()
            {
                return Err("invalid conflicts_with".into());
            }
        }
        Ok(())
    }

    pub fn occurrence_id(&self) -> String {
        stable_id(
            "knowledge-occurrence",
            &[&serde_json::to_string(self).expect("KnowledgeUnit serializes")],
        )
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeSpace {
    units: BTreeMap<String, KnowledgeUnit>,
    #[serde(skip)]
    categories: BTreeMap<String, BTreeSet<String>>,
    #[serde(skip)]
    goals: BTreeMap<String, BTreeSet<String>>,
    #[serde(skip)]
    rules: BTreeMap<i64, String>,
}

impl KnowledgeSpace {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn legacy() -> Self {
        let units: Vec<KnowledgeUnit> =
            serde_json::from_str(include_str!("../knowledge/legacy.json"))
                .expect("bundled legacy knowledge is JSON");
        let mut space = Self::empty();
        space
            .register_batch(units)
            .expect("bundled knowledge validates");
        space
    }
    pub fn len(&self) -> usize {
        self.units.len()
    }
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
    pub fn get(&self, id: &str) -> Option<&KnowledgeUnit> {
        self.units.get(id)
    }
    pub fn by_rule(&self, rule: i64) -> Option<&KnowledgeUnit> {
        self.rules.get(&rule).and_then(|id| self.units.get(id))
    }
    pub fn units(&self) -> impl Iterator<Item = &KnowledgeUnit> {
        self.units.values()
    }

    /// Registration is transactional; absent dependencies and cycles reject the batch.
    pub fn register_batch(&mut self, incoming: Vec<KnowledgeUnit>) -> Result<(), String> {
        let mut next = self.clone();
        for unit in incoming {
            unit.validate()?;
            if next.units.insert(unit.knowledge_id.clone(), unit).is_some() {
                return Err("duplicate Knowledge ID".into());
            }
        }
        next.validate_and_reindex()?;
        *self = next;
        Ok(())
    }

    pub fn validate_and_reindex(&mut self) -> Result<(), String> {
        self.categories.clear();
        self.goals.clear();
        self.rules.clear();
        // Older persisted catalogs predate capabilities. The bundled data is the migration table.
        let legacy_capabilities: BTreeMap<i64, Vec<String>> =
            serde_json::from_str::<Vec<KnowledgeUnit>>(include_str!("../knowledge/legacy.json"))
                .expect("bundled legacy knowledge is JSON")
                .into_iter()
                .filter_map(|unit| unit.runtime_rule_id.map(|rule| (rule, unit.capabilities)))
                .collect();
        for unit in self.units.values_mut() {
            if unit.capabilities.is_empty() {
                if let Some(rule) = unit.runtime_rule_id {
                    if let Some(capabilities) = legacy_capabilities.get(&rule) {
                        unit.capabilities = capabilities.clone();
                    }
                }
            }
        }
        for (id, unit) in &self.units {
            unit.validate()?;
            for dependency in &unit.dependencies {
                if !self.units.contains_key(dependency) {
                    return Err(format!("missing dependency: {dependency}"));
                }
            }
            if let Some(conflicts) = unit
                .activation_metadata
                .get("conflicts_with")
                .and_then(serde_json::Value::as_array)
            {
                for conflict in conflicts {
                    if !self.units.contains_key(conflict.as_str().unwrap()) {
                        return Err(format!("missing conflicting Knowledge: {conflict}"));
                    }
                }
            }
            if let Some(rule) = unit.runtime_rule_id {
                if self.rules.insert(rule, id.clone()).is_some() {
                    return Err("duplicate runtime Rule ID".into());
                }
            }
            self.categories
                .entry(unit.category.clone())
                .or_default()
                .insert(id.clone());
            for goal in &unit.applicable_goals {
                self.goals
                    .entry(goal.clone())
                    .or_default()
                    .insert(id.clone());
            }
            if unit.applicable_goals.is_empty() {
                self.goals.entry("*".into()).or_default().insert(id.clone());
            }
        }
        fn visit(
            id: &str,
            space: &KnowledgeSpace,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<(), String> {
            if done.contains(id) {
                return Ok(());
            }
            if !active.insert(id.into()) {
                return Err(format!("cyclic Knowledge dependency: {id}"));
            }
            for dependency in &space.units[id].dependencies {
                visit(dependency, space, active, done)?;
            }
            active.remove(id);
            done.insert(id.into());
            Ok(())
        }
        let mut done = BTreeSet::new();
        for id in self.units.keys() {
            visit(id, self, &mut BTreeSet::new(), &mut done)?;
        }
        Ok(())
    }

    pub fn query(&self, query: &KnowledgeQuery) -> KnowledgeQueryResult {
        let query_id = stable_id(
            "knowledge-query",
            &[&serde_json::to_string(query).expect("query serializes")],
        );
        let mut result = KnowledgeQueryResult {
            query_id,
            candidates: vec![],
            rejected: vec![],
            retrieval_trace: RetrievalTrace {
                total_knowledge: self.len(),
                ..Default::default()
            },
        };
        if query.goal.is_empty() || query.limit == 0 || !CATEGORIES.contains(&query.domain.as_str())
        {
            return result;
        }
        let mut category_ids = self
            .categories
            .get(&query.domain)
            .cloned()
            .unwrap_or_default();
        if query.domain != "FOUNDATION" {
            category_ids.extend(
                self.categories
                    .get("FOUNDATION")
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
        result.retrieval_trace.category_count = category_ids.len();
        let mut goal_ids = self.goals.get(&query.goal).cloned().unwrap_or_default();
        goal_ids.extend(self.goals.get("*").into_iter().flatten().cloned());
        let ids: Vec<_> = category_ids.intersection(&goal_ids).cloned().collect();
        result.retrieval_trace.goal_count = ids.len();
        result.retrieval_trace.examined = ids.len();
        for id in ids {
            let unit = &self.units[&id];
            let symbols = matched(&unit.pattern.symbols, &query.symbols);
            let relations = matched(&unit.pattern.relations, &query.relations);
            let properties = matched(&unit.pattern.properties, &query.required_properties);
            let premises = matched(&unit.premises, &query.required_properties);
            let constraints = matched(&unit.constraints, &query.constraints);
            let mut matched_features = symbols.1;
            matched_features.extend(relations.1);
            matched_features.extend(properties.1);
            matched_features.sort();
            matched_features.dedup();
            let activation_class = if !constraints.0 {
                ActivationClass::Inapplicable
            } else if symbols.0 && relations.0 && properties.0 && premises.0 {
                ActivationClass::Exact
            } else {
                ActivationClass::Related
            };
            let candidate = KnowledgeCandidate {
                knowledge_id: id,
                match_type: if activation_class == ActivationClass::Exact {
                    "FULL".into()
                } else {
                    "PARTIAL".into()
                },
                matched_features,
                activation_class,
            };
            if candidate.activation_class == ActivationClass::Inapplicable {
                result.rejected.push(candidate);
            } else {
                result.candidates.push(candidate);
            }
        }
        result.candidates.sort_by(|a, b| {
            a.activation_class
                .cmp(&b.activation_class)
                .then_with(|| {
                    self.units[&b.knowledge_id]
                        .priority
                        .cmp(&self.units[&a.knowledge_id].priority)
                })
                .then(a.knowledge_id.cmp(&b.knowledge_id))
        });
        result.retrieval_trace.exact_count = result
            .candidates
            .iter()
            .filter(|c| c.activation_class == ActivationClass::Exact)
            .count();
        result.retrieval_trace.related_count =
            result.candidates.len() - result.retrieval_trace.exact_count;
        result.retrieval_trace.rejected_count = result.rejected.len();
        result.retrieval_trace.truncated_count =
            result.candidates.len().saturating_sub(query.limit);
        result.candidates.truncate(query.limit);
        result
    }
}

fn matched(required: &[String], present: &[String]) -> (bool, Vec<String>) {
    let found: Vec<String> = required
        .iter()
        .filter(|value| present.contains(value))
        .cloned()
        .collect();
    (found.len() == required.len(), found)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub domain: String,
    pub goal: String,
    pub symbols: Vec<String>,
    pub relations: Vec<String>,
    pub required_properties: Vec<String>,
    pub constraints: Vec<String>,
    pub semantic_state_refs: Vec<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationClass {
    Exact,
    Related,
    Inapplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeCandidate {
    pub knowledge_id: String,
    pub match_type: String,
    pub matched_features: Vec<String>,
    pub activation_class: ActivationClass,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalTrace {
    pub total_knowledge: usize,
    pub category_count: usize,
    pub goal_count: usize,
    pub examined: usize,
    pub exact_count: usize,
    pub related_count: usize,
    pub rejected_count: usize,
    pub truncated_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeQueryResult {
    pub query_id: String,
    pub candidates: Vec<KnowledgeCandidate>,
    pub rejected: Vec<KnowledgeCandidate>,
    pub retrieval_trace: RetrievalTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Applicability {
    Applicable,
    PartiallyApplicable,
    NotApplicable,
    Unknown,
    Conflict,
}

pub struct ApplicabilityEngine;
impl ApplicabilityEngine {
    pub fn evaluate(
        unit: &KnowledgeUnit,
        candidate: &KnowledgeCandidate,
        query: &KnowledgeQuery,
        state: &SemanticState,
        has_knowledge: impl Fn(&str) -> bool,
    ) -> Applicability {
        if state.status == Status::Conflict {
            return Applicability::Conflict;
        }
        if query.semantic_state_refs.is_empty() && unit.knowledge_type == "RULE" {
            return Applicability::Unknown;
        }
        if query
            .semantic_state_refs
            .iter()
            .any(|id| state.get(id).is_none() && state.get_occurrence(id).is_none())
        {
            return Applicability::Unknown;
        }
        if unit
            .pattern
            .symbols
            .iter()
            .any(|symbol| symbol == "quadratic")
        {
            let quadratic = query.semantic_state_refs.iter().any(|id| {
                let Some(Object::Entity(entity)) = state.get(id) else { return false; };
                if entity.entity_type != "EQUATION" { return false; }
                matches!(state.value_of(id), Ok(Some((Value::Vector(values), _)))
                    if values.len() == 3 && values.iter().all(|value| matches!(value, Value::Integer(_)))
                        && matches!(&values[2], Value::Integer(a) if *a != 0))
            });
            if !quadratic {
                return Applicability::Unknown;
            }
        }
        if unit.pattern.relations.iter().any(|required| {
            !state.objects.iter().any(|object|
            matches!(object, Object::Relation(relation) if &relation.relation_type == required))
        }) {
            return Applicability::PartiallyApplicable;
        }
        if unit.category != query.domain && unit.category != "FOUNDATION"
            || !unit.applicable_goals.is_empty() && !unit.applicable_goals.contains(&query.goal)
            || unit.dependencies.iter().any(|id| !has_knowledge(id))
            || candidate.activation_class == ActivationClass::Inapplicable
            || !matched(&unit.constraints, &query.constraints).0
        {
            return Applicability::NotApplicable;
        }
        if candidate.activation_class == ActivationClass::Related
            || !matched(&unit.premises, &query.required_properties).0
        {
            return Applicability::PartiallyApplicable;
        }
        Applicability::Applicable
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeActivation {
    pub knowledge_id: String,
    pub knowledge_occurrence: String,
    pub source_state: String,
    pub selected_rus: String,
    pub capability_id: String,
    pub activation_id: String,
    pub ruo_ref: Option<String>,
    pub result_state: Option<String>,
    pub legacy_runtime_rule_id: Option<i64>,
}

impl KnowledgeActivation {
    pub fn from_applicable(unit: &KnowledgeUnit, state: &SemanticState) -> Option<Self> {
        Self::for_capability(unit, state, unit.capabilities.first()?)
    }

    pub fn for_capability(
        unit: &KnowledgeUnit,
        state: &SemanticState,
        capability_id: &str,
    ) -> Option<Self> {
        if !unit.capabilities.iter().any(|id| id == capability_id) {
            return None;
        }
        let source_state = stable_id("state", &[&state.canonical().ok()?]);
        let knowledge_occurrence = unit.occurrence_id();
        Some(Self {
            knowledge_id: unit.knowledge_id.clone(),
            knowledge_occurrence: knowledge_occurrence.clone(),
            source_state: source_state.clone(),
            selected_rus: unit.applicable_rus.first()?.clone(),
            capability_id: capability_id.into(),
            activation_id: stable_id(
                "activation",
                &[&knowledge_occurrence, capability_id, &source_state],
            ),
            ruo_ref: None,
            result_state: None,
            legacy_runtime_rule_id: unit.runtime_rule_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::run_reason_source;

    #[test]
    fn migrated_catalog_is_semantically_equivalent_to_reasonscript() {
        let space = KnowledgeSpace::legacy();
        let catalog = run_reason_source("Knowledge::KCatalog()", &[]).unwrap();
        let entries = catalog.as_array().unwrap();
        assert_eq!(entries.len(), 27);
        for entry in entries {
            let rule = entry["rule"].as_i64().unwrap();
            let unit = space.by_rule(rule).unwrap();
            assert_eq!(unit.knowledge_id, entry["id"]);
            assert_eq!(unit.applicable_rus[0], entry["rus"]);
            assert_eq!(unit.activation_metadata["legacy_domain"], entry["domain"]);
            assert_eq!(
                unit.activation_metadata["curriculum_source"],
                entry["curriculum_source"]
            );
            assert_eq!(unit.provenance.source_id, entry["source"]);
        }
        let comparison = run_reason_source("Knowledge::KMirpComparisonRu()", &[]).unwrap();
        let factor = run_reason_source("Knowledge::KByRule(35)", &[]).unwrap();
        for entry in [&comparison, &factor] {
            let unit = space.by_rule(entry["rule"].as_i64().unwrap()).unwrap();
            let name = entry.get("knowledge_id").unwrap_or(&entry["id"]);
            assert_eq!(unit.knowledge_id, *name);
            assert_eq!(unit.applicable_rus[0], entry["rus"]);
            assert_eq!(unit.activation_metadata["legacy_domain"], entry["domain"]);
            assert_eq!(unit.provenance.source_id, entry["source"]);
        }
        assert_eq!(space.len(), 29);
    }
}
