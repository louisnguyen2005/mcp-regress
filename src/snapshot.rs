use crate::{config::Config, protocol::Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format_version: u32,
    pub tools: Vec<Value>,
    pub cases: Vec<CaseResult>,
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

pub async fn capture(config: &Config) -> Result<Snapshot, String> {
    let mut client = Client::start(&config.server).await?;
    let result = async {
        let tools = client.list_tools().await?;
        let mut cases = Vec::new();
        for case in &config.cases {
            let outcome = match client.call_tool(&case.tool, case.arguments.clone()).await {
                Ok(mut value) => {
                    for pointer in &case.ignore_pointers {
                        remove_pointer(&mut value, pointer);
                    }
                    Outcome::Ok(value)
                }
                Err(error) => Outcome::RpcError(error),
            };
            cases.push(CaseResult {
                id: case.id.clone(),
                tool: case.tool.clone(),
                outcome,
            });
        }
        Ok(Snapshot {
            format_version: 1,
            tools,
            cases,
        })
    }
    .await;
    client.stop().await;
    result
}

// JSON Pointer tokens are decoded according to RFC 6901.
fn remove_pointer(value: &mut Value, pointer: &str) {
    let mut tokens: Vec<String> = pointer
        .split('/')
        .skip(1)
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect();
    let Some(last) = tokens.pop() else {
        return;
    };
    let mut parent = value;
    for token in tokens {
        parent = match parent {
            Value::Object(map) => match map.get_mut(&token) {
                Some(v) => v,
                None => return,
            },
            Value::Array(items) => match token.parse::<usize>().ok().and_then(|i| items.get_mut(i))
            {
                Some(v) => v,
                None => return,
            },
            _ => return,
        };
    }
    match parent {
        Value::Object(map) => {
            map.remove(&last);
        }
        Value::Array(items) => {
            if let Some(index) = last.parse::<usize>().ok().filter(|i| *i < items.len()) {
                items.remove(index);
            }
        }
        _ => {}
    }
}

impl Snapshot {
    pub fn load(path: &Path) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let snapshot: Self =
            serde_json::from_slice(&data).map_err(|e| format!("{}: {e}", path.display()))?;
        if snapshot.format_version != 1 {
            return Err("unsupported snapshot format version".into());
        }
        Ok(snapshot)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        write_file(
            path,
            &serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
    }
}

pub fn write_file(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))
}
