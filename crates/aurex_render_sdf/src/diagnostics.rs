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
    pub cache: CacheStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameDiagnostics {
    pub stages: Vec<&'static str>,
    pub stats: RenderStats,
}
