use crate::{
    config::Server,
    transport::{StdioTransport, Transport},
};
use serde_json::{json, Value};
use std::{fmt, time::Duration};
use tokio::time::timeout;

#[derive(Debug)]
pub enum RequestError {
    Execution(String),
    Rpc { code: i64 },
}
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(s) => write!(f, "{s}"),
            Self::Rpc { code } => write!(f, "RPC error code {code}"),
        }
    }
}
impl From<String> for RequestError {
    fn from(s: String) -> Self {
        Self::Execution(s)
    }
}
impl From<&str> for RequestError {
    fn from(s: &str) -> Self {
        Self::Execution(s.into())
    }
}

pub struct Client<T: Transport = StdioTransport> {
    transport: T,
    next_id: u64,
    deadline: Duration,
}
impl Client<StdioTransport> {
    pub async fn start(server: &Server) -> Result<Self, String> {
        Self::connect(
            StdioTransport::spawn(server)?,
            server.timeout_ms,
            &server.protocol_version,
        )
        .await
    }
}
impl<T: Transport> Client<T> {
    pub async fn connect(transport: T, timeout_ms: u64, version: &str) -> Result<Self, String> {
        if version != "2025-06-18" {
            return Err("unsupported protocol profile".into());
        }
        let mut client = Self {
            transport,
            next_id: 1,
            deadline: Duration::from_millis(timeout_ms),
        };
        let initialized = client.request("initialize", json!({"protocolVersion":version,
            "capabilities":{},"clientInfo":{"name":"mcp-regress","version":env!("CARGO_PKG_VERSION")}}))
            .await.map_err(|e| e.to_string())?;
        if initialized.get("protocolVersion").and_then(Value::as_str) != Some(version)
            || !initialized
                .pointer("/capabilities/tools")
                .is_some_and(Value::is_object)
            || !initialized
                .pointer("/serverInfo/name")
                .is_some_and(Value::is_string)
            || !initialized
                .pointer("/serverInfo/version")
                .is_some_and(Value::is_string)
        {
            return Err(
                "invalid initialization result or unsupported protocol/tools capability".into(),
            );
        }
        timeout(
            client.deadline,
            client
                .transport
                .send(json!({"jsonrpc":"2.0","method":"notifications/initialized"})),
        )
        .await
        .map_err(|_| "initialization notification timed out")??;
        Ok(client)
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, RequestError> {
        let id = self.next_id;
        self.next_id += 1;
        timeout(self.deadline, async {
            self.transport.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})).await?;
            for _ in 0..1024 {
                let value = self.transport.receive().await?;
                if value.get("jsonrpc") != Some(&json!("2.0")) || !value.is_object() {
                    return Err("invalid JSON-RPC envelope".into());
                }
                if let Some(method) = value.get("method") {
                    if !method.is_string() || value.get("result").is_some() || value.get("error").is_some() {
                        return Err("invalid server message".into());
                    }
                    if let Some(server_id) = value.get("id") {
                        if !server_id.is_string() && !server_id.is_number() { return Err("invalid server request ID".into()); }
                        // No sampling/elicitation capabilities were advertised.
                        self.transport.send(json!({"jsonrpc":"2.0","id":server_id,"error":{"code":-32601,"message":"Unsupported client method"}})).await?;
                    }
                    continue;
                }
                if value.get("id") != Some(&json!(id)) { return Err("unexpected response ID".into()); }
                match (value.get("result"), value.get("error")) {
                    (Some(result), None) if result.is_object() => return Ok(result.clone()),
                    (None, Some(error)) => {
                        let code = error.get("code").and_then(Value::as_i64).ok_or("invalid RPC error code")?;
                        if !error.get("message").is_some_and(Value::is_string) { return Err("invalid RPC error message".into()); }
                        return Err(RequestError::Rpc { code });
                    }
                    _ => return Err("response must contain exactly one valid result or error".into()),
                }
            }
            Err("too many server messages in one request".into())
        }).await.map_err(|_| RequestError::Execution(format!("{method} timed out")))?
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut cursor = None;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            let result = self
                .request(
                    "tools/list",
                    match &cursor {
                        Some(c) => json!({"cursor":c}),
                        None => json!({}),
                    },
                )
                .await
                .map_err(|e| e.to_string())?;
            let tools = result
                .get("tools")
                .and_then(Value::as_array)
                .ok_or("missing tools array")?;
            all.extend(tools.iter().cloned());
            if all.len() > 10_000 {
                return Err("tool count exceeds 10000".into());
            }
            cursor = match result.get("nextCursor") {
                None => None,
                Some(Value::String(c)) if !c.is_empty() => Some(c.clone()),
                _ => return Err("invalid pagination cursor".into()),
            };
            if cursor.is_none() {
                all.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
                let mut names = std::collections::HashSet::new();
                for tool in &all {
                    let name = tool
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .ok_or("invalid tool name")?;
                    if !tool.get("inputSchema").is_some_and(Value::is_object) {
                        return Err("invalid tool inputSchema".into());
                    }
                    if !names.insert(name) {
                        return Err("duplicate tool name".into());
                    }
                }
                return Ok(all);
            }
            if !seen.insert(cursor.clone()) {
                return Err("repeated pagination cursor".into());
            }
        }
        Err("tools/list exceeded 100 pages".into())
    }
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, RequestError> {
        self.request("tools/call", json!({"name":name,"arguments":arguments}))
            .await
    }
    pub async fn stop(mut self) {
        let _ = timeout(Duration::from_secs(2), self.transport.close()).await;
    }
}
