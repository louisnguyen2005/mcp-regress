# Local gate contract — v0.7

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

### Safe tool-call default (v0.7)

Tool calls are allowed by default only when the server marks that tool with `annotations.readOnlyHint = true`. If the hint is false, missing, or malformed, add `allow_side_effects = true` to that specific `[[cases]]` entry only after reviewing the tool and using disposable fixture data:

```toml
[[cases]]
id = "create-test-record"
tool = "create_record"
allow_side_effects = true
arguments = { name = "synthetic" }
```

The flag is included in fixture provenance. MCP annotations are server-supplied hints, not a security boundary: a server can mislabel a mutating tool as read-only, and a tool may have side effects that are not obvious from its name. The configured server remains trusted code with the runner's privileges. Review every fixture call and provide cleanup for disposable state. This release does not yet provide setup/teardown hooks.

`ignore_pointers` accepts RFC 6901 pointers and replaces selected result values with `[IGNORED]` rather than deleting them, keeping array indexes stable. Missing pointers are errors. This is a normalization change from v0.1; recapture and review baselines using exclusions when upgrading. Arguments and case order are not altered before execution; saved cases are sorted by ID for deterministic comparison.

## Snapshots

New captures use snapshot v2 with protocol/runner version, a digest of redacted fixture definitions, and SHA-256 of canonical `tools` plus `cases`. There are no timestamps or random IDs in snapshots. Content hashes detect accidental modification, not malicious edits: they are not signatures. Server binaries and dependencies are not hashed. Fixture digest changes are blocking findings; tool process command/environment are excluded so baseline and current versions can be launched differently. Pin the server artifact in CI separately.

V1 snapshots remain readable and comparable. They lack provenance; compare cannot detect fixture/config changes against a v1 baseline. Legacy saved RPC error strings are rejected for comparison: recapture using explicit expectations. V2 must have matching content integrity. Unknown versions fail with exit 1. Writes use a temporary file in the destination directory, sync it, and atomically replace the output; on Unix the file is private (0600). Destination directory trust is the operator's responsibility.

## Report v3 and comparison

JSON reports contain `format_version`, ordered `changes`, and each change's `severity`, `rule_id`, `kind`, `name`, `path`, `before`, `after`, plus optional `exception` metadata. Paths identify logical tool/case names rather than array offsets. JSON is a private artifact: values may include arbitrary server text. Console output is a count and policy status. Markdown contains finding names and paths, omitting payloads; externally supplied text is escaped for display.

Tool removals, case changes/removals, annotation changes and capture contract changes block the default policy. Additions and tool description/title edits are warnings. Input schemas are classified as warning only for a small proven compatibility subset: open object schemas, unchanged existing properties, no new required fields, and additions with unconstrained schemas. Other input schema changes and output schema changes are `inconclusive`, which also block `--fail-on breaking`. Full JSON Schema implication is not implemented. `any` fails on all findings and `none` suppresses policy failures only. A passing gate says nothing about uncaptured inputs or malicious tool behavior.

## Process cleanup

Unix stdio servers start in a separate process group. Windows servers start suspended and are assigned to a Job Object before they resume. Normal completion, request/run failure, cancellation, and dropped transports terminate the ordinary child process tree and reap the direct child. Descendants that deliberately detach into a new Unix session can escape; no sandbox is claimed. If Windows Job Object creation or assignment fails, startup fails rather than using direct-child-only cleanup. SIGINT is handled by the CLI so transport destruction runs. SIGKILL and host crashes cannot run cleanup code.

## Policy v1

`check` and `diff` accept `--policy policy.toml`. See `examples/policy.toml`. `[overrides]` maps a known rule ID to `breaking`, `warning`, or `inconclusive`. Rules are `tool.added`, `tool.removed`, `tool.input_schema`, `tool.output_schema`, `tool.description`, `tool.annotations`, `tool.metadata`, `case.added`, `case.removed`, `case.changed`, and `capture.contract`.

Each optional `[[exceptions]]` has `rule`, exact `name`, nonempty `reason`, and an `expires` string in YYYY-MM-DD format. Dates are validated, including leap years; expiry is inclusive through that UTC day. Expired policies fail with exit 1 even with `--fail-on none` or no matching findings. Unknown rules/fields and duplicate exception scopes are rejected. Exceptions remain in JSON/Markdown reports and suppress only matching policy findings. Execution failures cannot be waived. Policy loading is bounded to 1 MiB.

Before execution, outputs are resolved and checked against config/policy/input snapshots and each other. Existing symlink aliases are rejected. This avoids accidental input destruction but assumes parent directories do not change concurrently. Multiple output files are individually atomic, not a single transaction.

## Case assertions and deadlines (v0.4)

A case can specify `timeout_ms = 250` to impose a shorter tool-call deadline. The effective deadline is the minimum of this limit, the server request timeout, and the remaining total-run budget. Zero is invalid. Deadline failures abort capture and never replace a baseline. This does not extend the server request timeout or retry a tool call.

`[cases.assertions]` maps non-root RFC 6901 pointers to exact expected JSON values. Every path must exist and equal its expected value. Assertions run on the original tool result before ignore pointers and redaction; mismatches fail without printing actual or expected values. At most 1,000 assertions are allowed per case. Assertions cannot be combined with `expected_rpc_error`, since those cases have no tool result. They can be used with an explicitly expected tool-level error.

See `examples/assertions.toml`. Default-empty assertions and absent case deadlines are omitted from fixture serialization, preserving existing v2 fixture digests. Adding or changing an assertion/deadline changes fixture provenance and requires an intentional baseline review.

## Child environment isolation (v0.5)

`[server] env_clear = true` starts the configured child with an empty environment, then supplies only the key/value pairs under `server.env`. Because command lookup may depend on `PATH`, the command must be absolute in this mode. This flag does not create a sandbox or limit filesystem/network access. With `env_clear` absent or false, behavior remains compatible: the runner environment is inherited and `[server.env]` overrides/adds values. Avoid putting secrets directly in a checked-in config; use an external secret mechanism to generate runtime configuration. Diagnostics do not include configured values.
