//! Configuration generation for different CLI tools

use crate::templates;
use crate::CliTool;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml::Value;

/// Clean existing audit configurations
pub fn clean(target_dir: &Path) -> Result<()> {
    println!("Cleaning existing audit configurations...");

    let items = [
        (".audit", true),
        (".cline", true),
        (".mcp.json", false),
        ("codex.json", false),
        ("CLAUDE.md", false),
        ("AGENTS.md", false),
        (".clinerules", false),
        (".audit-viewpoints.md", false),
    ];

    for (name, is_dir) in items {
        let path = target_dir.join(name);
        if path.exists() {
            if is_dir {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            println!("  ✓ Removed {}", name);
        }
    }

    Ok(())
}

/// Setup the .audit directory
pub fn setup_audit_dir(target_dir: &Path) -> Result<()> {
    println!("Setting up audit directory...");

    let audit_dir = target_dir.join(".audit/artifacts");
    fs::create_dir_all(&audit_dir)
        .context("Failed to create .audit/artifacts directory")?;

    println!("  ✓ Created .audit/");
    Ok(())
}

/// Configure a specific CLI tool
pub fn configure(agent_dir: &Path, target_dir: &Path, tool: CliTool) -> Result<()> {
    match tool {
        CliTool::Claude => configure_claude(agent_dir, target_dir),
        CliTool::Codex => configure_codex(agent_dir, target_dir),
        CliTool::Cline => configure_cline(agent_dir, target_dir),
    }
}

fn configure_claude(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Configuring for Claude CLI...");

    // Generate MCP config
    let mcp_config = templates::mcp_json(agent_dir, target_dir);
    let mcp_path = target_dir.join(".mcp.json");
    fs::write(&mcp_path, mcp_config)
        .context("Failed to write .mcp.json")?;
    println!("  ✓ Created .mcp.json");

    // Copy CLAUDE.md from agent directory
    let claude_md_src = agent_dir.join("CLAUDE.md");
    let claude_md_dst = target_dir.join("CLAUDE.md");

    if claude_md_src.exists() {
        fs::copy(&claude_md_src, &claude_md_dst)
            .context("Failed to copy CLAUDE.md")?;
        println!("  ✓ Copied CLAUDE.md");
    } else {
        // Generate default CLAUDE.md
        let content = templates::claude_md();
        fs::write(&claude_md_dst, content)
            .context("Failed to write CLAUDE.md")?;
        println!("  ✓ Created CLAUDE.md");
    }

    Ok(())
}

fn configure_codex(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Configuring for Codex CLI...");

    let home_dir = dirs::home_dir().context("Could not determine home directory")?;

    // Create symlinks in ~/.local/bin/ (Codex requires commands in PATH)
    let local_bin = home_dir.join(".local/bin");
    fs::create_dir_all(&local_bin).context("Failed to create ~/.local/bin directory")?;

    let servers = [
        "mental-model-server",
        "methodology-kb-server",
        "sarif-tools-server",
        "codegraph-server",
    ];

    for server in &servers {
        let src = agent_dir.join("target/release").join(server);
        let dst = local_bin.join(server);

        // Remove existing symlink if present
        if dst.exists() || dst.is_symlink() {
            fs::remove_file(&dst).ok();
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&src, &dst)
            .with_context(|| format!("Failed to create symlink for {}", server))?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&src, &dst)
            .with_context(|| format!("Failed to create symlink for {}", server))?;
    }
    println!("  ✓ Created symlinks in ~/.local/bin/");

    // Get ~/.codex directory
    let codex_dir = home_dir.join(".codex");
    let config_path = codex_dir.join("config.toml");

    // Create ~/.codex directory if it doesn't exist
    if !codex_dir.exists() {
        fs::create_dir_all(&codex_dir).context("Failed to create ~/.codex directory")?;
    }

    // Load existing config or create empty one
    let mut config: HashMap<String, Value> = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .context("Failed to read existing ~/.codex/config.toml")?;
        toml::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Get or create mcp_servers table
    let mcp_servers = config
        .entry("mcp_servers".to_string())
        .or_insert_with(|| Value::Table(toml::map::Map::new()));

    let mcp_table = match mcp_servers {
        Value::Table(t) => t,
        _ => {
            *mcp_servers = Value::Table(toml::map::Map::new());
            match mcp_servers {
                Value::Table(t) => t,
                _ => unreachable!(),
            }
        }
    };

    // Generate our MCP server configs (using just binary names, not full paths)
    let our_servers = templates::codex_mcp_servers(agent_dir, target_dir);

    // Merge our servers into existing config
    for (name, server) in our_servers {
        let mut server_table = toml::map::Map::new();
        // Use just the binary name since it's now in PATH via symlink
        let command_name = Path::new(&server.command)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(server.command);
        server_table.insert("command".to_string(), Value::String(command_name));
        if !server.args.is_empty() {
            let args: Vec<Value> = server.args.into_iter().map(Value::String).collect();
            server_table.insert("args".to_string(), Value::Array(args));
        }
        mcp_table.insert(name, Value::Table(server_table));
    }

    // Write merged config back
    let config_str = toml::to_string_pretty(&config)
        .context("Failed to serialize config to TOML")?;
    fs::write(&config_path, config_str)
        .context("Failed to write ~/.codex/config.toml")?;
    println!("  ✓ Updated ~/.codex/config.toml (merged 4 MCP servers)");

    // Create AGENTS.md in target directory
    let agents_md = templates::agents_md();
    let agents_path = target_dir.join("AGENTS.md");
    fs::write(&agents_path, agents_md)
        .context("Failed to write AGENTS.md")?;
    println!("  ✓ Created AGENTS.md");

    Ok(())
}

fn configure_cline(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Configuring for Cline (VS Code Extension)...");

    // Create .cline directory
    let cline_dir = target_dir.join(".cline");
    fs::create_dir_all(&cline_dir)
        .context("Failed to create .cline directory")?;

    // Generate MCP settings
    let mcp_settings = templates::cline_mcp_settings(agent_dir, target_dir);
    let settings_path = cline_dir.join("mcp_settings.json");
    fs::write(&settings_path, mcp_settings)
        .context("Failed to write mcp_settings.json")?;
    println!("  ✓ Created .cline/mcp_settings.json");

    // Create .clinerules
    let clinerules = templates::clinerules();
    let rules_path = target_dir.join(".clinerules");
    fs::write(&rules_path, clinerules)
        .context("Failed to write .clinerules")?;
    println!("  ✓ Created .clinerules");

    Ok(())
}

/// Create shared viewpoints reference
pub fn create_viewpoints_reference(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Creating viewpoints reference...");

    let content = templates::viewpoints_reference(agent_dir);
    let path = target_dir.join(".audit-viewpoints.md");
    fs::write(&path, content)
        .context("Failed to write .audit-viewpoints.md")?;
    println!("  ✓ Created .audit-viewpoints.md");

    Ok(())
}

/// Update .gitignore to include .audit/
pub fn update_gitignore(target_dir: &Path) -> Result<()> {
    let gitignore_path = target_dir.join(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.contains(".audit/") {
            let mut new_content = content;
            new_content.push_str("\n# AI Code Audit Agent artifacts\n.audit/\n");
            fs::write(&gitignore_path, new_content)?;
            println!("  ✓ Added .audit/ to .gitignore");
        }
    }

    Ok(())
}
