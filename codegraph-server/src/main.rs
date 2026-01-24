//! Codegraph MCP Server
//!
//! This is the main entry point for the Codegraph MCP server,
//! which provides SCIP-based semantic code intelligence.

use codegraph_server::server;

use anyhow::Result;
use clap::Parser;
use rmcp::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Codegraph MCP Server for AI Code Audit
#[derive(Parser, Debug)]
#[command(name = "codegraph-server")]
#[command(author = "Light IT Global")]
#[command(version = "0.1.0")]
#[command(about = "MCP Server for SCIP-based semantic code intelligence")]
struct Args {
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

    tracing::info!("Starting Codegraph MCP Server");

    // Create the server
    let server = server::CodegraphServer::new();

    // Run with stdio transport
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("Codegraph MCP Server shutting down");

    Ok(())
}
