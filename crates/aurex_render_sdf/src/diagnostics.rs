use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CacheStats {
    pub pattern_hits: u64,
    pub pattern_misses: u64,
    pub field_hits: u64,
    pub field_misses: u64,
    pub effect_graph_evals: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderStats {
    pub raymarch_steps_total: u64,
    pub rays_traced: u64,
    pub stage_time_ms_total: f64,
    pub cache: CacheStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameDiagnostics {
    pub stages: Vec<&'static str>,
    pub stats: RenderStats,
    pub stage_durations_ms: BTreeMap<&'static str, f64>,
    pub stage_percentages: BTreeMap<&'static str, f64>,
    pub total_frame_time_ms: f64,
}

impl FrameDiagnostics {
    pub fn add_stage_duration(&mut self, stage: &'static str, millis: f64) {
        *self.stage_durations_ms.entry(stage).or_insert(0.0) += millis;
    }

    pub fn finalize_stage_percentages(&mut self) {
        self.stage_percentages.clear();
        if self.total_frame_time_ms <= f64::EPSILON {
            return;
        }
        for (stage, duration) in &self.stage_durations_ms {
            self.stage_percentages
                .insert(*stage, (duration / self.total_frame_time_ms) * 100.0);
        }
    }
}
