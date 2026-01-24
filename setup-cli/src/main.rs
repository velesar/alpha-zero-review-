//! Setup CLI for AI Code Audit Agent
//!
//! Configures MCP servers for different AI CLI tools (Claude, Codex, Cline).
//! Optionally builds SCIP indexes for code intelligence.

mod config;
mod indexer;
mod language;
mod templates;

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum CliTool {
    Claude,
    Codex,
    Cline,
}

impl std::fmt::Display for CliTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliTool::Claude => write!(f, "claude"),
            CliTool::Codex => write!(f, "codex"),
            CliTool::Cline => write!(f, "cline"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "setup-audit")]
#[command(version = "2.0")]
#[command(about = "Setup AI Code Audit Agent for a target project")]
#[command(long_about = "Configures MCP servers and instructions for Claude CLI, Codex CLI, or Cline.\n\n\
    Examples:\n  \
    setup-audit /path/to/project                    # Default (Claude)\n  \
    setup-audit /path/to/project --cli codex        # Codex CLI\n  \
    setup-audit /path/to/project --all              # All tools\n  \
    setup-audit /path/to/project --with-index       # Build SCIP indexes\n  \
    setup-audit /path/to/project --install-indexers # Install missing indexers")]
struct Args {
    /// Target project directory (default: current directory)
    #[arg(default_value = ".")]
    target: PathBuf,

    /// CLI tool to configure
    #[arg(long, short, value_enum, default_value = "claude")]
    cli: CliTool,

    /// Configure all CLI tools
    #[arg(long, conflicts_with = "cli")]
    all: bool,

    /// Remove existing audit configurations before setup
    #[arg(long)]
    clean: bool,

    /// Skip checking for built MCP servers
    #[arg(long)]
    skip_check: bool,

    /// Build SCIP indexes for detected languages
    #[arg(long)]
    with_index: bool,

    /// Install missing language indexers automatically
    #[arg(long, requires = "with_index")]
    install_indexers: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Resolve paths
    let target_dir = args
        .target
        .canonicalize()
        .context("Target directory does not exist")?;

    if !target_dir.is_dir() {
        bail!("Target path is not a directory: {}", target_dir.display());
    }

    // Find agent directory (where this binary is installed)
    let agent_dir = find_agent_dir()?;

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           AI Code Audit Agent - Setup CLI v2.0                 ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ Agent Directory: {}", agent_dir.display());
    println!("║ Target Project:  {}", target_dir.display());
    if args.all {
        println!("║ CLI Tools:       ALL (claude, codex, cline)");
    } else {
        println!("║ CLI Tool:        {}", args.cli);
    }
    if args.with_index {
        print!("║ SCIP Indexing:   Enabled");
        if args.install_indexers {
            println!(" (auto-install)");
        } else {
            println!();
        }
    }
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Check MCP servers if not skipped
    if !args.skip_check {
        check_servers(&agent_dir)?;
    }

    // Clean if requested
    if args.clean {
        config::clean(&target_dir)?;
    }

    // Setup audit directory
    config::setup_audit_dir(&target_dir)?;

    // Configure tools
    let tools: Vec<CliTool> = if args.all {
        vec![CliTool::Claude, CliTool::Codex, CliTool::Cline]
    } else {
        vec![args.cli]
    };

    for tool in &tools {
        config::configure(&agent_dir, &target_dir, *tool)?;
    }

    // Create shared viewpoints reference
    config::create_viewpoints_reference(&agent_dir, &target_dir)?;

    // Build SCIP indexes if requested
    let built_indexes = if args.with_index {
        build_indexes(&target_dir, args.install_indexers)?
    } else {
        vec![]
    };

    // Update .gitignore
    config::update_gitignore(&target_dir)?;

    // Print completion message
    print_completion(&target_dir, &tools, &built_indexes);

    Ok(())
}

fn find_agent_dir() -> Result<PathBuf> {
    // Try to find agent directory by looking for Cargo.toml with workspace
    let exe_path = std::env::current_exe()?;

    // If running from target/release, go up to workspace root
    if let Some(parent) = exe_path.parent() {
        if parent.ends_with("release") || parent.ends_with("debug") {
            if let Some(target_dir) = parent.parent() {
                if let Some(workspace) = target_dir.parent() {
                    if workspace.join("Cargo.toml").exists() {
                        return Ok(workspace.to_path_buf());
                    }
                }
            }
        }
    }

    // Fall back to AGENT_DIR env var
    if let Ok(dir) = std::env::var("AGENT_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Fall back to current directory if it looks like the agent dir
    let cwd = std::env::current_dir()?;
    if cwd.join("Cargo.toml").exists() && cwd.join("mental-model-server").exists() {
        return Ok(cwd);
    }

    bail!("Could not find agent directory. Set AGENT_DIR environment variable.")
}

fn check_servers(agent_dir: &Path) -> Result<()> {
    println!("Checking MCP servers...");

    let servers = [
        "mental-model-server",
        "methodology-kb-server",
        "sarif-tools-server",
        "codegraph-server",
    ];

    let mut all_found = true;
    for server in servers {
        let path = agent_dir.join("target/release").join(server);
        if path.exists() {
            println!("  ✓ {}", server);
        } else {
            println!("  ✗ {} (not found)", server);
            all_found = false;
        }
    }

    if !all_found {
        println!();
        println!("Some servers are missing. Build with:");
        println!("  cd {} && cargo build --release", agent_dir.display());
        bail!("MCP servers not built");
    }

    println!();
    Ok(())
}

fn build_indexes(
    target_dir: &Path,
    install_missing: bool,
) -> Result<Vec<(language::Language, PathBuf)>> {
    println!("Detecting project languages...");

    let languages = language::detect_languages(target_dir);

    if languages.is_empty() {
        println!("  No supported languages detected.");
        println!("  Supported: Rust, TypeScript, JavaScript, Python, Go, Java");
        return Ok(vec![]);
    }

    println!(
        "  Found: {}",
        languages
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();

    // Check indexer availability
    println!("Checking indexer availability...");
    let statuses = indexer::check_indexers(&languages);

    for status in &statuses {
        if status.installed {
            let version = status.version.as_deref().unwrap_or("unknown version");
            println!("  ✓ {} ({})", status.language, version);
        } else {
            println!("  ✗ {} ({} not found)", status.language, status.command);
        }
    }
    println!();

    // Build indexes
    println!("Building SCIP indexes...");
    let index_dir = target_dir.join(".audit/indexes");

    let results = indexer::build_all_indexes(&languages, target_dir, &index_dir, install_missing)?;

    if results.is_empty() {
        println!("  No indexes were built.");
        println!("  Use --install-indexers to install missing indexers automatically.");
    }

    println!();
    Ok(results)
}

fn print_completion(
    target_dir: &Path,
    tools: &[CliTool],
    indexes: &[(language::Language, PathBuf)],
) {
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      Setup Complete!                           ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    println!("║  Files created:                                                ║");

    for tool in tools {
        match tool {
            CliTool::Claude => {
                println!("║    • .mcp.json + CLAUDE.md      (Claude CLI)                  ║");
            }
            CliTool::Codex => {
                println!("║    • codex.json + AGENTS.md     (Codex CLI)                   ║");
            }
            CliTool::Cline => {
                println!("║    • .cline/ + .clinerules      (Cline VS Code)               ║");
            }
        }
    }

    println!("║    • .audit/                    (Artifact storage)             ║");
    println!("║    • .audit-viewpoints.md       (Viewpoint reference)          ║");

    if !indexes.is_empty() {
        println!("║                                                                ║");
        println!("║  SCIP indexes:                                                 ║");
        for (lang, _path) in indexes {
            println!("║    • .audit/indexes/{:<12}                            ║", lang.index_filename());
        }
    }

    println!("║                                                                ║");
    println!("║  To start the audit:                                           ║");
    println!("║                                                                ║");

    if tools.len() > 1 {
        println!("║    Claude: cd {} && claude", target_dir.display());
        println!("║    Codex:  cd {} && codex", target_dir.display());
        println!("║    Cline:  Open in VS Code with Cline extension              ║");
    } else {
        match tools[0] {
            CliTool::Claude => {
                println!("║    cd {} && claude", target_dir.display());
            }
            CliTool::Codex => {
                println!("║    cd {} && codex", target_dir.display());
            }
            CliTool::Cline => {
                println!("║    Open {} in VS Code", target_dir.display());
            }
        }
    }

    println!("║                                                                ║");
    println!("║  Then ask:                                                     ║");
    println!("║    \"Run a full code audit using the viewpoints framework\"     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
