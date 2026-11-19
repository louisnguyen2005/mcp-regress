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
    pub rule_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception: Option<crate::policy::Exception>,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Breaking,
    Warning,
    Inconclusive,
}

pub fn compare(before: &Snapshot, after: &Snapshot) -> Result<Report, String> {
    if before.cases.iter().chain(&after.cases).any(
        |c| matches!(&c.outcome, crate::snapshot::Outcome::RpcError(s) if !s.starts_with("code:")),
    ) {
        return Err(
            "legacy error snapshots must be recaptured with explicit error expectations".into(),
        );
    }
    before.validate()?;
    after.validate()?;
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
            (Some(old), Some(new)) if old != new => {
                for key in old
                    .as_object()
                    .ok_or("invalid tool")?
                    .keys()
                    .chain(new.as_object().ok_or("invalid tool")?.keys())
                    .collect::<std::collections::BTreeSet<_>>()
                {
                    if old.get(key) == new.get(key) {
                        continue;
                    }
                    let severity = match key.as_str() {
                        "description" | "title" => Severity::Warning,
                        "inputSchema" => schema_change(old.get(key), new.get(key)),
                        "outputSchema" => Severity::Inconclusive,
                        _ => Severity::Breaking,
                    };
                    let mut c = change(
                        severity,
                        "tool_changed",
                        &name,
                        old.get(key).cloned(),
                        new.get(key).cloned(),
                    );
                    c.rule_id = match key.as_str() {
                        "inputSchema" => "tool.input_schema",
                        "outputSchema" => "tool.output_schema",
                        "description" | "title" => "tool.description",
                        "annotations" => "tool.annotations",
                        _ => "tool.metadata",
                    }
                    .into();
                    c.path = format!("/tools/{}/{}", escape_pointer(&name), escape_pointer(key));
                    changes.push(c);
                }
            }
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
    if let (Some(a), Some(b)) = (&before.provenance, &after.provenance) {
        if a.fixture_digest != b.fixture_digest || a.protocol_version != b.protocol_version {
            changes.push(change(Severity::Breaking, "capture_contract_changed", "provenance",
                Some(serde_json::json!({"fixture_digest":a.fixture_digest,"protocol_version":a.protocol_version})),
                Some(serde_json::json!({"fixture_digest":b.fixture_digest,"protocol_version":b.protocol_version}))));
        }
    }
    Ok(Report {
        format_version: 3,
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
    let rule_id = match kind {
        "tool_added" => "tool.added",
        "tool_removed" => "tool.removed",
        "case_added" => "case.added",
        "case_removed" => "case.removed",
        "case_changed" => "case.changed",
        "capture_contract_changed" => "capture.contract",
        _ => "tool.metadata",
    };
    let prefix = if kind.starts_with("tool_") {
        "tools"
    } else if kind.starts_with("case_") {
        "cases"
    } else {
        "capture"
    };
    Change {
        severity,
        rule_id: rule_id.into(),
        exception: None,
        kind: kind.into(),
        name: name.into(),
        path: format!("/{}/{}", prefix, escape_pointer(name)),
        before,
        after,
    }
}

impl Report {
    pub fn should_fail(&self, policy: FailOn) -> bool {
        match policy {
            FailOn::Any => self.changes.iter().any(|c| c.exception.is_none()),
            FailOn::Breaking => self
                .changes
                .iter()
                .any(|c| c.exception.is_none() && c.severity != Severity::Warning),
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
                c.rule_id,
                escape_markdown(&c.name),
                match c.severity {
                    Severity::Breaking => "breaking",
                    Severity::Warning => "warning",
                    Severity::Inconclusive => "inconclusive",
                }
            ));
            if let Some(exception) = &c.exception {
                out.push_str(&format!(
                    "Exception until {}: {}\n\n",
                    escape_markdown(&exception.expires),
                    escape_markdown(&exception.reason)
                ));
            }
            out.push_str(&format!("Path: `{}`. Server-controlled values are available in the private JSON report.\n\n", escape_markdown(&c.path)));
        }
        out
    }
}

fn escape_pointer(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
fn escape_markdown(s: &str) -> String {
    s.chars()
        .flat_map(char::escape_default)
        .collect::<String>()
        .replace(['`', '<', '>', '[', ']', '*', '#'], "_")
}

// A deliberately small proof of input compatibility. Complex schemas are never
// assumed compatible. Output compatibility has the opposite direction and is
// conservatively inconclusive until a dedicated engine is implemented.
fn schema_change(old: Option<&Value>, new: Option<&Value>) -> Severity {
    let (Some(old), Some(new)) = (old, new) else {
        return Severity::Inconclusive;
    };
    if simple_input_superset(old, new) {
        Severity::Warning
    } else {
        Severity::Inconclusive
    }
}
fn simple_input_superset(old: &Value, new: &Value) -> bool {
    let (Some(a), Some(b)) = (old.as_object(), new.as_object()) else {
        return false;
    };
    const KEYS: &[&str] = &[
        "type",
        "properties",
        "required",
        "additionalProperties",
        "description",
        "title",
    ];
    if a.keys()
        .chain(b.keys())
        .any(|k| !KEYS.contains(&k.as_str()))
    {
        return false;
    }
    if a.get("type") != Some(&serde_json::json!("object")) || a.get("type") != b.get("type") {
        return false;
    }
    // Only additions to open object schemas are proven compatible here.
    if a.get("additionalProperties")
        .is_some_and(|v| v != &Value::Bool(true))
        || b.get("additionalProperties")
            .is_some_and(|v| v != &Value::Bool(true))
    {
        return false;
    }
    let required = |v: Option<&Value>| -> Option<std::collections::BTreeSet<String>> {
        match v {
            None => Some(Default::default()),
            Some(Value::Array(arr)) => arr.iter().map(|s| s.as_str().map(str::to_owned)).collect(),
            _ => None,
        }
    };
    let (Some(ar), Some(br)) = (required(a.get("required")), required(b.get("required"))) else {
        return false;
    };
    if !br.is_subset(&ar) {
        return false;
    }
    let empty = serde_json::Map::new();
    let ap = match a.get("properties") {
        None => &empty,
        Some(Value::Object(p)) => p,
        _ => return false,
    };
    let bp = match b.get("properties") {
        None => &empty,
        Some(Value::Object(p)) => p,
        _ => return false,
    };
    // New constrained properties can reject inputs previously allowed by the
    // open schema. Only true/empty schemas are safe additions.
    ap.iter().all(|(k, v)| bp.get(k) == Some(v))
        && bp.iter().all(|(k, v)| {
            ap.contains_key(k) || v == &Value::Bool(true) || v == &serde_json::json!({})
        })
}
