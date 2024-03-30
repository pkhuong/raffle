/// Generates pairs vouching and checking parameters.

/// Computes the modular inverse of (a | 1)  (mod 2**64).
const fn modinverse(a: u64) -> u64 {
    // Make sure `a` is odd, otherwise there's no inverse.
    let a = a | 1;
    // https://marc-b-reynolds.github.io/math/2017/09/18/ModInverse.html
    let mut x = a.wrapping_mul(3) ^ 2; // accurate to 5 bits

    // Repeat the Newton iteration 4 times, for 5 * 2**4 = 80 > 64 correct bits.
    x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));

    // Check that we indeed computed the modular inverse.
    assert!(a.wrapping_mul(x) == 1);
    x
}

/// Checks that the vouching and checking parameters are valid.
///
/// Vouching and then checking is the composition of two affine functions,
/// so another affine function.  This means we only need to check in two
/// points to confirm that the composition is the expected affine function:
/// we want `check(vouch(x)) == WANTED_SUM - x`.
///
/// We check in 4 points, just to be clear.
#[inline(never)] // make the function show up in profiles if it's an issue.
const fn check_parameters_or_die(offset: u64, scale: u64, checking: (u64, u64)) {
    use crate::vouch::vouch;

    // Each call to `vouch` internally checks that the voucher is correct.
    let _ = vouch(offset, scale, checking, 0);
    let _ = vouch(offset, scale, checking, 1);
    let _ = vouch(offset, scale, checking, 2);
    // and a "random" point.
    let _ = vouch(offset, scale, checking, 0x110d2ae90b38f555u64);
}

/// Given `scale`, the multiplier for the vouching step, and `unoffset`,
/// the addend for the checking step, computes matching vouching and
/// checking parameters.
///
/// Returns `(offset, scale, (unoffset, unscale))`, with the vouching
/// and checking tags applied.
#[inline(never)]
pub const fn derive_parameters(scale: u64, unoffset: u64) -> (u64, u64, (u64, u64)) {
    use crate::check::CHECKING_TAG;
    use crate::check::WANTED_SUM;
    use crate::vouch::VOUCHING_TAG;

    let scale = scale | 1; // scale must be odd
    let unscale = modinverse(scale).wrapping_neg(); // scale * unscale == -1

    // We want
    //    x + unscale * ([scale * (x + offset)] + unoffset)           == WANTED_SUM
    // == x + (unscale * scale) * (x + offset) + (unscale * unoffset)
    // == x - x - offset + (unscale * unoffset)
    // == -offset + (unscale * unoffset)
    //
    // offset = (unscale * unoffset) - WANTED_SUM

    let offset = unscale.wrapping_mul(unoffset).wrapping_sub(WANTED_SUM);

    // Apply the tags.
    let scale = scale ^ VOUCHING_TAG;
    let unscale = unscale ^ CHECKING_TAG;

    check_parameters_or_die(offset, scale, (unoffset, unscale));
    (offset, scale, (unoffset, unscale))
}

#[test]
fn test_inverse() {
    assert_eq!(modinverse(u64::MAX), u64::MAX);
    assert_eq!(modinverse(1u64), 1u64);
    assert_eq!(modinverse(3u64), 12297829382473034411u64);
}

#[test]
fn test_derive() {
    use crate::check::CHECKING_TAG;
    use crate::check::WANTED_SUM;
    use crate::vouch::VOUCHING_TAG;

    assert_eq!(
        derive_parameters(0, 0),
        (
            WANTED_SUM.wrapping_neg(),
            VOUCHING_TAG ^ 1,
            (0, !CHECKING_TAG)
        )
    );
    assert_eq!(
        derive_parameters(u64::MAX, 0),
        (
            WANTED_SUM.wrapping_neg(),
            !VOUCHING_TAG,
            (0, CHECKING_TAG ^ 1)
        )
    );
    assert_eq!(
        derive_parameters(1, 1),
        (
            13020151265475858601,
            7453010330410905431,
            (1, 10993733730414794684)
        )
    );
    assert_eq!(
        derive_parameters(37, 13),
        (
            12023029964194261217,
            7453010330410905459,
            (13, 10110629933032573968)
        )
    );
    assert_eq!(
        derive_parameters(u64::MAX, u64::MAX),
        (
            13020151265475858601,
            u64::MAX ^ VOUCHING_TAG,
            (u64::MAX, 7453010343294756930)
        )
    );
}

#[test]
#[should_panic(expected = "failed to check voucher; parameters incorrect.")]
fn test_swap_params() {
    // Swap vouching and checking parameters, `check_parameters_or_die` should fail.
    let mut params = derive_parameters(43, 123);
    std::mem::swap(&mut params.0, &mut params.2 .0);
    std::mem::swap(&mut params.1, &mut params.2 .1);

    check_parameters_or_die(params.0, params.1, params.2);
}

#[test]
#[should_panic(expected = "failed to check voucher; parameters incorrect.")]
fn test_swap_params_retag() {
    use crate::check::CHECKING_TAG;
    use crate::vouch::VOUCHING_TAG;

    // Swap vouching and checking parameters, `check_parameters_or_die` should fail,
    // even after swapping the tags
    let mut params = derive_parameters(43, 123);
    std::mem::swap(&mut params.0, &mut params.2 .0);
    std::mem::swap(&mut params.1, &mut params.2 .1);

    params.1 = params.1 ^ VOUCHING_TAG ^ CHECKING_TAG;
    params.2 .1 = params.2 .1 ^ VOUCHING_TAG ^ CHECKING_TAG;

    check_parameters_or_die(params.0, params.1, params.2);
}
