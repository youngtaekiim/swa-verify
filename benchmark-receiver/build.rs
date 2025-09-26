use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../proto/benchmark.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile(&[proto_file], &["../proto"])?;

    println!("cargo:rerun-if-changed={}", proto_file);
    Ok(())
}