use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let variant = std::env::var("MOCK_VARIANT").unwrap_or_default();
    let _descendant = if variant == "tree_hang" {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .env("MOCK_VARIANT", "hang")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        if let Ok(path) = std::env::var("MOCK_PID_FILE") {
            std::fs::write(path, child.id().to_string()).unwrap();
        }
        Some(child)
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
                json!({"tools":[{"name":"echo","description": if variant == "changed" { "Echoes text with a prefix" } else { "Echoes text" },"inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}}]})
            }
            Some("tools/call") => {
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
                json!({"content":[{"type":"text","text":text}],"isError":false})
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
