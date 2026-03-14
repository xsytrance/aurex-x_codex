#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioAction {
    Play,
    Stop,
    PlayOnce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCue {
    pub cue_id: String,
    pub action: AudioAction,
}

#[derive(Debug, Default, Clone)]
pub struct AudioTransport {
    pub active_tracks: Vec<String>,
    pub fired_cues: Vec<AudioCue>,
}

impl AudioTransport {
    pub fn apply_cue(&mut self, cue: AudioCue) {
        match cue.action {
            AudioAction::Play => {
                if !self.active_tracks.contains(&cue.cue_id) {
                    self.active_tracks.push(cue.cue_id.clone());
                }
            }
            AudioAction::Stop => {
                self.active_tracks.retain(|t| t != &cue.cue_id);
            }
            AudioAction::PlayOnce => {}
        }
        self.fired_cues.push(cue);
    }
}
