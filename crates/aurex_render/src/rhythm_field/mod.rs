pub mod modulation;
pub mod signals;
pub mod snapshot;

pub use modulation::{
    AtmosphereLayerParams, CameraLayerHints, GeneratorStackOutput, LightingLayerParams,
    ParticleLayerParams, StructureLayerParams, TerrainLayerParams, VisualTheme,
    apply_rhythm_modulation,
};
pub use signals::{SequencerState, sample_rhythm_field};
pub use snapshot::RhythmFieldSnapshot;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_time_produce_identical_snapshot() {
        let sequencer = SequencerState {
            bpm: 128.0,
            beat_index: 8,
            bar_index: 2,
            bass_energy: 0.66,
            mid_energy: 0.41,
            high_energy: 0.29,
        };

        let a = sample_rhythm_field(42, 4.0, sequencer);
        let b = sample_rhythm_field(42, 4.0, sequencer);

        assert_eq!(a, b);
    }

    #[test]
    fn snapshot_evolves_with_time() {
        let sequencer = SequencerState::default();

        let a = sample_rhythm_field(42, 1.0, sequencer);
        let b = sample_rhythm_field(42, 1.125, sequencer);

        assert_ne!(a.beat_phase, b.beat_phase);
        assert_ne!(a.pulse, b.pulse);
    }

    #[test]
    fn modulation_produces_bounded_predictable_deltas() {
        let base = GeneratorStackOutput::default();
        let snapshot = RhythmFieldSnapshot {
            beat_phase: 0.5,
            bar_phase: 0.75,
            pulse: 0.8,
            bass_energy: 0.7,
            mid_energy: 0.55,
            high_energy: 0.65,
            intensity: 0.72,
            accent: 0.2,
        };

        let modulated = apply_rhythm_modulation(&snapshot, &base, VisualTheme::Electronic);

        assert!(modulated.terrain.amplitude_hint > base.terrain.amplitude_hint);
        assert!(modulated.lighting.flash_envelope > base.lighting.flash_envelope);
        assert!(modulated.particles.density_multiplier > base.particles.density_multiplier);
        assert!(modulated.camera_hints.drift >= base.camera_hints.drift);

        for value in [
            modulated.terrain.amplitude_hint,
            modulated.terrain.roughness_hint,
            modulated.structures.density,
            modulated.structures.emissive,
            modulated.structures.structure_scale,
            modulated.structures.structure_height,
            modulated.structures.structure_complexity,
            modulated.atmosphere.hue_drift,
            modulated.atmosphere.fog_density,
            modulated.lighting.flash_envelope,
            modulated.lighting.exposure,
            modulated.particles.density_multiplier,
            modulated.particles.brightness,
            modulated.camera_hints.drift,
            modulated.camera_hints.fov_bias,
        ] {
            assert!((0.0..=1.0).contains(&value));
        }
    }
}
