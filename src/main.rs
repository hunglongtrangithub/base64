mod app;
mod decode;
mod encode;

const N: u8 = 64;
const TABLE: &[u8; N as usize] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD_CHAR: u8 = b'=';

/// Get the index of input base64 character in the base64 table.
/// The returned index is in the range `[0, 63]`.
/// If the input character is not in the base64 table, return None.
fn get_table_index(input_char: u8) -> Option<u8> {
    match input_char {
        // Uppercase letters
        b'A'..=b'Z' => Some(input_char - b'A'),
        // Lowercase letters
        b'a'..=b'z' => Some((input_char - b'a') + 26),
        // Digits
        b'0'..=b'9' => Some((input_char - b'0') + 52),
        // Special characters
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn main() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    app::setup_terminal(&mut stdout)?;
    let res = app::run(&mut stdout);
    app::restore_terminal(&mut stdout)?;
    res
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
