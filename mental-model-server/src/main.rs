//! Mental Model MCP Server
//!
//! This is the main entry point for the Mental Model MCP server,
//! which provides tools for managing the central mental model artifact
//! used in AI Code Audit.

use mental_model_server::server;

use anyhow::Result;
use clap::Parser;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Mental Model MCP Server for AI Code Audit
#[derive(Parser, Debug)]
#[command(name = "mental-model-server")]
#[command(author = "Light IT Global")]
#[command(version = "0.1.0")]
#[command(about = "MCP Server for managing Mental Model in AI Code Audit")]
struct Args {
    /// Path to the mental model YAML file
    #[arg(long, default_value = "./mental_model.yaml")]
    model_path: PathBuf,

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

    tracing::info!("Starting Mental Model MCP Server");
    tracing::info!("Model path: {:?}", args.model_path);

    // Create the server
    let server = server::MentalModelServer::new(args.model_path)?;

    // Run with stdio transport
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("Mental Model MCP Server shutting down");

    Ok(())
}
