use std::convert::Infallible;

use axum::response::sse::{Event, KeepAlive, Sse};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use gitron_core::event::EventEmitter;
use gitron_core::git::types::{RefsChangedPayload, StatusChangedPayload};

/// SSE broadcaster that implements EventEmitter.
/// Broadcasts repo change events to all connected SSE clients.
pub struct SseBroadcaster {
    tx: broadcast::Sender<SseEvent>,
}

#[derive(Clone, Debug)]
pub enum SseEvent {
    StatusChanged(String),   // JSON payload
    RefsChanged(String),     // JSON payload
}

impl SseBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self { tx }
    }

    pub fn subscribe(&self) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
        let rx = self.tx.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|result| {
            match result {
                Ok(event) => {
                    let sse_event = match event {
                        SseEvent::StatusChanged(data) => {
                            Event::default().event("repo:status-changed").data(data)
                        }
                        SseEvent::RefsChanged(data) => {
                            Event::default().event("repo:refs-changed").data(data)
                        }
                    };
                    Some(Ok(sse_event))
                }
                Err(_) => None,
            }
        });

        Sse::new(stream).keep_alive(KeepAlive::default())
    }
}

impl EventEmitter for SseBroadcaster {
    fn emit_status_changed(&self, payload: &StatusChangedPayload) {
        if let Ok(json) = serde_json::to_string(payload) {
            self.tx.send(SseEvent::StatusChanged(json)).ok();
        }
    }

    fn emit_refs_changed(&self, payload: &RefsChangedPayload) {
        if let Ok(json) = serde_json::to_string(payload) {
            self.tx.send(SseEvent::RefsChanged(json)).ok();
        }
    }
}
