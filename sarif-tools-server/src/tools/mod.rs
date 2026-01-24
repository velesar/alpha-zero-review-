//! Tool implementations
//!
//! Implements runners for various code analysis tools.

mod semgrep;
mod bandit;
mod ruff;
mod trivy;
mod clippy;

pub use semgrep::SemgrepRunner;
pub use bandit::BanditRunner;
pub use ruff::RuffRunner;
pub use trivy::TrivyRunner;
pub use clippy::ClippyRunner;

use crate::runner::ToolRunner;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of available tools
#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolRunner>>,
}

impl ToolRegistry {
    /// Create a new tool registry with all known tools
    pub fn new() -> Self {
        let mut tools: HashMap<String, Arc<dyn ToolRunner>> = HashMap::new();

        tools.insert("semgrep".to_string(), Arc::new(SemgrepRunner));
        tools.insert("bandit".to_string(), Arc::new(BanditRunner));
        tools.insert("ruff".to_string(), Arc::new(RuffRunner));
        tools.insert("trivy".to_string(), Arc::new(TrivyRunner));
        tools.insert("clippy".to_string(), Arc::new(ClippyRunner));

        Self { tools }
    }

    /// Get a tool runner by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolRunner>> {
        self.tools.get(name).cloned()
    }

    /// List all available tools with their status
    pub fn list_available(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|runner| ToolInfo {
                name: runner.name().to_string(),
                languages: runner.supported_languages(),
                installed: runner.is_available(),
                version: runner.version(),
            })
            .collect()
    }

    /// Get list of tool names
    #[allow(dead_code)]
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a tool
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub languages: Vec<String>,
    pub installed: bool,
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.get("semgrep").is_some());
        assert!(registry.get("bandit").is_some());
        assert!(registry.get("ruff").is_some());
        assert!(registry.get("trivy").is_some());
        assert!(registry.get("clippy").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_list_tools() {
        let registry = ToolRegistry::new();
        let tools = registry.list_available();
        assert_eq!(tools.len(), 5);
    }
}
