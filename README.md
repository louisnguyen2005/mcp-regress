# mcp-regress

`mcp-regress` records a trusted MCP server's tool surface and fixture responses, then compares a later run against that baseline. It is a deterministic regression gate for MCP server maintainers. The CLI and example server are written in Rust.

This repository is an early prototype. The [product roadmap](docs/ROADMAP.md), [delivery backlog](docs/BACKLOG.md), [security model](docs/SECURITY.md), and [compatibility ledger](docs/COMPATIBILITY.md) describe the work required before broader production use. The CLI is an MCP client; an agent-facing MCP server is planned as an optional adapter.

## Development milestone: v0.4

The local engine now has bounded stdout framing and run budgets, validated JSON-RPC responses, explicit expected errors, Unix process-group cleanup, recursive response key redaction, atomic private artifact writes, snapshot checksums/provenance, and conservative change classification. See the [behavior and migration contract](docs/CONTRACT.md) and [implementation evidence](docs/PROGRESS.md).

## Features

- MCP JSON-RPC client over stdio with initialization, tool listing, pagination, and tool calls.
- Versioned JSON snapshots and structured JSON or Markdown change reports.
- Stable fixture replay without an LLM or API key.
- Per-case JSON Pointer exclusions for values such as timestamps or generated IDs.
- CI exit codes: `0` for pass, `1` for execution/configuration error, `2` for a policy-triggering difference.
- Configurable `--fail-on any|breaking|none`; tool and case additions are warnings; removals and behavior changes block by default. Unproven schema changes are inconclusive and also block.
- Direct process execution without a shell. Child processes have request and total-run timeouts and are stopped after capture.

## Quick start

From this directory, with a Rust toolchain installed:

```sh
cargo build --release
cargo run -- baseline --config examples/fixture.toml --output snapshots/baseline.json
cargo run -- check --config examples/fixture.toml --baseline snapshots/baseline.json --json reports/change.json --markdown reports/change.md
```

The example server changes its tool description and response when `MOCK_VARIANT=changed` is set. To see a failing check:

```sh
MOCK_VARIANT=changed cargo run -- check --config examples/fixture.toml --baseline snapshots/baseline.json --fail-on breaking
```

`--` is required after `cargo run` to pass options to the CLI. The `examples/fixture.toml` server command invokes Cargo for the demo only. Point the configuration at your server executable in production.

## Commands

```text
mcp-regress baseline --config fixture.toml --output baseline.json
mcp-regress replay   --config fixture.toml --output current.json
mcp-regress diff     --baseline baseline.json --current current.json --json report.json --markdown report.md --fail-on breaking
mcp-regress check    --config fixture.toml --baseline baseline.json --json report.json --markdown report.md --fail-on breaking
```

`baseline` and `replay` intentionally use the same capture logic; their names distinguish the trusted recording from a later replay. `diff` only reads saved files. `check` captures and compares in one run.

## Configuration

```toml
version = 1

[server]
command = "./target/release/my-mcp-server"
args = ["--fixture-mode"]
timeout_ms = 10000
env = { FIXTURE_DIR = "./fixtures" }

[[cases]]
id = "lookup-known-user"
tool = "lookup_user"
arguments = { user_id = "fixture-1" }
ignore_pointers = ["/content/0/metadata/generated_at"]
```

Run fixture calls only against a server and test data that you control. The tool invokes each configured MCP tool; it does not sandbox the server. Keep snapshots and reports private if fixture responses may contain secrets. `ignore_pointers` replaces selected response values with `[IGNORED]` and fails if a path is absent. Common secret keys are redacted recursively in responses; unstructured text and schema metadata still require sanitized fixtures. Server stderr and remote RPC error messages are discarded.

The server must speak newline-delimited MCP JSON-RPC over stdio. Protocol messages on stdout must be valid JSON; diagnostics should go to stderr. The client requests MCP protocol version `2025-06-18`. Only stdio is supported in this release.

## Repository layout

```text
src/lib.rs        Reusable capture/comparison library
src/main.rs       CLI and exit codes
src/config.rs     TOML configuration and validation
src/protocol.rs   MCP profile and JSON-RPC validation
src/transport.rs  Bounded stdio transport and process lifecycle
src/snapshot.rs   Capture and snapshot storage
src/diff.rs       Change classification and reports
examples/         Runnable mock MCP server and fixture
```

## CI example

Commit a reviewed baseline and fixture configuration. In CI, build the server and run:

```sh
cargo run --release -- check \
  --config fixture.toml \
  --baseline baseline.json \
  --json reports/mcp-regress.json \
  --markdown reports/mcp-regress.md \
  --fail-on breaking
```

Review baseline updates as code changes. A passing comparison means the configured tool surface and fixture results did not change in the observed run; it is not a security certification or a sandbox.

## Result assertions

Cases can assert exact result fields before ignored values are normalized:

```toml
[[cases]]
id = "echo-basic"
tool = "echo"
arguments = { text = "hello" }
timeout_ms = 250

[cases.assertions]
"/content/0/text" = "hello"
"/isError" = false
```

Missing paths, mismatches, and case timeouts abort capture without updating a baseline. Diagnostics omit the expected and actual values. See `examples/assertions.toml` for a runnable fixture.

## Policy and validation

```sh
mcp-regress validate --config examples/fixture.toml --policy examples/policy.toml
mcp-regress check --config examples/fixture.toml --baseline fixtures/baseline.json --policy examples/policy.toml
```

Policies provide stable rule IDs, explicit severity overrides, and exact-name exceptions with a reason and UTC expiry date. Expired exceptions fail the run. Findings remain visible in reports. Output paths are checked before execution to prevent overwriting input snapshots or configuration.

## Development checks

```sh
python3 scripts/check.py
```

CI compares the mock server with the reviewed `fixtures/baseline.json`. Regenerate that file only as an intentional baseline update and review its diff.

## Enhancement planning

The editable [enhancement tracker](outputs/mcp-regress-planning-20261119/mcp-regress-enhancement-tracker.xlsx) contains 24 proposed ideas and the current engineering backlog. Use filters and amber cells to assign priorities, owners, statuses and target releases. The engineering sheet is a snapshot; synchronize accepted work with `docs/BACKLOG.md` when implementation changes.

## License

MIT. See [LICENSE](LICENSE).
