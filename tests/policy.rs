use mcp_regress::{
    diff::{compare, FailOn, Severity},
    paths::ensure_separate,
    policy::Policy,
    snapshot::Snapshot,
};
use serde_json::json;

fn report() -> mcp_regress::diff::Report {
    let a: Snapshot = serde_json::from_value(
        json!({"format_version":1,"tools":[{"name":"echo","inputSchema":{}}],"cases":[]}),
    )
    .unwrap();
    let b: Snapshot =
        serde_json::from_value(json!({"format_version":1,"tools":[],"cases":[]})).unwrap();
    compare(&a, &b).unwrap()
}
fn policy(s: &str) -> Policy {
    toml::from_str(s).unwrap()
}

#[test]
fn scoped_exceptions_expire_and_remain_visible() {
    let p=policy("version=1\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason='Migration approval'\nexpires='2026-11-19'");
    let mut r = report();
    p.apply_on(&mut r, "2026-11-19").unwrap();
    assert!(!r.should_fail(FailOn::Any));
    assert!(r.changes[0].exception.is_some());
    assert!(r.markdown().contains("Migration approval"));
    assert!(p.apply_on(&mut r, "2026-11-20").is_err());
    let p=policy("version=1\n[[exceptions]]\nrule='tool.removed'\nname='other'\nreason='Migration approval'\nexpires='2026-11-19'");
    p.apply_on(&mut r, "2026-11-19").unwrap();
    assert!(r.should_fail(FailOn::Breaking));
}
#[test]
fn overrides_are_explicit_and_any_still_blocks_warnings() {
    let p = policy("version=1\n[overrides]\n\"tool.removed\"='warning'");
    let mut r = report();
    p.apply_on(&mut r, "2026-01-01").unwrap();
    assert_eq!(r.changes[0].severity, Severity::Warning);
    assert!(!r.should_fail(FailOn::Breaking));
    assert!(r.should_fail(FailOn::Any));
}
#[test]
fn invalid_and_ambiguous_policies_fail_closed() {
    for text in ["version=2", "version=1\n[overrides]\nunknown='warning'",
        "version=1\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason=''\nexpires='2026-01-01'",
        "version=1\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason='x'\nexpires='2026-02-29'",
        "version=1\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason='x'\nexpires='2026-13-01'"] {
        assert!(policy(text).validate().is_err(),"{text}");
    }
    let exception =
        "\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason='x'\nexpires='2028-02-29'";
    assert!(policy(&format!("version=1{exception}")).validate().is_ok());
    assert!(policy(&format!("version=1{exception}{exception}"))
        .validate()
        .is_err());
}
#[test]
fn dates_do_not_panic_on_non_ascii_or_truncated_strings() {
    for date in [
        "",
        "2026",
        "💥💥xx",
        "2026-00-00",
        "1969-12-31",
        "2100-02-29",
    ] {
        let p=policy(&format!("version=1\n[[exceptions]]\nrule='tool.removed'\nname='echo'\nreason='x'\nexpires='{date}'"));
        assert!(p.validate().is_err());
    }
}
#[test]
fn artifact_aliases_and_duplicate_reports_are_rejected_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("baseline.json");
    std::fs::write(&input, b"keep").unwrap();
    assert!(ensure_separate(&[&input], &[&input]).is_err());
    let report = temp.path().join("new").join("report.json");
    assert!(ensure_separate(&[&input], &[&report, &report]).is_err());
    assert!(ensure_separate(&[&input], &[&report]).is_ok());
    #[cfg(unix)]
    {
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&input, &alias).unwrap();
        assert!(ensure_separate(&[&input], &[&alias]).is_err());
        let directory = temp.path().join("directory");
        std::os::unix::fs::symlink(temp.path(), &directory).unwrap();
        assert!(ensure_separate(&[&input], &[&directory.join("baseline.json")]).is_err());
    }
    assert_eq!(std::fs::read(&input).unwrap(), b"keep");
}
#[test]
fn snapshot_identity_validation_prevents_invalid_success() {
    let invalid:Snapshot=serde_json::from_value(json!({"format_version":1,"tools":[],"cases":[{"id":"x","tool":"absent","outcome":{"status":"ok","value":{}}}]})).unwrap();
    assert!(invalid.validate().is_err());
}
