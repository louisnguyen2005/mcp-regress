# Implementation checkpoint — 2026-11-19

## Scope delivered

v0.2 is the first local-engine milestone toward the requested larger product. The full roadmap includes research, independent compatibility, remote execution, agent integration, and release operations; no evidence supports calling exactly 50% of that entire product complete. This checkpoint makes the completed scope and remaining work explicit.

Implemented:

- Reusable Rust library and generic transport interface, with thin CLI orchestration.
- Bounded incremental frames, cumulative stdout budget, request/run deadlines, tool/page/message/config/snapshot limits.
- Strict JSON-RPC result/error/ID validation and explicit 2025-06-18 initialization profile.
- Expected RPC and tool errors; transport and unexpected errors abort without replacing a baseline.
- Unix process groups and SIGINT-aware CLI cleanup; suppressed stderr and remote error messages.
- Stable case/tool ordering and strict RFC 6901 normalization that preserves array positions.
- Snapshot v2 with protocol/runner/fixture provenance and content checksums; v1 read compatibility.
- Atomic private output files, built-in/custom recursive response key redaction, console/Markdown without raw response bodies.
- Per-field tool changes, conservative input schema compatibility proof, blocking inconclusive classifications, stable exit codes.
- Reviewed baseline fixture and a CI matrix configured for Linux/macOS/Windows.
- Contract/migration notes, ADR, updated security gaps and backlog evidence.

## Local validation

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --all-targets`: 11 Rust tests.
- `cargo build --locked --bins --examples` then `python3 tests/e2e.py`: 13 real-process tests.
- Reviewed baseline check passes; changed mock behavior exits 2.
- Validation ran on the local Linux environment. Hosted CI and agent compatibility have not run here.

Test cases cover invalid envelopes/IDs, unsupported revisions, repeated cursors, oversized frames, total/request timeout, explicit error expectations, no baseline replacement on failure, deterministic capture, secret canaries, atomic permissions/symlinks, corrupted content, schema fallback, exit behavior, and Unix descendant termination.

## Next work

1. Independent MCP server corpus and full conformance evidence for the supported profile.
2. Unix detached-session containment, explicit fixture side-effect control and setup/teardown.
3. Full conservative schema rule engine, versioned policy files, secret scanning, signed/binary provenance.
4. Streamable HTTP/auth/network policy, then narrow agent adapter and per-agent installation evidence.
5. External pilot and signed releases.

See [BACKLOG.md](BACKLOG.md) for statuses. Items with external evidence or unfinished subfeatures remain `in_progress` or `todo` even when a foundation exists.

## v0.3 follow-up

Implemented versioned policy files, stable finding rule IDs and report v3, explicit severity overrides, exact-name expiring exceptions, and a config/policy validation command. Added pre-execution output alias protection and stronger snapshot identity/provenance checks. Hashes now stream borrowed data without copying full results, preserving the checked-in v2 baseline. Narrowed Tokio features and added weekly Cargo/Actions dependency updates.

The shared automated command is `python3 scripts/check.py`: formatting, Clippy, 20 Rust tests, 19 real-process CLI tests, build, reviewed-baseline pass, and intentional-drift exit 2 all pass locally on Linux. Tests include policy expiry/leap dates, alias/symlink protection, canonical hash compatibility, notification floods, schema narrowing, and SIGINT process cleanup. CI runs the same command on three operating systems; hosted runs are recorded below.

Hosted validation for `f805b69` passed on all three operating systems ([run](https://github.com/louisnguyen2005/mcp-regress/actions/runs/44485383205)). Deprecated action runtimes were then upgraded to official Node 24 releases and all action revisions pinned; checkout credentials are not retained. The final follow-up CI run validates these workflow-only updates.

## v0.4 fixture checks

FIX-02 adds exact JSON Pointer assertions evaluated before normalization/redaction and optional per-case tool deadlines. Five additional process tests verify success, mismatch privacy, missing paths, shorter deadlines, and invalid settings. Existing captures with no new settings retain their fixture digests. Future enhancement management is delivered in an editable XLSX tracker under `outputs/mcp-regress-planning-20261119/`; engineering status remains authoritative in the Markdown backlog and tracker edits are not automatically synchronized.

Local v0.4 validation passes formatting, Clippy, 20 Rust tests, 24 real-process tests and the unchanged clean/drift baseline gates. The tracker contains 24 proposals and 30 engineering records, with recalculated counts, dropdowns, filters and frozen headers verified after export.


## v0.5 clean child environment

SEC-01 records the default execution/data policy and gives each open threat a responsible owner role. Individual security contacts remain a pre-release task. SEC-02 adds opt-in `server.env_clear`; explicit config variables are the only values forwarded and absolute executables are required. The compatible default keeps inherited environment behavior. Four E2E cases cover parent secret removal, explicit allowlisted values, unchanged legacy inheritance, and invalid relative commands. `python3 scripts/check.py` passes formatting, Clippy, 20 Rust tests, 28 real-process CLI tests, and clean/drift baseline gates on local Linux. The later three-platform run for v0.6 also validates this behavior; no new agent interoperability claim is made.

## v0.6 Windows process-tree cleanup

CORE-05 adds a Windows-only Job Object process group. The child is assigned before it resumes, so children and grandchildren inherit job membership; startup fails if job setup fails. Linux/macOS retain their existing process groups. A Windows-only E2E test verifies both descendants exit after a server timeout; Linux timeout and SIGINT tests now verify a grandchild as well. Rust 1.88 is the declared minimum due to the target-specific process-management dependency. Local validation passes formatting, Clippy, 20 Rust tests, and 29 E2E cases (28 passed, one Windows-only case skipped on Linux), plus clean/drift baseline gates. Linux-to-Windows `cargo check --all-targets` passes for both GNU and MSVC targets. Hosted validation passed on Ubuntu, macOS, and Windows, including the Windows descendant test ([run](https://github.com/louisnguyen2005/mcp-regress/actions/runs/44676697169)). Detached Unix sessions remain an open risk, so CORE-05 stays `in_progress`.

## v0.7 public-server evidence and fixture safety

One current public MCP server package, `@modelcontextprotocol/server-filesystem@2.4.0`, passed a repeatable Linux stdio run on the implemented `2025-06-18` profile. The probe uses a lockfile-pinned package, `npm ci --ignore-scripts`, `--read-only`, synthetic temporary files, two captures with an identical content digest, and a separate zero-change `check`. It lists 10 tools and passes directory-list/read cases; the [evidence record](COMPATIBILITY.md#independent-server-run-2026-11-20) intentionally claims only this subset.

FIX-01 now blocks case calls unless the server marks a tool `readOnlyHint = true` or that individual case sets `allow_side_effects = true`. The flag participates in fixture provenance. E2E tests cover both blocked calls (false and missing hints) and an explicitly approved test mutation. Server annotations remain untrusted hints, the server is not sandboxed, and setup/teardown remain outstanding. Broader adoption evidence still requires consented maintainer fixtures; this public sample does not satisfy DISC-02.

Full local validation for v0.7 passes `python3 scripts/check.py`: formatting, Clippy, 20 Rust tests, 31 real-process E2E cases (one Windows-only case skipped on Linux), and clean/drift checks against the updated reviewed baseline. The optional pinned public-server probe also passes separately; it is not part of offline CI.

## v0.8 fixture effect policy and lifecycle

SEC-IDEA-007 / FIX-01 adds explicit `auto`, `read_only`, `mutating`, and `destructive` classifications. The default only permits a server tool marked `readOnlyHint = true`; mutating and destructive cases require `allow_side_effects = true` and a cleanup tool action. Optional setup runs before the case. Cleanup is attempted after setup or case failures, and cleanup failure aborts capture before baseline storage. Lifecycle responses are not persisted. The runnable [fixture example](../examples/fixture-lifecycle.toml), [contract and migration note](CONTRACT.md#fixture-effects-and-lifecycle-v08), and [ADR 0007](decisions/0007-fixture-effect-lifecycle.md) document the behavior.

This is not rollback or process isolation: the server can lie about tool behavior, external actions may be irreversible, and process or total-run termination may interrupt cleanup. Use disposable data and credentials. The config fixture digest includes explicit classifications and lifecycle actions; default `auto` remains omitted to preserve existing read-only fixture digests. v0.7 configurations that opted in with only `allow_side_effects = true` must add an effect classification and cleanup action.

`python3 scripts/check.py` passes formatting, Clippy, 21 Rust tests, 35 real-process E2E cases (one Windows-only case skipped on Linux), and clean/drift checks. A manual run of `examples/fixture-lifecycle.toml` confirmed setup, case execution, and cleanup remove the synthetic marker. The v0.8 baseline diff changes only the runner version; content and fixture digests are unchanged for the default read-only example.

## v0.9 hostile MCP output regression corpus

SEC-03 adds deterministic mock variants that place instruction-like text and a canary in tool descriptions, input-schema metadata, tool results, and JSON-RPC error message/data. E2E checks verify that terminal and Markdown output omit these values, the private JSON report retains them only in an owner-only Unix artifact, and a failed hostile RPC capture leaves a previously reviewed baseline byte-for-byte unchanged. CLI output assertions use generic failure messages so a regression does not print the fixture payload into CI logs. The test does not evaluate an LLM or provide general free-text secret scanning.

`python3 scripts/check.py` passes formatting, Clippy, 21 Rust tests, 37 real-process E2E cases (one Windows-only case skipped on local Linux), and the reviewed clean/drift baseline gates. Research notes and the synthesis in [`reports/MCP nâng cao kiểm thử agent.md`](../reports/MCP%20n%C3%A2ng%20cao%20ki%E1%BB%83m%20th%E1%BB%AD%20agent.md) informed SEC-03 and three follow-on ideas in the [46-item tracker](../outputs/mcp-regress-real-world-ideas-20261119/mcp-regress-real-world-ideas.xlsx). The report distinguishes benchmark/preprint findings from protocol conformance and customer-demand evidence.
