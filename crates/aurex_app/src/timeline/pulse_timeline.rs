use super::events::TimelineEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct PulseTimeline {
    pub name: String,
    pub duration_seconds: f32,
    pub events: Vec<TimelineEvent>,
}

impl PulseTimeline {
    pub fn new(
        name: impl Into<String>,
        duration_seconds: f32,
        mut events: Vec<TimelineEvent>,
    ) -> Self {
        events.sort_by(|a, b| {
            a.at_seconds
                .total_cmp(&b.at_seconds)
                .then_with(|| a.priority.cmp(&b.priority))
                .then_with(|| a.id.cmp(&b.id))
        });

        Self {
            name: name.into(),
            duration_seconds: duration_seconds.max(0.0),
            events,
        }
    }
}
