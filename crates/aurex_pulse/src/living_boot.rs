use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resonance::{PrimeFaction, ResonanceProfile};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LivingBootPresentation {
    pub dominant_prime: Option<PrimeFaction>,
    pub top_three_primes: Vec<PrimeFaction>,
    pub visual_bias_weights: HashMap<PrimeFaction, f32>,
    pub audio_bias_weights: HashMap<PrimeFaction, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdleResonanceEventState {
    pub idle_time_seconds: f32,
    pub warning_issued: bool,
    pub event_count: u32,
    pub last_event_time: Option<f32>,
    pub resonance_event_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LivingBootState {
    pub presentation: LivingBootPresentation,
    pub idle_state: IdleResonanceEventState,
}

impl LivingBootState {
    pub fn from_profile(profile: &ResonanceProfile) -> Self {
        Self {
            presentation: derive_presentation(profile),
            idle_state: IdleResonanceEventState {
                idle_time_seconds: 0.0,
                warning_issued: false,
                event_count: 0,
                last_event_time: None,
                resonance_event_ready: false,
            },
        }
    }

    pub fn refresh_presentation(&mut self, profile: &ResonanceProfile) {
        self.presentation = derive_presentation(profile);
    }

    pub fn update_idle(&mut self, dt: f32, meaningful_interaction: bool, runtime_seconds: f32) {
        if meaningful_interaction {
            self.idle_state.idle_time_seconds = 0.0;
            self.idle_state.resonance_event_ready = false;
            return;
        }

        self.idle_state.idle_time_seconds += dt.max(0.0);

        const FIRST_WARNING_THRESHOLD: f32 = 20.0;
        const EVENT_THRESHOLD_STEP: f32 = 15.0;

        if !self.idle_state.warning_issued
            && self.idle_state.idle_time_seconds >= FIRST_WARNING_THRESHOLD
        {
            self.idle_state.warning_issued = true;
            self.idle_state.last_event_time = Some(runtime_seconds);
            self.idle_state.resonance_event_ready = false;
            return;
        }

        if self.idle_state.warning_issued {
            let next_event_threshold = FIRST_WARNING_THRESHOLD
                + (self.idle_state.event_count + 1) as f32 * EVENT_THRESHOLD_STEP;
            if self.idle_state.idle_time_seconds >= next_event_threshold {
                self.idle_state.event_count = self.idle_state.event_count.saturating_add(1);
                self.idle_state.last_event_time = Some(runtime_seconds);
                self.idle_state.resonance_event_ready = true;
            }
        }
    }
}

fn derive_presentation(profile: &ResonanceProfile) -> LivingBootPresentation {
    let dominant_prime = profile.dominant_prime();
    let top = profile.top_primes(3);
    let top_three_primes = top.iter().map(|(p, _)| *p).collect::<Vec<_>>();

    let mut visual_bias_weights = HashMap::new();
    let mut audio_bias_weights = HashMap::new();

    for p in PrimeFaction::all() {
        let r = profile.get_resonance(p).value;
        visual_bias_weights.insert(p, r);
        audio_bias_weights.insert(p, (r * 0.8 + 0.2 * r.powf(0.5)).clamp(0.0, 1.0));
    }

    LivingBootPresentation {
        dominant_prime,
        top_three_primes,
        visual_bias_weights,
        audio_bias_weights,
    }
}

#[cfg(test)]
mod tests {
    use super::LivingBootState;
    use crate::resonance::{PrimeFaction, ResonanceProfile, ResonanceTracker};

    #[test]
    fn presentation_derives_from_resonance_profile() {
        let mut tracker = ResonanceTracker::default();
        tracker.record_pulse_launch(PrimeFaction::Electronic);
        tracker.record_pulse_launch(PrimeFaction::Electronic);
        tracker.record_pulse_launch(PrimeFaction::Jazz);
        let state = LivingBootState::from_profile(tracker.profile());
        assert_eq!(
            state.presentation.dominant_prime,
            Some(PrimeFaction::Electronic)
        );
        assert_eq!(
            state.presentation.top_three_primes[0],
            PrimeFaction::Electronic
        );
    }

    #[test]
    fn first_idle_threshold_is_warning_only() {
        let profile = ResonanceProfile::default();
        let mut state = LivingBootState::from_profile(&profile);
        state.update_idle(21.0, false, 21.0);
        assert!(state.idle_state.warning_issued);
        assert_eq!(state.idle_state.event_count, 0);
        assert!(!state.idle_state.resonance_event_ready);
    }

    #[test]
    fn later_idle_threshold_sets_event_ready() {
        let profile = ResonanceProfile::default();
        let mut state = LivingBootState::from_profile(&profile);
        state.update_idle(21.0, false, 21.0);
        state.update_idle(15.0, false, 36.0);
        assert!(state.idle_state.warning_issued);
        assert_eq!(state.idle_state.event_count, 1);
        assert!(state.idle_state.resonance_event_ready);
    }

    #[test]
    fn idle_updates_are_deterministic_for_fixed_dt() {
        let profile = ResonanceProfile::default();
        let mut a = LivingBootState::from_profile(&profile);
        let mut b = LivingBootState::from_profile(&profile);
        for i in 0..400 {
            let t = i as f32 * 0.25;
            a.update_idle(0.25, false, t);
            b.update_idle(0.25, false, t);
        }
        assert_eq!(a, b);
    }
}
