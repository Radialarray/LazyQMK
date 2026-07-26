//! Server-Sent Events endpoint for hot-reload layout changes.
//!
//! Subscribes to the [`crate::web::app_state::AppState::layout_events`]
//! broadcast channel and streams each event as an SSE `data:` line.
//! Sends a periodic `:keepalive` ping so intermediate proxies don't
//! drop the connection.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::web::app_state::AppState;

/// How often the keepalive ping is sent to keep the SSE connection open.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);

/// `GET /api/events` — Server-Sent Events stream of layout file changes.
///
/// Each connected client gets its own broadcast subscription. When a
/// `.json` file in the workspace changes (because of an external
/// write, an agent CLI, or another TUI session) a `LayoutEvent` is
/// forwarded to every subscriber as a JSON `data:` line.
///
/// The stream also emits a keepalive ping every 15 seconds so that
/// corporate proxies and load balancers don't time out an idle
/// connection.
///
/// # Cancellation
///
/// If the broadcast channel is closed (server shutting down) the
/// stream terminates cleanly. The client will reconnect automatically
/// (browsers' `EventSource` does this by default).
pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.layout_events.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Some(Ok(Event::default().event("layout").data(json)))
        }
        // Lagged means the receiver fell behind and some messages
        // were dropped; skip and wait for the next real event. If
        // the broadcast channel closes the stream will naturally
        // terminate.
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(KEEPALIVE_INTERVAL))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::app_state::AppState;
    use crate::web::events::LayoutEvent;

    fn make_state() -> AppState {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let config = crate::config::Config::default();
        AppState::new(config, tmp.path().to_path_buf()).expect("state")
    }

    #[tokio::test]
    async fn test_subscribe_receives_published_event() {
        let state = make_state();
        let mut rx = state.layout_events.subscribe();
        state
            .layout_events
            .send(LayoutEvent::Changed {
                filename: "foo.json".to_string(),
                mtime: 42,
            })
            .expect("send");

        let event = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("no timeout")
            .expect("event");

        match event {
            LayoutEvent::Changed { filename, mtime } => {
                assert_eq!(filename, "foo.json");
                assert_eq!(mtime, 42);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn test_event_serializes_with_kind_tag() {
        let event = LayoutEvent::Removed {
            filename: "bar.json".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"kind\":\"removed\""));
        assert!(json.contains("\"filename\":\"bar.json\""));
    }
}
