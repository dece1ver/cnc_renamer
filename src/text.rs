use std::fs;
use std::io;
use std::path::Path;

use encoding_rs::{UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1251};

/// Читает файл УП и декодирует его с поддержкой кодировок Windows.
pub fn read_program(path: &Path) -> io::Result<String> {
    Ok(decode_bytes(&fs::read(path)?))
}

/// Декодирует байты УП в строку.
///
/// Порядок: BOM (UTF-16LE/BE, UTF-8) → валидный UTF-8 → CP1251.
pub fn decode_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode(UTF_16LE, &bytes[2..]);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode(UTF_16BE, &bytes[2..]);
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return decode(UTF_8, &bytes[3..]);
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    decode(WINDOWS_1251, bytes)
}

fn decode(encoding: &'static encoding_rs::Encoding, bytes: &[u8]) -> String {
    let (cow, _, _) = encoding.decode(bytes);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_passthrough() {
        assert_eq!(decode_bytes(b"N10 G0 X0\n"), "N10 G0 X0\n");
    }

    #[test]
    fn utf8_bom_stripped() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"O0001(NAME)");
        assert_eq!(decode_bytes(&bytes), "O0001(NAME)");
    }

    #[test]
    fn cp1251_cyrillic_decoded() {
        // "T1 M6(ФРЕЗА D16)" в CP1251
        let bytes = b"T1 M6(\xd4\xd0\xc5\xc7\xc0 D16)";
        assert_eq!(decode_bytes(bytes), "T1 M6(ФРЕЗА D16)");
    }

    #[test]
    fn utf16le_bom_decoded() {
        let mut bytes = vec![0xFF, 0xFE];
        for u in "T1 M6(DRILL)".encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        assert_eq!(decode_bytes(&bytes), "T1 M6(DRILL)");
    }
}
