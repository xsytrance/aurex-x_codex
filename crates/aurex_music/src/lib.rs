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
        assert_eq!(a.rhythm_field.beat_phase, b.rhythm_field.beat_phase);
        assert_eq!(a.rhythm_field.beat_strength, b.rhythm_field.beat_strength);
    }
}
