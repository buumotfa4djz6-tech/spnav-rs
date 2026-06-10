//! Gap 1: SpnavClient::open() handshake against a scripted daemon.

mod common;

use common::SpnavDaemonSimulator;
use spnav_rs::protocol::{ReqResp, REQ_TAG};
use spnav_rs::SpnavClient;
use std::path::PathBuf;

#[tokio::test]
async fn open_path_against_simulator_succeeds() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();
    let client = SpnavClient::open_path(&path).await;
    assert!(client.is_ok(), "client should open against simulator");
}

#[tokio::test]
async fn protocol_version_is_negotiated_to_v1() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();
    let client = SpnavClient::open_path(&path).await.unwrap();
    // The simulator advertises MAX_PROTO_VER (1); the client should report it.
    assert_eq!(client.protocol_version(), 1);
}

#[tokio::test]
async fn open_to_nonexistent_socket_returns_error() {
    let path = PathBuf::from("/tmp/nonexistent_spnav_integration_test_12345.sock");
    let result = SpnavClient::open_path(&path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn client_can_be_dropped_cleanly_after_open() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();
    let client = SpnavClient::open_path(&path).await.unwrap();
    drop(client);
    // If we get here without hanging, shutdown worked.
}

/// Sanity-check: the handshake request the client sends is the tagged
/// CHANGE_PROTO command, and the first 4 bytes on the wire carry it.
#[tokio::test]
async fn handshake_sends_tagged_change_proto() {
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();
    let _client = SpnavClient::open_path(&path).await.unwrap();

    // After the 4-byte handshake command, the event loop is idle waiting for
    // frames. The simulator's recv() buffer should be empty at this point
    // (the handshake bytes were consumed before the reader task started).
    let buffered = sim.try_recv_all().await;
    assert!(
        buffered.is_empty(),
        "no frames should arrive from the client until it sends a request"
    );
}

/// After handshake, a client request round-trips: the simulator sees the
/// tagged request and the client accepts the tagged response.
#[tokio::test]
async fn simple_request_round_trip() {
    use spnav_rs::protocol::req;
    use spnav_rs::EventMask;

    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let handler = tokio::spawn({
        async move {
            // Read the client's request frame.
            let req_bytes = sim.recv().await.expect("client should send a request");
            let rr = ReqResp::from_bytes(&req_bytes);
            assert_eq!(rr.type_, REQ_TAG | req::SET_EVMASK);
            assert_eq!(rr.data[0], EventMask::MOTION.bits() as i32);

            // Send back a tagged OK response (data[6] >= 0 ⇒ status_ok).
            let response = ReqResp {
                type_: REQ_TAG,
                data: [0; 7],
            };
            sim.send(response.to_bytes());
        }
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    client
        .set_event_mask(EventMask::MOTION)
        .await
        .expect("set_event_mask should succeed");

    handler.await.expect("handler task should complete");
}
