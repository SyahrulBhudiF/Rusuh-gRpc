fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/auth.proto");

    let descriptor_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin");

    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .out_dir("src/pb")
        .compile_protos(&["proto/auth.proto"], &["proto"])?;

    Ok(())
}
