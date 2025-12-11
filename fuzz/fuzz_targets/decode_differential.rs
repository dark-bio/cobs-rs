// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

#![no_main]

use darkbio_cobs::{decode, decode_buffer};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip empty data, it's invalid
    if data.is_empty() {
        return;
    }
    // Skip data containing 0x00, we expect single frames
    if data.contains(&0) {
        return;
    }
    // Decode with local implementation
    let mut local_dec = vec![0u8; decode_buffer(data.len())];
    let local_result = decode(data, &mut local_dec);

    // Decode with reference cobs crate
    let mut ref_dec = vec![0u8; data.len()];
    let ref_result = cobs::decode(data, &mut ref_dec);

    // Both should agree on success/failure
    match (&local_result, &ref_result) {
        (Ok(local_len), Ok(ref_report)) => {
            // Reference must have consumed all input (no partial frames)
            assert_eq!(
                ref_report.parsed_size(),
                data.len(),
                "reference only consumed {} of {} bytes for input {:?}",
                ref_report.parsed_size(),
                data.len(),
                data
            );
            assert_eq!(
                &local_dec[..*local_len],
                &ref_dec[..ref_report.frame_size()],
                "decode mismatch for input {:?}",
                data
            );
        }
        (Err(_), Err(_)) => {}
        (Ok(local_len), Err(ref_err)) => {
            panic!(
                "local succeeded ({} bytes) but reference failed ({:?}) for input {:?}",
                local_len, ref_err, data
            );
        }
        (Err(local_err), Ok(ref_report)) => {
            panic!(
                "local failed ({:?}) but reference succeeded ({} bytes) for input {:?}",
                local_err,
                ref_report.frame_size(),
                data
            );
        }
    }
});
