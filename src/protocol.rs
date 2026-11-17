use crate::config::Server;
use serde_json::{json, Value};
use std::{process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::timeout,
};

pub struct Client {
    child: Child,
    input: ChildStdin,
    output: Lines<BufReader<ChildStdout>>,
    next_id: u64,
    deadline: Duration,
}

impl Client {
    pub async fn start(server: &Server) -> Result<Self, String> {
        let mut command = Command::new(&server.command);
        command
            .args(&server.args)
            .envs(&server.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot start {}: {e}", server.command))?;
        let input = child.stdin.take().ok_or("missing child stdin")?;
        let output = BufReader::new(child.stdout.take().ok_or("missing child stdout")?).lines();
        let mut client = Self {
            child,
            input,
            output,
            next_id: 1,
            deadline: Duration::from_millis(server.timeout_ms),
        };
        client
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "mcp-regress", "version": env!("CARGO_PKG_VERSION")}
                }),
            )
            .await?;
        client
            .notify("notifications/initialized", json!({}))
            .await?;
        Ok(client)
    }

    async fn send(&mut self, message: &Value) -> Result<(), String> {
        let mut line = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        line.push(b'\n');
        timeout(self.deadline, self.input.write_all(&line))
            .await
            .map_err(|_| "server stdin write timed out".to_string())?
            .map_err(|e| format!("server stdin write failed: {e}"))
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
            .await
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await?;
        // One deadline covers the whole response, including unrelated notifications.
        let response = timeout(self.deadline, async {
            loop {
                let line = self
                    .output
                    .next_line()
                    .await
                    .map_err(|e| format!("server stdout read failed: {e}"))?
                    .ok_or_else(|| format!("server closed stdout during {method}"))?;
                if line.len() > 8 * 1024 * 1024 {
                    return Err("MCP response exceeds 8 MiB".into());
                }
                let value: Value = serde_json::from_str(&line)
                    .map_err(|e| format!("invalid JSON on server stdout: {e}"))?;
                if value.get("id") != Some(&json!(id)) {
                    continue;
                }
                if let Some(error) = value.get("error") {
                    return Err(format!("{method} RPC error: {error}"));
                }
                return value
                    .get("result")
                    .cloned()
                    .ok_or_else(|| format!("{method} response has no result"));
            }
        })
        .await;
        response
            .map_err(|_| format!("{method} timed out after {} ms", self.deadline.as_millis()))?
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        let mut seen = std::collections::HashSet::new();
        loop {
            let params = match &cursor {
                Some(c) => json!({"cursor": c}),
                None => json!({}),
            };
            let result = self.request("tools/list", params).await?;
            let tools = result
                .get("tools")
                .and_then(Value::as_array)
                .ok_or("tools/list result has no tools array")?;
            all.extend(tools.iter().cloned());
            cursor = result
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            match &cursor {
                Some(c) if !seen.insert(c.clone()) => {
                    return Err("tools/list repeated a pagination cursor".into())
                }
                Some(_) => {}
                None => break,
            }
            if seen.len() > 100 {
                return Err("tools/list exceeded 100 pages".into());
            }
        }
        all.sort_by(|a, b| {
            a.get("name")
                .and_then(Value::as_str)
                .cmp(&b.get("name").and_then(Value::as_str))
        });
        let mut names = std::collections::HashSet::new();
        for tool in &all {
            let name = tool
                .get("name")
                .and_then(Value::as_str)
                .ok_or("tool has no name")?;
            if !names.insert(name.to_owned()) {
                return Err(format!("duplicate tool name: {name}"));
            }
        }
        Ok(all)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, String> {
        self.request("tools/call", json!({"name": name, "arguments": arguments}))
            .await
    }

    pub async fn stop(mut self) {
        let _ = self.input.shutdown().await;
        let _ = timeout(Duration::from_secs(1), self.child.wait()).await;
        let _ = self.child.kill().await;
    }
}
