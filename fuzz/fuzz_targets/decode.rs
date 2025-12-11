// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

#![no_main]

use darkbio_cobs::{decode, decode_buffer};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let mut dec_buf = vec![0u8; decode_buffer(data.len())];
    let _ = decode(data, &mut dec_buf);
});
