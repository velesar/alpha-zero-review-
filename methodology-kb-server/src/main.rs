//! Methodology KB MCP Server
//!
//! This is the main entry point for the Methodology Knowledge Base MCP server,
//! which provides tools for interpreting metrics, classifying findings,
//! and checking compliance against standards.

mod acquisition;
mod server;
mod types;

use anyhow::Result;
use clap::Parser;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Methodology KB MCP Server for AI Code Audit
#[derive(Parser, Debug)]
#[command(name = "methodology-kb-server")]
#[command(author = "Light IT Global")]
#[command(version = "0.1.0")]
#[command(about = "MCP Server for Methodology Knowledge Base in AI Code Audit")]
struct Args {
    /// Path to the methodology KB directory
    #[arg(long, default_value = "./methodology_kb/")]
    kb_path: PathBuf,

    /// Enable debug logging
    #[arg(long, short)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.debug {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Methodology KB MCP Server");
    tracing::info!("KB path: {:?}", args.kb_path);

    // Create the server
    let server = server::MethodologyKBServer::new(args.kb_path);

    // Run with stdio transport
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("Methodology KB MCP Server shutting down");

    Ok(())
}
