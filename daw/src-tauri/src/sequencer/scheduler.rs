//! Event scheduling for sequencer
//!
//! Grown-up Script: Manages priority queue of MIDI events for precise playback timing.

use crate::core::midi::types::MidiMessage;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A scheduled MIDI event with timing information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduledEvent {
    pub message: MidiMessage,
    pub tick: u64,
    pub track_id: i32,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.tick == other.tick
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other.tick.cmp(&self.tick)
    }
}

/// Manages scheduling and retrieval of MIDI events
///
/// Uses a priority queue (min-heap) to efficiently retrieve events in chronological order.
/// Thread-safe using Mutex for concurrent access.
pub struct EventScheduler {
    events: Arc<Mutex<BinaryHeap<ScheduledEvent>>>,
}

impl EventScheduler {
    /// Create a new empty event scheduler
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    /// Schedule a MIDI event at a specific tick
    ///
    /// # Arguments
    /// * `message` - The MIDI message to schedule
    /// * `tick` - Absolute tick position when event should fire
    /// * `track_id` - ID of the track this event belongs to
    pub async fn schedule(&self, message: MidiMessage, tick: u64, track_id: i32) {
        let event = ScheduledEvent {
            message,
            tick,
            track_id,
        };

        let mut events = self.events.lock().await;
        events.push(event);
    }

    /// Schedule multiple events at once
    ///
    /// More efficient than calling schedule() repeatedly.
    pub async fn schedule_many(&self, events: Vec<ScheduledEvent>) {
        let mut heap = self.events.lock().await;
        for event in events {
            heap.push(event);
        }
    }

    /// Get the next event at or before the current tick
    ///
    /// Returns None if no events are ready or queue is empty.
    ///
    /// # Arguments
    /// * `current_tick` - Current playback position
    pub async fn pop_next(&self, current_tick: u64) -> Option<ScheduledEvent> {
        let mut events = self.events.lock().await;

        // Peek at next event
        if let Some(next) = events.peek() {
            if next.tick <= current_tick {
                return events.pop();
            }
        }

        None
    }

    /// Get all events at or before the current tick
    ///
    /// Returns a vector of events in chronological order.
    /// More efficient than calling pop_next() repeatedly.
    ///
    /// # Arguments
    /// * `current_tick` - Current playback position
    pub async fn pop_ready(&self, current_tick: u64) -> Vec<ScheduledEvent> {
        let mut events = self.events.lock().await;
        let mut ready = Vec::new();

        while let Some(next) = events.peek() {
            if next.tick <= current_tick {
                // Safe to unwrap here since peek() returned Some, but we use if let for safety
                if let Some(event) = events.pop() {
                    ready.push(event);
                }
            } else {
                break;
            }
        }

        ready
    }

    /// Peek at the next event without removing it
    pub async fn peek_next(&self) -> Option<u64> {
        let events = self.events.lock().await;
        events.peek().map(|e| e.tick)
    }

    /// Get the number of scheduled events
    pub async fn len(&self) -> usize {
        let events = self.events.lock().await;
        events.len()
    }

    /// Check if scheduler is empty
    pub async fn is_empty(&self) -> bool {
        let events = self.events.lock().await;
        events.is_empty()
    }

    /// Clear all scheduled events
    pub async fn clear(&self) {
        let mut events = self.events.lock().await;
        events.clear();
    }

    /// Remove all events for a specific track
    pub async fn clear_track(&self, track_id: i32) {
        let mut events = self.events.lock().await;
        let filtered: Vec<_> = events
            .drain()
            .filter(|e| e.track_id != track_id)
            .collect();

        events.clear();
        for event in filtered {
            events.push(event);
        }
    }
}

impl Default for EventScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::MidiEventType;

    fn create_test_message(note: u8, velocity: u8) -> MidiMessage {
        MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel: 0,
            data1: note,
            data2: velocity,
            timestamp: 0,
        }
    }

    #[tokio::test]
    async fn test_schedule_and_pop() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;

        let event = scheduler.pop_next(100).await.unwrap();
        assert_eq!(event.tick, 100);
        assert_eq!(event.message.data1, 60);
    }

    #[tokio::test]
    async fn test_pop_before_ready() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;

        // Try to pop at tick 50 (before event is ready)
        let event = scheduler.pop_next(50).await;
        assert!(event.is_none());

        // Event should still be there at tick 100
        let event = scheduler.pop_next(100).await.unwrap();
        assert_eq!(event.tick, 100);
    }

    #[tokio::test]
    async fn test_chronological_order() {
        let scheduler = EventScheduler::new();

        // Schedule events out of order
        scheduler
            .schedule(create_test_message(64, 100), 300, 1)
            .await;
        scheduler
            .schedule(create_test_message(62, 100), 200, 1)
            .await;
        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;

        // Should come out in chronological order
        let e1 = scheduler.pop_next(500).await.unwrap();
        assert_eq!(e1.tick, 100);
        assert_eq!(e1.message.data1, 60);

        let e2 = scheduler.pop_next(500).await.unwrap();
        assert_eq!(e2.tick, 200);
        assert_eq!(e2.message.data1, 62);

        let e3 = scheduler.pop_next(500).await.unwrap();
        assert_eq!(e3.tick, 300);
        assert_eq!(e3.message.data1, 64);
    }

    #[tokio::test]
    async fn test_pop_ready() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;
        scheduler
            .schedule(create_test_message(62, 100), 200, 1)
            .await;
        scheduler
            .schedule(create_test_message(64, 100), 300, 1)
            .await;

        // Get all events up to tick 250
        let ready = scheduler.pop_ready(250).await;
        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0].tick, 100);
        assert_eq!(ready[1].tick, 200);

        // One event should remain
        assert_eq!(scheduler.len().await, 1);
    }

    #[tokio::test]
    async fn test_peek_next() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;
        scheduler
            .schedule(create_test_message(62, 100), 200, 1)
            .await;

        let next_tick = scheduler.peek_next().await.unwrap();
        assert_eq!(next_tick, 100);

        // Peek doesn't remove
        assert_eq!(scheduler.len().await, 2);
    }

    #[tokio::test]
    async fn test_clear_track() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;
        scheduler
            .schedule(create_test_message(62, 100), 200, 2)
            .await;
        scheduler
            .schedule(create_test_message(64, 100), 300, 1)
            .await;

        scheduler.clear_track(1).await;

        // Only track 2 event should remain
        assert_eq!(scheduler.len().await, 1);
        let event = scheduler.pop_next(500).await.unwrap();
        assert_eq!(event.track_id, 2);
    }

    #[tokio::test]
    async fn test_schedule_many() {
        let scheduler = EventScheduler::new();

        let events = vec![
            ScheduledEvent {
                message: create_test_message(60, 100),
                tick: 100,
                track_id: 1,
            },
            ScheduledEvent {
                message: create_test_message(62, 100),
                tick: 200,
                track_id: 1,
            },
            ScheduledEvent {
                message: create_test_message(64, 100),
                tick: 300,
                track_id: 1,
            },
        ];

        scheduler.schedule_many(events).await;

        assert_eq!(scheduler.len().await, 3);
    }

    #[tokio::test]
    async fn test_clear() {
        let scheduler = EventScheduler::new();

        scheduler
            .schedule(create_test_message(60, 100), 100, 1)
            .await;
        scheduler
            .schedule(create_test_message(62, 100), 200, 1)
            .await;

        scheduler.clear().await;

        assert!(scheduler.is_empty().await);
    }
}
