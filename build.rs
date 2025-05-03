fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/auth.proto");
    tonic_build::configure()
        .file_descriptor_set_path("descriptor.bin")
        .out_dir("src/pb")
        .compile_protos(&["proto/auth.proto"], &["proto"])?;
    Ok(())
}
