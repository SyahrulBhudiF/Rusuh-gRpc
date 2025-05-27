fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/auth.proto");
    println!("cargo:rerun-if-changed=proto/buf/validate/validate.proto");

    tonic_build::configure()
        .out_dir("src/pb")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/auth.proto"], &["proto"])?;

    Ok(())
}
