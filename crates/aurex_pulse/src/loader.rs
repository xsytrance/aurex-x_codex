use std::path::Path;

use crate::schema::PulseDefinition;

pub fn load_pulse_from_str(contents: &str) -> Result<PulseDefinition, Box<dyn std::error::Error>> {
    let pulse: PulseDefinition = serde_json::from_str(contents)?;
    pulse
        .validate()
        .map_err(|e| format!("pulse validation failed: {e}"))?;
    Ok(pulse)
}

pub fn load_pulse_from_path(
    path: impl AsRef<Path>,
) -> Result<PulseDefinition, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    load_pulse_from_str(&data)
}
