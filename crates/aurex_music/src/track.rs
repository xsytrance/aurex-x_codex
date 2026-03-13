use serde::{Deserialize, Serialize};

use crate::{instrument::InstrumentKind, pattern::Pattern};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub name: String,
    pub instrument: InstrumentKind,
    pub pattern: Pattern,
    #[serde(default = "default_volume")]
    pub volume: f32,
}

fn default_volume() -> f32 {
    1.0
}
