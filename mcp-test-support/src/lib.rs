//! In-process MCP test harness.
//!
//! Connects an rmcp client to a server over an in-memory duplex stream, so
//! tests exercise the same path a real MCP client does: JSON-RPC framing,
//! parameter deserialization against the tool's schema, the tool router and
//! the handler's error mapping.

use rmcp::model::{CallToolRequestParam, CallToolResult};
use rmcp::service::{RunningService, ServiceError};
use rmcp::{RoleClient, ServerHandler, ServiceExt};
use serde_json::Value;

/// Outcome of a tool call as a client sees it
#[derive(Debug)]
pub enum ToolOutcome {
    /// Successful result: concatenated text content
    Ok(String),
    /// Protocol error (e.g. invalid_params) or a result flagged is_error
    Err(String),
}

/// A connected MCP client talking to an in-process server
pub struct McpClient {
    client: RunningService<RoleClient, ()>,
}

impl McpClient {
    /// Start `server` on an in-memory transport and connect a client to it
    pub async fn start<S>(server: S) -> Self
    where
        S: ServerHandler + Send + Sync + 'static,
    {
        let (server_io, client_io) = tokio::io::duplex(1 << 20);
        tokio::spawn(async move {
            if let Ok(running) = server.serve(server_io).await {
                let _ = running.waiting().await;
            }
        });
        let client = ().serve(client_io).await.expect("MCP client failed to connect");
        Self { client }
    }

    /// Names of all tools the server lists, sorted
    pub async fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .client
            .list_all_tools()
            .await
            .expect("tools/list failed")
            .into_iter()
            .map(|t| t.name.to_string())
            .collect();
        names.sort();
        names
    }

    /// Call a tool with JSON object arguments
    pub async fn call(&self, name: &str, arguments: Value) -> ToolOutcome {
        let arguments = match arguments {
            Value::Object(map) => Some(map),
            Value::Null => None,
            other => panic!("tool arguments must be a JSON object, got {other}"),
        };
        let result = self
            .client
            .call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments,
            })
            .await;
        match result {
            Ok(result) => {
                let text = text_of(&result);
                if result.is_error == Some(true) {
                    ToolOutcome::Err(text)
                } else {
                    ToolOutcome::Ok(text)
                }
            }
            Err(ServiceError::McpError(e)) => ToolOutcome::Err(e.message.to_string()),
            Err(e) => panic!("transport failure calling {name}: {e}"),
        }
    }

    /// Call a tool that must succeed; returns its text
    pub async fn ok(&self, name: &str, arguments: Value) -> String {
        match self.call(name, arguments).await {
            ToolOutcome::Ok(text) => text,
            ToolOutcome::Err(e) => panic!("{name} failed: {e}"),
        }
    }

    /// Call a tool that must succeed with a JSON body (after an optional
    /// text prefix such as "Found 3 symbols\n\n")
    pub async fn ok_json(&self, name: &str, arguments: Value) -> Value {
        let text = self.ok(name, arguments).await;
        parse_json_body(&text).unwrap_or_else(|| panic!("{name} did not return JSON: {text}"))
    }

    /// Call a tool that must fail; returns the error message
    pub async fn err(&self, name: &str, arguments: Value) -> String {
        match self.call(name, arguments).await {
            ToolOutcome::Err(e) => e,
            ToolOutcome::Ok(text) => panic!("{name} unexpectedly succeeded: {text}"),
        }
    }
}

fn text_of(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse a JSON document, or the JSON after a human-readable prefix line
pub fn parse_json_body(text: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str(text) {
        return Some(value);
    }
    let start = text.find(['{', '['])?;
    serde_json::from_str(&text[start..]).ok()
}
