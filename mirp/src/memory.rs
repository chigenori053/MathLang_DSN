//! Persistent MIRP snapshots with DSN_Test's native ReasonScript memory codec.

use crate::{session::run_reason_source, stable_id, MirpError, SemanticState, VERSION};
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemorySpace {
    entries: BTreeMap<String, Stored>,
}

impl MemorySpace {
    pub fn new() -> Self {
        Self::default()
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
            let rule = ruo["source_ru"]
                .as_i64()
                .ok_or_else(|| error("MirpValidationError", "RUO source_ru"))?;
            let sequence = ruo["source_rus"]
                .as_str()
                .ok_or_else(|| error("MirpValidationError", "RUO source_rus"))?;
            let execution = ruo["knowledge_id"]
                .as_str()
                .ok_or_else(|| error("MirpValidationError", "RUO knowledge_id"))?;
            rules.push(rule.to_string());
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
        for object in &mut state.objects {
            object.common_mut().provenance.memory_id = Some(memory_id.into());
        }
        state.validate()?;
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
        let memory: Self =
            serde_json::from_str(&text).map_err(|exc| error("MirpParseError", exc.to_string()))?;
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
}
