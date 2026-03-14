#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeometryModeOverride {
    Flat,
    #[default]
    Safe,
    Legacy,
}

impl GeometryModeOverride {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "flat" => Some(Self::Flat),
            "safe" => Some(Self::Safe),
            "legacy" => Some(Self::Legacy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDebugFlags {
    pub stop_after_procedural_stage: bool,
    pub stop_after_sdf_stage: bool,
    pub bypass_procedural_setup: bool,
    pub diagnostic_gpu_triangle: bool,
    pub force_flat_render: bool,
    pub disable_audio: bool,
    pub disable_gpu_error_scopes: bool,
    pub log_only_procedural_transition: bool,
    pub skip_root_tree_build: bool,
    pub skip_procedural_camera: bool,
    pub geometry_sdf_mode: GeometryModeOverride,
}

impl Default for RuntimeDebugFlags {
    fn default() -> Self {
        Self {
            stop_after_procedural_stage: false,
            stop_after_sdf_stage: false,
            bypass_procedural_setup: false,
            diagnostic_gpu_triangle: false,
            force_flat_render: false,
            disable_audio: false,
            disable_gpu_error_scopes: false,
            log_only_procedural_transition: false,
            skip_root_tree_build: false,
            skip_procedural_camera: false,
            geometry_sdf_mode: GeometryModeOverride::Safe,
        }
    }
}

impl RuntimeDebugFlags {
    pub fn from_env() -> Self {
        Self {
            stop_after_procedural_stage: env_flag("AUREX_STOP_AFTER_PROCEDURAL_STAGE"),
            stop_after_sdf_stage: env_flag("AUREX_STOP_AFTER_SDF_STAGE"),
            bypass_procedural_setup: env_flag("AUREX_BYPASS_PROCEDURAL_SETUP"),
            diagnostic_gpu_triangle: env_flag("AUREX_DIAGNOSTIC_GPU_TRIANGLE"),
            force_flat_render: env_flag("AUREX_FORCE_FLAT_RENDER"),
            disable_audio: env_flag("AUREX_DISABLE_AUDIO"),
            disable_gpu_error_scopes: env_flag("AUREX_DISABLE_GPU_ERROR_SCOPES"),
            log_only_procedural_transition: env_flag("AUREX_LOG_ONLY_PROCEDURAL_TRANSITION"),
            skip_root_tree_build: env_flag("AUREX_SKIP_ROOT_TREE_BUILD"),
            skip_procedural_camera: env_flag("AUREX_SKIP_PROCEDURAL_CAMERA"),
            geometry_sdf_mode: std::env::var("AUREX_GEOMETRY_SDF_MODE")
                .ok()
                .as_deref()
                .and_then(GeometryModeOverride::parse)
                .unwrap_or(GeometryModeOverride::Safe),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "procedural_stop={} sdf_stop={} bypass_setup={} gpu_triangle={} flat={} disable_audio={} disable_gpu_scopes={} transition_logs_only={} skip_root={} skip_camera={} geometry_mode={:?}",
            self.stop_after_procedural_stage,
            self.stop_after_sdf_stage,
            self.bypass_procedural_setup,
            self.diagnostic_gpu_triangle,
            self.force_flat_render,
            self.disable_audio,
            self.disable_gpu_error_scopes,
            self.log_only_procedural_transition,
            self.skip_root_tree_build,
            self.skip_procedural_camera,
            self.geometry_sdf_mode,
        )
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{GeometryModeOverride, RuntimeDebugFlags};

    #[test]
    fn defaults_are_safe_and_disabled() {
        let flags = RuntimeDebugFlags::default();
        assert_eq!(flags.geometry_sdf_mode, GeometryModeOverride::Safe);
        assert!(!flags.force_flat_render);
    }
}
