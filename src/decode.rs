use crate::{PAD_CHAR, get_table_index};

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The input length (after trimming padding) is invalid for decoding.
    InvalidLength,
    /// An invalid base64 character was encountered (byte value returned).
    InvalidByte(u8),
}

/// Decode up to 4 base64 characters from input slice into up to 3 bytes in output slice.
///
/// `[0..6..12..18..]`
///
/// `[0...8...16....]`
///
/// Only care about the first 4 bytes. If input slice's length is less than 4, consider the
/// remaining bytes to be padding characters (`=`).
///
/// # Arguments
/// * `input_slice` - A slice of input base64 characters (as bytes).
/// * `is_last` - A boolean indicating whether this is the last chunk to decode.
/// * `output_slice` - A mutable array slice to hold the decoded bytes.
///
/// # Returns
/// An `Option<usize>` indicating the number of bytes written to output slice,
/// or `None` if the input is invalid.
/// * `Some(n)` - Number of bytes written to output slice (0 <= n <= 3).
/// * `None` - Invalid input:
///   - Invalid base64 character found in the input slice (except trailing padding character).
///   - Input slice length (excluding trailing padding characters) less than 4 and not the last chunk.
///   - Input length of 1 (not enough to form a byte).
fn decode_4_bytes(
    input_slice: &[u8],
    is_last: bool,
    output_slice: &mut [u8; 3],
) -> Result<usize, DecodeError> {
    let input_slice = &input_slice[..input_slice.len().min(4)];
    let input_len = {
        let mut input_len = input_slice.len();
        while input_len > 0 {
            if input_slice[input_len - 1] == PAD_CHAR {
                input_len -= 1;
            } else {
                break;
            }
        }
        input_len
    };

    // Hold the converted table indices
    let index_slice = &mut [0u8; 4];

    match input_len {
        4 => {
            for i in 0..4 {
                // If any character is invalid, return InvalidByte with the offending byte
                index_slice[i] = match get_table_index(input_slice[i]) {
                    Some(b) => b,
                    None => return Err(DecodeError::InvalidByte(input_slice[i])),
                };
            }

            output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
            output_slice[1] = (index_slice[1] << 4) | (index_slice[2] >> 2);
            output_slice[2] = (index_slice[2] << 6) | index_slice[3];
            Ok(3)
        }
        _ => {
            if !is_last {
                // If not the last chunk, input length must be 4
                return Err(DecodeError::InvalidLength);
            }

            for i in 0..input_len {
                index_slice[i] = match get_table_index(input_slice[i]) {
                    Some(b) => b,
                    None => return Err(DecodeError::InvalidByte(input_slice[i])),
                };
            }

            match input_len {
                3 => {
                    output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
                    output_slice[1] = (index_slice[1] << 4) | (index_slice[2] >> 2);
                    Ok(2)
                }
                2 => {
                    output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
                    Ok(1)
                }
                0 => Ok(0),
                // Only one base64 character. Not enough to form a byte.
                _ => Err(DecodeError::InvalidLength),
            }
        }
    }
}

/// Decode input base64 bytes into original bytes.
/// Returns `None` if the input is invalid.
fn decode_bytes(input_bytes: &[u8]) -> Result<Box<[u8]>, DecodeError> {
    let (chunks, remainder) = input_bytes.as_chunks::<4>();

    let output_len = if remainder.is_empty() {
        3 * chunks.len()
    } else {
        3 * chunks.len() + 3
    };
    let mut output_bytes = vec![0u8; output_len];

    // Scratch buffer to hold decoded 3 bytes
    let mut output_chunk = [0u8; 3];
    // Current index showing the number of bytes written to output_bytes
    let mut output_index = 0;

    for (idx, chunk) in chunks.iter().enumerate() {
        let is_last = remainder.is_empty() && idx == chunks.len() - 1;
        let num_bytes = decode_4_bytes(chunk, is_last, &mut output_chunk)?;
        output_bytes[output_index..output_index + num_bytes]
            .copy_from_slice(&output_chunk[..num_bytes]);
        output_index += num_bytes;
    }

    if !remainder.is_empty() {
        let num_bytes = decode_4_bytes(remainder, true, &mut output_chunk)?;
        output_bytes[output_index..output_index + num_bytes]
            .copy_from_slice(&output_chunk[..num_bytes]);
    }

    output_bytes.truncate(output_index);
    Ok(output_bytes.into_boxed_slice())
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
    fn test_encode_bytes() {
        assert_eq!(decode_bytes(b"").ok().as_deref(), Some(b"".as_ref()));
        assert_eq!(decode_bytes(b"Zg==").ok().as_deref(), Some(b"f".as_ref()));
    }
}
