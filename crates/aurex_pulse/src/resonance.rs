use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::PulseDefinition;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrimeFaction {
    Pop,
    Rock,
    #[serde(rename = "HipHopRap", alias = "Hip-Hop / Rap")]
    HipHopRap,
    Electronic,
    #[serde(rename = "RnBSoulFunk", alias = "R&B / Soul / Funk")]
    RnBSoulFunk,
    Classical,
    Jazz,
    #[serde(rename = "CountryFolk", alias = "Country / Folk")]
    CountryFolk,
    #[serde(
        rename = "ReggaeCaribbeanAfrobeat",
        alias = "Reggae / Caribbean / Afrobeat"
    )]
    ReggaeCaribbeanAfrobeat,
    #[serde(rename = "WorldTraditional", alias = "World / Traditional")]
    WorldTraditional,
    #[serde(rename = "AmbientExperimental", alias = "Ambient / Experimental")]
    #[serde(alias = "Ambient")]
    AmbientExperimental,
    #[serde(rename = "PlayToyCartoon", alias = "Play / Toy / Cartoon")]
    Kazoom,
}

impl PrimeFaction {
    pub fn all() -> [Self; 12] {
        [
            Self::Pop,
            Self::Rock,
            Self::HipHopRap,
            Self::Electronic,
            Self::RnBSoulFunk,
            Self::Classical,
            Self::Jazz,
            Self::CountryFolk,
            Self::ReggaeCaribbeanAfrobeat,
            Self::WorldTraditional,
            Self::AmbientExperimental,
            Self::Kazoom,
        ]
    }

    fn rank(self) -> u8 {
        match self {
            Self::Pop => 0,
            Self::Rock => 1,
            Self::HipHopRap => 2,
            Self::Electronic => 3,
            Self::RnBSoulFunk => 4,
            Self::Classical => 5,
            Self::Jazz => 6,
            Self::CountryFolk => 7,
            Self::ReggaeCaribbeanAfrobeat => 8,
            Self::WorldTraditional => 9,
            Self::AmbientExperimental => 10,
            Self::Kazoom => 11,
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label.trim().to_ascii_lowercase().as_str() {
            "pop" | "aurielle" => Some(Self::Pop),
            "rock" | "lord riffion" => Some(Self::Rock),
            "hiphoprap" | "hip-hop / rap" | "hip-hop" | "rap" | "mc baraka" => {
                Some(Self::HipHopRap)
            }
            "electronic" | "djinn" => Some(Self::Electronic),
            "rnbsoulfunk" | "r&b / soul / funk" | "r&b" | "soul" | "funk" => {
                Some(Self::RnBSoulFunk)
            }
            "classical" | "octavius audius rudwig" => Some(Self::Classical),
            "jazz" | "blue rondo" => Some(Self::Jazz),
            "countryfolk" | "country / folk" | "country" | "folk" | "dust strummer" => {
                Some(Self::CountryFolk)
            }
            "reggaecaribbeanafrobeat"
            | "reggae / caribbean / afrobeat"
            | "reggae"
            | "caribbean"
            | "afrobeat"
            | "oba fyah irie" => Some(Self::ReggaeCaribbeanAfrobeat),
            "worldtraditional" | "world / traditional" | "world" | "traditional" => {
                Some(Self::WorldTraditional)
            }
            "ambientexperimental"
            | "ambient / experimental"
            | "ambient"
            | "experimental"
            | "aetherion"
            | "cinematic" => Some(Self::AmbientExperimental),
            "playtoycartoon" | "play / toy / cartoon" | "play" | "toy" | "cartoon" | "kazoom" => {
                Some(Self::Kazoom)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ResonanceValue {
    pub value: f32,
    pub activity_score: f32,
    pub last_update_time: f32,
}

impl Default for ResonanceValue {
    fn default() -> Self {
        Self {
            value: 0.0,
            activity_score: 0.0,
            last_update_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResonanceProfile {
    pub resonance_map: HashMap<PrimeFaction, ResonanceValue>,
    pub total_playtime: f32,
    pub pulse_count: u32,
    pub district_visits: HashMap<PrimeFaction, u32>,
}

impl Default for ResonanceProfile {
    fn default() -> Self {
        let mut resonance_map = HashMap::new();
        let mut district_visits = HashMap::new();
        for p in PrimeFaction::all() {
            resonance_map.insert(p, ResonanceValue::default());
            district_visits.insert(p, 0);
        }
        Self {
            resonance_map,
            total_playtime: 0.0,
            pulse_count: 0,
            district_visits,
        }
    }
}

impl ResonanceProfile {
    pub fn get_resonance(&self, prime: PrimeFaction) -> ResonanceValue {
        self.resonance_map.get(&prime).copied().unwrap_or_default()
    }

    pub fn set_resonance(&mut self, prime: PrimeFaction, resonance: ResonanceValue) {
        self.resonance_map.insert(prime, resonance);
    }

    pub fn top_primes(&self, n: usize) -> Vec<(PrimeFaction, ResonanceValue)> {
        let mut items: Vec<(PrimeFaction, ResonanceValue)> =
            self.resonance_map.iter().map(|(k, v)| (*k, *v)).collect();
        items.sort_by(|(pa, va), (pb, vb)| {
            vb.value
                .partial_cmp(&va.value)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| pa.rank().cmp(&pb.rank()))
        });
        items.truncate(n);
        items
    }

    pub fn dominant_prime(&self) -> Option<PrimeFaction> {
        let (prime, resonance) = self.top_primes(1).first().copied()?;
        (resonance.value > f32::EPSILON).then_some(prime)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResonanceTracker {
    profile: ResonanceProfile,
}

impl ResonanceTracker {
    pub fn update_from_pulse(&mut self, dt: f32, pulse_definition: &PulseDefinition) {
        self.update_time(dt);
        if let Some(prime) = pulse_definition.metadata.prime_affinity {
            let mut r = self.profile.get_resonance(prime);
            let rate = 0.02_f32;
            r.value = (r.value + dt.max(0.0) * rate).clamp(0.0, 1.0);
            r.activity_score += dt.max(0.0) * (0.3 + r.value);
            r.last_update_time = self.profile.total_playtime;
            self.profile.set_resonance(prime, r);
        }
    }

    pub fn record_district_visit(&mut self, prime: PrimeFaction) {
        let visits = self.profile.district_visits.entry(prime).or_insert(0);
        *visits += 1;

        let mut r = self.profile.get_resonance(prime);
        r.activity_score += 1.0;
        r.value = (r.value + 0.015).clamp(0.0, 1.0);
        r.last_update_time = self.profile.total_playtime;
        self.profile.set_resonance(prime, r);
    }

    pub fn record_pulse_launch(&mut self, prime: PrimeFaction) {
        self.profile.pulse_count = self.profile.pulse_count.saturating_add(1);
        let mut r = self.profile.get_resonance(prime);
        r.activity_score += 2.0;
        r.value = (r.value + 0.03).clamp(0.0, 1.0);
        r.last_update_time = self.profile.total_playtime;
        self.profile.set_resonance(prime, r);
    }

    pub fn update_time(&mut self, dt: f32) {
        self.profile.total_playtime = (self.profile.total_playtime + dt).max(0.0);
    }

    pub fn profile(&self) -> &ResonanceProfile {
        &self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::{PrimeFaction, ResonanceProfile, ResonanceTracker, ResonanceValue};
    use crate::schema::{
        Interactivity, PulseDefinition, PulseKind, PulseMetadata, PulseSceneSource,
    };

    fn pulse_with_affinity(prime: PrimeFaction) -> PulseDefinition {
        PulseDefinition {
            metadata: PulseMetadata {
                title: "x".into(),
                author: "x".into(),
                description: String::new(),
                tags: vec![],
                seed: 1,
                pulse_kind: PulseKind::World,
                duration_hint: None,
                interactivity: Interactivity::Passive,
                prime_affinity: Some(prime),
            },
            pulse_kind: PulseKind::World,
            scene: PulseSceneSource::ScenePath {
                scene_path: "examples/infinite_circuit_megacity.json".into(),
            },
            audio: None,
            timeline: None,
            generators: vec![],
            music: None,
            boot_world: None,
            parameters: Default::default(),
        }
    }

    #[test]
    fn resonance_accumulates_over_time() {
        let mut t = ResonanceTracker::default();
        let p = pulse_with_affinity(PrimeFaction::Electronic);
        t.update_from_pulse(1.0, &p);
        t.update_from_pulse(1.0, &p);
        assert!(t.profile().get_resonance(PrimeFaction::Electronic).value > 0.03);
    }

    #[test]
    fn district_visit_and_pulse_launch_are_recorded() {
        let mut t = ResonanceTracker::default();
        t.record_district_visit(PrimeFaction::Jazz);
        t.record_pulse_launch(PrimeFaction::Jazz);
        assert_eq!(t.profile().district_visits[&PrimeFaction::Jazz], 1);
        assert_eq!(t.profile().pulse_count, 1);
    }

    #[test]
    fn dominant_prime_detected() {
        let mut t = ResonanceTracker::default();
        t.record_pulse_launch(PrimeFaction::Rock);
        t.record_pulse_launch(PrimeFaction::Rock);
        t.record_pulse_launch(PrimeFaction::AmbientExperimental);
        assert_eq!(t.profile().dominant_prime(), Some(PrimeFaction::Rock));
    }

    #[test]
    fn dominant_prime_is_none_for_default_profile() {
        let profile = ResonanceProfile::default();
        assert_eq!(profile.dominant_prime(), None);
    }

    #[test]
    fn dominant_prime_is_reported_when_value_is_positive() {
        let mut profile = ResonanceProfile::default();
        profile.set_resonance(
            PrimeFaction::Jazz,
            ResonanceValue {
                value: 0.1,
                activity_score: 0.5,
                last_update_time: 1.0,
            },
        );
        assert_eq!(profile.dominant_prime(), Some(PrimeFaction::Jazz));
    }

    #[test]
    fn deterministic_for_fixed_dt() {
        let p = pulse_with_affinity(PrimeFaction::Electronic);
        let mut a = ResonanceTracker::default();
        let mut b = ResonanceTracker::default();
        for _ in 0..120 {
            a.update_from_pulse(1.0 / 60.0, &p);
            b.update_from_pulse(1.0 / 60.0, &p);
        }
        assert_eq!(a.profile(), b.profile());
    }
}
