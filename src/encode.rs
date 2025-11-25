use crate::{N, PAD_CHAR, TABLE};

const MASK_6_BITS: u8 = 0b0011_1111;

/// Encode input bytes into base64 bytes.
fn encode_bytes(input_bytes: &[u8]) -> Box<[u8]> {
    let (chunks, remainder) = input_bytes.as_chunks::<3>();

    // Calculate output length
    let output_len = if remainder.is_empty() {
        4 * chunks.len()
    } else {
        4 * chunks.len() + 4
    };
    let mut output_bytes = Box::<[u8]>::new_uninit_slice(output_len);

    // Process each chunk of 3 bytes
    for (i, chunk) in chunks.iter().enumerate() {
        let start_idx = 4 * i;
        output_bytes[start_idx].write(chunk[0] >> 2);
        output_bytes[start_idx + 1].write((chunk[0] << 4) & MASK_6_BITS | (chunk[1] >> 4));
        output_bytes[start_idx + 2].write((chunk[1] << 2) & MASK_6_BITS | (chunk[2] >> 6));
        output_bytes[start_idx + 3].write(chunk[2] & MASK_6_BITS);
    }

    // Process remainder bytes
    match remainder.len() {
        // Skip if no remainder
        0 => {}
        1 => {
            let start_idx = 4 * chunks.len();
            output_bytes[start_idx].write(remainder[0] >> 2);
            output_bytes[start_idx + 1].write((remainder[0] << 4) & MASK_6_BITS);
            output_bytes[start_idx + 2].write(N);
            output_bytes[start_idx + 3].write(N);
        }
        2 => {
            let start_idx = 4 * chunks.len();
            output_bytes[start_idx].write(remainder[0] >> 2);
            output_bytes[start_idx + 1]
                .write((remainder[0] << 4) & MASK_6_BITS | (remainder[1] >> 4));
            output_bytes[start_idx + 2].write((remainder[1] << 2) & MASK_6_BITS);
            output_bytes[start_idx + 3].write(N);
        }
        // Can only be length 0, 1, or 2. Guaranteed by as_chunks.
        _ => unreachable!(),
    }

    // SAFETY: All elements of output_bytes have been initialized.
    let mut output_bytes = unsafe { output_bytes.assume_init() };

    // Map 6-bit values to base64 characters
    (0..output_len).for_each(|i| {
        let table_index = output_bytes[i] as usize;
        output_bytes[i] = *TABLE.get(table_index).unwrap_or(&PAD_CHAR);
    });

    output_bytes
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
        for (expected, input) in cases {
            let encoded = encode_bytes(input);
            assert_eq!(&encoded[..], *expected);
        }
    }
}
