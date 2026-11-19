# mcp-regress product and delivery roadmap

Updated 2026-11-19. This is a planning document, not a claim that future capabilities already exist. Track execution in [BACKLOG.md](BACKLOG.md). Security controls and compatibility evidence live in [SECURITY.md](SECURITY.md) and [COMPATIBILITY.md](COMPATIBILITY.md).

## Product definition

**Primary user:** maintainers of MCP servers who need a repeatable CI gate before releases. Secondary users: platform teams checking many servers and agent developers verifying that tool behavior remains usable. The current program is a Rust **CLI that acts as an MCP client**. It is not itself an MCP server; an optional agent-facing MCP adapter is a later product surface.

**Core promise:** capture a reviewed contract, replay safe fixtures, explain meaningful changes, and produce a reproducible pass/fail result. Avoid promising that a passing run proves safety, semantic correctness, or compatibility with every agent.

**Initial target environment:** local or CI-owned servers via stdio, nonproduction fixture data, Linux/macOS/Windows runners, then remote Streamable HTTP after a security design. Existing `2025-06-18` behavior remains supported while newer protocol versions are added deliberately.

## Evidence before expansion

Interview at least 10 maintainers across open-source, internal platform, and commercial MCP servers. Collect one anonymized real-world fixture set from at least three teams. Ask where their current release checks fail, whether they need transport checks, semantic schema compatibility, output redaction, agent setup, or fleet governance. Record alternatives (custom scripts, MCP Inspector, contract tests) and willingness to adopt/pay. Promote features by repeated pain and usage, not by marketplace copy. A free/open core and paid hosting or team features are options to validate, not predetermined pricing.

Success signals for the first public release: at least three external maintainers use the gate on real PRs; all three can reproduce a failing change locally; no known secret leakage in default output paths; an actionable report is produced for every supported failure class; and setup works from a clean machine using the documented release artifact. Track setup time, flaky-run rate, report usefulness, and false positive/negative examples. Do not infer market size from Agensi's outreach traffic claim.

## Delivery sequence and release gates

| Stage | Outcome and scope | Exit gate |
| --- | --- | --- |
| 0. Baseline and discovery | Inventory current behavior, write compatibility contracts, gather user cases, define threat model and success metrics. | Supported/unsupported matrix and at least three representative fixture repositories exist; open questions have owners. |
| 1. Reliable local gate | Harden stdio framing, JSON-RPC validation, negotiated protocol behavior, limits, process cleanup, deterministic capture and failure taxonomy. | Golden and adversarial fixtures pass on supported OSes; repeated runs yield identical snapshots; failures never silently become passing snapshots. |
| 2. Contract intelligence | Versioned snapshots, migration, semantic schema diff, configurable policies, redaction, secret detection, rich machine and human reports. | Reviewers can distinguish additive, breaking, behavioral, and inconclusive results with evidence and stable exit codes. |
| 3. Remote and enterprise safety | Streamable HTTP client, explicit network policy, authentication and secret handling, bounded concurrency, telemetry controls. | Remote runs meet the security controls below; transport and protocol conformance is demonstrated with independent servers. |
| 4. Agent experience | Agent-neutral skill instructions and optional local MCP adapter that invokes the same core engine; documented setup for named agents. | Each supported agent passes a recorded install/list/run/review workflow; no agent may silently approve baseline updates. |
| 5. Distribution and operations | Signed release artifacts, package channels, CI templates, support policy, release process, feedback loop. | Clean-machine install, upgrade/rollback, license and vulnerability response are documented; external pilot gate is met. |

Each stage may be split into releases. Do not advance a stage merely because its code exists: require the evidence in the gate. After stage 2, run a pilot before committing to stage 3/4 scope. HTTP server hosting, SaaS dashboards, multi-tenant control planes, and paid skill listings require separate demand and security decisions.

## Functional requirements

1. **Capture:** enumerate tools with pagination; preserve the complete tool contract, metadata, input/output schemas, annotations, and protocol version. Capture fixtures with stable ordering and content hashes. Distinguish tool-level `isError`, JSON-RPC errors, process failures, and timeouts.
2. **Comparison:** classify additions, removals, schema compatibility, descriptions/annotations, and fixture behavior. A configurable policy decides what blocks a release. Unknown or unsupported schema constructs must be `inconclusive`, not silently accepted.
3. **Fixture control:** support isolated data, setup/teardown, deterministic seeds/time, per-case deadlines, explicit expected errors, safe ordering and optional parallelism. Side-effecting calls require opt-in and documented cleanup.
4. **Snapshot governance:** human-reviewed baseline promotion, format version/migration, provenance (server artifact and config digest), file integrity, redaction before persistence, and readable diffs. CI must compare against a checked-in or immutable trusted baseline.
5. **Reports:** JSON with stable schema, Markdown for PRs, concise console summary, structured diagnostics, source and rule IDs, redacted samples, and exit codes that separate policy findings from execution/infrastructure errors.
6. **Interoperability:** stdio first; Streamable HTTP later. Implement protocol versions as explicit profiles and test negotiated behavior. Agent integrations consume the stable CLI/library and share the same policy engine.
7. **Operations:** bounded memory/time/output, no shell interpolation, process-tree termination, offline mode where possible, release artifacts, dependency and vulnerability monitoring, and a documented support window.

## Architecture direction

Refactor into a reusable Rust library with `transport`, `protocol`, `capture`, `normalize`, `compare`, `policy`, `report`, and `storage` modules. Keep the CLI thin. An optional agent-facing MCP server invokes the library and exposes a small read-first tool surface such as `inspect_config`, `run_check`, and `summarize_report`; baseline promotion remains an explicit user action. Use a transport trait so stdio and HTTP share protocol semantics. Pin snapshot/report schema versions independently of the CLI version. Document an ADR before selecting an MCP SDK or replacing the current client implementation.

Boundary rule: the tool being inspected executes with the runner's privileges. The product can enforce path, network, timeout, and logging controls around its own behavior, but a true sandbox needs an external container/VM or OS policy. Do not label the runner as sandboxed without one.

## Detailed stage work

### Stage 0: define the contract

Map current commands (`baseline`, `replay`, `diff`, `check`), exit codes, config v1, snapshot v1, and report v1. Build a corpus of compliant and malformed servers. Decide supported protocol versions from observed clients and independent conformance tests. Document lifecycle differences, including the 2026-07-28 request metadata/stateless model; do not simply swap the hard-coded version string.

### Stage 1: correctness and determinism

Add bounded streaming reads (the current 8 MiB check happens after a full line is read), total run budgets, cancellation, strict response ID/JSON-RPC validation, notification handling, pagination limits, protocol negotiation, and process-tree cleanup. Make execution/configuration errors fatal to capture unless a case explicitly expects an error. Normalize only fields a user has explicitly declared volatile. Record ignored paths and flag missing pointers. Verify same inputs produce byte-identical snapshots across repeated runs and OSes where promised.

### Stage 2: useful change decisions

Introduce a compatibility rule engine rather than treating every tool change as breaking. Compare JSON Schema under an explicit draft/profile; evaluate required fields, property additions/removals, types, enums, defaults, constraints, and output schema. Track unknown constructs as inconclusive. Add policy files with rule IDs, severity overrides, allowlists with expiry, and a strict mode. Add secret-safe artifact handling, report schema documentation, migration commands, and baseline review workflow. Keep `--fail-on` behavior through a documented migration.

### Stage 3: remote execution

Support Streamable HTTP with TLS verification, target allowlists, redirect limits, authentication suited to the selected spec version, proxy behavior, and SSRF controls. Keep OAuth and token storage outside plain fixture files. Add retry only for clearly safe idempotent operations; never replay a side-effecting tool call automatically. Record transport and auth provenance without persisting credentials. Design hosted execution only after tenancy, isolation, data retention, and threat review are approved.

### Stage 4: agents

Publish an agent-neutral skill that explains when to inspect, check, interpret, and propose a baseline update. Provide validated examples for Claude Code, Cursor, Codex, and VS Code Copilot. The optional local MCP adapter should expose a narrow API over the library, with explicit workspace and command allowlists and no arbitrary command execution. The skill should guide the agent; the deterministic gate must remain useful without an LLM or marketplace dependency. Verify each agent independently because MCP feature support and configuration differ.

### Stage 5: release

Ship reproducible builds for supported platforms, checksum/signature verification, SBOM, license inventory, changelog, installation/upgrade docs, CI templates, and a vulnerability disclosure path. Define semantic versioning for CLI, config, snapshot, report, and adapter interfaces. Maintain a small public example and a private adversarial corpus. Review adoption and support costs after the pilot before adding commercial features.

## Decisions still open

| Decision | Needed by | Evidence required |
| --- | --- | --- |
| Which MCP versions to support simultaneously | Stage 1 | Official spec plus real server/client conformance runs |
| JSON Schema compatibility profile | Stage 2 | Corpus showing false positives/negatives and conservative fallback |
| Baseline storage/signing model | Stage 2 | CI threat model and maintainer workflow |
| HTTP auth mechanism and credential source | Stage 3 | Current MCP auth spec and deployment scenarios |
| Agent adapter versus skill-only distribution | Stage 4 | Pilot usage and agent compatibility runs |
| Paid packaging and Agensi listing | Stage 5 | User demand, license review, and support cost |

## Sources checked on 2026-11-19

- MCP 2026-07-28 specification and changes: https://modelcontextprotocol.io/specification/2026-07-28 and https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/changelog.mdx
- MCP client feature matrix: https://modelcontextprotocol.io/clients
- OWASP MCP security guidance: https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html
- VS Code MCP configuration: https://code.visualstudio.com/docs/agents/reference/mcp-configuration

Recheck these sources at implementation time; client and protocol support changes.
