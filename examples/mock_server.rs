use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let variant = std::env::var("MOCK_VARIANT").unwrap_or_default();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = request.get("id") else {
            continue;
        };
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
                let text = if variant == "changed" {
                    format!("echo: {text}")
                } else {
                    text.to_owned()
                };
                json!({"content":[{"type":"text","text":text}],"isError":false})
            }
            _ => json!({}),
        };
        let response = json!({"jsonrpc":"2.0","id":id,"result":result});
        if writeln!(stdout, "{response}").is_err() || stdout.flush().is_err() {
            break;
        }
    }
}
