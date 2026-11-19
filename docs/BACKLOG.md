# Delivery backlog

Update this file when work starts or a gate is completed. IDs are stable; split an item into child IDs if it grows. Status values: `todo`, `in_progress`, `blocked`, `done`. `Done` requires evidence linked in the final column. Dependencies refer to IDs below. Priority: P0 blocks a trustworthy release; P1 is needed for broad adoption; P2 follows pilot evidence.

| ID | Stage / priority | Work item | Depends on | Acceptance evidence | Status |
| --- | --- | --- | --- | --- | --- |
| DISC-01 | 0 / P0 | Interview 10 maintainers; record jobs, alternatives, adoption and pricing signals without private data | — | Interview summary and ranked problem list | todo |
| DISC-02 | 0 / P0 | Collect 3 consented representative fixture projects and failure cases | DISC-01 | Reproducible, sanitized corpus with expected outcomes | todo |
| SPEC-01 | 0 / P0 | Specify CLI/config/snapshot/report v1, errors and exit codes | — | [Contract](CONTRACT.md) and [ADR](decisions/0001-local-gate-v02.md) | done |
| SPEC-02 | 0 / P0 | Choose protocol compatibility profiles and test matrix | DISC-02 | ADR with independent server evidence | todo |
| SEC-01 | 0 / P0 | Complete threat model and default data policy | — | Reviewed [SECURITY.md](SECURITY.md) with owners for open risks | todo |
| CORE-01 | 1 / P0 | Extract library and transport interface without changing CLI behavior | SPEC-01 | `src/lib.rs`, generic `Transport`, CLI/E2E equivalence | done |
| CORE-02 | 1 / P0 | Bound line/frame size before allocation, page count, duration and total run resource use | CORE-01, SEC-01 | Frame/output/run/tool limits implemented; child resource isolation remains | in_progress |
| CORE-03 | 1 / P0 | Validate JSON-RPC envelope, IDs, protocol lifecycle and negotiated version | SPEC-02, CORE-01 | Local 2025 profile validation tested; independent conformance outstanding | in_progress |
| CORE-04 | 1 / P0 | Separate RPC/tool errors from transport, timeout and process failures | CORE-03 | `tests/core.rs` and `tests/e2e.py`: unexpected errors abort capture | done |
| CORE-05 | 1 / P0 | Terminate child process trees; handle cancellation and stderr safely | CORE-01, SEC-01 | Linux group timeout tested; Windows descendants still outstanding | in_progress |
| CORE-06 | 1 / P0 | Prove deterministic ordering and normalization rules | CORE-03 | Repeated capture passes on Linux/macOS/Windows: [CI evidence](COMPATIBILITY.md#hosted-ci-evidence) | done |
| CI-01 | 1 / P0 | Stop generating the baseline inside the same CI check that compares against it | SPEC-01 | Checked-in `fixtures/baseline.json`, clean/drift checks in CI | done |
| FIX-01 | 1 / P1 | Add case setup/teardown, expected errors, run budgets and safe side-effect opt-in | CORE-04, SEC-01 | Expected errors/run budgets implemented; setup/teardown and side-effect policy remain | in_progress |
| DATA-01 | 2 / P0 | Version and migrate snapshot/report schemas; record provenance and integrity | SPEC-01, CORE-06 | V2 checksum/fixture provenance; v1 reads; binary/signature/migration work remains | in_progress |
| DATA-02 | 2 / P0 | Redact secrets before persistence and report emission | SEC-01, DATA-01 | Recursive key redaction/private files; free-text scanning remains | in_progress |
| DIFF-01 | 2 / P0 | Implement conservative JSON Schema compatibility rules | DATA-01, DISC-02 | Conservative object-schema proof and inconclusive fallback; full engine remains | in_progress |
| DIFF-02 | 2 / P0 | Compare annotations, descriptions, output schema and behavioral cases with rule IDs | DIFF-01 | Stable rule IDs/per-field paths implemented; full schema engine remains | in_progress |
| POL-01 | 2 / P1 | Add policy file, expiry-bound exceptions and stable exit mapping | DIFF-02 | [Contract](CONTRACT.md), `src/policy.rs`, exact-name expiry tests and CLI E2E | done |
| UX-01 | 2 / P1 | Improve console/JSON/Markdown reports and baseline review workflow | DATA-02, POL-01 | Private JSON plus payload-free console/Markdown; pilot evidence outstanding | in_progress |
| HTTP-01 | 3 / P1 | Implement Streamable HTTP transport and version-specific lifecycle | CORE-03, SEC-01 | Independent servers pass transport conformance corpus | todo |
| HTTP-02 | 3 / P1 | Add URL/TLS/redirect/proxy/network policy and credential provider | HTTP-01 | SSRF/auth threat cases rejected; no credential persistence | todo |
| SCALE-01 | 3 / P2 | Bounded parallelism, cache and observability with opt-in telemetry | HTTP-02 | Resource budgets and reproducible results under load | todo |
| AGENT-01 | 4 / P1 | Write agent-neutral skill for inspect/check/report workflow | UX-01 | Skill runs against a clean example without changing baseline | todo |
| AGENT-02 | 4 / P1 | Build optional local MCP adapter over library with narrow tools | AGENT-01, SEC-01 | No arbitrary command/path reachability; audit of tool inputs | todo |
| AGENT-03 | 4 / P1 | Validate Claude Code, Cursor, Codex and VS Code Copilot guides | AGENT-01, AGENT-02 | Per-agent evidence in [COMPATIBILITY.md](COMPATIBILITY.md) | todo |
| REL-01 | 5 / P0 | Publish signed, reproducible platform artifacts and install docs | UX-01 | Clean-machine install/rollback verified per supported platform | todo |
| REL-02 | 5 / P0 | Add supply-chain checks, SBOM, advisory and support process | SEC-01, REL-01 | Release checklist and response owner | todo |
| PILOT-01 | 5 / P0 | Run external pilot and decide release scope | UX-01, AGENT-03, REL-02 | Three real PR gates, feedback metrics and release decision | todo |
| BIZ-01 | 5 / P2 | Evaluate hosted/team features and marketplace listing | PILOT-01 | Demand, license, security and cost decision recorded | todo |

## Working cadence

- Weekly: review top P0 items, blockers, user evidence, security issues, and scope changes.
- Per PR: cite ID, explain user-visible behavior, record verification evidence, update docs and status.
- Per milestone: check exit gate in [ROADMAP.md](ROADMAP.md), update [COMPATIBILITY.md](COMPATIBILITY.md), decide next scope in a decision record.
- Per release: review default data handling, vulnerabilities, supported environments, migration notes, and rollback path.

## Decision record template

Create `docs/decisions/NNNN-short-title.md` with: **Status**, **Date**, **Context**, **Options**, **Decision**, **Consequences**, **Evidence**, and **Revisit trigger**. Link it from the relevant backlog item. Record uncertainty rather than implying that a planned feature is already supported.
