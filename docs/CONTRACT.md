# Local gate contract — v0.3

## Supported scope

The library and CLI inspect trusted local MCP servers using the 2025-06-18 stdio profile. A server must return that exact protocol revision, tools capability and server identity in initialization. Unsupported revisions fail; the runner does not claim modern MCP compatibility or independently verified conformance. It advertises no sampling or elicitation capabilities and answers unsupported inbound requests with method-not-found.

Commands are `baseline`, `replay`, `diff`, `check`, and `validate`. `validate --config fixture.toml [--policy policy.toml]` performs validation without launching a server. Exit `0` means the requested operation succeeded and the chosen policy passed; `1` means configuration, protocol, storage, execution, integrity, or expectation failure; `2` means policy findings. `--fail-on none` does not suppress execution errors. `baseline`/`replay` have identical capture behavior. No command silently promotes or overwrites a baseline after a failed capture.

## Config v1 additions

Existing fixture files remain parseable. Optional server settings:

```toml
[server]
command = "./my-server"
timeout_ms = 10000
run_timeout_ms = 60000
max_frame_bytes = 8388608
max_output_bytes = 67108864
protocol_version = "2025-06-18"
```

The total run deadline includes startup and fixture execution; shutdown has a separate maximum two-second wait after termination. Frame bytes include the newline. The output budget is cumulative across all stdout frames. Other limits: config 1 MiB, saved snapshot input 128 MiB, 100 tool pages, 10,000 tools/cases, and 1,024 inbound messages per request. These bound program-controlled buffers; they do not limit the child process's own memory/CPU. Server stderr is discarded to avoid secret-bearing logs. The child inherits the runner environment plus configured overrides: use disposable, least-privilege CI environments.

Optional root `redact_keys = ["customer_email"]` adds case-insensitive response key names to built-in recursive redaction of `password`, `secret`, `token`, `api_key`, `apikey`, `authorization`, `access_token`, and `refresh_token`. Tool schema property names are preserved to retain the contract. Schema defaults/descriptions and unstructured text may contain secrets: sanitize fixtures and server metadata before capture. Redaction is not a secret detector.

Optional per-case `expected_rpc_error = -32602` allows only that JSON-RPC error code; remote messages/data are discarded. `expect_tool_error = true` expects `isError: true` in a normal tool result. These expectations are mutually exclusive. Unexpected errors always abort capture. A successful tool result must contain a `content` array.

`ignore_pointers` accepts RFC 6901 pointers and replaces selected result values with `[IGNORED]` rather than deleting them, keeping array indexes stable. Missing pointers are errors. This is a normalization change from v0.1; recapture and review baselines using exclusions when upgrading. Arguments and case order are not altered before execution; saved cases are sorted by ID for deterministic comparison.

## Snapshots

New captures use snapshot v2 with protocol/runner version, a digest of redacted fixture definitions, and SHA-256 of canonical `tools` plus `cases`. There are no timestamps or random IDs in snapshots. Content hashes detect accidental modification, not malicious edits: they are not signatures. Server binaries and dependencies are not hashed. Fixture digest changes are blocking findings; tool process command/environment are excluded so baseline and current versions can be launched differently. Pin the server artifact in CI separately.

V1 snapshots remain readable and comparable. They lack provenance; compare cannot detect fixture/config changes against a v1 baseline. Legacy saved RPC error strings are rejected for comparison: recapture using explicit expectations. V2 must have matching content integrity. Unknown versions fail with exit 1. Writes use a temporary file in the destination directory, sync it, and atomically replace the output; on Unix the file is private (0600). Destination directory trust is the operator's responsibility.

## Report v3 and comparison

JSON reports contain `format_version`, ordered `changes`, and each change's `severity`, `rule_id`, `kind`, `name`, `path`, `before`, `after`, plus optional `exception` metadata. Paths identify logical tool/case names rather than array offsets. JSON is a private artifact: values may include arbitrary server text. Console output is a count and policy status. Markdown contains finding names and paths, omitting payloads; externally supplied text is escaped for display.

Tool removals, case changes/removals, annotation changes and capture contract changes block the default policy. Additions and tool description/title edits are warnings. Input schemas are classified as warning only for a small proven compatibility subset: open object schemas, unchanged existing properties, no new required fields, and additions with unconstrained schemas. Other input schema changes and output schema changes are `inconclusive`, which also block `--fail-on breaking`. Full JSON Schema implication is not implemented. `any` fails on all findings and `none` suppresses policy failures only. A passing gate says nothing about uncaptured inputs or malicious tool behavior.

## Process cleanup

Unix stdio servers start in a separate process group. Normal completion, request/run failure, cancellation, and dropped transports kill the group and reap the direct child. Descendants that deliberately detach into another session can escape; no sandbox is claimed. Windows currently terminates the direct child only; Job Object support is a remaining release blocker for full tree cleanup. SIGINT is handled by the CLI so transport destruction runs. SIGKILL and host crashes cannot run cleanup code.

## Policy v1

`check` and `diff` accept `--policy policy.toml`. See `examples/policy.toml`. `[overrides]` maps a known rule ID to `breaking`, `warning`, or `inconclusive`. Rules are `tool.added`, `tool.removed`, `tool.input_schema`, `tool.output_schema`, `tool.description`, `tool.annotations`, `tool.metadata`, `case.added`, `case.removed`, `case.changed`, and `capture.contract`.

Each optional `[[exceptions]]` has `rule`, exact `name`, nonempty `reason`, and an `expires` string in YYYY-MM-DD format. Dates are validated, including leap years; expiry is inclusive through that UTC day. Expired policies fail with exit 1 even with `--fail-on none` or no matching findings. Unknown rules/fields and duplicate exception scopes are rejected. Exceptions remain in JSON/Markdown reports and suppress only matching policy findings. Execution failures cannot be waived. Policy loading is bounded to 1 MiB.

Before execution, outputs are resolved and checked against config/policy/input snapshots and each other. Existing symlink aliases are rejected. This avoids accidental input destruction but assumes parent directories do not change concurrently. Multiple output files are individually atomic, not a single transaction.
