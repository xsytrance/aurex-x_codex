use std::collections::BTreeSet;

use aurex_scene::Vec3;
use serde::{Deserialize, Serialize};

use crate::pulse_graph::PulseGraphRunner;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistrictPrime {
    Electronic,
    Jazz,
    Rock,
    Ambient,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct District {
    pub id: String,
    pub prime: DistrictPrime,
    pub center: Vec3,
    pub radius: f32,
    #[serde(default)]
    pub pulse_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PulsePortal {
    pub id: String,
    pub trigger: String,
    pub target_node: String,
    pub position: Vec3,
    pub activation_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BootWorldGenerator {
    #[serde(default)]
    pub seed: u32,
    #[serde(default)]
    pub districts: Vec<District>,
    #[serde(default)]
    pub portals: Vec<PulsePortal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BootWorldState {
    #[serde(default)]
    pub player_position: Vec3,
    #[serde(default)]
    pub active_district: Option<String>,
    #[serde(default)]
    pub nearest_portal: Option<String>,
    #[serde(default)]
    pub launched_portals: BTreeSet<String>,
    #[serde(default)]
    pub resonance_tracker: Option<serde_json::Value>,
    #[serde(default)]
    pub living_boot_screen: Option<serde_json::Value>,
}

impl Default for BootWorldState {
    fn default() -> Self {
        Self {
            player_position: Vec3::new(0.0, 0.0, 0.0),
            active_district: None,
            nearest_portal: None,
            launched_portals: BTreeSet::new(),
            resonance_tracker: None,
            living_boot_screen: None,
        }
    }
}

impl BootWorldState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_player_position(&mut self, cfg: &BootWorldGenerator, player_position: Vec3) {
        self.player_position = player_position;

        self.active_district = cfg
            .districts
            .iter()
            .find(|d| distance(player_position, d.center) <= d.radius.max(0.0))
            .map(|d| d.id.clone());

        self.nearest_portal = cfg
            .portals
            .iter()
            .min_by(|a, b| {
                distance(player_position, a.position)
                    .partial_cmp(&distance(player_position, b.position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.id.clone());
    }

    pub fn emit_portal_triggers(
        &mut self,
        cfg: &BootWorldGenerator,
        graph_runner: &mut PulseGraphRunner,
    ) {
        for portal in &cfg.portals {
            let hit = distance(self.player_position, portal.position) <= portal.activation_radius;
            if hit && !self.launched_portals.contains(&portal.id) {
                graph_runner.trigger_manual(portal.trigger.clone());
                self.launched_portals.insert(portal.id.clone());
            }
        }
    }
}

fn distance(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}
