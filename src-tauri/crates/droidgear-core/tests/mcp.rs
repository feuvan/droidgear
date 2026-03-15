use droidgear_core::mcp::{self, McpServer, McpServerConfig, McpServerType};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn mcp_save_load_and_toggle_roundtrip() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let mut env = HashMap::new();
    env.insert("EXAMPLE".to_string(), "1".to_string());

    let server = McpServer {
        name: "test".to_string(),
        config: McpServerConfig {
            server_type: McpServerType::Stdio,
            disabled: false,
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "exa-mcp-server".to_string()]),
            env: Some(env),
            url: None,
            headers: None,
        },
    };

    mcp::save_mcp_server_for_home(home, server).unwrap();

    let loaded = mcp::load_mcp_servers_for_home(home).unwrap();
    let loaded = loaded.into_iter().find(|s| s.name == "test").unwrap();
    assert_eq!(loaded.config.server_type, McpServerType::Stdio);
    assert_eq!(loaded.config.command.as_deref(), Some("npx"));
    assert_eq!(
        loaded.config.args.as_deref(),
        Some(&["-y".to_string(), "exa-mcp-server".to_string()][..])
    );
    assert_eq!(
        loaded
            .config
            .env
            .as_ref()
            .and_then(|m| m.get("EXAMPLE"))
            .map(String::as_str),
        Some("1")
    );

    mcp::toggle_mcp_server_for_home(home, "test", true).unwrap();
    let loaded2 = mcp::load_mcp_servers_for_home(home).unwrap();
    let loaded2 = loaded2.into_iter().find(|s| s.name == "test").unwrap();
    assert!(loaded2.config.disabled);
}
