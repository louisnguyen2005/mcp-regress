use crate::snapshot::{CaseResult, Snapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum FailOn {
    Any,
    Breaking,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub format_version: u32,
    pub changes: Vec<Change>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Change {
    pub severity: Severity,
    pub kind: String,
    pub name: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Breaking,
    Warning,
}

pub fn compare(before: &Snapshot, after: &Snapshot) -> Result<Report, String> {
    let old_tools = tool_map(&before.tools)?;
    let new_tools = tool_map(&after.tools)?;
    let old_cases = case_map(&before.cases)?;
    let new_cases = case_map(&after.cases)?;
    let mut changes = Vec::new();
    for name in old_tools
        .keys()
        .chain(new_tools.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
    {
        match (old_tools.get(&name), new_tools.get(&name)) {
            (None, Some(new)) => changes.push(change(
                Severity::Warning,
                "tool_added",
                &name,
                None,
                Some((*new).clone()),
            )),
            (Some(old), None) => changes.push(change(
                Severity::Breaking,
                "tool_removed",
                &name,
                Some((*old).clone()),
                None,
            )),
            (Some(old), Some(new)) if old != new => changes.push(change(
                Severity::Breaking,
                "tool_changed",
                &name,
                Some((*old).clone()),
                Some((*new).clone()),
            )),
            _ => {}
        }
    }
    for id in old_cases
        .keys()
        .chain(new_cases.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
    {
        match (old_cases.get(&id), new_cases.get(&id)) {
            (None, Some(new)) => changes.push(change(
                Severity::Warning,
                "case_added",
                &id,
                None,
                Some(serde_json::to_value(new).map_err(|e| e.to_string())?),
            )),
            (Some(old), None) => changes.push(change(
                Severity::Breaking,
                "case_removed",
                &id,
                Some(serde_json::to_value(old).map_err(|e| e.to_string())?),
                None,
            )),
            (Some(old), Some(new)) if old.tool != new.tool || old.outcome != new.outcome => {
                changes.push(change(
                    Severity::Breaking,
                    "case_changed",
                    &id,
                    Some(serde_json::to_value(old).map_err(|e| e.to_string())?),
                    Some(serde_json::to_value(new).map_err(|e| e.to_string())?),
                ));
            }
            _ => {}
        }
    }
    Ok(Report {
        format_version: 1,
        changes,
    })
}

fn tool_map(tools: &[Value]) -> Result<BTreeMap<String, &Value>, String> {
    let mut map = BTreeMap::new();
    for tool in tools {
        let name = tool
            .get("name")
            .and_then(Value::as_str)
            .ok_or("snapshot tool has no name")?;
        if map.insert(name.to_owned(), tool).is_some() {
            return Err(format!("duplicate tool: {name}"));
        }
    }
    Ok(map)
}

fn case_map(cases: &[CaseResult]) -> Result<BTreeMap<String, &CaseResult>, String> {
    let mut map = BTreeMap::new();
    for case in cases {
        if map.insert(case.id.clone(), case).is_some() {
            return Err(format!("duplicate case: {}", case.id));
        }
    }
    Ok(map)
}

fn change(
    severity: Severity,
    kind: &str,
    name: &str,
    before: Option<Value>,
    after: Option<Value>,
) -> Change {
    Change {
        severity,
        kind: kind.into(),
        name: name.into(),
        before,
        after,
    }
}

impl Report {
    pub fn should_fail(&self, policy: FailOn) -> bool {
        match policy {
            FailOn::Any => !self.changes.is_empty(),
            FailOn::Breaking => self
                .changes
                .iter()
                .any(|c| c.severity == Severity::Breaking),
            FailOn::None => false,
        }
    }

    pub fn markdown(&self) -> String {
        let mut out = format!(
            "# MCP regression report\n\n{} change(s) found.\n\n",
            self.changes.len()
        );
        if self.changes.is_empty() {
            out.push_str("No differences detected.\n");
        }
        for c in &self.changes {
            out.push_str(&format!(
                "## {}: {} ({})\n\n",
                c.kind,
                c.name,
                match c.severity {
                    Severity::Breaking => "breaking",
                    Severity::Warning => "warning",
                }
            ));
            for (label, value) in [("Before", &c.before), ("After", &c.after)] {
                if let Some(value) = value {
                    out.push_str(&format!(
                        "**{label}**\n\n```json\n{}\n```\n\n",
                        serde_json::to_string_pretty(value).unwrap_or_default()
                    ));
                }
            }
        }
        out
    }
}
