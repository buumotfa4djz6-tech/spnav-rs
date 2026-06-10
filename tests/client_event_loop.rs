//! Gap 6: client event-loop behavior.
//!
//! The event loop is the background task that multiplexes daemon frames and
//! client requests. These tests verify it forwards events correctly, handles
//! connection close, and interleaves events with in-flight requests.

mod common;

use common::{encode_button_frame, encode_motion_frame, SpnavDaemonSimulator};
use spnav_rs::protocol::{ReqResp, REQ_TAG};
use spnav_rs::{SpnavClient, SpnavEvent};

#[tokio::test]
async fn motion_event_is_forwarded_to_wait_event() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn(async move {
        // Small delay to let the client finish opening and call wait_event.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        sim.send(encode_motion_frame(10, -20, 30, 1, 2, 3, 16));
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    let event = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
        .await
        .expect("wait_event should complete within 1s")
        .expect("wait_event should return an event");

    match event {
        SpnavEvent::Motion(m) => {
            assert_eq!(m.x, 10);
            assert_eq!(m.y, -20);
            assert_eq!(m.z, 30);
            assert_eq!(m.rx, 1);
            assert_eq!(m.ry, 2);
            assert_eq!(m.rz, 3);
            assert_eq!(m.period, 16);
        }
        other => panic!("expected Motion, got {:?}", other),
    }

    handler.await.unwrap();
}

#[tokio::test]
async fn button_press_event_is_forwarded() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        sim.send(encode_button_frame(true, 5));
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    let event = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
        .await
        .unwrap()
        .unwrap();

    match event {
        SpnavEvent::Button(b) => {
            assert!(b.press);
            assert_eq!(b.bnum, 5);
        }
        other => panic!("expected Button, got {:?}", other),
    }

    handler.await.unwrap();
}

#[tokio::test]
async fn multiple_events_are_forwarded_in_order() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        sim.send(encode_motion_frame(1, 0, 0, 0, 0, 0, 0));
        sim.send(encode_motion_frame(2, 0, 0, 0, 0, 0, 0));
        sim.send(encode_button_frame(true, 0));
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();

    for expected_x in [1, 2] {
        let event = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
            .await
            .unwrap()
            .unwrap();
        match event {
            SpnavEvent::Motion(m) => assert_eq!(m.x, expected_x),
            other => panic!("expected Motion({}), got {:?}", expected_x, other),
        }
    }

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(event, SpnavEvent::Button(b) if b.press && b.bnum == 0));

    handler.await.unwrap();
}

#[tokio::test]
async fn events_interleaved_with_request_are_not_lost() {
    use spnav_rs::protocol::req;
    use spnav_rs::EventMask;

    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn(async move {
        // Wait for the client to send a request, then:
        //   1. Send an event BEFORE the response
        //   2. Send the response
        //   3. Send another event AFTER the response
        // The event loop should forward both events and the request should
        // still complete.
        let _req = sim.recv().await.expect("request should arrive");

        sim.send(encode_motion_frame(100, 0, 0, 0, 0, 0, 0));
        sim.send(
            ReqResp {
                type_: REQ_TAG,
                data: [0; 7],
            }
            .to_bytes(),
        );
        sim.send(encode_motion_frame(200, 0, 0, 0, 0, 0, 0));
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();

    // The request should succeed (response arrives between two events).
    client
        .set_event_mask(EventMask::MOTION)
        .await
        .expect("request should complete despite interleaved events");

    // Both events should be receivable.
    let e1 = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
        .await
        .unwrap()
        .unwrap();
    let e2 = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event())
        .await
        .unwrap()
        .unwrap();

    let xs = match (&e1, &e2) {
        (SpnavEvent::Motion(a), SpnavEvent::Motion(b)) => (a.x, b.x),
        other => panic!("expected two Motion events, got {:?} and {:?}", e1, e2),
    };
    assert_eq!(xs, (100, 200), "both interleaved events should arrive");

    handler.await.unwrap();
}

#[tokio::test]
async fn wait_event_returns_error_after_daemon_closes_connection() {
    use std::sync::Arc;

    let sim = Arc::new(SpnavDaemonSimulator::start().await);
    let path = sim.path().to_path_buf();

    let mut client = SpnavClient::open_path(&path).await.unwrap();

    // Drop the simulator: its listener, channels, and spawned tasks all go
    // away, closing the underlying socket. The event loop should see EOF and
    // the broadcast channel should close, causing wait_event to fail.
    drop(sim);

    let result = tokio::time::timeout(std::time::Duration::from_secs(1), client.wait_event()).await;

    match result {
        Ok(Err(_)) => {} // expected: connection closed
        Ok(Ok(ev)) => panic!("expected error after daemon closed, got event {:?}", ev),
        Err(_) => panic!("wait_event hung instead of returning after daemon closed"),
    }
}

#[tokio::test]
async fn poll_event_returns_none_when_idle() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    // Give the event loop a moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    assert!(
        client.poll_event().is_none(),
        "poll_event should return None when no events are queued"
    );
}

#[tokio::test]
async fn poll_event_returns_event_when_queued() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        sim.send(encode_motion_frame(42, 0, 0, 0, 0, 0, 0));
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();

    // Spin until the event arrives (bounded by timeout).
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
    let event = loop {
        if let Some(ev) = client.poll_event() {
            break Some(ev);
        }
        if tokio::time::Instant::now() > deadline {
            break None;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    };

    match event {
        Some(SpnavEvent::Motion(m)) => assert_eq!(m.x, 42),
        other => panic!("expected Motion(42, ...), got {:?}", other),
    }

    handler.await.unwrap();
}
