use aurex_core::Tick;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackendMode {
    MockSilence,
    Cpal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioTransition {
    Noop,
    Transitioned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioEngineStatus {
    pub mode: AudioBackendMode,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBackendReadiness {
    pub has_device_io: bool,
    pub has_stream_graph: bool,
    pub can_emit_sound: bool,
}

impl AudioBackendReadiness {
    pub fn for_mode(mode: AudioBackendMode) -> Self {
        match mode {
            AudioBackendMode::MockSilence => Self {
                has_device_io: false,
                has_stream_graph: false,
                can_emit_sound: false,
            },
            AudioBackendMode::Cpal => Self {
                has_device_io: true,
                has_stream_graph: true,
                can_emit_sound: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioClock {
    pub tick: Tick,
}

impl Default for AudioClock {
    fn default() -> Self {
        Self { tick: Tick(0) }
    }
}

impl AudioClock {
    pub fn advance(&mut self) {
        self.tick.0 += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatEvent {
    pub tick: Tick,
    pub pulse: f32,
}

#[derive(Debug, Clone)]
pub struct MockAudioEngine {
    mode: AudioBackendMode,
    ready: bool,
    clock: AudioClock,
}

impl Default for MockAudioEngine {
    fn default() -> Self {
        Self {
            mode: AudioBackendMode::MockSilence,
            ready: true,
            clock: AudioClock::default(),
        }
    }
}

impl MockAudioEngine {
    pub fn status(&self) -> AudioEngineStatus {
        AudioEngineStatus {
            mode: self.mode,
            ready: self.ready,
        }
    }

    pub fn transition_mode(&mut self, mode: AudioBackendMode) -> AudioTransition {
        if self.mode == mode {
            return AudioTransition::Noop;
        }

        self.mode = mode;
        self.ready = mode == AudioBackendMode::MockSilence;
        AudioTransition::Transitioned
    }

    pub fn next_beat(&mut self) -> BeatEvent {
        self.clock.advance();
        let phase = (self.clock.tick.0 % 8) as f32 / 8.0;
        let pulse = (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        BeatEvent {
            tick: self.clock.tick,
            pulse,
        }
    }
}

#[cfg(feature = "real_audio")]
mod real_audio {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    pub struct CpalAudioEngine {
        stream: cpal::Stream,
    }

    impl CpalAudioEngine {
        pub fn start() -> Result<Self, String> {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or_else(|| "no default output audio device found".to_string())?;
            let config = device
                .default_output_config()
                .map_err(|e| format!("failed to query default output config: {e}"))?;

            let sample_rate = config.sample_rate().0 as f32;
            let channels = config.channels() as usize;
            let mut phase = 0.0_f32;
            let phase_step = 440.0_f32 * std::f32::consts::TAU / sample_rate;

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device
                    .build_output_stream(
                        &config.clone().into(),
                        move |data: &mut [f32], _| {
                            write_sine_f32(data, channels, &mut phase, phase_step)
                        },
                        move |err| eprintln!("cpal output stream error: {err}"),
                        None,
                    )
                    .map_err(|e| format!("failed to build f32 output stream: {e}"))?,
                cpal::SampleFormat::I16 => device
                    .build_output_stream(
                        &config.clone().into(),
                        move |data: &mut [i16], _| {
                            write_sine_i16(data, channels, &mut phase, phase_step)
                        },
                        move |err| eprintln!("cpal output stream error: {err}"),
                        None,
                    )
                    .map_err(|e| format!("failed to build i16 output stream: {e}"))?,
                cpal::SampleFormat::U16 => device
                    .build_output_stream(
                        &config.into(),
                        move |data: &mut [u16], _| {
                            write_sine_u16(data, channels, &mut phase, phase_step)
                        },
                        move |err| eprintln!("cpal output stream error: {err}"),
                        None,
                    )
                    .map_err(|e| format!("failed to build u16 output stream: {e}"))?,
                other => {
                    return Err(format!("unsupported output sample format: {other:?}"));
                }
            };

            stream
                .play()
                .map_err(|e| format!("failed to start output stream: {e}"))?;

            Ok(Self { stream })
        }

        pub fn stream(&self) -> &cpal::Stream {
            &self.stream
        }
    }

    fn write_sine_f32(data: &mut [f32], channels: usize, phase: &mut f32, phase_step: f32) {
        for frame in data.chunks_mut(channels) {
            let sample = (*phase).sin() * 0.2;
            *phase = (*phase + phase_step) % std::f32::consts::TAU;
            for out in frame {
                *out = sample;
            }
        }
    }

    fn write_sine_i16(data: &mut [i16], channels: usize, phase: &mut f32, phase_step: f32) {
        for frame in data.chunks_mut(channels) {
            let sample = ((*phase).sin() * i16::MAX as f32 * 0.2) as i16;
            *phase = (*phase + phase_step) % std::f32::consts::TAU;
            for out in frame {
                *out = sample;
            }
        }
    }

    fn write_sine_u16(data: &mut [u16], channels: usize, phase: &mut f32, phase_step: f32) {
        for frame in data.chunks_mut(channels) {
            let centered = (*phase).sin() * 0.5 + 0.5;
            let sample = (centered * u16::MAX as f32 * 0.2) as u16;
            *phase = (*phase + phase_step) % std::f32::consts::TAU;
            for out in frame {
                *out = sample;
            }
        }
    }

    pub use CpalAudioEngine as PublicCpalAudioEngine;
}

#[cfg(feature = "real_audio")]
pub use real_audio::PublicCpalAudioEngine as CpalAudioEngine;

#[cfg(test)]
mod tests {
    use super::{AudioBackendMode, AudioBackendReadiness, AudioTransition, MockAudioEngine};

    #[test]
    fn readiness_contract_tracks_audio_mode() {
        let silent = AudioBackendReadiness::for_mode(AudioBackendMode::MockSilence);
        assert!(!silent.has_device_io);
        assert!(!silent.has_stream_graph);
        assert!(!silent.can_emit_sound);

        let planned = AudioBackendReadiness::for_mode(AudioBackendMode::Cpal);
        assert!(planned.has_device_io);
        assert!(planned.has_stream_graph);
        assert!(planned.can_emit_sound);
    }

    #[test]
    fn transition_marks_cpal_not_ready() {
        let mut engine = MockAudioEngine::default();
        let transitioned = engine.transition_mode(AudioBackendMode::Cpal);

        assert_eq!(transitioned, AudioTransition::Transitioned);
        let status = engine.status();
        assert_eq!(status.mode, AudioBackendMode::Cpal);
        assert!(!status.ready);
    }

    #[test]
    fn beat_sequence_progresses_tick_and_range() {
        let mut engine = MockAudioEngine::default();
        let first = engine.next_beat();
        let second = engine.next_beat();

        assert_eq!(first.tick.0, 1);
        assert_eq!(second.tick.0, 2);
        assert!((0.0..=1.0).contains(&first.pulse));
        assert!((0.0..=1.0).contains(&second.pulse));
    }
}
