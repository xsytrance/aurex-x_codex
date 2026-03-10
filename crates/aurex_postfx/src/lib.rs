#[derive(Debug, Clone)]
pub struct BloomSettings {
    pub intensity: f32,
    pub low_frequency_boost: f32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            intensity: 0.25,
            low_frequency_boost: 0.7,
        }
    }
}
