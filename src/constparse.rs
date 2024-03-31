/// This module exposes const-fn methods to convert bytes and string-as-bytes
/// to u64 numbers at compile-time.

/// Interprets the first up to 8 characters in `name` as a little-endian u64.
pub const fn named_u64(name: &[u8; 8], expected: u64) -> u64 {
    let ret = u64::from_le_bytes(*name);
    assert!(ret == expected);
    ret
}

/// Parses ASCII encoded big-endian hex (e.g., the result of
/// formatting an integer to hex) to a u64 value.
///
/// Returns None on parse failure.
pub const fn parse_hex(bytes: &[u8], base: usize) -> Option<u64> {
    const fn update(acc: Option<u64>, bytes: &[u8], base: usize, idx: usize) -> Option<u64> {
        if base >= bytes.len() || idx >= bytes.len() - base {
            return None;
        }

        if let Some(acc) = acc {
            let byte = bytes[base + idx];
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => 10 + (byte - b'a'),
                b'A'..=b'F' => 10 + (byte - b'A'),
                _ => return None,
            };
            Some(acc + (digit as u64).wrapping_shl((4 * (15 - idx)) as u32))
        } else {
            None
        }
    }

    let mut acc = Some(0u64);
    acc = update(acc, bytes, base, 0);
    acc = update(acc, bytes, base, 1);
    acc = update(acc, bytes, base, 2);
    acc = update(acc, bytes, base, 3);
    acc = update(acc, bytes, base, 4);
    acc = update(acc, bytes, base, 5);
    acc = update(acc, bytes, base, 6);
    acc = update(acc, bytes, base, 7);
    acc = update(acc, bytes, base, 8);
    acc = update(acc, bytes, base, 9);
    acc = update(acc, bytes, base, 10);
    acc = update(acc, bytes, base, 11);
    acc = update(acc, bytes, base, 12);
    acc = update(acc, bytes, base, 13);
    acc = update(acc, bytes, base, 14);
    acc = update(acc, bytes, base, 15);

    acc
}

#[test]
fn test_named_u64() {
    // These are the three strings we care about.
    assert_eq!(u64::from_le_bytes(*b"Vouch!OK"), 0x4b4f216863756f56u64);
    assert_eq!(
        named_u64(b"Vouch!OK", 0x4b4f216863756f56u64),
        u64::from_le_bytes(*b"Vouch!OK")
    );

    assert_eq!(u64::from_le_bytes(*b"Checking"), 0x676e696b63656843u64);
    assert_eq!(
        named_u64(b"Checking", 0x676e696b63656843u64),
        u64::from_le_bytes(*b"Checking")
    );

    assert_eq!(u64::from_le_bytes(*b"Vouching"), 0x676e696863756f56u64);
    assert_eq!(
        named_u64(b"Vouching", 0x676e696863756f56u64),
        u64::from_le_bytes(*b"Vouching")
    );
}

#[test]
fn test_parse_hex() {
    assert_eq!(parse_hex(format!("{:016x}", 42).as_bytes(), 0), Some(42));
    assert_eq!(parse_hex(format!("--{:016x}", 42).as_bytes(), 2), Some(42));
    assert_eq!(
        parse_hex(format!("{:016x}", u64::MAX).as_bytes(), 0),
        Some(u64::MAX)
    );
    assert_eq!(
        parse_hex(format!("{:016x}", 0x123456789abcdef0u64).as_bytes(), 0),
        Some(0x123456789abcdef0)
    );
    assert_eq!(
        parse_hex(format!("{:016X}", 0x123456789abcdef0u64).as_bytes(), 0),
        Some(0x123456789abcdef0)
    );
    assert_eq!(
        parse_hex(
            format!("VOUCH-{:016x}", 0xa0b1c2d3e4f56789u64).as_bytes(),
            6
        ),
        Some(0xa0b1c2d3e4f56789)
    );
}

#[test]
fn test_parse_hex_bad() {
    assert_eq!(parse_hex(format!("{:016x}", 42).as_bytes(), 1), None);
    assert_eq!(parse_hex(format!("{:016x}", 42).as_bytes(), 16), None);
    assert_eq!(parse_hex(format!("{:015x}g", 42).as_bytes(), 0), None);
    assert_eq!(parse_hex(format!("x{:015x}", 42).as_bytes(), 0), None);
}
