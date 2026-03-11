use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scene {
    pub name: String,
    pub nodes: Vec<SceneNode>,
    pub camera: Option<CameraDefinition>,
    pub sync: Option<AudioSyncBindings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneNode {
    pub id: String,
    pub node_type: String,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraDefinition {
    pub path: Vec<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSyncBindings {
    pub kick: Option<String>,
    pub snare: Option<String>,
    pub bass: Option<String>,
}

#[derive(Debug)]
pub enum SceneError {
    Parse(serde_json::Error),
    Serialize(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "failed to parse scene json: {err}"),
            Self::Serialize(err) => write!(f, "failed to serialize scene json: {err}"),
            Self::Validation(message) => write!(f, "scene validation failed: {message}"),
        }
    }
}

impl std::error::Error for SceneError {}

impl Scene {
    pub fn from_json(input: &str) -> Result<Self, SceneError> {
        let scene: Self = serde_json::from_str(input).map_err(SceneError::Parse)?;
        scene.validate()?;
        Ok(scene)
    }

    pub fn to_json(&self) -> Result<String, SceneError> {
        serde_json::to_string_pretty(self).map_err(SceneError::Serialize)
    }

    fn validate(&self) -> Result<(), SceneError> {
        if self.name.trim().is_empty() {
            return Err(SceneError::Validation(
                "scene name must not be empty".to_owned(),
            ));
        }

        for (index, node) in self.nodes.iter().enumerate() {
            if node.id.trim().is_empty() {
                return Err(SceneError::Validation(format!(
                    "node at index {index} has an empty id"
                )));
            }

            if node.node_type.trim().is_empty() {
                return Err(SceneError::Validation(format!(
                    "node at index {index} has an empty node_type"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Scene;

    #[test]
    fn scene_round_trips_json() {
        let input = r#"{
            "name":"neon_tunnel",
            "nodes":[
                {
                    "id":"tunnel1",
                    "node_type":"TunnelGenerator",
                    "parameters": {
                        "radius": 5.0,
                        "twist": 0.2,
                        "glow": 3.0
                    }
                }
            ],
            "camera":{"path":[[0.0,0.0,10.0],[5.0,2.0,4.0],[12.0,0.0,-6.0]]},
            "sync":{"kick":"tunnel1.glow","snare":null,"bass":"tunnel1.radius"}
        }"#;

        let scene = Scene::from_json(input).expect("scene should parse");
        let json = scene.to_json().expect("scene should serialize");
        let reparsed = Scene::from_json(&json).expect("serialized scene should parse");
        assert_eq!(scene, reparsed);
    }
}
