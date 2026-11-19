use crate::{
    config::Config,
    protocol::{Client, RequestError},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path, time::Duration};

const MAX_SNAPSHOT_BYTES: u64 = 128 * 1024 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    pub tools: Vec<Value>,
    pub cases: Vec<CaseResult>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub protocol_version: String,
    pub runner_version: String,
    pub fixture_digest: String,
    pub content_digest: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseResult {
    pub id: String,
    pub tool: String,
    pub outcome: Outcome,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Outcome {
    Ok(Value),
    RpcError(String),
}

struct HashWriter(Sha256);
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn digest(value: &impl Serialize) -> String {
    let mut writer = HashWriter(Sha256::new());
    serde_json::to_writer(&mut writer, value).expect("snapshot data serializes");
    format!("{:x}", writer.0.finalize())
}
impl Snapshot {
    fn content_digest(&self) -> String {
        #[derive(Serialize)]
        struct CanonicalCase<'a> {
            id: &'a str,
            outcome: &'a Outcome,
            tool: &'a str,
        }
        #[derive(Serialize)]
        struct Contents<'a> {
            cases: Vec<CanonicalCase<'a>>,
            tools: &'a [Value],
        }
        // Match canonical object key order at every level, including case fields.
        // Only a small reference index is allocated; result payloads stay borrowed.
        digest(&Contents {
            cases: self
                .cases
                .iter()
                .map(|c| CanonicalCase {
                    id: &c.id,
                    outcome: &c.outcome,
                    tool: &c.tool,
                })
                .collect(),
            tools: &self.tools,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.format_version, 1 | 2) {
            return Err("unsupported snapshot format version".into());
        }
        if self.tools.len() > 10000 || self.cases.len() > 10000 {
            return Err("snapshot item count exceeds limit".into());
        }
        if self.format_version == 1 && self.provenance.is_some() {
            return Err("v1 snapshots cannot carry v2 provenance".into());
        }
        if self.format_version == 2 {
            let p = self
                .provenance
                .as_ref()
                .ok_or("v2 snapshot missing provenance")?;
            let valid_digest = |s: &str| {
                s.len() == 64
                    && s.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            };
            if p.protocol_version != "2025-06-18"
                || p.runner_version.trim().is_empty()
                || !valid_digest(&p.fixture_digest)
            {
                return Err("invalid snapshot provenance".into());
            }
            if p.content_digest != self.content_digest() {
                return Err("snapshot integrity mismatch".into());
            }
        }
        let mut names = std::collections::HashSet::new();
        for tool in &self.tools {
            let name = tool
                .get("name")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .ok_or("snapshot tool has no name")?;
            if !tool.get("inputSchema").is_some_and(Value::is_object) {
                return Err("snapshot tool missing inputSchema".into());
            }
            if !names.insert(name) {
                return Err("duplicate snapshot tool".into());
            }
        }
        let tools = names;
        let mut names = std::collections::HashSet::new();
        for case in &self.cases {
            if case.id.trim().is_empty() || !tools.contains(case.tool.as_str()) {
                return Err("invalid snapshot case identity or tool reference".into());
            }
            if self.format_version == 2 {
                match &case.outcome {
                    Outcome::Ok(value)
                        if value.get("content").is_some_and(Value::is_array)
                            && value.get("isError").is_none_or(Value::is_boolean) => {}
                    Outcome::RpcError(code)
                        if code
                            .strip_prefix("code:")
                            .is_some_and(|s| s.parse::<i64>().is_ok()) => {}
                    _ => return Err("invalid snapshot outcome".into()),
                }
            }
            if !names.insert(&case.id) {
                return Err("duplicate snapshot case".into());
            }
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = fs::File::open(path).map_err(|_| "cannot open snapshot")?;
        use std::io::Read;
        let mut data = Vec::new();
        file.take(MAX_SNAPSHOT_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(|_| "cannot read snapshot")?;
        if data.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err("snapshot exceeds size limit".into());
        }
        let snapshot: Self = serde_json::from_slice(&data).map_err(|_| "invalid snapshot JSON")?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let data = serde_json::to_vec_pretty(self).map_err(|_| "snapshot encoding failed")?;
        if data.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err("snapshot exceeds size limit".into());
        }
        write_file(path, &data)
    }
}

pub async fn capture(config: &Config) -> Result<Snapshot, String> {
    config.validate()?;
    tokio::time::timeout(
        Duration::from_millis(config.server.run_timeout_ms),
        capture_inner(config),
    )
    .await
    .map_err(|_| "total run deadline exceeded".to_string())?
}
async fn capture_inner(config: &Config) -> Result<Snapshot, String> {
    let mut client = Client::start(&config.server).await?;
    let result = async {
        let mut tools = client.list_tools().await?;
        let mut cases = Vec::new();
        let tool_names: std::collections::HashSet<_> = tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        for case in &config.cases {
            if !tool_names.contains(case.tool.as_str()) {
                return Err("fixture refers to an unknown tool".into());
            }
            let outcome = match client.call_tool(&case.tool, case.arguments.clone()).await {
                Ok(mut value) => {
                    if case.expected_rpc_error.is_some() {
                        return Err("expected RPC error was not returned".into());
                    }
                    let is_error = match value.get("isError") {
                        None => false,
                        Some(Value::Bool(b)) => *b,
                        _ => return Err("invalid tool isError flag".into()),
                    };
                    if !value.get("content").is_some_and(Value::is_array) {
                        return Err("tool result missing content array".into());
                    }
                    if is_error != case.expect_tool_error {
                        return Err("tool error did not match case expectation".into());
                    }
                    for pointer in &case.ignore_pointers {
                        // Replace instead of remove: array indexes stay stable across exclusions.
                        let field = value
                            .pointer_mut(pointer)
                            .ok_or("ignore pointer does not exist in tool result")?;
                        *field = json!("[IGNORED]");
                    }
                    redact(&mut value, &config.redact_keys);
                    Outcome::Ok(value)
                }
                Err(RequestError::Rpc { code }) if case.expected_rpc_error == Some(code) => {
                    Outcome::RpcError(format!("code:{code}"))
                }
                Err(error) => return Err(error.to_string()),
            };
            cases.push(CaseResult {
                id: case.id.clone(),
                tool: case.tool.clone(),
                outcome,
            });
        }
        cases.sort_by(|a, b| a.id.cmp(&b.id));
        for tool in &mut tools {
            if let Some(fields) = tool.as_object_mut() {
                for (key, value) in fields {
                    // Schema property names are contracts, not response secrets.
                    if key != "inputSchema" && key != "outputSchema" {
                        let mut field = json!({key: value.clone()});
                        redact(&mut field, &config.redact_keys);
                        *value = field.get(key).unwrap().clone();
                    }
                }
            }
        }
        let mut fixtures = json!({"cases":config.cases,"redact_keys":config.redact_keys});
        redact(&mut fixtures, &config.redact_keys);
        let mut snapshot = Snapshot {
            format_version: 2,
            provenance: None,
            tools,
            cases,
        };
        snapshot.provenance = Some(Provenance {
            protocol_version: config.server.protocol_version.clone(),
            runner_version: env!("CARGO_PKG_VERSION").into(),
            fixture_digest: digest(&fixtures),
            content_digest: snapshot.content_digest(),
        });
        Ok(snapshot)
    }
    .await;
    client.stop().await;
    result
}

/// Explicit key redaction applies recursively before output reaches storage.
/// This is not a general secret detector; use sanitized fixture data.
pub fn redact(value: &mut Value, extra_keys: &[String]) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if [
                    "password",
                    "secret",
                    "token",
                    "api_key",
                    "apikey",
                    "authorization",
                    "access_token",
                    "refresh_token",
                ]
                .iter()
                .any(|k| key.eq_ignore_ascii_case(k))
                    || extra_keys.iter().any(|k| key.eq_ignore_ascii_case(k))
                {
                    *value = json!("[REDACTED]");
                } else {
                    redact(value, extra_keys);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact(value, extra_keys);
            }
        }
        _ => {}
    }
}

pub fn write_file(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|_| "cannot create output directory")?;
    let mut temp =
        tempfile::NamedTempFile::new_in(parent).map_err(|_| "cannot create temporary output")?;
    temp.write_all(data).map_err(|_| "cannot write output")?;
    temp.as_file()
        .sync_all()
        .map_err(|_| "cannot sync output")?;
    temp.persist(path)
        .map_err(|_| "cannot atomically replace output")?;
    Ok(())
}
