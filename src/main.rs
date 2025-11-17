mod decode;
mod encode;

use std::io::{Read, Write};

use crate::decode::decode_string;
use crate::encode::encode_string;

const N: u8 = 64;
const TABLE: &[u8; N as usize] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD_CHAR: u8 = b'=';

/// Get the index of input base64 character in the base64 table.
/// The returned index is in the range `[0, 63]`.
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
                    Some(decoded_str) => {
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
    fn test_get_table_index_valid_chars() {
        let mut i = 0u8;

        // Uppercase alphabet
        for c in b'A'..=b'Z' {
            assert_eq!(get_table_index(c).unwrap(), i);
            i += 1;
        }

        // Lowercase alphabet
        for c in b'a'..=b'z' {
            assert_eq!(get_table_index(c).unwrap(), i);
            i += 1;
        }

        // Digits
        for c in b'0'..=b'9' {
            assert_eq!(get_table_index(c).unwrap(), i);
            i += 1;
        }

        assert_eq!(get_table_index(b'+').unwrap(), 62);
        assert_eq!(get_table_index(b'/').unwrap(), 63);
    }

    #[test]
    fn test_get_table_index_invalid_chars() {
        let invalid_chars = [b'=', b'!', b' ', b'\n', b'-', b'@', b'[', b'`', b'{', 255u8];
        for &c in &invalid_chars {
            assert!(get_table_index(c).is_none());
        }
    }
}
