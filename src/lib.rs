// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

/// Error types that can be returned from encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EncodeError {
    #[error("buffer too small: have {have} bytes, want {want} bytes")]
    BufferTooSmall { have: usize, want: usize },
}

/// Error types that can be returned from decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("empty input")]
    EmptyInput,
    #[error("buffer too small: have {have} bytes, want {want} bytes")]
    BufferTooSmall { have: usize, want: usize },
    #[error("zero marker at position {at}")]
    ZeroMarker { at: usize },
    #[error("zero byte in data at position {at}")]
    ZeroBinary { at: usize },
    #[error("chunk overflow at position {at}: chunk {marker} exceeds data length {len}")]
    ChunkOverflow { at: usize, marker: u8, len: usize },
}

/// Computes the maximum size needed to COBS encode a blind input blob.
#[inline]
pub const fn encode_buffer(size: usize) -> usize {
    size + size.div_ceil(254) + 1
}

/// Computes the maximum size needed to COBS decode a blind input data.
///
/// Note, 0 is not a valid CODE data size and the method will panic.
#[inline]
pub const fn decode_buffer(size: usize) -> usize {
    if size == 0 {
        panic!("size cannot be zero");
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
    Ok(encode_unsafe(data, encoded))
}

/// Encodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the encoding took.
///
/// # Safety
/// The caller must ensure `encoded` has at least `encode_buffer(data.len())` bytes.
#[inline]
pub fn encode_unsafe(data: &[u8], encoded: &mut [u8]) -> usize {
    // The empty blob is always encoded as 0x01
    if data.is_empty() {
        encoded[0] = 0x01;
        return 1;
    }
    // Sanity check in debug builds that the user called it correctly
    debug_assert!(encoded.len() >= encode_buffer(data.len()));

    // Start pushing the bytes into the output array, skipping each marker byte
    // and backfilling it later
    unsafe {
        let mut marker_pos = 0usize;
        let mut output_pos = 1usize;
        let mut run_length = 1u8;

        for &b in data {
            // If the next byte is non-zero, append it to the output
            if b > 0 {
                *encoded.get_unchecked_mut(output_pos) = b;
                output_pos += 1;
                run_length += 1;

                // If an entire chunk was non-zero, mark and start the next chunk
                if run_length == 0xff {
                    *encoded.get_unchecked_mut(marker_pos) = run_length;
                    marker_pos = output_pos;
                    output_pos += 1;
                    run_length = 1;
                }
            } else {
                // Next byte is zero, terminate the chunk and start the next chunk
                *encoded.get_unchecked_mut(marker_pos) = run_length;
                marker_pos = output_pos;
                output_pos += 1;
                run_length = 1;
            }
        }
        // Terminate any unfinished chunk
        let last_byte = *data.get_unchecked(data.len() - 1);
        if run_length > 1 || last_byte == 0 {
            *encoded.get_unchecked_mut(marker_pos) = run_length;
        } else {
            // Just finished at the chunk boundary, revert last open
            output_pos -= 1;
        }
        // Return the number of bytes written to the output stream
        output_pos
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
    decode_unsafe(data, decoded)
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the decoding took.
///
/// # Safety
/// The caller must ensure `decoded` has at least `decode_buffer(data.len())` bytes.
#[inline]
pub fn decode_unsafe(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
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
}
