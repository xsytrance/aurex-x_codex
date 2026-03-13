use std::collections::BTreeMap;

use crate::V3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub t: i32,
    pub seed: u32,
}

impl SampleKey {
    pub(crate) fn from_world(p: V3, t: f32, seed: u32) -> Self {
        Self {
            x: (p.x * 128.0) as i32,
            y: (p.y * 128.0) as i32,
            z: (p.z * 128.0) as i32,
            t: (t * 120.0) as i32,
            seed,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PatternSampleCache {
    store: BTreeMap<SampleKey, f32>,
    pub hits: u64,
    pub misses: u64,
}

impl PatternSampleCache {
    pub fn get(&mut self, key: SampleKey) -> Option<f32> {
        if let Some(v) = self.store.get(&key).copied() {
            self.hits += 1;
            Some(v)
        } else {
            self.misses += 1;
            None
        }
    }
    pub fn insert(&mut self, key: SampleKey, value: f32) {
        self.store.insert(key, value);
    }
    pub fn clear(&mut self) {
        self.store.clear();
    }
}

#[derive(Debug, Clone, Default)]
pub struct FieldSampleCache {
    store: BTreeMap<SampleKey, [f32; 5]>,
    pub hits: u64,
    pub misses: u64,
}

impl FieldSampleCache {
    pub fn get(&mut self, key: SampleKey) -> Option<[f32; 5]> {
        if let Some(v) = self.store.get(&key).copied() {
            self.hits += 1;
            Some(v)
        } else {
            self.misses += 1;
            None
        }
    }
    pub fn insert(&mut self, key: SampleKey, value: [f32; 5]) {
        self.store.insert(key, value);
    }
}

#[derive(Debug, Clone, Default)]
pub struct EffectGraphEvalCache {
    pub last_scene_seed: Option<u32>,
    pub last_time_tick: Option<i32>,
    pub eval_count: u64,
}

impl EffectGraphEvalCache {
    pub fn should_reuse(&self, scene_seed: u32, time_seconds: f32) -> bool {
        self.last_scene_seed == Some(scene_seed)
            && self.last_time_tick == Some((time_seconds * 120.0) as i32)
    }
    pub fn mark_evaluated(&mut self, scene_seed: u32, time_seconds: f32) {
        self.last_scene_seed = Some(scene_seed);
        self.last_time_tick = Some((time_seconds * 120.0) as i32);
        self.eval_count += 1;
    }
}

#[derive(Debug, Clone, Default)]
pub struct SceneBoundsCache {
    pub entries: BTreeMap<u32, f32>,
}

impl SceneBoundsCache {
    pub fn get_or_insert_with<F: FnOnce() -> f32>(&mut self, seed: u32, f: F) -> f32 {
        if let Some(v) = self.entries.get(&seed).copied() {
            v
        } else {
            let v = f();
            self.entries.insert(seed, v);
            v
        }
    }
}
