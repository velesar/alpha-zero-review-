//! Configuration generation for different CLI tools

use crate::templates;
use crate::CliTool;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml::Value;

/// MCP server names registered by the audit agent
const SERVER_NAMES: [&str; 4] = ["mental-model", "methodology-kb", "sarif-tools", "codegraph"];

const BLOCK_BEGIN: &str = "<!-- ai-code-audit:begin -->";
const BLOCK_END: &str = "<!-- ai-code-audit:end -->";

/// Insert or replace the audit agent's section in an instructions file
/// (CLAUDE.md, AGENTS.md, .clinerules), preserving the project's own content.
pub fn upsert_marked_block(path: &Path, content: &str) -> Result<()> {
    let block = format!("{}\n{}\n{}\n", BLOCK_BEGIN, content.trim_end(), BLOCK_END);
    let existing = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?
    } else {
        String::new()
    };

    let updated = match (existing.find(BLOCK_BEGIN), existing.find(BLOCK_END)) {
        (Some(start), Some(end)) if end > start => {
            let end = end + BLOCK_END.len();
            let rest = existing[end..]
                .strip_prefix('\n')
                .unwrap_or(&existing[end..]);
            format!("{}{}{}", &existing[..start], block, rest)
        }
        _ if existing.trim().is_empty() => block,
        _ => format!("{}\n\n{}", existing.trim_end(), block),
    };

    fs::write(path, updated).with_context(|| format!("Failed to write {}", path.display()))
}

/// Remove the audit agent's section; deletes the file if nothing else remains.
pub fn remove_marked_block(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let existing = fs::read_to_string(path)?;
    let (Some(start), Some(end)) = (existing.find(BLOCK_BEGIN), existing.find(BLOCK_END)) else {
        return Ok(false);
    };
    if end < start {
        return Ok(false);
    }
    let remaining = format!(
        "{}{}",
        existing[..start].trim_end(),
        &existing[end + BLOCK_END.len()..]
    );
    if remaining.trim().is_empty() {
        fs::remove_file(path)?;
    } else {
        fs::write(path, format!("{}\n", remaining.trim_end()))?;
    }
    Ok(true)
}

/// Merge the audit agent's servers into an MCP JSON config
/// (`{"mcpServers": {...}}`), keeping any other servers already configured.
pub fn merge_mcp_servers(path: &Path, generated: &str) -> Result<()> {
    let generated: serde_json::Value = serde_json::from_str(generated)?;
    let mut config: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).with_context(|| {
            format!(
                "{} is not valid JSON; fix or remove it and re-run",
                path.display()
            )
        })?
    } else {
        serde_json::json!({})
    };

    let root = config
        .as_object_mut()
        .with_context(|| format!("{} must contain a JSON object", path.display()))?;
    let servers = root
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));
    let servers = servers
        .as_object_mut()
        .with_context(|| format!("mcpServers in {} must be an object", path.display()))?;

    if let Some(ours) = generated.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, server) in ours {
            servers.insert(name.clone(), server.clone());
        }
    }

    fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

/// Remove the audit agent's servers from an MCP JSON config; deletes the
/// file if no servers or other settings remain.
pub fn remove_mcp_servers(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(path)?;
    let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(false);
    };
    let Some(servers) = config.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
        return Ok(false);
    };
    let removed = SERVER_NAMES
        .iter()
        .filter(|name| servers.remove(**name).is_some())
        .count();
    if removed == 0 {
        return Ok(false);
    }

    let only_empty_servers = servers.is_empty()
        && config
            .as_object()
            .is_some_and(|o| o.keys().all(|k| k == "mcpServers"));
    if only_empty_servers {
        fs::remove_file(path)?;
    } else {
        fs::write(path, serde_json::to_string_pretty(&config)?)?;
    }
    Ok(true)
}

/// Clean existing audit configurations.
///
/// Only removes what the audit agent created: the .audit directory, the
/// viewpoints reference, its MCP servers and its marked instruction blocks.
pub fn clean(target_dir: &Path) -> Result<()> {
    println!("Cleaning existing audit configurations...");

    let audit_dir = target_dir.join(".audit");
    if audit_dir.exists() {
        fs::remove_dir_all(&audit_dir)?;
        println!("  ✓ Removed .audit/");
    }

    let viewpoints = target_dir.join(".audit-viewpoints.md");
    if viewpoints.exists() {
        fs::remove_file(&viewpoints)?;
        println!("  ✓ Removed .audit-viewpoints.md");
    }

    for name in [".mcp.json", ".cline/mcp_settings.json"] {
        if remove_mcp_servers(&target_dir.join(name))? {
            println!("  ✓ Removed audit MCP servers from {}", name);
        }
    }

    for name in ["CLAUDE.md", "AGENTS.md", ".clinerules"] {
        if remove_marked_block(&target_dir.join(name))? {
            println!("  ✓ Removed audit section from {}", name);
        }
    }

    Ok(())
}

/// Setup the .audit directory
pub fn setup_audit_dir(target_dir: &Path) -> Result<()> {
    println!("Setting up audit directory...");

    let audit_dir = target_dir.join(".audit/artifacts");
    fs::create_dir_all(&audit_dir).context("Failed to create .audit/artifacts directory")?;

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
    merge_mcp_servers(&target_dir.join(".mcp.json"), &mcp_config)
        .context("Failed to update .mcp.json")?;
    println!("  ✓ Added audit MCP servers to .mcp.json");

    upsert_marked_block(&target_dir.join("CLAUDE.md"), templates::claude_md())?;
    println!("  ✓ Added audit section to CLAUDE.md");

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
    let config_str =
        toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;
    fs::write(&config_path, config_str).context("Failed to write ~/.codex/config.toml")?;
    println!("  ✓ Updated ~/.codex/config.toml (merged 4 MCP servers)");

    // Create AGENTS.md in target directory
    upsert_marked_block(&target_dir.join("AGENTS.md"), templates::agents_md())?;
    println!("  ✓ Added audit section to AGENTS.md");

    Ok(())
}

fn configure_cline(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Configuring for Cline (VS Code Extension)...");

    // Create .cline directory
    let cline_dir = target_dir.join(".cline");
    fs::create_dir_all(&cline_dir).context("Failed to create .cline directory")?;

    // Generate MCP settings
    let mcp_settings = templates::cline_mcp_settings(agent_dir, target_dir);
    merge_mcp_servers(&cline_dir.join("mcp_settings.json"), &mcp_settings)
        .context("Failed to update .cline/mcp_settings.json")?;
    println!("  ✓ Added audit MCP servers to .cline/mcp_settings.json");

    upsert_marked_block(&target_dir.join(".clinerules"), templates::clinerules())?;
    println!("  ✓ Added audit section to .clinerules");

    Ok(())
}

/// Create shared viewpoints reference
pub fn create_viewpoints_reference(agent_dir: &Path, target_dir: &Path) -> Result<()> {
    println!();
    println!("Creating viewpoints reference...");

    let content = templates::viewpoints_reference(agent_dir);
    let path = target_dir.join(".audit-viewpoints.md");
    fs::write(&path, content).context("Failed to write .audit-viewpoints.md")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn marked_block_preserves_existing_content_and_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "# My project\n\nOwn rules.\n").unwrap();

        upsert_marked_block(&path, "audit v1").unwrap();
        upsert_marked_block(&path, "audit v2").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My project\n\nOwn rules.\n"));
        assert!(content.contains("audit v2"));
        assert!(!content.contains("audit v1"));
        assert_eq!(content.matches(BLOCK_BEGIN).count(), 1);

        assert!(remove_marked_block(&path).unwrap());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "# My project\n\nOwn rules.\n"
        );
    }

    #[test]
    fn removing_only_block_deletes_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("AGENTS.md");
        upsert_marked_block(&path, "audit").unwrap();
        assert!(remove_marked_block(&path).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn mcp_servers_merge_and_remove_keep_other_servers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(
            &path,
            r#"{"mcpServers": {"github": {"command": "gh-mcp"}}}"#,
        )
        .unwrap();

        let generated = templates::mcp_json(Path::new("/agent"), dir.path());
        merge_mcp_servers(&path, &generated).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let servers = config["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("github"));
        for name in SERVER_NAMES {
            assert!(servers.contains_key(name), "missing {}", name);
        }

        assert!(remove_mcp_servers(&path).unwrap());
        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            config["mcpServers"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            vec!["github"]
        );
    }

    #[test]
    fn invalid_existing_mcp_json_is_not_overwritten() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(&path, "{ not json").unwrap();
        let generated = templates::mcp_json(Path::new("/agent"), dir.path());
        assert!(merge_mcp_servers(&path, &generated).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
    }
}
