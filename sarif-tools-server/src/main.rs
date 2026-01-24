//! SARIF Tools MCP Server
//!
//! This is the main entry point for the SARIF Tools MCP server,
//! which provides tools for running code analysis tools and working
//! with SARIF output format.

use sarif_tools_server::server;

use anyhow::Result;
use clap::Parser;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// SARIF Tools MCP Server for AI Code Audit
#[derive(Parser, Debug)]
#[command(name = "sarif-tools-server")]
#[command(author = "Light IT Global")]
#[command(version = "0.1.0")]
#[command(about = "MCP Server for SARIF-based code analysis tools")]
struct Args {
    /// Path to rule mappings YAML file
    #[arg(long)]
    mappings_path: Option<PathBuf>,

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

    tracing::info!("Starting SARIF Tools MCP Server");

    // Create the server
    let server = server::SarifToolsServer::new(args.mappings_path);

    // Run with stdio transport
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("SARIF Tools MCP Server shutting down");

    Ok(())
}
