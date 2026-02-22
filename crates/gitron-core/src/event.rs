use crate::git::types::{StatusChangedPayload, RefsChangedPayload};

/// Trait for emitting real-time events to connected clients.
/// Implementations can use Tauri events, SSE broadcasting, etc.
pub trait EventEmitter: Send + Sync + 'static {
    fn emit_status_changed(&self, payload: &StatusChangedPayload);
    fn emit_refs_changed(&self, payload: &RefsChangedPayload);
}
