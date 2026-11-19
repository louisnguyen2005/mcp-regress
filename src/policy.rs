//! Explicit, versioned policy changes; exceptions are scoped and expire in UTC.
use crate::diff::{Report, Severity};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::Read,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const RULES: &[&str] = &[
    "tool.added",
    "tool.removed",
    "tool.input_schema",
    "tool.output_schema",
    "tool.description",
    "tool.annotations",
    "tool.metadata",
    "case.added",
    "case.removed",
    "case.changed",
    "capture.contract",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u32,
    #[serde(default)]
    pub overrides: BTreeMap<String, Severity>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    pub rule: String,
    pub name: String,
    pub reason: String,
    pub expires: String,
}

impl Policy {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut text = String::new();
        std::fs::File::open(path)
            .map_err(|_| "cannot open policy")?
            .take(1024 * 1024 + 1)
            .read_to_string(&mut text)
            .map_err(|_| "cannot read policy")?;
        if text.len() > 1024 * 1024 {
            return Err("policy exceeds 1 MiB".into());
        }
        let policy: Self = toml::from_str(&text).map_err(|_| "invalid policy TOML")?;
        policy.validate()?;
        Ok(policy)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported policy version".into());
        }
        if self.overrides.keys().any(|r| !RULES.contains(&r.as_str())) {
            return Err("unknown policy rule".into());
        }
        let mut scopes = std::collections::HashSet::new();
        for e in &self.exceptions {
            if !RULES.contains(&e.rule.as_str())
                || e.name.trim().is_empty()
                || e.reason.trim().is_empty()
            {
                return Err(
                    "exceptions require a known rule, exact name, and nonempty reason".into(),
                );
            }
            date_days(&e.expires)?;
            if !scopes.insert((&e.rule, &e.name)) {
                return Err("duplicate exception scope".into());
            }
        }
        Ok(())
    }
    pub fn apply(&self, report: &mut Report) -> Result<(), String> {
        let today = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system clock before epoch")?
            .as_secs()
            / 86400;
        self.apply_at_day(report, today)
    }
    /// Deterministic time injection for automation and policy tests.
    pub fn apply_on(&self, report: &mut Report, date: &str) -> Result<(), String> {
        self.apply_at_day(report, date_days(date)?)
    }
    fn apply_at_day(&self, report: &mut Report, today: u64) -> Result<(), String> {
        self.validate()?;
        // Fail closed even if an expired exception currently matches no finding.
        for e in &self.exceptions {
            if date_days(&e.expires)? < today {
                return Err("policy contains an expired exception".into());
            }
        }
        for change in &mut report.changes {
            if let Some(severity) = self.overrides.get(&change.rule_id) {
                change.severity = *severity;
            }
            change.exception = self
                .exceptions
                .iter()
                .find(|e| e.rule == change.rule_id && e.name == change.name)
                .cloned();
        }
        Ok(())
    }
}

fn date_days(s: &str) -> Result<u64, String> {
    let invalid = || "dates must be valid YYYY-MM-DD between 1970 and 9999".to_string();
    let bytes = s.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit())
    {
        return Err(invalid());
    }
    let year: u32 = s[..4].parse().map_err(|_| invalid())?;
    let month: usize = s[5..7].parse().map_err(|_| invalid())?;
    let day: u32 = s[8..].parse().map_err(|_| invalid())?;
    let leap = |y: u32| {
        u32::from(y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)))
    };
    if !(1970..=9999).contains(&year) || !(1..=12).contains(&month) {
        return Err(invalid());
    }
    let lengths = [31, 28 + leap(year), 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if day == 0 || day > lengths[month - 1] {
        return Err(invalid());
    }
    Ok((1970..year).map(|y| 365 + u64::from(leap(y))).sum::<u64>()
        + lengths[..month - 1]
            .iter()
            .map(|n| u64::from(*n))
            .sum::<u64>()
        + u64::from(day - 1))
}
