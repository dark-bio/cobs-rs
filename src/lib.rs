// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

// Pull in the README as the package doc
#![doc = include_str!("../README.md")]
// Build without the standard library unless the std feature asks for it
#![cfg_attr(not(feature = "std"), no_std)]

// The tests allocate, so they link the standard library in every configuration
#[cfg(test)]
extern crate std;

/// Error types that can be returned from encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EncodeError {
    /// The output buffer holds `have` bytes but the worst case encoding of the
    /// input needs `want`, as computed by [`encode_buffer`]. Nothing was
    /// written. A smaller buffer is refused even if the actual encoding would
    /// have fit.
    #[error("buffer too small: have {have} bytes, want {want} bytes")]
    BufferTooSmall { have: usize, want: usize },
}

/// Error types that can be returned from decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// The input is empty. A COBS stream is never shorter than one byte, since
    /// the empty payload encodes as a single `0x01`.
    #[error("empty input")]
    EmptyInput,

    /// The output buffer holds `have` bytes but the worst case decoding of the
    /// input needs `want`, as computed by [`decode_buffer`]. Nothing was
    /// written. Inputs of a single byte skip this check, they never produce
    /// output.
    #[error("buffer too small: have {have} bytes, want {want} bytes")]
    BufferTooSmall { have: usize, want: usize },

    /// The chunk marker at input offset `at` is zero, which no COBS stream
    /// contains. Both [`decode`] and [`decode_nonzero`] report it. The latter
    /// keeps this one check to stay memory safe.
    #[error("zero marker at position {at}")]
    ZeroMarker { at: usize },

    /// A chunk payload contains a zero byte at input offset `at`, which the
    /// encoder never produces. Only [`decode`] reports it. [`decode_nonzero`]
    /// trusts the caller and decodes such input to garbage instead.
    #[error("zero byte in data at position {at}")]
    ZeroBinary { at: usize },

    /// The chunk marker at input offset `at` announces `marker - 1` payload
    /// bytes, which run past the `len` bytes of input. Truncated frames end up
    /// here.
    #[error("chunk overflow at position {at}: chunk {marker} exceeds data length {len}")]
    ChunkOverflow { at: usize, marker: u8, len: usize },
}

/// Computes the maximum size needed to COBS encode a blind input blob.
#[inline]
pub const fn encode_buffer(size: usize) -> usize {
    size + size.div_ceil(254) + 1
}

/// Computes the maximum size needed to COBS decode a blind input data.
#[inline]
pub const fn decode_buffer(size: usize) -> usize {
    if size == 0 {
        // Zero length COBS is invalid. We could panic here, but that makes call
        // sites brittle when parsing potentially malicious input. We could also
        // return an error, but that makes the method so much uglier. Returning
        // zero is safe however, because the caller can still alloc a zero-byte
        // buffer and the decoder will error anyway.
        return 0;
    }
    size - 1
}

/// Encodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the encoding took. Returns an error if the output buffer
/// is too small.
#[inline]
pub fn encode(data: &[u8], encoded: &mut [u8]) -> Result<usize, EncodeError> {
    let want = encode_buffer(data.len());
    if encoded.len() < want {
        return Err(EncodeError::BufferTooSmall {
            have: encoded.len(),
            want,
        });
    }
    // The output was checked to hold the worst case encoding
    Ok(unsafe { encode_unchecked(data, encoded) })
}

/// Encodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the encoding took.
///
/// # Safety
/// The caller must ensure `encoded` has at least `encode_buffer(data.len())` bytes.
#[inline]
pub unsafe fn encode_unchecked(data: &[u8], encoded: &mut [u8]) -> usize {
    // The empty blob is always encoded as 0x01
    if data.is_empty() {
        encoded[0] = 0x01;
        return 1;
    }
    // Sanity check in debug builds that the user called it correctly
    debug_assert!(encoded.len() >= encode_buffer(data.len()));

    // Consume the input stream one zero delimited run at a time, copying whole
    // chunks into the output instead of individual bytes
    unsafe {
        let mut input_pos = 0usize;
        let mut output_pos = 0usize;

        loop {
            // Look up the next zero, skipping the scanner call for zero runs
            let run = if *data.get_unchecked(input_pos) == 0 {
                Some(0)
            } else {
                memchr::memchr(0, data.get_unchecked(input_pos..))
            };
            // Copy over all the full chunks preceding the zero or the end
            let mut rem = run.unwrap_or(data.len() - input_pos);
            while rem >= 254 {
                *encoded.get_unchecked_mut(output_pos) = 0xff;
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(input_pos),
                    encoded.as_mut_ptr().add(output_pos + 1),
                    254,
                );
                input_pos += 254;
                output_pos += 255;
                rem -= 254;
            }
            if run.is_some() {
                // Copy over the partial chunk and consume the zero closing it
                *encoded.get_unchecked_mut(output_pos) = rem as u8 + 1;
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(input_pos),
                    encoded.as_mut_ptr().add(output_pos + 1),
                    rem,
                );
                input_pos += rem + 1;
                output_pos += rem + 1;

                // If the zero was the last byte, terminate with an empty chunk
                if input_pos == data.len() {
                    *encoded.get_unchecked_mut(output_pos) = 0x01;
                    return output_pos + 1;
                }
            } else {
                // Copy over any partial chunk at the tail. Data ending exactly
                // on a chunk boundary was fully consumed by the full chunks.
                if rem > 0 {
                    *encoded.get_unchecked_mut(output_pos) = rem as u8 + 1;
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(input_pos),
                        encoded.as_mut_ptr().add(output_pos + 1),
                        rem,
                    );
                    output_pos += rem + 1;
                }
                return output_pos;
            }
        }
    }
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the decoding took. Returns an error if the output buffer
/// is too small or if the input is malformed.
#[inline]
pub fn decode(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::EmptyInput);
    }
    if data.len() > 1 {
        let want = decode_buffer(data.len());
        if decoded.len() < want {
            return Err(DecodeError::BufferTooSmall {
                have: decoded.len(),
                want,
            });
        }
    }
    // The output was checked to hold the worst case decoding, a lone byte
    // never produces any
    unsafe { decode_unchecked(data, decoded) }
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the decoding took.
///
/// # Safety
/// The caller must ensure `decoded` has at least `decode_buffer(data.len())` bytes.
#[inline]
pub unsafe fn decode_unchecked(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
    // The empty blob is not a valid COBS encoding
    if data.is_empty() {
        return Err(DecodeError::EmptyInput);
    }
    // The empty text is always encoded as 0x01
    if data.len() == 1 && data[0] == 0x01 {
        return Ok(0);
    }
    // Sanity check in debug builds that the user called it correctly
    debug_assert!(decoded.len() >= decode_buffer(data.len()));

    // A valid COBS stream cannot contain any zero bytes, neither as chunk
    // markers nor as chunk content, so a single scan up front can validate the
    // entire input. Streams failing it are handed off to the byte by byte
    // decoder to pinpoint the error. Clean streams skip all further checks.
    if memchr::memchr(0, data).is_some() {
        return decode_scalar(data, decoded);
    }
    decode_chunked::<false>(data, decoded)
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value,
/// assuming the input contains no zero bytes, a guarantee usually provided by
/// a zero delimited framing layer. Skipping the validation scan makes this
/// faster than `decode`, but violating the assumption yields either a decode
/// error or garbage output, never memory unsafety. Returns the number of bytes
/// the decoding took. Returns an error if the output buffer is too small or if
/// the input is malformed.
#[inline]
pub fn decode_nonzero(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::EmptyInput);
    }
    if data.len() > 1 {
        let want = decode_buffer(data.len());
        if decoded.len() < want {
            return Err(DecodeError::BufferTooSmall {
                have: decoded.len(),
                want,
            });
        }
    }
    // The output was checked to hold the worst case decoding, a lone byte
    // never produces any
    unsafe { decode_nonzero_unchecked(data, decoded) }
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value,
/// assuming the input contains no zero bytes. Returns the number of bytes the
/// decoding took.
///
/// # Safety
/// The caller must ensure `decoded` has at least `decode_buffer(data.len())` bytes.
#[inline]
pub unsafe fn decode_nonzero_unchecked(
    data: &[u8],
    decoded: &mut [u8],
) -> Result<usize, DecodeError> {
    // The empty blob is not a valid COBS encoding
    if data.is_empty() {
        return Err(DecodeError::EmptyInput);
    }
    // The empty text is always encoded as 0x01
    if data.len() == 1 && data[0] == 0x01 {
        return Ok(0);
    }
    // Sanity check in debug builds that the user called it correctly
    debug_assert!(decoded.len() >= decode_buffer(data.len()));

    decode_chunked::<true>(data, decoded)
}

/// Decodes an opaque data blob with COBS one chunk at a time, copying whole
/// chunks into the output instead of individual bytes. With `CHECKED` the
/// chunk markers are verified to not be zero, without it the caller vouches
/// that the input contains no zero bytes at all.
///
/// # Safety
/// The caller must ensure `decoded` has at least `decode_buffer(data.len())`
/// bytes and that `data` is not empty. Without `CHECKED`, the caller must also
/// ensure that `data` contains no zero bytes.
#[inline]
fn decode_chunked<const CHECKED: bool>(
    data: &[u8],
    decoded: &mut [u8],
) -> Result<usize, DecodeError> {
    unsafe {
        let mut input_pos = 0usize;
        let mut output_pos = 0usize;

        // Consume the bulk of the stream with fixed size copies per chunk. The
        // copies intentionally cover a maximum size no matter the real one,
        // making them straight inline copies without memcpy calls. Short
        // chunks copy 16 bytes and anything longer the full 254, keeping the
        // write amplification of tiny chunk streams in check. Garbage copied
        // past a chunk is overwritten by the next chunk or falls beyond the
        // length returned to the caller. The stream cannot end nor overflow
        // within this loop, so the separator zero can also be written blindly,
        // dropped again for full chunks by not advancing over it.
        while input_pos + 255 < data.len() {
            let marker = *data.get_unchecked(input_pos);
            if CHECKED && marker == 0 {
                return Err(DecodeError::ZeroMarker { at: input_pos });
            }
            let chunk = marker as usize - 1;
            input_pos += 1;

            core::ptr::copy_nonoverlapping(
                data.as_ptr().add(input_pos),
                decoded.as_mut_ptr().add(output_pos),
                16,
            );
            if chunk > 16 {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(input_pos),
                    decoded.as_mut_ptr().add(output_pos),
                    254,
                );
            }
            input_pos += chunk;
            output_pos += chunk;

            *decoded.get_unchecked_mut(output_pos) = 0;
            output_pos += (marker != 0xff) as usize;
        }
        // Consume the stream tail one chunk at a time with exact copies
        loop {
            // Read the length marker and ensure the chunk fits the input
            let marker = *data.get_unchecked(input_pos);
            if CHECKED && marker == 0 {
                return Err(DecodeError::ZeroMarker { at: input_pos });
            }
            let chunk = marker as usize - 1;
            input_pos += 1;

            if input_pos + chunk > data.len() {
                return Err(DecodeError::ChunkOverflow {
                    at: input_pos - 1,
                    marker,
                    len: data.len(),
                });
            }
            // Copy over the entire chunk
            core::ptr::copy_nonoverlapping(
                data.as_ptr().add(input_pos),
                decoded.as_mut_ptr().add(output_pos),
                chunk,
            );
            input_pos += chunk;
            output_pos += chunk;

            // If the stream is done, so is the decoder
            if input_pos == data.len() {
                return Ok(output_pos);
            }
            // If we had a partial chunk, there must be a zero following
            if marker != 0xff {
                *decoded.get_unchecked_mut(output_pos) = 0;
                output_pos += 1;
            }
        }
    }
}

/// Decodes an opaque data blob with COBS one byte at a time. This is the path
/// for streams known to contain zero bytes, walking the chunks to pinpoint
/// whether a zero marker, a zero binary or an overflow triggers first.
///
/// # Safety
/// The caller must ensure `decoded` has at least `decode_buffer(data.len())` bytes.
#[cold]
#[inline(never)]
fn decode_scalar(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
    // Consume the input stream one chunk at a time
    unsafe {
        let mut output_pos = 0usize;
        let mut i = 0usize;

        while i < data.len() {
            // Zero cannot be part of a COBS encoded stream
            let marker = *data.get_unchecked(i);
            if marker == 0 {
                return Err(DecodeError::ZeroMarker { at: i });
            }
            i += 1;

            // If the marker defines an overflowing chunk, abort
            if i + (marker as usize) - 1 > data.len() {
                return Err(DecodeError::ChunkOverflow {
                    at: i - 1,
                    marker,
                    len: data.len(),
                });
            }
            // Consume the entire chunk, ensuring there's no zero in it
            for _ in 1..marker {
                let b = *data.get_unchecked(i);
                if b == 0 {
                    return Err(DecodeError::ZeroBinary { at: i });
                }
                *decoded.get_unchecked_mut(output_pos) = b;
                output_pos += 1;
                i += 1;
            }
            // If we had a partial chunk, there must be a zero following
            if i < data.len() && marker != 0xff {
                *decoded.get_unchecked_mut(output_pos) = 0;
                output_pos += 1;
            }
        }
        Ok(output_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn test_roundtrip_empty() {
        let data = [];
        let mut enc_buf = [0u8; 1];
        let len = encode(&data, &mut enc_buf).unwrap();
        assert_eq!(len, 1);
        assert_eq!(enc_buf[0], 0x01);

        let mut dec_buf = [0u8; 0];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(dec_len, 0);
    }

    #[test]
    fn test_roundtrip_no_zeros() {
        let data = [1, 2, 3, 4, 5];
        let mut enc_buf = [0u8; encode_buffer(5)];
        let len = encode(&data, &mut enc_buf).unwrap();

        let mut dec_buf = [0u8; decode_buffer(encode_buffer(5))];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data);
    }

    #[test]
    fn test_roundtrip_with_zeros() {
        let data = [0, 1, 0, 2, 0, 0, 3];
        let mut enc_buf = [0u8; encode_buffer(7)];
        let len = encode(&data, &mut enc_buf).unwrap();

        let mut dec_buf = [0u8; decode_buffer(encode_buffer(7))];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data);
    }

    #[test]
    fn test_roundtrip_254_nonzero() {
        let data: Vec<u8> = (1..=254).collect();
        let mut enc_buf = vec![0u8; encode_buffer(254)];
        let len = encode(&data, &mut enc_buf).unwrap();

        let mut dec_buf = vec![0u8; decode_buffer(enc_buf.len())];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data[..]);
    }

    #[test]
    fn test_roundtrip_255_nonzero() {
        let data: Vec<u8> = (1..=254).chain(std::iter::once(1)).collect();
        let mut enc_buf = vec![0u8; encode_buffer(255)];
        let len = encode(&data, &mut enc_buf).unwrap();

        let mut dec_buf = vec![0u8; decode_buffer(enc_buf.len())];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data[..]);
    }

    #[test]
    fn test_roundtrip_chunk_boundaries() {
        let sizes: Vec<usize> = if cfg!(miri) {
            (0..=64)
                .chain([253, 254, 255, 256, 507, 508, 509, 510, 1021, 1024])
                .collect()
        } else {
            (0..=515)
                .chain([1021, 1024, 4093, 4096, 8191, 65536])
                .collect()
        };
        for size in sizes {
            for period in [1usize, 2, 3, 253, 254, 255, 256] {
                for phase in [0, period - 1] {
                    let data: Vec<u8> = (0..size)
                        .map(|i| {
                            if i % period == phase {
                                0
                            } else {
                                (i % 251 + 1) as u8
                            }
                        })
                        .collect();
                    roundtrip_reference(&data);
                }
            }
            let data: Vec<u8> = (0..size).map(|i| (i % 251 + 1) as u8).collect();
            roundtrip_reference(&data);
        }
    }

    /// Encodes and decodes a blob with both this crate and the reference cobs
    /// crate, cross checking all the outputs against one another.
    fn roundtrip_reference(data: &[u8]) {
        let mut encoded = vec![0u8; encode_buffer(data.len())];
        let encoded_len = encode(data, &mut encoded).unwrap();

        let mut reference = vec![0u8; cobs::max_encoding_length(data.len())];
        let reference_len = cobs::encode(data, &mut reference);
        assert_eq!(&encoded[..encoded_len], &reference[..reference_len]);

        let mut decoded = vec![0u8; decode_buffer(encoded_len)];
        let decoded_len = decode(&encoded[..encoded_len], &mut decoded).unwrap();
        assert_eq!(&decoded[..decoded_len], data);

        let mut nonzero = vec![0u8; decode_buffer(encoded_len)];
        let nonzero_len = decode_nonzero(&encoded[..encoded_len], &mut nonzero).unwrap();
        assert_eq!(&nonzero[..nonzero_len], data);
    }

    #[test]
    fn test_decode_malformed() {
        let mut buffer = [0u8; 16];

        assert_eq!(decode(&[], &mut buffer), Err(DecodeError::EmptyInput));
        assert_eq!(
            decode(&[0x00], &mut buffer),
            Err(DecodeError::ZeroMarker { at: 0 })
        );
        assert_eq!(
            decode(&[0x02, 0x41, 0x00], &mut buffer),
            Err(DecodeError::ZeroMarker { at: 2 })
        );
        assert_eq!(
            decode(&[0x02, 0x00], &mut buffer),
            Err(DecodeError::ZeroBinary { at: 1 })
        );
        assert_eq!(
            decode(&[0x03, 0x41, 0x00, 0x41], &mut buffer),
            Err(DecodeError::ZeroBinary { at: 2 })
        );
        assert_eq!(
            decode(&[0x03, 0x41], &mut buffer),
            Err(DecodeError::ChunkOverflow {
                at: 0,
                marker: 3,
                len: 2
            })
        );
        assert_eq!(
            decode(&[0x05, 0x41, 0x00, 0x41], &mut buffer),
            Err(DecodeError::ChunkOverflow {
                at: 0,
                marker: 5,
                len: 4
            })
        );
    }

    #[test]
    fn test_decode_nonzero_malformed() {
        let mut buffer = [0u8; 128];

        assert_eq!(
            decode_nonzero(&[], &mut buffer),
            Err(DecodeError::EmptyInput)
        );
        assert_eq!(
            decode_nonzero(&[0x03, 0x41], &mut buffer),
            Err(DecodeError::ChunkOverflow {
                at: 0,
                marker: 3,
                len: 2
            })
        );
        // Zero free truncated streams long enough for the chunked decoder must
        // error the same way as the scanning decoder
        let mut long = Vec::new();
        for _ in 0..26 {
            long.extend_from_slice(&[0x03, 0x41, 0x42]);
        }
        long.extend_from_slice(&[0x05, 0x41]);
        assert_eq!(
            decode_nonzero(&long, &mut buffer),
            decode(&long, &mut [0u8; 128])
        );
        // Feeding zeroes violates the contract, the result is unspecified but
        // the call must remain memory safe
        long[40] = 0;
        let _ = decode_nonzero(&long, &mut buffer);
    }

    #[test]
    fn test_buffer_too_small() {
        let mut buffer = [0u8; 2];

        assert_eq!(
            encode(&[1, 2, 3], &mut buffer),
            Err(EncodeError::BufferTooSmall {
                have: 2,
                want: encode_buffer(3)
            })
        );
        assert_eq!(
            decode(&[0x02, 0x41, 0x02, 0x42], &mut buffer),
            Err(DecodeError::BufferTooSmall { have: 2, want: 3 })
        );
    }
}
