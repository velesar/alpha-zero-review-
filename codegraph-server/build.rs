//! Build script for codegraph-server
//!
//! Compiles the SCIP protobuf definitions if protoc is available.

fn main() {
    // Check if proto file exists
    let proto_path = "proto/scip.proto";
    if std::path::Path::new(proto_path).exists() {
        // Try to compile, but don't fail if protoc is missing
        match prost_build::compile_protos(&[proto_path], &["proto/"]) {
            Ok(_) => {
                println!("cargo:warning=SCIP proto compiled successfully");
            }
            Err(e) => {
                // Check if the error is due to missing protoc
                let err_str = e.to_string();
                if err_str.contains("protoc") || err_str.contains("Could not find") {
                    println!("cargo:warning=protoc not found, using simplified SCIP types. Install protoc for full SCIP support.");
                } else {
                    // Re-panic for other errors
                    panic!("Failed to compile SCIP proto: {}", e);
                }
            }
        }
    } else {
        println!("cargo:warning=SCIP proto not found at {}. Using simplified types.", proto_path);
    }
}
