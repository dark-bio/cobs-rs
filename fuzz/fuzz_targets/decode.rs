#![no_main]

use libfuzzer_sys::fuzz_target;
use darkbio_cobs::{decode, decode_buffer};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let mut dec_buf = vec![0u8; decode_buffer(data.len())];
    let _ = decode(data, &mut dec_buf);
});
