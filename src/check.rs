/// This module implements the voucher checking logic.
use crate::constparse::named_u64;
use crate::constparse::parse_hex;

/// The vouching and checking transform is such that
///   x + check(vouch(x)) == WANTED_SUM
pub const WANTED_SUM: u64 = named_u64(b"Vouch!OK", 0x4b4f216863756f56u64);

/// The checking multiplier is xor-ed with this other constant.
pub const CHECKING_TAG: u64 = named_u64(b"Checking", 0x676e696b63656843u64);

/// Determines whether the `voucher` value was generated for
/// `expected`, and with vouching parameters that correspond to the
/// checking parameters `unoffset` and `unscale`.
///
/// Returns true on match, and false on mismatch.
#[must_use]
#[inline(always)]
pub const fn check(unoffset: u64, unscale: u64, expected: u64, voucher: u64) -> bool {
    let unvouched_value = voucher
        .wrapping_add(unoffset)
        .wrapping_mul(unscale ^ CHECKING_TAG);

    unvouched_value.wrapping_add(expected) == WANTED_SUM
}

pub const REPRESENTATION_BYTE_COUNT: usize = 39;

/// Parses the `bytes` as the serialised ASCII representation of checking parameters.
///
/// Returns a pair of `(unoffset, unscale)` on success or a failure reason string.
pub const fn parse_bytes(bytes: &[u8]) -> Result<(u64, u64), &'static str> {
    // Expected length:
    //  "CHECK-"     [ 0,  6)
    //  hex unoffset [ 6, 22)
    //  "-"          [22, 23)
    //  hex unscale  [23, 39)

    if bytes.len() < REPRESENTATION_BYTE_COUNT {
        return Err("Too few bytes in serialized raffle::CheckingParameters");
    }

    if bytes.len() > REPRESENTATION_BYTE_COUNT {
        return Err("Too many bytes in serialized raffle::CheckingParameters");
    }

    if bytes[0] != b'C'
        || bytes[1] != b'H'
        || bytes[2] != b'E'
        || bytes[3] != b'C'
        || bytes[4] != b'K'
        || bytes[5] != b'-'
    {
        return Err("Incorrect prefix for raffle::CheckingParameters. Expected CHECK-");
    }

    let Some(unoffset) = parse_hex(bytes, 6) else {
        return Err("Failed to parse hex unoffset in raffle::CheckingParameters.");
    };

    if bytes[22] != b'-' {
        return Err("Missing dash separator after unoffset in raffle::CheckingParameters");
    }

    let Some(unscale) = parse_hex(bytes, 23) else {
        return Err("Failed to parse hex uscale in raffle::CheckingParameters.");
    };

    Ok((unoffset, unscale))
}

#[test]
fn test_parse_bytes() {
    assert_eq!(
        parse_bytes(format!("CHECK-{:016x}-{:016x}", 1234, 5678).as_bytes()),
        Ok((1234, 5678))
    );
    // Too long
    assert!(parse_bytes(format!("CHECK-{:016x}-{:016x}-suffix", 1234, 5678).as_bytes()).is_err());
    // Too short
    assert!(parse_bytes(format!("CHECK-{:016x}-{:015x}", 1234, 5678).as_bytes()).is_err());
    // Bad prefix
    assert!(parse_bytes(format!("VOUCH-{:016x}-{:016x}-", 1234, 5678).as_bytes()).is_err());

    assert!(parse_bytes(format!("CHEC-{:016x}-{:016x}-", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("AHECK-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CCECK-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHHCK-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHEKK-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHEKC-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHECK.{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());

    // Wrong format
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    // Bad dashes
    assert!(parse_bytes(format!("CHECK-{:016x}{:016x}-", 1234, 5678).as_bytes()).is_err());
    // Wrong hex length
    assert!(parse_bytes(format!("CHECK-{:015x}-{:017x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHECK-{:017x}-{:015x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHECK-{:016x}-{:017x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHECK-{:016x}-{:015x}", 1234, 5678).as_bytes()).is_err());
    assert!(parse_bytes(format!("CHECK-{:016x}-{:015x}-", 1234, 5678).as_bytes()).is_err());
}
