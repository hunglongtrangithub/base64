use crate::{PAD_CHAR, get_table_index};

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The input length (after trimming padding) is invalid for decoding.
    /// This occurs when the length mod 4 is 1 (after trimming padding).
    InputLength,
    /// Padding character found in a non-final chunk, or incorrect amount of
    /// trailing padding characters for the final chunk.
    WrongPadding,
    /// An invalid base64 character was encountered (byte value returned).
    InvalidByte(u8),
}

/// Decode input base64 bytes into original bytes.
/// Returns `None` if the input is invalid.
fn decode_bytes(input_bytes: &[u8]) -> Result<Box<[u8]>, DecodeError> {
    // Trim trailing padding characters first
    let (input_bytes, trailing_len) = {
        let mut end = input_bytes.len();
        while end > 0 {
            if input_bytes[end - 1] == PAD_CHAR {
                end -= 1;
            } else {
                break;
            }
        }
        (&input_bytes[..end], input_bytes.len() - end)
    };

    let (chunks, remainder) = input_bytes.as_chunks::<4>();

    // Calculate output length
    let output_len = match remainder.len() {
        // No remainder bytes, output length only from full chunks
        0 => 3 * chunks.len(),
        // Only one base64 character left. Not enough to form a byte.
        1 => return Err(DecodeError::InputLength),
        2 => {
            if trailing_len < 2 {
                // Need at least 2 padding characters
                return Err(DecodeError::WrongPadding);
            }
            // Two 6-bit values forms 1 byte ((2 * 6) / 8 = 1)
            3 * chunks.len() + 1
        }
        3 => {
            if trailing_len < 1 {
                // Need at least 1 padding character
                return Err(DecodeError::WrongPadding);
            }
            // Three 6-bit values forms 2 bytes ((3 * 6) / 8 = 2)
            3 * chunks.len() + 2
        }
        // Can only be length 0, 1, 2, or 3. Guaranteed by as_chunks.
        _ => unreachable!(),
    };
    let mut output_bytes = Box::<[u8]>::new_uninit_slice(output_len);

    // Helper closure to return table index or invalid byte error
    let get_index = |b: u8| -> Result<u8, DecodeError> {
        get_table_index(b).ok_or(DecodeError::InvalidByte(b))
    };

    // Process each chunk of 4 bytes
    for (idx, chunk) in chunks.iter().enumerate() {
        if chunk.contains(&PAD_CHAR) {
            return Err(DecodeError::WrongPadding);
        }
        // Chunk is valid, decode all 4 bytes
        let byte0 = (get_index(chunk[0])? << 2) | (get_index(chunk[1])? >> 4);
        let byte1 = (get_index(chunk[1])? << 4) | (get_index(chunk[2])? >> 2);
        let byte2 = (get_index(chunk[2])? << 6) | get_index(chunk[3])?;
        let start_idx = 3 * idx;
        output_bytes[start_idx].write(byte0);
        output_bytes[start_idx + 1].write(byte1);
        output_bytes[start_idx + 2].write(byte2);
    }

    // Process remainder bytes
    match remainder.len() {
        0 => {}
        1 => return Err(DecodeError::InputLength),
        2 => {
            let start_index = 3 * chunks.len();
            let byte0 = (get_index(remainder[0])? << 2) | (get_index(remainder[1])? >> 4);
            output_bytes[start_index].write(byte0);
        }
        3 => {
            let start_index = 3 * chunks.len();
            let byte0 = (get_index(remainder[0])? << 2) | (get_index(remainder[1])? >> 4);
            let byte1 = (get_index(remainder[1])? << 4) | (get_index(remainder[2])? >> 2);
            output_bytes[start_index].write(byte0);
            output_bytes[start_index + 1].write(byte1);
        }
        _ => unreachable!(),
    };

    // SAFETY: All elements of output_bytes have been initialized.
    let output_bytes = unsafe { output_bytes.assume_init() };

    // Truncate output bytes to actual output length
    Ok(output_bytes)
}

/// Decode input base64 string into original string.
/// This function tries to decode the input string as UTF-8 after decoding the base64 bytes.
/// Replacement characters will be used for invalid UTF-8 sequences.
/// Returns `None` if the input is invalid.
pub fn decode_string(input_string: &str) -> Result<String, DecodeError> {
    let input_bytes = input_string.as_bytes();
    let output_bytes = decode_bytes(input_bytes)?;
    Ok(String::from_utf8_lossy(&output_bytes).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decode_valid_lengths() {
        // Valid base64 encodings for 'a' repeated lengths 0..9
        let cases: &[(&[u8], &[u8])] = &[
            (b"", b""),
            (b"YQ==", b"a"),
            (b"YWE=", b"aa"),
            (b"YWFh", b"aaa"),
            (b"YWFhYQ==", b"aaaa"),
            (b"YWFhYWE=", b"aaaaa"),
            (b"YWFhYWFh", b"aaaaaa"),
            (b"YWFhYWFhYQ==", b"aaaaaaa"),
            (b"YWFhYWFhYWE=", b"aaaaaaaa"),
            (b"YWFhYWFhYWFh", b"aaaaaaaaa"),
        ];

        for (enc, expected) in cases {
            let got = decode_bytes(enc)
                .unwrap_or_else(|e| panic!("Decoding failed for {:?}: {:?}", enc, e));
            assert_eq!(&*got, *expected);
        }
    }

    #[test]
    fn test_decode_valid_with_padding() {
        assert_eq!(decode_bytes(b"Zig=="), decode_bytes(b"Zig==="));
    }

    #[test]
    fn test_decode_invalid_byte() {
        assert_eq!(decode_bytes(b"Zig!"), Err(DecodeError::InvalidByte(b'!')));
        assert_eq!(decode_bytes(b"Zig!"), Err(DecodeError::InvalidByte(b'!')));
    }

    #[test]
    fn test_decode_wrong_padding_in_middle() {
        assert_eq!(decode_bytes(b"ab==cdef"), Err(DecodeError::WrongPadding));
        assert_eq!(decode_bytes(b"abcd==ef"), Err(DecodeError::WrongPadding));
        assert_eq!(decode_bytes(b"abcdef="), Err(DecodeError::WrongPadding));
        assert_eq!(decode_bytes(b"abcdefg"), Err(DecodeError::WrongPadding));
    }

    #[test]
    fn test_decode_invalid_length_single_char() {
        assert_eq!(decode_bytes(b"a"), Err(DecodeError::InputLength));
        assert_eq!(decode_bytes(b"abcde"), Err(DecodeError::InputLength));
    }
}
