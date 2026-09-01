// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

#![no_main]

use darkbio_cobs::{decode, decode_buffer, decode_nonzero};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip empty data, it's invalid
    if data.is_empty() {
        return;
    }
    let mut dec_buf = vec![0u8; decode_buffer(data.len())];
    let _ = decode(data, &mut dec_buf);

    // The nonzero decoder documents garbage output for zero laced inputs, but
    // it must remain memory safe on arbitrary data
    let _ = decode_nonzero(data, &mut dec_buf);
});
