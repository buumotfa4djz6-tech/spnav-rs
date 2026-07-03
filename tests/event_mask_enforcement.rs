//! Gap 4: verify the client sends the right event-mask request and correctly
//! handles the daemon's response.
//!
//! (The daemon is what actually filters events; the client's responsibility is
//! to encode the chosen mask and propagate daemon failures.)

mod common;

use common::SpnavDaemonSimulator;
use spnav_rs::protocol::{req, ReqResp, REQ_TAG};
use spnav_rs::{EventMask, SpnavClient};

async fn open_and_handle_mask<F>(mask: EventMask, handler: F) -> Result<(), spnav_rs::Error>
where
    F: FnOnce(&SpnavDaemonSimulator, [u8; 32]) + Send + 'static,
{
    let sim = SpnavDaemonSimulator::start().await;
    let path = sim.path().to_path_buf();

    let sim_for_handler = sim;
    let handler_task = tokio::spawn(async move {
        let req_bytes = sim_for_handler.recv().await.expect("request should arrive");
        handler(&sim_for_handler, req_bytes);
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    let result = client.set_event_mask(mask).await;

    handler_task.await.expect("handler should complete");
    result.map(|_| ())
}

#[tokio::test]
async fn set_event_mask_default_encodes_correctly() {
    let result = open_and_handle_mask(EventMask::DEFAULT, |sim, req_bytes| {
        let rr = ReqResp::from_bytes(&req_bytes);
        assert_eq!(rr.type_, REQ_TAG | req::SET_EVMASK);
        assert_eq!(rr.data[0], EventMask::DEFAULT.bits() as i32);
        sim.send(
            ReqResp {
                type_: REQ_TAG | req::SET_EVMASK,
                data: [0; 7],
            }
            .to_bytes(),
        );
    })
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn set_event_mask_all_encodes_correctly() {
    let result = open_and_handle_mask(EventMask::ALL, |sim, req_bytes| {
        let rr = ReqResp::from_bytes(&req_bytes);
        assert_eq!(rr.type_, REQ_TAG | req::SET_EVMASK);
        assert_eq!(rr.data[0], EventMask::ALL.bits() as i32);
        sim.send(
            ReqResp {
                type_: REQ_TAG | req::SET_EVMASK,
                data: [0; 7],
            }
            .to_bytes(),
        );
    })
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn set_event_mask_empty_encodes_zero() {
    let result = open_and_handle_mask(EventMask::empty(), |sim, req_bytes| {
        let rr = ReqResp::from_bytes(&req_bytes);
        assert_eq!(rr.data[0], 0, "empty mask should encode as zero");
        sim.send(
            ReqResp {
                type_: REQ_TAG | req::SET_EVMASK,
                data: [0; 7],
            }
            .to_bytes(),
        );
    })
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn set_event_mask_daemon_failure_propagates() {
    let result = open_and_handle_mask(EventMask::MOTION, |sim, _req_bytes| {
        // Respond with status_ok == false (data[6] < 0).
        let mut data = [0i32; 7];
        data[6] = -1;
        let response = ReqResp {
            type_: REQ_TAG | req::SET_EVMASK,
            data,
        };
        sim.send(response.to_bytes());
    })
    .await;
    assert!(
        result.is_err(),
        "daemon failure should propagate as an error"
    );
}

#[tokio::test]
async fn set_event_mask_only_writes_one_frame() {
    use std::sync::Arc;

    let sim = Arc::new(SpnavDaemonSimulator::start().await);
    let path = sim.path().to_path_buf();

    let sim_for_handler = sim.clone();
    let handler = tokio::spawn(async move {
        let req_bytes = sim_for_handler.recv().await.expect("request");
        let rr = ReqResp::from_bytes(&req_bytes);
        assert_eq!(rr.type_, REQ_TAG | req::SET_EVMASK);
        sim_for_handler.send(
            ReqResp {
                type_: REQ_TAG | req::SET_EVMASK,
                data: [0; 7],
            }
            .to_bytes(),
        );
    });

    let mut client = SpnavClient::open_path(&path).await.unwrap();
    client.set_event_mask(EventMask::MOTION).await.unwrap();

    handler.await.unwrap();

    // Give the event loop a tick to make sure it doesn't write anything else.
    tokio::task::yield_now().await;
    let extra = sim.try_recv_all().await;
    assert!(
        extra.is_empty(),
        "set_event_mask should write exactly one frame, got {} extras",
        extra.len()
    );
}
