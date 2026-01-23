//! Build script for codegraph-server
//!
//! Compiles the SCIP protobuf definitions.

fn main() {
    // Check if proto file exists
    let proto_path = "proto/scip.proto";
    if std::path::Path::new(proto_path).exists() {
        prost_build::compile_protos(&[proto_path], &["proto/"])
            .expect("Failed to compile SCIP proto");
    } else {
        // Create a placeholder - the actual proto should be downloaded
        println!("cargo:warning=SCIP proto not found at {}. Using simplified types.", proto_path);
    }
}
