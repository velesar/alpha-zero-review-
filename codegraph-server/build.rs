//! Build script for codegraph-server
//!
//! Compiles the SCIP protobuf definitions. Uses the system `protoc` when
//! `PROTOC` is set, otherwise falls back to a vendored binary so the crate
//! builds on machines without protobuf tooling installed.

fn main() {
    let proto_path = "proto/scip.proto";
    println!("cargo:rerun-if-changed={}", proto_path);
    println!("cargo:rerun-if-env-changed=PROTOC");

    if std::env::var_os("PROTOC").is_none() {
        let protoc = protoc_bin_vendored::protoc_bin_path()
            .expect("no vendored protoc for this platform; set PROTOC to a protoc binary");
        std::env::set_var("PROTOC", protoc);
    }

    // Upstream SCIP proto doc comments contain examples that are not valid
    // Rust and would break doctests, so drop all generated comments.
    let mut config = prost_build::Config::new();
    config.disable_comments(["."]);
    config
        .compile_protos(&[proto_path], &["proto/"])
        .expect("failed to compile SCIP proto");
}
