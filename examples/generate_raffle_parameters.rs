#[derive(Debug)]
enum Never {}

fn main() {
    use raffle::VouchingParameters;

    let args = std::env::args().skip(1); // skip the program name

    // No arguments -> use OS entropy.
    let params: VouchingParameters = if args.len() == 0 {
        use rand::Rng;

        let mut rng = rand::rngs::OsRng {};
        VouchingParameters::generate(|| Ok::<u64, Never>(rng.gen())).unwrap()
    } else {
        // Got some arguments, feed that in blake3.
        let mut hasher = blake3::Hasher::new_derive_key("generate_raffle_parameters");

        for arg in args {
            hasher.update(arg.as_bytes());
            hasher.update(b"\0");
        }

        let mut reader = hasher.finalize_xof();
        let generator = || {
            let mut buf = [0u8; 8];
            reader.fill(&mut buf);

            Ok::<u64, Never>(u64::from_le_bytes(buf))
        };
        VouchingParameters::generate(generator).unwrap()
    };

    println!("{}", params);
    println!("{}", params.checking_parameters());
}
