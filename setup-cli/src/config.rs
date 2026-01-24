//! Configuration generation for different CLI tools

use crate::templates;
use crate::CliTool;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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

    // Generate Codex config
    let codex_config = templates::codex_json(agent_dir, target_dir);
    let codex_path = target_dir.join("codex.json");
    fs::write(&codex_path, codex_config)
        .context("Failed to write codex.json")?;
    println!("  ✓ Created codex.json");

    // Create AGENTS.md
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
