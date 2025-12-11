// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EmptyInput,
    ZeroMarker { at: usize },
    ZeroBinary { at: usize },
    ChunkOverflow { at: usize, marker: u8, len: usize },
}

/// Computes the maximum size needed to COBS encode a blind input blob.
#[inline]
pub const fn encode_buffer(size: usize) -> usize {
    size + (size + 253) / 254 + 1
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
/// the number of bytes the encoding took. The output buffer is expected to have
/// enough space pre-allocated to hold everything.
pub fn encode(data: &[u8], encoded: &mut [u8]) -> usize {
    // The empty blob is always encoded as 0x01
    if data.is_empty() {
        encoded[0] = 0x01;
        return 1;
    }
    // Start pushing the bytes into the output array, skipping each marker byte
    // and backfilling it later
    let mut marker_pos = 0usize;
    let mut output_pos = 1usize;
    let mut run_length = 1u8;

    for &b in data {
        // If the next byte is non-zero, append it to the output
        if b > 0 {
            encoded[output_pos] = b;
            output_pos += 1;
            run_length += 1;

            // If en entire chunk was non-zero, mark and start the next chunk
            if run_length == 0xff {
                encoded[marker_pos] = run_length;
                marker_pos = output_pos;
                output_pos += 1;
                run_length = 1;
            }
        } else {
            // Next byte is zero, terminate the chunk and start the next chunk
            encoded[marker_pos] = run_length;
            marker_pos = output_pos;
            output_pos += 1;
            run_length = 1;
        }
    }
    // Terminate any unfinished chunk
    if run_length > 1 || data[data.len() - 1] == 0 {
        encoded[marker_pos] = run_length;
    } else {
        // Just finished at the chunk boundary, revert last open
        output_pos -= 1;
    }
    // Return the number of bytes written to the output stream
    output_pos
}

/// Decodes an opaque data blob with COBS using 0 as the sentinel value. Returns
/// the number of bytes the decoding took. The output buffer is expected to have
/// enough space pre-allocated to hold everything.
pub fn decode(data: &[u8], decoded: &mut [u8]) -> Result<usize, DecodeError> {
    // The empty blob is not a valid COBS encoding
    if data.is_empty() {
        return Err(DecodeError::EmptyInput);
    }
    // The empty text is always encoded as 0x01
    if data.len() == 1 && data[0] == 0x01 {
        return Ok(0);
    }
    // Consume the input stream one chunk at a time
    let mut output_pos = 0usize;
    let mut i = 0usize;

    while i < data.len() {
        // Zero cannot be part of a COBS encoded stream
        let marker = data[i];
        if marker == 0 {
            return Err(DecodeError::ZeroMarker { at: i - 1 });
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
            if data[i] == 0 {
                return Err(DecodeError::ZeroBinary { at: i });
            }
            decoded[output_pos] = data[i];
            output_pos += 1;
            i += 1;
        }
        if i < data.len() && marker != 0xff {
            decoded[output_pos] = 0;
            output_pos += 1;
        }
    }
    Ok(output_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_empty() {
        let data = [];
        let mut enc_buf = [0u8; 1];
        let len = encode(&data, &mut enc_buf);
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
        let len = encode(&data, &mut enc_buf);

        let mut dec_buf = [0u8; 5];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data);
    }

    #[test]
    fn test_roundtrip_with_zeros() {
        let data = [0, 1, 0, 2, 0, 0, 3];
        let mut enc_buf = [0u8; encode_buffer(7)];
        let len = encode(&data, &mut enc_buf);

        let mut dec_buf = [0u8; 7];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data);
    }

    #[test]
    fn test_roundtrip_254_nonzero() {
        let data: Vec<u8> = (1..=254).collect();
        let mut enc_buf = vec![0u8; encode_buffer(254)];
        let len = encode(&data, &mut enc_buf);

        let mut dec_buf = vec![0u8; 254];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data[..]);
    }

    #[test]
    fn test_roundtrip_255_nonzero() {
        let data: Vec<u8> = (1..=254).chain(std::iter::once(1)).collect();
        let mut enc_buf = vec![0u8; encode_buffer(255)];
        let len = encode(&data, &mut enc_buf);

        let mut dec_buf = vec![0u8; 255];
        let dec_len = decode(&enc_buf[..len], &mut dec_buf).unwrap();
        assert_eq!(&dec_buf[..dec_len], &data[..]);
    }
}
