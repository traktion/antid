fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["proto/archive.proto", "proto/tarchive.proto", "proto/pnr.proto"],
            &["proto"],
        )?;
    Ok(())
}
