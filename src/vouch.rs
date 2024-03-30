use crate::constparse::named_u64;
use crate::constparse::parse_hex;

/// The vouching multiplier is xor-ed with this constant.
pub const VOUCHING_TAG: u64 = named_u64("Vouching");

/// Returns the voucher representation of `value`, given the vouching
/// parameters `offset` and `scale`.
///
/// The result is always checked with the `checking` parameters, `(unoffset, unscale)`.
#[must_use]
#[inline(always)]
pub const fn vouch(offset: u64, scale: u64, checking: (u64, u64), value: u64) -> u64 {
    let ret = value
        .wrapping_add(offset)
        .wrapping_mul(scale ^ VOUCHING_TAG);

    // This only fails when the parameters are invalid.
    assert!(
        crate::check::check(checking.0, checking.1, value, ret),
        "failed to check voucher; parameters incorrect."
    );
    ret
}

pub const REPRESENTATION_BYTE_COUNT: usize = 73;

pub const fn parse_bytes(bytes: &[u8]) -> Result<(u64, u64, (u64, u64)), &'static str> {
    // Expected length:
    //  "VOUCH-"     [ 0,  6)
    //  hex offset   [ 6, 22)
    //  "-"          [22, 23)
    //  hex scale    [23, 39)
    //  "-"          [39, 40)
    //  hex unoffset [40, 56)
    //  "-"          [56, 57)
    //  hex unscale  [57, 73)

    if bytes.len() < REPRESENTATION_BYTE_COUNT {
        return Err("Too few bytes in serialized raffle::VouchingParameters");
    }

    if bytes.len() > REPRESENTATION_BYTE_COUNT {
        return Err("Too many bytes in serialized raffle::VouchingParameters");
    }

    if bytes[0] != b'V'
        || bytes[1] != b'O'
        || bytes[2] != b'U'
        || bytes[3] != b'C'
        || bytes[4] != b'H'
        || bytes[5] != b'-'
    {
        return Err("Incorrect prefix for serialized raffle::VouchingParameters. Expected VOUCH-");
    }

    let Some(offset) = parse_hex(bytes, 6) else {
        return Err("Failed to parse hex offset in serialized raffle::VouchingParameters.");
    };

    if bytes[22] != b'-' {
        return Err("Missing dash separator after offset in serialized raffle::VouchingParameters");
    }

    let Some(scale) = parse_hex(bytes, 23) else {
        return Err("Failed to parse hex scale in serialized raffle::VouchingParameters.");
    };

    if bytes[39] != b'-' {
        return Err("Missing dash separator after scale in serialized raffle::VouchingParameters");
    }

    let Some(unoffset) = parse_hex(bytes, 40) else {
        return Err("Failed to parse hex unoffset in serialized raffle::VouchingParameters.");
    };

    if bytes[56] != b'-' {
        return Err(
            "Missing dash separator after unoffset in serialized raffle::VouchingParameters",
        );
    }

    let Some(unscale) = parse_hex(bytes, 57) else {
        return Err("Failed to parse hex unscale in serialized raffle::VouchingParameters.");
    };

    Ok((offset, scale, (unoffset, unscale)))
}

#[test]
fn test_parse_bytes() {
    assert_eq!(
        parse_bytes(
            format!(
                "VOUCH-{:016x}-{:016x}-{:016x}-{:016x}",
                1234, 5678, 987, 432
            )
            .as_bytes()
        ),
        Ok((1234, 5678, (987, 432)))
    );
    // Wrong prefix
    assert!(parse_bytes(
        format!(
            "CHECK-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());

    // Wrong format
    assert!(parse_bytes(format!("CHECK-{:016x}-{:016x}", 1234, 5678).as_bytes()).is_err());

    // Too short
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}-{:016x}-{:015x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    // Too long
    assert!(parse_bytes(
        format!("VOUCH-{:016x}-{:016x}-{:016x}-{:017}", 1234, 5678, 987, 432).as_bytes()
    )
    .is_err());
}

#[test]
fn test_parse_bytes_fail_prefix() {
    assert!(parse_bytes(
        format!(
            "OOUCH-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VUUCH-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOCUH-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUDH-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCC-{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH.{:016x}-{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
}

#[test]
fn test_parse_bytes_fail_hyphens() {
    // Bad hyphens
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}.{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}.{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}-{:016x}.{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
}

#[test]
fn test_parse_bytes_fail_hex() {
    // Bad hex
    assert!(parse_bytes(
        format!(
            "VOUCH-{:015x}--{:016x}-{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:015x}--{:016x}-{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}-{:015x}--{:016x}",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
    assert!(parse_bytes(
        format!(
            "VOUCH-{:016x}-{:016x}-{:016x}-{:015x}-",
            1234, 5678, 987, 432
        )
        .as_bytes()
    )
    .is_err());
}
