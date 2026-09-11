# Fast COBS encoder and decoder

[![](https://img.shields.io/crates/v/darkbio-cobs.svg)](https://crates.io/crates/darkbio-cobs)
[![](https://docs.rs/darkbio-cobs/badge.svg)](https://docs.rs/darkbio-cobs)
[![](https://github.com/dark-bio/cobs-rs/workflows/tests/badge.svg)](https://github.com/dark-bio/cobs-rs/actions/workflows/ci.yml)

This crate is a *fast* implementation of [Consistent Overhead Byte Stuffing (COBS)](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing). It doesn't do much, but it does it fast. Although there might be eventual fixups and feature expansions for streaming codecs, assume the library is "done".

The library is `no_std` and never allocates. Disabling the default `std` feature
only costs runtime SIMD detection on x86_64, where `memchr` then picks SSE2
instead of AVX2.

## Usage

The codec works on caller provided buffers and returns how many bytes it wrote.
The two sizing helpers compute the worst case output for an input length, so a
buffer of that size always fits.

```rust
use darkbio_cobs::{decode, decode_buffer, encode, encode_buffer};

let data = [0x11, 0x00, 0x22, 0x33, 0x00];

let mut encoded = vec![0u8; encode_buffer(data.len())];
let len = encode(&data, &mut encoded).unwrap();
assert_eq!(&encoded[..len], &[0x02, 0x11, 0x03, 0x22, 0x33, 0x01]);

let mut decoded = vec![0u8; decode_buffer(len)];
let len = decode(&encoded[..len], &mut decoded).unwrap();
assert_eq!(&decoded[..len], &data);
```

An encoding never contains a zero byte, so a framing layer can delimit packets
with zeros and hand each frame to the decoder. Frames from such a layer contain
no zeros by construction, which lets `decode_nonzero` skip the validation scan.
Feeding it a zero anyway yields an error or garbage output, never memory
unsafety.

```rust
use darkbio_cobs::{decode_buffer, decode_nonzero};

let frame = [0x02, 0x11, 0x03, 0x22, 0x33, 0x01];

let mut decoded = vec![0u8; decode_buffer(frame.len())];
let len = decode_nonzero(&frame, &mut decoded).unwrap();
assert_eq!(&decoded[..len], &[0x11, 0x00, 0x22, 0x33, 0x00]);
```

Malformed input is reported through `DecodeError`, naming the offending
position. Both directions refuse an undersized output buffer with a
`BufferTooSmall` error before writing anything.

```rust
use darkbio_cobs::{decode, DecodeError};

let mut decoded = [0u8; 16];
let error = DecodeError::ChunkOverflow { at: 0, marker: 3, len: 2 };
assert_eq!(decode(&[0x03, 0x41], &mut decoded), Err(error));
```

The `encode_unchecked`, `decode_unchecked` and `decode_nonzero_unchecked` variants
skip the buffer size checks and keep only a debug assertion, so they are `unsafe`
to call. The caller must size the buffers with the helpers, anything smaller is
undefined behavior in release builds.

```rust
use darkbio_cobs::{encode_buffer, encode_unchecked};

let data = [0x11, 0x00, 0x22];

let mut encoded = vec![0u8; encode_buffer(data.len())];
let len = unsafe { encode_unchecked(&data, &mut encoded) };
assert_eq!(&encoded[..len], &[0x02, 0x11, 0x02, 0x22]);
```

## Performance

You can run the benchmarks to see the performance of the checked versions, unchecked versions and the currently most popular Rust `cobs` package (`v0.5.1`).

```text
% cargo bench -- --quiet
```

This crate, measured on an Apple M2 Max with rustc 1.98.0 in a release build.

|                          | 16 B      | 256 B      | 4 KiB      | 64 KiB     | 256 KiB    | 1 MiB      | 4 MiB      |
|--------------------------|-----------|------------|------------|------------|------------|------------|------------|
| encode                   | 3.1 GiB/s | 28.9 GiB/s | 20.4 GiB/s | 21.5 GiB/s | 19.4 GiB/s | 10.6 GiB/s | 8.5 GiB/s  |
| encode_unchecked         | 3.1 GiB/s | 28.9 GiB/s | 25.9 GiB/s | 20.3 GiB/s | 19.0 GiB/s | 11.9 GiB/s | 8.7 GiB/s  |
| decode                   | 2.7 GiB/s | 14.2 GiB/s | 24.9 GiB/s | 21.4 GiB/s | 17.9 GiB/s | 18.0 GiB/s | 17.8 GiB/s |
| decode_unchecked         | 2.7 GiB/s | 20.9 GiB/s | 25.6 GiB/s | 20.3 GiB/s | 17.8 GiB/s | 18.1 GiB/s | 17.9 GiB/s |
| decode_nonzero           | 4.6 GiB/s | 31.2 GiB/s | 34.4 GiB/s | 25.7 GiB/s | 22.7 GiB/s | 22.7 GiB/s | 22.3 GiB/s |
| decode_nonzero_unchecked | 5.0 GiB/s | 57.9 GiB/s | 35.0 GiB/s | 26.4 GiB/s | 22.8 GiB/s | 22.2 GiB/s | 22.6 GiB/s |

The reference `cobs` crate at 0.5.1 on the same machine.

|        | 16 B      | 256 B     | 4 KiB     | 64 KiB    | 256 KiB   | 1 MiB     | 4 MiB     |
|--------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| encode | 1.5 GiB/s | 1.5 GiB/s | 1.5 GiB/s | 1.6 GiB/s | 1.6 GiB/s | 1.6 GiB/s | 1.6 GiB/s |
| decode | 1.0 GiB/s | 1.1 GiB/s | 1.1 GiB/s | 1.1 GiB/s | 1.1 GiB/s | 1.1 GiB/s | 1.1 GiB/s |

## License

This library is licensed under the [BSD 3-Clause License](https://github.com/dark-bio/cobs-rs/blob/main/LICENSE).
