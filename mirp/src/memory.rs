//! Persistent MIRP snapshots with DSN_Test's native ReasonScript memory codec.

use crate::{
    knowledge::{KnowledgeQuery, KnowledgeQueryResult, KnowledgeSpace, KnowledgeUnit},
    session::run_reason_source,
    stable_id, Common, Evidence, MirpError, Object, Provenance, SemanticDelta, SemanticState,
    Status, VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn error(category: &'static str, message: impl Into<String>) -> MirpError {
    MirpError {
        category,
        message: message.into(),
        source_id: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Stored {
    state: SemanticState,
    packet: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpace {
    entries: BTreeMap<String, Stored>,
    #[serde(default = "KnowledgeSpace::legacy")]
    knowledge_space: KnowledgeSpace,
}

impl Default for MemorySpace {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            knowledge_space: KnowledgeSpace::legacy(),
        }
    }
}

impl MemorySpace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_knowledge_space(mut knowledge_space: KnowledgeSpace) -> Result<Self, MirpError> {
        knowledge_space
            .validate_and_reindex()
            .map_err(|message| error("MirpValidationError", message))?;
        Ok(Self {
            entries: BTreeMap::new(),
            knowledge_space,
        })
    }

    pub fn knowledge_space(&self) -> &KnowledgeSpace {
        &self.knowledge_space
    }

    pub fn get_knowledge(&self, knowledge_id: &str) -> Option<&KnowledgeUnit> {
        self.knowledge_space.get(knowledge_id)
    }

    /// Compatibility bridge for ReasonScript's numeric KByRule interface.
    pub fn legacy_by_rule(&self, runtime_rule_id: i64) -> Option<&KnowledgeUnit> {
        self.knowledge_space.by_rule(runtime_rule_id)
    }

    pub fn register_knowledge(&mut self, units: Vec<KnowledgeUnit>) -> Result<(), MirpError> {
        self.knowledge_space
            .register_batch(units)
            .map_err(|message| error("MirpValidationError", message))
    }

    pub fn query_knowledge(&self, query: &KnowledgeQuery) -> KnowledgeQueryResult {
        self.knowledge_space.query(query)
    }

    pub fn store(
        &mut self,
        memory_id: &str,
        state: &SemanticState,
        ruos: &[Json],
    ) -> Result<(), MirpError> {
        if memory_id.is_empty() || ruos.is_empty() {
            return Err(error(
                "MirpValidationError",
                "memory ID and RUO trace required",
            ));
        }
        state.validate()?;
        let mut rules = vec![];
        let mut sequences = vec![];
        let mut executions = vec![];
        for ruo in ruos {
            let rule = ruo["ru_ref"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| ruo["source_ru"].as_i64().map(|id| id.to_string()))
                .ok_or_else(|| error("MirpValidationError", "RUO ru_ref"))?;
            let sequence = ruo["rus_ref"]
                .as_str()
                .or_else(|| ruo["source_rus"].as_str())
                .ok_or_else(|| error("MirpValidationError", "RUO rus_ref"))?;
            let execution = ruo["knowledge_id"]
                .as_str()
                .ok_or_else(|| error("MirpValidationError", "RUO knowledge_id"))?;
            rules.push(rule);
            sequences.push(sequence.to_string());
            executions.push(execution.to_string());
        }
        let canonical = state.canonical()?;
        let digest = stable_id("state", &[&canonical]);
        let source = format!("Memory_codec_f4::Encode(Memory_codec_f4::New({}, {}, {}, {}, [], [{}], {}, {}, \"\", {}, \"mirp\", 0, 0), \"DICTIONARY\")",
            quote(memory_id)?, quote(&rules)?, quote(&sequences)?, quote(&executions)?,
            quote(&digest)?, quote(state.goals.first().cloned().unwrap_or_default())?,
            quote(format!("{:?}", state.status).to_uppercase())?, quote(VERSION)?);
        let packet = run_reason_source(&source, &[("memory_codec_f4", "Memory_codec_f4")])?;
        self.entries.insert(
            memory_id.into(),
            Stored {
                state: state.clone(),
                packet,
            },
        );
        Ok(())
    }

    pub fn retrieve(&self, memory_id: &str) -> Result<SemanticState, MirpError> {
        let stored = self
            .entries
            .get(memory_id)
            .ok_or_else(|| error("MirpReferenceError", memory_id))?;
        stored.state.validate()?;
        let packet = &stored.packet;
        let fields = [
            "memory_id",
            "mode",
            "dictionary",
            "ru_codes",
            "rus_codes",
            "ruo_codes",
            "raw_ru",
            "raw_rus",
            "raw_ruo",
            "relations",
            "context",
            "stored_goal",
            "stored_status",
            "stored_concept",
            "knowledge_version",
            "provenance",
            "logical_sequence",
            "usage_count",
        ];
        let arguments: Result<Vec<String>, MirpError> = fields
            .iter()
            .map(|name| {
                packet
                    .get(*name)
                    .ok_or_else(|| error("MirpParseError", format!("memory packet missing {name}")))
                    .and_then(quote)
            })
            .collect();
        let source = format!(
            "Memory_codec_f4::Decode(Memory_codec_f4::FromSerialized({}))",
            arguments?.join(", ")
        );
        let decoded = run_reason_source(&source, &[("memory_codec_f4", "Memory_codec_f4")])?;
        if decoded["valid"] != true
            || decoded["unit"]["memory_id"] != memory_id
            || decoded["unit"]["knowledge_version"] != VERSION
        {
            return Err(error(
                "MirpValidationError",
                "invalid or stale memory packet",
            ));
        }
        let canonical = stored.state.canonical()?;
        if decoded["unit"]["context"][0] != stable_id("state", &[&canonical]) {
            return Err(error(
                "MirpValidationError",
                "memory packet does not match semantic state",
            ));
        }
        let mut state = stored.state.clone();
        let occurrence_ids: Vec<String> = state
            .objects
            .iter()
            .map(|object| object.occurrence_id().into())
            .collect();
        let semantic_ids: Vec<String> = state
            .objects
            .iter()
            .map(|object| object.id().into())
            .collect();
        let mut provenance = Provenance::input("memory_retrieval", memory_id);
        provenance.memory_id = Some(memory_id.into());
        provenance.parent_occurrence_ids = occurrence_ids.clone();
        let mut evidence = Object::Evidence(Evidence {
            common: Common::unidentified(Status::Known, provenance),
            evidence_type: "MEMORY_RETRIEVAL".into(),
            source: "MemorySpace".into(),
            supports: semantic_ids.clone(),
            contradicts: vec![],
        });
        evidence.common_mut().metadata.insert(
            "retrieved_occurrence_ids".into(),
            serde_json::json!(occurrence_ids),
        );
        evidence.common_mut().metadata.insert(
            "retrieved_semantic_ids".into(),
            serde_json::json!(semantic_ids),
        );
        evidence.normalize_identity(&format!("memory-retrieval:{memory_id}"));
        let dependencies = BTreeMap::from([(evidence.occurrence_id().to_owned(), occurrence_ids)]);
        state.apply(SemanticDelta {
            added: vec![evidence],
            dependencies,
            ..Default::default()
        })?;
        Ok(state)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), MirpError> {
        let path = path.as_ref();
        let temporary = path.with_extension("memory.tmp");
        let text =
            serde_json::to_string(self).map_err(|exc| error("MirpParseError", exc.to_string()))?;
        fs::write(&temporary, format!("{text}\n"))
            .and_then(|_| fs::rename(&temporary, path))
            .map_err(|exc| error("MirpValidationError", exc.to_string()))
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, MirpError> {
        let text =
            fs::read_to_string(path).map_err(|exc| error("MirpParseError", exc.to_string()))?;
        let mut memory: Self =
            serde_json::from_str(&text).map_err(|exc| error("MirpParseError", exc.to_string()))?;
        memory
            .knowledge_space
            .validate_and_reindex()
            .map_err(|message| error("MirpValidationError", message))?;
        for id in memory.entries.keys() {
            memory.retrieve(id)?;
        }
        Ok(memory)
    }
}

fn quote(value: impl Serialize) -> Result<String, MirpError> {
    serde_json::to_string(&value).map_err(|exc| error("MirpParseError", exc.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Provenance, Status, Value};

    #[test]
    fn stale_and_mismatched_memory_is_rejected() {
        let mut state = SemanticState::default();
        let p = Provenance::input("test", "x=1");
        let x = state
            .bind("x", "n", "global", "VARIABLE", p.clone())
            .unwrap();
        state
            .assert_value(&x, Value::Integer(1), p, Status::Known)
            .unwrap();
        let trace = serde_json::json!({"source_ru": 12, "source_rus": "PolynomialRUS", "knowledge_id": "POLY_ADD"});
        let mut memory = MemorySpace::new();
        memory.store("m", &state, &[trace]).unwrap();
        memory.entries.get_mut("m").unwrap().packet["knowledge_version"] =
            Json::String("stale".into());
        assert!(memory.retrieve("m").is_err());
        memory.entries.get_mut("m").unwrap().packet["knowledge_version"] =
            Json::String(VERSION.into());
        memory
            .entries
            .get_mut("m")
            .unwrap()
            .state
            .goals
            .push("changed".into());
        assert!(memory.retrieve("m").is_err());
    }

    #[test]
    fn malformed_trace_and_invalid_state_are_rejected() {
        let mut state = SemanticState::default();
        let provenance = Provenance::input("test", "x=1");
        let x = state
            .bind("x", "n", "global", "VARIABLE", provenance.clone())
            .unwrap();
        state
            .assert_value(&x, Value::Integer(1), provenance, Status::Known)
            .unwrap();
        let trace = serde_json::json!({"source_ru": 12, "source_rus": "PolynomialRUS", "knowledge_id": "POLY_ADD"});
        let mut memory = MemorySpace::new();
        memory.store("m", &state, &[trace]).unwrap();
        let packet = memory.entries["m"].packet.clone();
        memory.entries.get_mut("m").unwrap().packet["ru_codes"] = serde_json::json!([]);
        assert!(memory.retrieve("m").is_err());
        memory.entries.get_mut("m").unwrap().packet = packet;
        memory.entries.get_mut("m").unwrap().state.objects[0]
            .common_mut()
            .confidence = 2.0;
        assert!(memory.retrieve("m").is_err());
    }
}
