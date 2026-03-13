use aurex_scene::Vec3;
use serde::{Deserialize, Serialize};

use crate::resonance::{PrimeFaction, ResonanceProfile};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimePulseLayer {
    pub layer_index: u32,
    pub required_prime_count: u32,
    pub required_resonance: f32,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimePulseState {
    pub position: Vec3,
    pub active_layer: u32,
    pub total_layers: u32,
    pub unlocked_layers: u32,
    pub proximity_distance: f32,
    pub layers: Vec<PrimePulseLayer>,
}

impl PrimePulseState {
    pub fn default_boot_world() -> Self {
        let layers = vec![
            PrimePulseLayer {
                layer_index: 1,
                required_prime_count: 3,
                required_resonance: 0.25,
                unlocked: false,
            },
            PrimePulseLayer {
                layer_index: 2,
                required_prime_count: 6,
                required_resonance: 0.35,
                unlocked: false,
            },
            PrimePulseLayer {
                layer_index: 3,
                required_prime_count: 9,
                required_resonance: 0.50,
                unlocked: false,
            },
            PrimePulseLayer {
                layer_index: 4,
                required_prime_count: 12,
                required_resonance: 0.60,
                unlocked: false,
            },
        ];

        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            active_layer: 1,
            total_layers: layers.len() as u32,
            unlocked_layers: 0,
            proximity_distance: f32::MAX,
            layers,
        }
    }

    pub fn update(&mut self, player_position: Vec3, profile: &ResonanceProfile) {
        self.proximity_distance = distance(player_position, self.position);

        for layer in &mut self.layers {
            if !layer.unlocked
                && satisfies_layer(
                    profile,
                    layer.required_prime_count,
                    layer.required_resonance,
                )
            {
                layer.unlocked = true;
            }
        }

        self.unlocked_layers = self.layers.iter().filter(|l| l.unlocked).count() as u32;
        self.active_layer = (self.unlocked_layers + 1).min(self.total_layers);
    }

    pub fn force_field_active(&self) -> bool {
        self.unlocked_layers < self.total_layers
    }

    pub fn prime_pulse_proximity(&self) -> f32 {
        (1.0 / (1.0 + self.proximity_distance.max(0.0))).clamp(0.0, 1.0)
    }

    pub fn prime_pulse_intensity(&self) -> f32 {
        let unlock_factor = if self.total_layers > 0 {
            self.unlocked_layers as f32 / self.total_layers as f32
        } else {
            0.0
        };
        (self.prime_pulse_proximity() * (0.3 + 0.7 * unlock_factor)).clamp(0.0, 1.0)
    }
}

fn satisfies_layer(
    profile: &ResonanceProfile,
    required_prime_count: u32,
    required_resonance: f32,
) -> bool {
    let count = PrimeFaction::all()
        .iter()
        .filter(|p| profile.get_resonance(**p).value >= required_resonance)
        .count() as u32;
    count >= required_prime_count
}

fn distance(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::PrimePulseState;
    use crate::resonance::{PrimeFaction, ResonanceProfile, ResonanceValue};

    fn profile_with_value(v: f32, count: usize) -> ResonanceProfile {
        let mut p = ResonanceProfile::default();
        for prime in PrimeFaction::all().iter().take(count) {
            p.set_resonance(
                *prime,
                ResonanceValue {
                    value: v,
                    activity_score: 1.0,
                    last_update_time: 1.0,
                },
            );
        }
        p
    }

    #[test]
    fn thresholds_unlock_layers() {
        let mut s = PrimePulseState::default_boot_world();
        let p = profile_with_value(0.61, 12);
        s.update(aurex_scene::Vec3::new(0.0, 0.0, 3.0), &p);
        assert_eq!(s.unlocked_layers, s.total_layers);
        assert!(!s.force_field_active());
    }

    #[test]
    fn deterministic_progression_for_fixed_profile() {
        let mut a = PrimePulseState::default_boot_world();
        let mut b = PrimePulseState::default_boot_world();
        let p = profile_with_value(0.36, 6);
        a.update(aurex_scene::Vec3::new(2.0, 0.0, 0.0), &p);
        b.update(aurex_scene::Vec3::new(2.0, 0.0, 0.0), &p);
        assert_eq!(a, b);
    }

    #[test]
    fn proximity_tracking_works() {
        let mut s = PrimePulseState::default_boot_world();
        let p = ResonanceProfile::default();
        s.update(aurex_scene::Vec3::new(3.0, 4.0, 0.0), &p);
        assert!((s.proximity_distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn layers_stay_locked_when_unmet() {
        let mut s = PrimePulseState::default_boot_world();
        let p = profile_with_value(0.2, 12);
        s.update(aurex_scene::Vec3::new(0.0, 0.0, 1.0), &p);
        assert_eq!(s.unlocked_layers, 0);
        assert!(s.force_field_active());
    }
}
