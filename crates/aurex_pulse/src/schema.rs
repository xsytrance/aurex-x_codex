use std::collections::BTreeMap;

use aurex_audio::ProceduralAudioConfig;
use aurex_music::sequencer::MusicSequenceConfig;
use aurex_scene::{Scene, SceneTimeline};
use serde::{Deserialize, Serialize};

use crate::boot_world::BootWorldGenerator;
use crate::resonance::PrimeFaction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PulseKind {
    Game,
    World,
    VisualMusic,
    Demo,
    Ambient,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Interactivity {
    Interactive,
    Passive,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseMetadata {
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub seed: u32,
    pub pulse_kind: PulseKind,
    #[serde(default)]
    pub duration_hint: Option<f32>,
    pub interactivity: Interactivity,
    #[serde(default, deserialize_with = "deserialize_prime_affinity_opt")]
    pub prime_affinity: Option<PrimeFaction>,
}

fn deserialize_prime_affinity_opt<'de, D>(deserializer: D) -> Result<Option<PrimeFaction>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Option::<String>::deserialize(deserializer)?;
    match raw {
        Some(label) => PrimeFaction::from_label(&label).map(Some).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid metadata.prime_affinity '{}'; expected a known PrimeFaction label",
                label
            ))
        }),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum PulseSceneSource {
    Inline(Scene),
    ScenePath { scene_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseGeneratorBinding {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseDefinition {
    pub metadata: PulseMetadata,
    pub pulse_kind: PulseKind,
    pub scene: PulseSceneSource,
    #[serde(default)]
    pub audio: Option<ProceduralAudioConfig>,
    #[serde(default)]
    pub timeline: Option<SceneTimeline>,
    #[serde(default)]
    pub generators: Vec<PulseGeneratorBinding>,
    #[serde(default)]
    pub music: Option<MusicSequenceConfig>,
    #[serde(default)]
    pub boot_world: Option<BootWorldGenerator>,
    #[serde(default)]
    pub parameters: BTreeMap<String, serde_json::Value>,
}

impl PulseDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.pulse_kind != self.pulse_kind {
            return Err("metadata.pulse_kind must match pulse_kind".to_string());
        }
        if self.metadata.title.trim().is_empty() {
            return Err("metadata.title must not be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{PrimeFaction, PulseDefinition};

    #[test]
    fn valid_prime_affinity_deserializes() {
        let json = r#"{
            "metadata": {
                "title": "x",
                "author": "x",
                "pulse_kind": "World",
                "interactivity": "Passive",
                "prime_affinity": "Electronic"
            },
            "pulse_kind": "World",
            "scene": { "scene_path": "examples/infinite_circuit_megacity.json" }
        }"#;

        let pulse: PulseDefinition = serde_json::from_str(json).expect("json should parse");
        assert_eq!(
            pulse.metadata.prime_affinity,
            Some(PrimeFaction::Electronic)
        );
    }

    #[test]
    fn invalid_prime_affinity_fails_deserialization() {
        let json = r#"{
            "metadata": {
                "title": "x",
                "author": "x",
                "pulse_kind": "World",
                "interactivity": "Passive",
                "prime_affinity": "NotARealPrime"
            },
            "pulse_kind": "World",
            "scene": { "scene_path": "examples/infinite_circuit_megacity.json" }
        }"#;

        let err = serde_json::from_str::<PulseDefinition>(json).expect_err("invalid prime");
        assert!(
            err.to_string()
                .contains("invalid metadata.prime_affinity 'NotARealPrime'")
        );
    }
}
