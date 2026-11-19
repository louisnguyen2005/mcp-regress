use mcp_regress::{
    config::{valid_pointer, Config},
    diff::{compare, FailOn, Severity},
    protocol::{Client, RequestError},
    snapshot::{redact, write_file, Snapshot},
    transport::{read_frame, IoFuture, Transport},
};
use serde_json::{json, Value};
use std::collections::VecDeque;

struct Fake {
    messages: VecDeque<Value>,
}
impl Transport for Fake {
    fn send(&mut self, _: Value) -> IoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn receive(&mut self) -> IoFuture<'_, Value> {
        Box::pin(async { self.messages.pop_front().ok_or("EOF".into()) })
    }
    fn close(&mut self) -> IoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
fn initialized() -> Value {
    json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"test","version":"1"}}})
}
async fn client(messages: Vec<Value>) -> Client<Fake> {
    let mut all = vec![initialized()];
    all.extend(messages);
    Client::connect(
        Fake {
            messages: all.into(),
        },
        100,
        "2025-06-18",
    )
    .await
    .unwrap()
}
fn snapshot(tools: Value) -> Snapshot {
    serde_json::from_value(json!({"format_version":1,"tools":tools,"cases":[]})).unwrap()
}
#[tokio::test]
async fn bounded_frames_reject_unterminated_input() {
    let mut reader = tokio::io::BufReader::new(&b"123456789"[..]);
    assert!(read_frame(&mut reader, 4)
        .await
        .unwrap_err()
        .contains("limit"));
    let mut reader = tokio::io::BufReader::new(&b"{}\n{}\n"[..]);
    assert_eq!(read_frame(&mut reader, 3).await.unwrap(), b"{}\n");
    assert_eq!(read_frame(&mut reader, 3).await.unwrap(), b"{}\n");
}
#[tokio::test]
async fn rejects_wrong_id_and_ambiguous_envelopes() {
    for response in [
        json!({"jsonrpc":"2.0","id":99,"result":{}}),
        json!({"jsonrpc":"1.0","id":2,"result":{}}),
        json!({"jsonrpc":"2.0","id":2,"result":{},"error":{"code":-1,"message":"secret"}}),
    ] {
        assert!(matches!(
            client(vec![response]).await.request("x", json!({})).await,
            Err(RequestError::Execution(_))
        ));
    }
}
#[tokio::test]
async fn errors_do_not_expose_remote_messages() {
    let error = client(vec![json!({"jsonrpc":"2.0","id":2,"error":{"code":-32602,"message":"CANARY_SECRET","data":"CANARY_SECRET"}})]).await.request("x",json!({})).await.unwrap_err();
    assert!(matches!(error, RequestError::Rpc { code: -32602 }));
    assert!(!error.to_string().contains("CANARY"));
}
#[tokio::test]
async fn rejects_unsupported_protocol() {
    let mut init = initialized();
    init["result"]["protocolVersion"] = json!("future");
    assert!(Client::connect(
        Fake {
            messages: vec![init].into()
        },
        100,
        "2025-06-18"
    )
    .await
    .is_err());
}
#[tokio::test]
async fn sorts_tools_and_rejects_repeating_cursor() {
    let tools = json!([{"name":"z","inputSchema":{}},{"name":"a","inputSchema":{}}]);
    let mut c = client(vec![
        json!({"jsonrpc":"2.0","id":2,"result":{"tools":tools}}),
    ])
    .await;
    assert_eq!(c.list_tools().await.unwrap()[0]["name"], "a");
    let mut c = client(vec![
        json!({"jsonrpc":"2.0","id":2,"result":{"tools":[],"nextCursor":"x"}}),
        json!({"jsonrpc":"2.0","id":3,"result":{"tools":[],"nextCursor":"x"}}),
    ])
    .await;
    assert!(c.list_tools().await.unwrap_err().contains("repeated"));
}
#[test]
fn validates_pointer_escapes_and_config_limits() {
    assert!(valid_pointer("/a~1b/~0"));
    assert!(!valid_pointer("/bad~2"));
    let config: Config = toml::from_str(
        "version=1\n[server]\ncommand='x'\nmax_frame_bytes=200\nmax_output_bytes=100",
    )
    .unwrap();
    assert!(config.validate().is_err());
}
#[test]
fn redacts_nested_secrets_and_custom_keys() {
    let mut value = json!({"nested":[{"TOKEN":"CANARY","customer":"PRIVATE"}],"normal":"ok"});
    redact(&mut value, &["customer".into()]);
    assert!(!value.to_string().contains("CANARY"));
    assert!(!value.to_string().contains("PRIVATE"));
    assert_eq!(value["normal"], "ok");
}
#[test]
fn atomic_storage_replaces_without_following_output_symlink() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.json");
    write_file(&output, b"one").unwrap();
    write_file(&output, b"two").unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), b"two");
    #[cfg(unix)]
    {
        use std::os::unix::fs::{symlink, PermissionsExt};
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let link = temp.path().join("link");
        symlink(&output, &link).unwrap();
        write_file(&link, b"three").unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), b"two");
    }
}
#[test]
fn checksum_mismatch_and_unknown_versions_are_rejected() {
    let mut s = snapshot(json!([]));
    s.format_version = 9;
    assert!(s.validate().is_err());
    let s:Snapshot = serde_json::from_value(json!({"format_version":2,"provenance":{"protocol_version":"2025-06-18","runner_version":"0.2.0","fixture_digest":"0000000000000000000000000000000000000000000000000000000000000000","content_digest":"bad"},"tools":[],"cases":[]})).unwrap();
    assert!(s.validate().unwrap_err().contains("integrity"));
}
#[test]
fn conservative_schema_and_metadata_policy() {
    let old = snapshot(
        json!([{"name":"x","description":"before","inputSchema":{"type":"object","properties":{},"required":["a"]}}]),
    );
    let new = snapshot(
        json!([{"name":"x","description":"after","inputSchema":{"type":"object","properties":{},"required":[]}}]),
    );
    let report = compare(&old, &new).unwrap();
    assert_eq!(report.changes.len(), 2);
    assert!(!report.should_fail(FailOn::Breaking));
    assert!(report.should_fail(FailOn::Any));
    let new = snapshot(json!([{"name":"x","inputSchema":{"type":"object","allOf":[{}]}}]));
    let report = compare(&old, &new).unwrap();
    assert!(report
        .changes
        .iter()
        .any(|c| c.severity == Severity::Inconclusive));
    assert!(report.should_fail(FailOn::Breaking));
    assert!(!report.should_fail(FailOn::None));
}
#[test]
fn reports_do_not_embed_tool_payloads_in_markdown() {
    let old = snapshot(json!([]));
    let new = snapshot(json!([{"name":"x","description":"CANARY_SECRET","inputSchema":{}}]));
    let report = compare(&old, &new).unwrap();
    assert!(!report.markdown().contains("CANARY_SECRET"));
}

#[test]
fn checked_in_v2_checksum_remains_compatible_with_streaming_hashes() {
    let snapshot = Snapshot::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/baseline.json"),
    )
    .unwrap();
    assert_eq!(snapshot.format_version, 2);
}

#[tokio::test]
async fn server_notifications_are_bounded_and_unsupported_requests_are_answered() {
    let messages = vec![
        json!({"jsonrpc":"2.0","method":"notifications/message","params":{}}),
        json!({"jsonrpc":"2.0","id":"server-1","method":"sampling/createMessage","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"result":{}}),
    ];
    assert!(client(messages).await.request("x", json!({})).await.is_ok());
    let messages = vec![json!({"jsonrpc":"2.0","method":"notifications/message"}); 1024];
    assert!(client(messages)
        .await
        .request("x", json!({}))
        .await
        .unwrap_err()
        .to_string()
        .contains("too many"));
}
#[test]
fn narrowing_and_complex_schema_changes_cannot_pass_default_policy() {
    for schema in [
        json!({"type":"object","required":["x"]}),
        json!({"type":"object","additionalProperties":false}),
        json!({"type":"object","properties":{"x":{"type":"string"}}}),
        json!({"type":"object","$ref":"https://example.invalid/schema"}),
    ] {
        let old = snapshot(json!([{"name":"x","inputSchema":{"type":"object"}}]));
        let new = snapshot(json!([{"name":"x","inputSchema":schema}]));
        assert!(compare(&old, &new).unwrap().should_fail(FailOn::Breaking));
    }
}
