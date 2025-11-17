use std::io::{Read, Write};

const N: u8 = 64;
const TABLE: &[u8; N as usize] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD_CHAR: u8 = b'=';

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
fn encode_string(input_string: &str) -> String {
    let input_bytes = input_string.as_bytes();
    let output_bytes = encode_bytes(input_bytes);
    String::from_utf8_lossy(&output_bytes).to_string()
}

/// Get the index of input base64 character in the base64 table.
/// If the input character is not in the base64 table, return None.
fn get_table_index(input_char: u8) -> Option<u8> {
    match input_char {
        b'A'..=b'z' => TABLE[0..52]
            .binary_search(&input_char)
            .map(|i| i as u8)
            .ok(),
        b'0'..=b'9' => TABLE[52..62]
            .binary_search(&input_char)
            .map(|i| (i + 52) as u8)
            .ok(),
        b'+' => Some(N - 2),
        b'/' => Some(N - 1),
        _ => None,
    }
}

/// Decode up to 4 base64 characters from input slice into up to 3 bytes in output slice.
/// `[0..6..12..18..]`
/// `[0...8...16....]`
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
fn decode_4_bytes(input_slice: &[u8], is_last: bool, output_slice: &mut [u8; 3]) -> Option<usize> {
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
                // If any character is invalid, return None
                index_slice[i] = get_table_index(input_slice[i])?;
            }

            output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
            output_slice[1] = (index_slice[1] << 4) | (index_slice[2] >> 2);
            output_slice[2] = (index_slice[2] << 6) | index_slice[3];
            Some(3)
        }
        _ => {
            if !is_last {
                // If not the last chunk, input length must be 4
                return None;
            }

            for i in 0..input_len {
                index_slice[i] = get_table_index(input_slice[i])?;
            }

            match input_len {
                3 => {
                    output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
                    output_slice[1] = (index_slice[1] << 4) | (index_slice[2] >> 2);
                    Some(2)
                }
                2 => {
                    output_slice[0] = (index_slice[0] << 2) | (index_slice[1] >> 4);
                    Some(1)
                }
                0 => Some(0),
                // Only one base64 character. Not enough to form a byte.
                _ => None,
            }
        }
    }
}

fn decode_bytes(input_bytes: &[u8]) -> Option<Box<[u8]>> {
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
    Some(output_bytes.into_boxed_slice())
}

fn decode_string(input_string: &str) -> Option<Box<[u8]>> {
    let input_bytes = input_string.as_bytes();
    decode_bytes(input_bytes)
}

fn main() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let mut stdin = std::io::stdin();

    println!("Base64 Encoder/Decoder");
    print!("Type your input first: ");
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;
    input.pop(); // Remove new-line char

    loop {
        print!("Encode (1) or Decode (2): ");
        stdout.flush()?;

        let mut choice = [0u8];
        stdin.read_exact(&mut choice)?;

        let first_char = choice[0];
        match first_char {
            b'1' => {
                println!(
                    "Encoding input: {}. Number of bytes: {}",
                    input,
                    input.len()
                );
                let encoded_string = encode_string(&input);
                println!("Encoded string: {}", encoded_string);
                break;
            }
            b'2' => {
                println!(
                    "Decoding input: {}. Number of bytes: {}",
                    input,
                    input.len()
                );
                let decoded_string = decode_string(&input);
                match decoded_string {
                    Some(bytes) => {
                        let decoded_str = String::from_utf8_lossy(&bytes);
                        println!("Decoded string: {}", decoded_str);
                    }
                    None => {
                        println!("Invalid base64 input string.");
                    }
                }
                break;
            }
            _ => {
                println!("Invalid input. Try again.");
                continue;
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let mut i = 0;

        // Uppercase alphabet
        for c in b'A'..=b'Z' {
            assert_eq!(TABLE[i], c);
            i += 1;
        }

        // Lowercase alphabet
        for c in b'a'..=b'z' {
            assert_eq!(TABLE[i], c);
            i += 1;
        }

        // Digits
        for c in b'0'..=b'9' {
            assert_eq!(TABLE[i], c);
            i += 1;
        }

        // '+' and '/'
        for c in b"+/" {
            assert_eq!(TABLE[i], *c);
            i += 1;
        }
    }

    #[test]
    fn test_encode_bytes() {
        assert_eq!(encode_bytes(&[]).as_ref(), b"");
        assert_eq!(encode_bytes(b"f").as_ref(), b"Zg==");
    }
}
