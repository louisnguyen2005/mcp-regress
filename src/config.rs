use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub server: Server,
    #[serde(default)]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub tool: String,
    #[serde(default = "empty_object")]
    pub arguments: Value,
    #[serde(default)]
    pub ignore_pointers: Vec<String>,
}

fn default_timeout() -> u64 {
    10_000
}
fn empty_object() -> Value {
    serde_json::json!({})
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let config: Self =
            toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
        if config.version != 1 {
            return Err("config version must be 1".into());
        }
        if config.server.command.trim().is_empty() {
            return Err("server.command cannot be empty".into());
        }
        if config.server.timeout_ms == 0 {
            return Err("server.timeout_ms must be greater than zero".into());
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
            for pointer in &case.ignore_pointers {
                if !pointer.starts_with('/') {
                    return Err(format!(
                        "case {} ignore pointer must start with /: {pointer}",
                        case.id
                    ));
                }
            }
        }
        Ok(config)
    }
}
