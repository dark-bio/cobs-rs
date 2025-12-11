#![no_main]

use libfuzzer_sys::fuzz_target;
use darkbio_cobs::{decode, encode, encode_buffer, decode_buffer};

fuzz_target!(|data: &[u8]| {
    let mut enc_buf = vec![0u8; encode_buffer(data.len())];
    let enc_len = encode(data, &mut enc_buf).unwrap();

    let mut dec_buf = vec![0u8; if enc_len > 0 { decode_buffer(enc_len) } else { 0 }];
    let dec_len = decode(&enc_buf[..enc_len], &mut dec_buf).unwrap();

    assert_eq!(&dec_buf[..dec_len], data, "roundtrip mismatch");
});
