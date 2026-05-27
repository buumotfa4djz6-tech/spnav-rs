//! Integration tests for string encoding/decoding in the protocol module.

use spnav_rs::protocol::*;

// ─── String encoder tests ───────────────────────────────────────────────────

#[test]
fn encode_empty_string() {
    let chunks = encode_string_chunks(req::SET_NAME, "");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].data[6], 0);
}

#[test]
fn encode_short_string() {
    let chunks = encode_string_chunks(req::SET_NAME, "hello");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].data[6], 5); // length
}

#[test]
fn encode_exact_chunk_size() {
    // 24 bytes = REQSTR_CHUNK_SIZE
    let s = "a".repeat(REQSTR_CHUNK_SIZE);
    let chunks = encode_string_chunks(req::SET_NAME, &s);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].data[6], REQSTR_CHUNK_SIZE as i32);
}

#[test]
fn encode_multi_chunk_string() {
    // 48 bytes = 2 chunks
    let s = "a".repeat(REQSTR_CHUNK_SIZE * 2);
    let chunks = encode_string_chunks(req::SET_NAME, &s);
    assert_eq!(chunks.len(), 2);
    // First chunk has total length
    assert_eq!(chunks[0].data[6], (REQSTR_CHUNK_SIZE * 2) as i32);
    // Second chunk has remaining with continuation bit
    assert_eq!(chunks[1].data[6] & 0xFFFF, REQSTR_CHUNK_SIZE as i32);
    assert_ne!(chunks[1].data[6] & REQSTR_CONT_BIT, 0);
}

#[test]
fn encode_multi_chunk_with_remainder() {
    // 30 bytes = 1 full chunk + 6 bytes
    let s = "a".repeat(30);
    let chunks = encode_string_chunks(req::SET_NAME, &s);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].data[6], 30);
    assert_eq!(chunks[1].data[6] & 0xFFFF, 6);
}

#[test]
fn encode_sets_request_type() {
    let chunks = encode_string_chunks(req::DEV_NAME, "test");
    assert_eq!(chunks[0].type_, REQ_TAG | req::DEV_NAME);
}

#[test]
fn encode_preserves_string_content() {
    let s = "hello world";
    let chunks = encode_string_chunks(req::SET_NAME, s);

    // Decode the first chunk manually
    let rr = &chunks[0];
    let mut bytes = Vec::new();
    let len = (rr.data[6] & 0xFFFF) as usize;
    for i in 0..len {
        let byte = ((rr.data[i / 4] >> ((i % 4) * 8)) & 0xff) as u8;
        bytes.push(byte);
    }
    let decoded = String::from_utf8(bytes).unwrap();
    assert_eq!(decoded, "hello world");
}

// ─── StringDecoder tests ────────────────────────────────────────────────────

#[test]
fn decoder_single_chunk() {
    let chunks = encode_string_chunks(req::SET_NAME, "hello");
    let mut decoder = StringDecoder::new();
    let result = decoder.feed(&chunks[0]).unwrap();
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn decoder_multi_chunk() {
    let s = "a".repeat(REQSTR_CHUNK_SIZE * 2);
    let chunks = encode_string_chunks(req::SET_NAME, &s);
    let mut decoder = StringDecoder::new();

    let result1 = decoder.feed(&chunks[0]).unwrap();
    assert!(result1.is_none()); // more chunks expected

    let result2 = decoder.feed(&chunks[1]).unwrap();
    assert_eq!(result2.unwrap(), s);
}

#[test]
fn decoder_multi_chunk_with_remainder() {
    let s = "b".repeat(30);
    let chunks = encode_string_chunks(req::SET_NAME, &s);
    let mut decoder = StringDecoder::new();

    let result1 = decoder.feed(&chunks[0]).unwrap();
    assert!(result1.is_none());

    let result2 = decoder.feed(&chunks[1]).unwrap();
    assert_eq!(result2.unwrap(), s);
}

#[test]
fn decoder_empty_string() {
    let chunks = encode_string_chunks(req::SET_NAME, "");
    let mut decoder = StringDecoder::new();
    let result = decoder.feed(&chunks[0]).unwrap();
    assert_eq!(result.unwrap(), "");
}

#[test]
fn decoder_failure_status_returns_error() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = REQ_TAG | req::SET_NAME;
    rr.data[6] = -1; // failure status

    let mut decoder = StringDecoder::new();
    let result = decoder.feed(&rr);
    assert!(result.is_err());
}

#[test]
fn decoder_uninitialized_returns_error() {
    // Feed a continuation chunk without initializing
    let mut rr = ReqResp::zeroed();
    rr.type_ = REQ_TAG | req::SET_NAME;
    rr.data[6] = REQSTR_CONT_BIT | 5; // continuation with 5 remaining

    let mut decoder = StringDecoder::new();
    let result = decoder.feed(&rr);
    assert!(result.is_err());
}

#[test]
fn decoder_reuse_after_completion() {
    let chunks1 = encode_string_chunks(req::SET_NAME, "first");
    let chunks2 = encode_string_chunks(req::DEV_NAME, "second");

    let mut decoder = StringDecoder::new();
    let result1 = decoder.feed(&chunks1[0]).unwrap();
    assert_eq!(result1.unwrap(), "first");

    // Decoder should be reusable
    let result2 = decoder.feed(&chunks2[0]).unwrap();
    assert_eq!(result2.unwrap(), "second");
}

#[test]
fn decoder_preserves_utf8() {
    let s = "Hello, 世界! 🌍";
    let chunks = encode_string_chunks(req::SET_NAME, s);
    let mut decoder = StringDecoder::new();
    let result = decoder.feed(&chunks[0]).unwrap();
    assert_eq!(result.unwrap(), s);
}

// ─── Roundtrip tests ────────────────────────────────────────────────────────

#[test]
fn encode_decode_roundtrip_short() {
    let original = "test string";
    let chunks = encode_string_chunks(req::SET_NAME, original);
    let mut decoder = StringDecoder::new();
    let decoded = decoder.feed(&chunks[0]).unwrap().unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn encode_decode_roundtrip_long() {
    let original = "x".repeat(100);
    let chunks = encode_string_chunks(req::SET_NAME, &original);
    let mut decoder = StringDecoder::new();
    let mut last_result = None;
    for chunk in &chunks {
        last_result = decoder.feed(chunk).unwrap();
    }
    assert_eq!(last_result.unwrap(), original);
}

#[test]
fn encode_decode_roundtrip_empty() {
    let original = "";
    let chunks = encode_string_chunks(req::SET_NAME, original);
    let mut decoder = StringDecoder::new();
    let decoded = decoder.feed(&chunks[0]).unwrap().unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn encode_decode_roundtrip_special_chars() {
    let original = "path/to/file\nwith\ttabs";
    let chunks = encode_string_chunks(req::SET_NAME, original);
    let mut decoder = StringDecoder::new();
    let decoded = decoder.feed(&chunks[0]).unwrap().unwrap();
    assert_eq!(decoded, original);
}

// ─── REQSTR constants tests ─────────────────────────────────────────────────

#[test]
fn reqstr_chunk_size_is_24() {
    assert_eq!(REQSTR_CHUNK_SIZE, 24);
}

#[test]
fn reqstr_cont_bit_is_0x10000() {
    assert_eq!(REQSTR_CONT_BIT, 0x10000);
}
