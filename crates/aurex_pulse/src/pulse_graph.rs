use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use aurex_render_sdf::{RenderConfig, RenderedFrame};
use serde::{Deserialize, Serialize};

use crate::{loader::load_pulse_from_path, runner::PulseRunner};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseGraph {
    pub name: String,
    #[serde(default)]
    pub seed: u32,
    pub entry_node: String,
    pub nodes: Vec<PulseNode>,
    #[serde(default)]
    pub transitions: Vec<PulseTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseNode {
    pub id: String,
    pub pulse_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulseTransition {
    pub from: String,
    pub to: String,
    #[serde(flatten)]
    pub kind: PulseTransitionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "transition_type")]
pub enum PulseTransitionKind {
    Manual { trigger: String },
    Timeline { after_seconds: f32 },
    MusicalCue { cue: MusicalCue },
    GeneratorTrigger { event: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cue")]
pub enum MusicalCue {
    BeatStrengthAbove { threshold: f32 },
    BassEnergyAbove { threshold: f32 },
    BarMultiple { multiple: u64 },
}

impl PulseGraph {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("graph name must not be empty".into());
        }
        if self.nodes.is_empty() {
            return Err("graph must contain at least one node".into());
        }

        let node_ids: BTreeSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        if !node_ids.contains(self.entry_node.as_str()) {
            return Err("entry_node must reference an existing node id".into());
        }
        for t in &self.transitions {
            if !node_ids.contains(t.from.as_str()) || !node_ids.contains(t.to.as_str()) {
                return Err(format!(
                    "transition references unknown nodes: {} -> {}",
                    t.from, t.to
                ));
            }
        }
        Ok(())
    }
}

pub fn load_pulse_graph_from_str(contents: &str) -> Result<PulseGraph, Box<dyn std::error::Error>> {
    let graph: PulseGraph = serde_json::from_str(contents)?;
    graph
        .validate()
        .map_err(|e| format!("pulse graph validation failed: {e}"))?;
    Ok(graph)
}

pub fn load_pulse_graph_from_path(
    path: impl AsRef<Path>,
) -> Result<PulseGraph, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    load_pulse_graph_from_str(&data)
}

pub fn electronic_journey_graph() -> PulseGraph {
    PulseGraph {
        name: "ElectronicJourneyGraph".into(),
        seed: 424242,
        entry_node: "AmbientIntroPulse".into(),
        nodes: vec![
            PulseNode {
                id: "AmbientIntroPulse".into(),
                pulse_path: "examples/pulses/ambient_intro.pulse.json".into(),
            },
            PulseNode {
                id: "PsytranceTunnelPulse".into(),
                pulse_path: "examples/pulses/psytrance_tunnel.pulse.json".into(),
            },
            PulseNode {
                id: "InfiniteCircuitCityPulse".into(),
                pulse_path: "examples/pulses/infinite_circuit_megacity.pulse.json".into(),
            },
            PulseNode {
                id: "FractalFinalePulse".into(),
                pulse_path: "examples/pulses/fractal_finale.pulse.json".into(),
            },
        ],
        transitions: vec![
            PulseTransition {
                from: "AmbientIntroPulse".into(),
                to: "PsytranceTunnelPulse".into(),
                kind: PulseTransitionKind::Timeline {
                    after_seconds: 12.0,
                },
            },
            PulseTransition {
                from: "PsytranceTunnelPulse".into(),
                to: "InfiniteCircuitCityPulse".into(),
                kind: PulseTransitionKind::MusicalCue {
                    cue: MusicalCue::BeatStrengthAbove { threshold: 0.9 },
                },
            },
            PulseTransition {
                from: "InfiniteCircuitCityPulse".into(),
                to: "FractalFinalePulse".into(),
                kind: PulseTransitionKind::GeneratorTrigger {
                    event: "city_gate_opened".into(),
                },
            },
            PulseTransition {
                from: "FractalFinalePulse".into(),
                to: "AmbientIntroPulse".into(),
                kind: PulseTransitionKind::Manual {
                    trigger: "restart".into(),
                },
            },
        ],
    }
}

pub struct PulseGraphRunner {
    pub graph: PulseGraph,
    pub active_node_id: String,
    pub active_runner: PulseRunner,
    elapsed_in_node_seconds: f32,
    manual_triggers: BTreeSet<String>,
    generator_events: BTreeSet<String>,
    graph_file_path: Option<PathBuf>,
    node_map: BTreeMap<String, PulseNode>,
}

impl PulseGraphRunner {
    pub fn load(
        graph: PulseGraph,
        graph_file_path: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        graph
            .validate()
            .map_err(|e| format!("pulse graph validation failed: {e}"))?;

        let node_map: BTreeMap<String, PulseNode> = graph
            .nodes
            .iter()
            .cloned()
            .map(|n| (n.id.clone(), n))
            .collect();

        let entry = node_map
            .get(&graph.entry_node)
            .ok_or_else(|| format!("entry node '{}' not found", graph.entry_node))?;

        let graph_file_path_buf = graph_file_path.map(|p| p.to_path_buf());
        let mut active_runner = load_runner_for_node(entry, graph_file_path)?;
        active_runner.initialize();

        Ok(Self {
            graph,
            active_node_id: entry.id.clone(),
            active_runner,
            elapsed_in_node_seconds: 0.0,
            manual_triggers: BTreeSet::new(),
            generator_events: BTreeSet::new(),
            graph_file_path: graph_file_path_buf,
            node_map,
        })
    }

    pub fn trigger_manual(&mut self, trigger: impl Into<String>) {
        self.manual_triggers.insert(trigger.into());
    }

    pub fn emit_generator_event(&mut self, event: impl Into<String>) {
        self.generator_events.insert(event.into());
    }

    pub fn update(&mut self, delta_seconds: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.active_runner.update(delta_seconds);
        self.elapsed_in_node_seconds = (self.elapsed_in_node_seconds + delta_seconds).max(0.0);

        if let Some(next_node) = self.select_transition_target() {
            self.switch_to_node(next_node)?;
        }

        Ok(())
    }

    pub fn render(&mut self, config: RenderConfig) -> RenderedFrame {
        self.active_runner.render(config)
    }

    fn select_transition_target(&self) -> Option<String> {
        let rhythm = self.active_runner.rhythm_field();
        self.graph
            .transitions
            .iter()
            .find(|t| {
                if t.from != self.active_node_id {
                    return false;
                }
                match &t.kind {
                    PulseTransitionKind::Manual { trigger } => {
                        self.manual_triggers.contains(trigger)
                    }
                    PulseTransitionKind::Timeline { after_seconds } => {
                        self.elapsed_in_node_seconds >= (*after_seconds).max(0.0)
                    }
                    PulseTransitionKind::MusicalCue { cue } => {
                        let Some(rf) = rhythm else {
                            return false;
                        };
                        match cue {
                            MusicalCue::BeatStrengthAbove { threshold } => {
                                rf.beat_strength >= *threshold
                            }
                            MusicalCue::BassEnergyAbove { threshold } => {
                                rf.bass_energy >= *threshold
                            }
                            MusicalCue::BarMultiple { multiple } => {
                                let m = (*multiple).max(1);
                                rf.bar_index > 0 && rf.bar_index % m == 0
                            }
                        }
                    }
                    PulseTransitionKind::GeneratorTrigger { event } => {
                        self.generator_events.contains(event)
                    }
                }
            })
            .map(|t| t.to.clone())
    }

    fn switch_to_node(&mut self, node_id: String) -> Result<(), Box<dyn std::error::Error>> {
        if node_id == self.active_node_id {
            return Ok(());
        }

        let node = self
            .node_map
            .get(&node_id)
            .ok_or_else(|| format!("node '{node_id}' not found"))?
            .clone();

        self.active_runner.shutdown();
        let mut next_runner = load_runner_for_node(&node, self.graph_file_path.as_deref())?;
        next_runner.initialize();

        self.active_runner = next_runner;
        self.active_node_id = node.id;
        self.elapsed_in_node_seconds = 0.0;
        self.manual_triggers.clear();
        self.generator_events.clear();

        Ok(())
    }
}

fn load_runner_for_node(
    node: &PulseNode,
    graph_file_path: Option<&Path>,
) -> Result<PulseRunner, Box<dyn std::error::Error>> {
    let resolved = if Path::new(&node.pulse_path).is_absolute() {
        PathBuf::from(&node.pulse_path)
    } else if let Some(base) = graph_file_path.and_then(|p| p.parent()) {
        base.join(&node.pulse_path)
    } else {
        PathBuf::from(&node.pulse_path)
    };

    let pulse = load_pulse_from_path(&resolved)?;
    PulseRunner::load(pulse, Some(&resolved))
}

#[cfg(test)]
mod tests {
    use super::{PulseGraph, electronic_journey_graph, load_pulse_graph_from_str};

    #[test]
    fn electronic_journey_graph_is_valid() {
        let graph = electronic_journey_graph();
        graph.validate().expect("graph should validate");
        assert_eq!(graph.entry_node, "AmbientIntroPulse");
    }

    #[test]
    fn graph_json_parses_and_validates() {
        let graph_json = r#"{
          "name": "mini_graph",
          "entry_node": "a",
          "nodes": [
            {"id": "a", "pulse_path": "a.pulse.json"},
            {"id": "b", "pulse_path": "b.pulse.json"}
          ],
          "transitions": [
            {"from": "a", "to": "b", "transition_type": "Manual", "trigger": "go"}
          ]
        }"#;

        let graph: PulseGraph = load_pulse_graph_from_str(graph_json).expect("graph parse");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.transitions.len(), 1);
    }
}
