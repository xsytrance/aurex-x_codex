use crate::{MidiNote, analysis::midi_analysis::PitchRange};

pub fn compute_pitch_range(notes: &[MidiNote]) -> PitchRange {
    if notes.is_empty() {
        return PitchRange { min: 0, max: 0 };
    }

    let mut min = u8::MAX;
    let mut max = u8::MIN;

    for note in notes {
        min = min.min(note.pitch);
        max = max.max(note.pitch);
    }

    PitchRange { min, max }
}
