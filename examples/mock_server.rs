use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let variant = std::env::var("MOCK_VARIANT").unwrap_or_default();
    let _descendant = if variant == "tree_hang" {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .env("MOCK_VARIANT", "tree_hang_child")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        if let Ok(path) = std::env::var("MOCK_PID_FILE") {
            std::fs::write(path, child.id().to_string()).unwrap();
        }
        Some(child)
    } else if variant == "tree_hang_child" {
        let grandchild = std::process::Command::new(std::env::current_exe().unwrap())
            .env("MOCK_VARIANT", "hang")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        if let Ok(path) = std::env::var("MOCK_GRANDCHILD_PID_FILE") {
            std::fs::write(path, grandchild.id().to_string()).unwrap();
        }
        Some(grandchild)
    } else {
        None
    };
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = request.get("id") else {
            continue;
        };
        if variant == "hang"
            || variant == "tree_hang"
            || variant == "tree_hang_child"
            || (variant == "call_hang" && request["method"] == "tools/call")
        {
            std::thread::sleep(std::time::Duration::from_secs(120));
        }
        if variant == "oversized" {
            writeln!(stdout, "{}", "x".repeat(65536)).unwrap();
            stdout.flush().unwrap();
            continue;
        }
        if request["method"] == "tools/call" && variant == "rpc_error" {
            writeln!(
                stdout,
                "{}",
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":"CANARY_SECRET"}})
            )
            .unwrap();
            stdout.flush().unwrap();
            continue;
        }
        let result = match request.get("method").and_then(Value::as_str) {
            Some("initialize") => {
                json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"mock-server","version":"1.0"}})
            }
            Some("tools/list") => {
                let mut tools = vec![
                    json!({"name":"echo","description": if variant == "changed" { "Echoes text with a prefix" } else { "Echoes text" },"annotations":{"readOnlyHint":true},"inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}}),
                ];
                if variant == "side_effect" {
                    tools.push(json!({"name":"write_marker","description":"Writes the test marker file","annotations":{"readOnlyHint":false},"inputSchema":{"type":"object"}}));
                    tools.push(json!({"name":"unclassified_marker","description":"Writes the test marker file without a read-only annotation","inputSchema":{"type":"object"}}));
                }
                if variant.starts_with("effect_") {
                    for name in ["setup_marker", "run_marker", "cleanup_marker"] {
                        tools.push(json!({"name":name,"description":"Runs a fixture lifecycle action","annotations":{"readOnlyHint":false},"inputSchema":{"type":"object"}}));
                    }
                }
                json!({"tools":tools})
            }
            Some("tools/call") => {
                let tool = request
                    .pointer("/params/name")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let marker = std::env::var("MOCK_MUTATION_FILE").ok();
                match tool {
                    "setup_marker" => {
                        std::fs::write(marker.as_deref().unwrap(), "setup").unwrap();
                    }
                    "run_marker" => {
                        std::fs::write(marker.as_deref().unwrap(), "case ran").unwrap();
                    }
                    "cleanup_marker" if variant != "effect_cleanup_failure" => {
                        let _ = std::fs::remove_file(marker.as_deref().unwrap());
                    }
                    _ => {}
                }
                if matches!(tool, "write_marker" | "unclassified_marker") {
                    let marker = marker.as_deref().unwrap();
                    std::fs::write(marker, "side effect completed").unwrap();
                }
                let text = request
                    .pointer("/params/arguments/text")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let text = if text == "env-check" {
                    std::env::var("MCP_TEST_SECRET").unwrap_or_else(|_| "isolated".to_owned())
                } else if variant == "changed" {
                    format!("echo: {text}")
                } else {
                    text.to_owned()
                };
                let is_error = (variant == "effect_setup_failure" && tool == "setup_marker")
                    || (variant == "effect_main_failure" && tool == "run_marker")
                    || (variant == "effect_cleanup_failure" && tool == "cleanup_marker");
                json!({"content":[{"type":"text","text":text}],"isError":is_error})
            }
            _ => json!({}),
        };
        let response_id = if variant == "wrong_id" {
            json!(999)
        } else {
            id.clone()
        };
        let response = json!({"jsonrpc":"2.0","id":response_id,"result":result});
        if writeln!(stdout, "{response}").is_err() || stdout.flush().is_err() {
            break;
        }
    }
}
