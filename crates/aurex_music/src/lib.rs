pub mod instrument;
pub mod pattern;
pub mod rhythm_field;
pub mod sequencer;
pub mod tempo;
pub mod track;

#[cfg(test)]
mod tests {
    use crate::sequencer::{MusicSequencer, default_electronic_sequence};

    #[test]
    fn sequencer_emits_deterministically() {
        let cfg = default_electronic_sequence(42);
        let mut a = MusicSequencer::new(cfg.clone());
        let mut b = MusicSequencer::new(cfg);

        a.update(0.125);
        b.update(0.125);

        assert_eq!(a.emitted_events.len(), b.emitted_events.len());
        assert_eq!(a.rhythm_field.tempo, b.rhythm_field.tempo);
        assert_eq!(a.rhythm_field.beat_phase, b.rhythm_field.beat_phase);
        assert_eq!(a.rhythm_field.beat_strength, b.rhythm_field.beat_strength);
        assert_eq!(a.rhythm_field.beat_index, b.rhythm_field.beat_index);
        assert_eq!(a.rhythm_field.bar_index, b.rhythm_field.bar_index);
        assert_eq!(a.rhythm_field.phrase_index, b.rhythm_field.phrase_index);
        assert_eq!(a.rhythm_field.bass_energy, b.rhythm_field.bass_energy);
        assert_eq!(
            a.rhythm_field.harmonic_energy,
            b.rhythm_field.harmonic_energy
        );
        assert_eq!(a.rhythm_field.spectral_flux, b.rhythm_field.spectral_flux);
        assert_eq!(a.rhythm_field.groove_vector, b.rhythm_field.groove_vector);
    }
}
