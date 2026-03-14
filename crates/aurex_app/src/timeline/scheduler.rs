use std::collections::HashSet;

use super::{events::TimelineEvent, pulse_timeline::PulseTimeline};

#[derive(Debug, Default, Clone)]
pub struct EventScheduler {
    fired: HashSet<u64>,
}

impl EventScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_due_events<'a>(
        &mut self,
        timeline: &'a PulseTimeline,
        time_seconds: f32,
    ) -> Vec<&'a TimelineEvent> {
        let mut due = Vec::new();
        for event in &timeline.events {
            if event.at_seconds <= time_seconds && !self.fired.contains(&event.id) {
                self.fired.insert(event.id);
                due.push(event);
            }
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{PulseTimeline, TimelineEvent, TimelineEventKind};

    #[test]
    fn triggers_once_when_time_reaches_event() {
        let timeline = PulseTimeline::new(
            "t",
            5.0,
            vec![TimelineEvent {
                id: 1,
                at_seconds: 1.0,
                priority: 0,
                kind: TimelineEventKind::Trigger {
                    key: "x".to_string(),
                },
            }],
        );

        let mut scheduler = EventScheduler::new();
        assert_eq!(scheduler.collect_due_events(&timeline, 0.9).len(), 0);
        assert_eq!(scheduler.collect_due_events(&timeline, 1.0).len(), 1);
        assert_eq!(scheduler.collect_due_events(&timeline, 2.0).len(), 0);
    }
}
