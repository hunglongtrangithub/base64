use crate::{N, PAD_CHAR, TABLE};

/// Encode up to 3 bytes from input slice into 4 base64 indices in output slice.
/// `[0...8....16...]`
/// `[0..6..12..18..]`
/// Only care about the first 3 bytes. If input slice's length is less than 3,
/// consider the remaining bytes to be 0.
/// If input slice is empty, all bytes of the output slice are set to be [`N`].
/// After this function, all bytes of the output slice are always less than or
/// equal to [`N`].
/// # Arguments
/// * `input_slice` - A slice of input bytes.
/// * `output_slice` - A mutable array slice to hold the encoded base64 indices.
fn encode_3_bytes(input_slice: &[u8], output_slice: &mut [u8; 4]) {
    let input_slice = &input_slice[..input_slice.len().min(3)];
    let mask_6_bit = 0b0011_1111;

    match input_slice.len() {
        3 => {
            output_slice[0] = input_slice[0] >> 2;
            output_slice[1] = (input_slice[0] << 4) & mask_6_bit | (input_slice[1] >> 4);
            output_slice[2] = (input_slice[1] << 2) & mask_6_bit | (input_slice[2] >> 6);
            output_slice[3] = input_slice[2] & mask_6_bit;
        }
        2 => {
            output_slice[0] = input_slice[0] >> 2;
            output_slice[1] = (input_slice[0] << 4) & mask_6_bit | (input_slice[1] >> 4);
            output_slice[2] = (input_slice[1] << 2) & mask_6_bit;
            output_slice[3] = N;
        }
        1 => {
            output_slice[0] = input_slice[0] >> 2;
            output_slice[1] = (input_slice[0] << 4) & mask_6_bit;
            output_slice[2] = N;
            output_slice[3] = N;
        }
        // input slice is empty. All output bytes set to N.
        _ => {
            output_slice[0] = N;
            output_slice[1] = N;
            output_slice[2] = N;
            output_slice[3] = N;
        }
    }
}

/// Encode input bytes into base64 bytes.
fn encode_bytes(input_bytes: &[u8]) -> Box<[u8]> {
    let (chunks, remainder) = input_bytes.as_chunks::<3>();

    let output_len = if remainder.is_empty() {
        4 * chunks.len()
    } else {
        4 * chunks.len() + 4
    };
    let mut output_bytes = vec![0u8; output_len];

    // Scratch buffer to hold encoded 4 bytes
    let mut output_chunk = [0u8; 4];

    for i in 0..chunks.len() {
        encode_3_bytes(&chunks[i], &mut output_chunk);
        output_bytes[4 * i..4 * i + 4].copy_from_slice(&output_chunk);
    }

    if !remainder.is_empty() {
        encode_3_bytes(remainder, &mut output_chunk);
        output_bytes[4 * chunks.len()..].copy_from_slice(&output_chunk);
    }

    (0..output_len).for_each(|i| {
        let table_index = output_bytes[i] as usize;
        output_bytes[i] = *TABLE.get(table_index).unwrap_or(&PAD_CHAR);
    });

    output_bytes.into_boxed_slice()
}

/// Encode input string into base64 string.
pub fn encode_string(input_string: &str) -> String {
    let input_bytes = input_string.as_bytes();
    let output_bytes = encode_bytes(input_bytes);
    String::from_utf8_lossy(&output_bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode_bytes() {
        assert_eq!(encode_bytes(&[]).as_ref(), b"");
        assert_eq!(encode_bytes(b"f").as_ref(), b"Zg==");
    }
}
