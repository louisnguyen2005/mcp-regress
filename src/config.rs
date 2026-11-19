use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub server: Server,
    #[serde(default)]
    pub cases: Vec<Case>,
    #[serde(default)]
    pub redact_keys: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_clear: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_run_timeout")]
    pub run_timeout_ms: u64,
    #[serde(default = "default_frame_limit")]
    pub max_frame_bytes: usize,
    #[serde(default = "default_total_limit")]
    pub max_output_bytes: usize,
    #[serde(default = "default_protocol")]
    pub protocol_version: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub tool: String,
    #[serde(default = "empty_object")]
    pub arguments: Value,
    #[serde(default)]
    pub ignore_pointers: Vec<String>,
    #[serde(default)]
    pub expected_rpc_error: Option<i64>,
    #[serde(default)]
    pub expect_tool_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assertions: BTreeMap<String, Value>,
}

fn default_protocol() -> String {
    "2025-06-18".into()
}
fn default_run_timeout() -> u64 {
    60_000
}
fn default_frame_limit() -> usize {
    8 * 1024 * 1024
}
fn default_total_limit() -> usize {
    64 * 1024 * 1024
}
fn default_timeout() -> u64 {
    10_000
}
fn empty_object() -> Value {
    serde_json::json!({})
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let mut content = String::new();
        fs::File::open(path)
            .map_err(|_| "cannot open config")?
            .take(1024 * 1024 + 1)
            .read_to_string(&mut content)
            .map_err(|_| "cannot read config")?;
        if content.len() > 1024 * 1024 {
            return Err("config exceeds 1 MiB".into());
        }
        let config: Self = toml::from_str(&content)
            .map_err(|_| "invalid TOML configuration (values omitted for privacy)")?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        let config = self;
        if config.version != 1 {
            return Err("config version must be 1".into());
        }
        if config.server.command.trim().is_empty() {
            return Err("server.command cannot be empty".into());
        }
        if config.server.env_clear && !Path::new(&config.server.command).is_absolute() {
            return Err("server.command must be an absolute path when env_clear is enabled".into());
        }
        if config.server.timeout_ms == 0 {
            return Err("server.timeout_ms must be greater than zero".into());
        }
        if config.server.run_timeout_ms == 0
            || config.server.max_frame_bytes == 0
            || config.server.max_output_bytes < config.server.max_frame_bytes
        {
            return Err("invalid run timeout or output limits".into());
        }
        if config.server.protocol_version != "2025-06-18" {
            return Err("only protocol profile 2025-06-18 is implemented".into());
        }
        if config.cases.len() > 10000 {
            return Err("case count exceeds 10000".into());
        }
        let mut ids = std::collections::HashSet::new();
        for case in &config.cases {
            if case.id.trim().is_empty() || case.tool.trim().is_empty() {
                return Err("case id and tool cannot be empty".into());
            }
            if !ids.insert(&case.id) {
                return Err(format!("duplicate case id: {}", case.id));
            }
            if !case.arguments.is_object() {
                return Err(format!("case {} arguments must be a JSON object", case.id));
            }
            if case.expect_tool_error && case.expected_rpc_error.is_some() {
                return Err("case cannot expect both RPC and tool errors".into());
            }
            if case.timeout_ms == Some(0) {
                return Err("case timeout_ms must be greater than zero".into());
            }
            if case.assertions.len() > 1000 {
                return Err("case assertions exceed 1000".into());
            }
            if case.expected_rpc_error.is_some() && !case.assertions.is_empty() {
                return Err("RPC error cases cannot assert result fields".into());
            }
            if case
                .assertions
                .keys()
                .any(|pointer| !valid_pointer(pointer))
            {
                return Err("assertions require valid non-root JSON Pointers".into());
            }
            for pointer in &case.ignore_pointers {
                if !valid_pointer(pointer) {
                    return Err(format!(
                        "case {} ignore pointer must start with /: {pointer}",
                        case.id
                    ));
                }
            }
        }
        Ok(())
    }
}

// Reject malformed RFC 6901 escape sequences rather than silently ignoring them.
pub fn valid_pointer(pointer: &str) -> bool {
    if !pointer.starts_with('/') {
        return false;
    }
    let mut chars = pointer.chars();
    while let Some(c) = chars.next() {
        if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}
