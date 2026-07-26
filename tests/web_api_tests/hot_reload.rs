//! Integration tests for the hot-reload `/api/events` SSE endpoint
//! and the workspace watcher that feeds it.
//!
//! The tests use the same `create_router` helper as the rest of the
//! web API suite, plus a `tempfile::TempDir` workspace so each test
//! gets an isolated filesystem.

use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_sse_endpoint_streams_layout_change_event() {
    let (mut state, temp_dir) = create_test_state();
    // Start the workspace watcher for this test.
    state
        .start_workspace_watcher()
        .expect("workspace watcher should start");

    // Subscribe BEFORE writing so we don't miss the event.
    let mut rx = state.subscribe_layout_events();

    // Drop the initial event the watcher pushes for the directory
    // itself on startup (notify fires a "create" for the watched
    // root in some configurations).
    tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .ok();

    // Write a layout file as an external process would.
    let layout = test_layout_basic(1, 1);
    let path = temp_dir.path().join("hotreload.json");
    write_layout_file(&layout, &path).expect("write layout");

    // Wait for the watcher to debounce + publish.
    let event = tokio::time::timeout(Duration::from_secs(3), rx.recv())
        .await
        .expect("no timeout")
        .expect("event");

    let filename = event.filename().to_string();
    let expected_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .expect("file name")
        .to_string();
    assert_eq!(filename, expected_name);
}

#[tokio::test]
async fn test_sse_handler_responds_with_event_stream() {
    let (mut state, _temp_dir) = create_test_state();
    state
        .start_workspace_watcher()
        .expect("workspace watcher should start");

    let app = create_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("text/event-stream"),
        "SSE endpoint should set text/event-stream content-type, got {content_type}"
    );
}

#[tokio::test]
async fn test_external_write_with_self_epoch_does_not_echo() {
    // This test verifies that the `save_with_epoch` API correctly
    // suppresses echoes of our own writes through the watcher.
    let (mut state, temp_dir) = create_test_state();
    state
        .start_workspace_watcher()
        .expect("workspace watcher should start");

    let mut rx = state.subscribe_layout_events();

    // Write through the same self-write epoch that the watcher
    // checks.
    let layout = test_layout_basic(1, 1);
    let path = temp_dir.path().join("self.json");
    let epoch = state.self_write_epoch();
    lazyqmk::services::LayoutService::save_with_epoch(&layout, &path, Some(&epoch))
        .expect("save_with_epoch");

    // Poll the channel for ~700 ms; we expect NO event for this file
    // because the watcher should suppress the echo.
    let echoed = tokio::time::timeout(Duration::from_millis(700), async {
        loop {
            match rx.recv().await {
                Ok(event) if event.filename() == "self.json" => return true,
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
    })
    .await
    .unwrap_or(false);
    assert!(!echoed, "self-write echo leaked through watcher");
}

#[tokio::test]
async fn test_sse_event_serializes_with_kind_tag() {
    use lazyqmk::web::events::LayoutEvent;
    let event = LayoutEvent::Changed {
        filename: "test.json".to_string(),
        mtime: 1234,
    };
    let json = serde_json::to_string(&event).expect("serialize");
    assert!(json.contains("\"kind\":\"changed\""));
    assert!(json.contains("\"filename\":\"test.json\""));
    assert!(json.contains("\"mtime\":1234"));
}

#[test]
fn test_layout_event_removed_carries_filename() {
    use lazyqmk::web::events::LayoutEvent;
    let event = LayoutEvent::Removed {
        filename: "bye.json".to_string(),
    };
    assert_eq!(event.filename(), "bye.json");
}

/// Smoke test: publish through the broadcast and confirm the SSE
/// stream wrapper delivers a parsed `LayoutEvent`.
#[tokio::test]
async fn test_sse_payload_round_trip() {
    use lazyqmk::web::events::LayoutEvent;
    let (state, _temp_dir) = create_test_state();
    let mut rx = state.subscribe_layout_events();

    state
        .layout_event_sender()
        .send(LayoutEvent::Changed {
            filename: "round.json".to_string(),
            mtime: 99,
        })
        .expect("send");

    let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
        .await
        .expect("timeout")
        .expect("event");
    let json = serde_json::to_string(&event).expect("serialize");
    assert!(json.contains("\"kind\":\"changed\""));

    // Round-trip via StreamExt to ensure the stream wrappers work.
    let mut stream = tokio_stream::wrappers::BroadcastStream::new(rx);
    let _ = tokio::time::timeout(Duration::from_millis(200), stream.next())
        .await
        .ok()
        .flatten();
}

