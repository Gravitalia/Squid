fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
                &["proto/squid/squid.proto"],
                &["proto"]
        )
        .unwrap();
    Ok(())
}