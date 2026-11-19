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
2. Windows Job Object cleanup, explicit fixture side-effect control and setup/teardown.
3. Full conservative schema rule engine, versioned policy files, secret scanning, signed/binary provenance.
4. Streamable HTTP/auth/network policy, then narrow agent adapter and per-agent installation evidence.
5. External pilot and signed releases.

See [BACKLOG.md](BACKLOG.md) for statuses. Items with external evidence or unfinished subfeatures remain `in_progress` or `todo` even when a foundation exists.

## v0.3 follow-up

Implemented versioned policy files, stable finding rule IDs and report v3, explicit severity overrides, exact-name expiring exceptions, and a config/policy validation command. Added pre-execution output alias protection and stronger snapshot identity/provenance checks. Hashes now stream borrowed data without copying full results, preserving the checked-in v2 baseline. Narrowed Tokio features and added weekly Cargo/Actions dependency updates.

The shared automated command is `python3 scripts/check.py`: formatting, Clippy, 20 Rust tests, 19 real-process CLI tests, build, reviewed-baseline pass, and intentional-drift exit 2 all pass locally on Linux. Tests include policy expiry/leap dates, alias/symlink protection, canonical hash compatibility, notification floods, schema narrowing, and SIGINT process cleanup. CI runs the same command on three operating systems; remote job results must be checked after push.

Hosted validation for `f805b69` passed on all three operating systems ([run](https://github.com/louisnguyen2005/mcp-regress/actions/runs/44485383205)). Deprecated action runtimes were then upgraded to official Node 24 releases and all action revisions pinned; checkout credentials are not retained. The final follow-up CI run validates these workflow-only updates.
