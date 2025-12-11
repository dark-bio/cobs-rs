// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

#![no_main]

use darkbio_cobs::{decode, decode_buffer, encode, encode_buffer};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Encode with local implementation
    let mut local_enc = vec![0u8; encode_buffer(data.len())];
    let local_enc_len = encode(data, &mut local_enc).unwrap();

    // Encode with reference cobs crate
    let mut ref_enc = vec![0u8; cobs::max_encoding_length(data.len())];
    let ref_enc_len = cobs::encode(data, &mut ref_enc);

    // Compare encoded outputs
    assert_eq!(
        &local_enc[..local_enc_len],
        &ref_enc[..ref_enc_len],
        "encode mismatch for input {:?}",
        data
    );
    // Decode with local implementation
    let mut local_dec = vec![
        0u8;
        if local_enc_len > 0 {
            decode_buffer(local_enc_len)
        } else {
            0
        }
    ];
    let local_dec_len = decode(&local_enc[..local_enc_len], &mut local_dec).unwrap();

    // Decode with reference cobs crate
    let mut ref_dec = vec![0u8; ref_enc_len];
    let ref_dec_report = cobs::decode(&ref_enc[..ref_enc_len], &mut ref_dec).unwrap();

    // Compare decoded outputs
    assert_eq!(
        &local_dec[..local_dec_len],
        &ref_dec[..ref_dec_report.frame_size()],
        "decode mismatch for encoded {:?}",
        &local_enc[..local_enc_len]
    );
    // Verify roundtrip matches original
    assert_eq!(&local_dec[..local_dec_len], data, "roundtrip mismatch");
});
