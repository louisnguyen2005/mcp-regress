# Security model and controls

Reviewed 2026-11-19. This file is a design checklist for planned work and a statement of current limits. Open risks have a responsible owner role and backlog link below; individual incident-response contacts still need to be named before a public release. See [ROADMAP.md](ROADMAP.md) for sequencing.

## Trust boundaries

1. The CLI reads a local TOML config and launches its configured command directly. Whoever controls config controls a process with the runner's privileges. The current tool is **not a sandbox**.
2. The inspected MCP server controls its stdout, stderr, tool catalog, and tool results. All of these are untrusted. Tool text may contain prompt injection aimed at an agent that later reads a report.
3. Fixture arguments can invoke side-effecting tools. A CI runner should use disposable credentials and test data.
4. Snapshots and reports can contain sensitive tool responses, arguments, URLs, identifiers, and error strings. Current `ignore_pointers` replaces selected values from successful tool responses; it is not comprehensive redaction.
5. Future HTTP support creates an additional URL, network, OAuth and redirect boundary. Future agent adapter support creates a model-to-local-command boundary.

## Implemented controls in v0.5

Bounded incremental frames and total stdout/run limits replace unbounded line reads. JSON-RPC envelopes, response IDs, initialization identity/tools/revision and pagination are validated. Unexpected RPC/tool errors and all transport failures abort capture. Files are atomically replaced with private Unix permissions and snapshot content checksums. CLI SIGINT triggers transport cleanup, and Unix child groups are killed. Server stderr and remote RPC message/data are suppressed. Common response secret keys are redacted; console/Markdown omit response payloads. CI uses a reviewed baseline. Output aliases to inputs are rejected before execution. Policy exceptions are exact-name scoped, retain finding evidence, and expire; expired exceptions cannot suppress checks. Parent directories must still be trusted against concurrent replacement.

An opt-in `server.env_clear` mode launches with an empty child environment, then adds only explicit server values. It requires an absolute executable. The default remains inherited for compatibility. Environment clearing reduces accidental secret forwarding but is not process isolation.

## Default data and execution policy

- Treat the configured child command as trusted code because it runs with the runner's operating-system permissions. Prefer a disposable CI worker and fixture account. `server.env_clear = true` reduces inherited-variable exposure, but does not restrict files, network, credentials explicitly supplied, or child capabilities.
- Use synthetic or disposable test data. Keep fixture arguments, snapshots, reports, and server output private when they may contain sensitive data. Do not put credentials in checked-in TOML. The current key-based redaction does not detect secrets embedded in free text.
- Treat all server-controlled output as untrusted data. A passing check is evidence of observed contract consistency only; it does not prove that a server is safe or that an AI agent will use it correctly.
- CI must compare with a reviewed immutable baseline. Baseline generation and promotion remain deliberate maintainer actions, separate from the check job.

## Open risk register and owners

Ownership below is assigned by role so the work is not ownerless. The repository maintainer is accountable for keeping the register current and for naming individuals when the project establishes a release and incident-response process. This is an internal threat-model review, not an independent security audit.

| Risk | Owner role | Follow-up |
| --- | --- | --- |
| Configured server command can access runner files, network and OS privileges; no CPU/memory sandbox exists | Project maintainer | [CORE-02](BACKLOG.md#delivery-backlog), [SEC-02](BACKLOG.md#delivery-backlog) |
| Windows child descendants and detached Unix descendants may outlive cancellation | Cross-platform runtime maintainer | [CORE-05](BACKLOG.md#delivery-backlog) |
| Secrets in free text, schemas, arguments or private reports can escape key-based redaction | Artifact and reporting maintainer | [DATA-02](BACKLOG.md#delivery-backlog), [UX-01](BACKLOG.md#delivery-backlog) |
| Implemented MCP profile lacks independent server/client conformance evidence and newer protocol behavior | Protocol compatibility maintainer | [SPEC-02](BACKLOG.md#delivery-backlog), [CORE-03](BACKLOG.md#delivery-backlog) |
| Future remote transport could create SSRF, redirect, DNS rebinding and OAuth credential risks | Transport security maintainer | [HTTP-01](BACKLOG.md#delivery-backlog), [HTTP-02](BACKLOG.md#delivery-backlog) |
| Future agent adapter could expose local commands or treat malicious server text as instructions | Agent integration maintainer | [AGENT-02](BACKLOG.md#delivery-backlog), [UX-01](BACKLOG.md#delivery-backlog) |
| No published private vulnerability channel or named incident response contact | Project maintainer / security contact | [REL-02](BACKLOG.md#delivery-backlog); name contacts before public release |

## Remaining gaps before broad production use

- No sandbox or child CPU/memory limits. The default inherited environment may expose CI credentials; opt-in clean environment mode reduces that exposure.
- Windows direct-child termination does not yet cover descendants; detached Unix sessions may also escape process groups.
- Redaction is key-based and cannot detect secrets in text or schema metadata; private JSON reports still contain server data.
- Content checksums are not signatures; server binary and dependency provenance are not captured.
- Only the 2025-06-18 profile is implemented and has only local mock evidence. Independent conformance is outstanding.
- Full schema compatibility reasoning, secret scanning, external credential providers, HTTP/network controls, and agent adapter isolation are outstanding.
- No published private vulnerability reporting channel or assigned incident response owner exists yet.

## Required controls by boundary

| Boundary | Control | Verification |
| --- | --- | --- |
| Config and command | Validate paths and values; no shell expansion; explicit executable/workspace allowlist in agent adapter; warn on side effects | Malicious and malformed config corpus |
| Child process | Total/per-request deadlines, resource limits where available, process-tree termination, stderr cap and redaction | Hang, fork, flood and cancellation cases |
| MCP wire | Bounded incremental framing, envelope/ID/version checks, page and message limits, conservative errors | Protocol conformance and adversarial server corpus |
| Fixture data | Disposable test identities; no production secrets by default; explicit side-effect policy; setup/cleanup | Fixture review and post-run cleanup checks |
| Artifacts | Redact before write; atomic restrictive-permission writes; schema/provenance/hash; safe path handling | Secret canaries and artifact permission checks |
| Reports to agents | Escape/label server-controlled text as data; avoid interpreting it as instructions; trim large payloads | Prompt-injection corpus and manual review |
| HTTP targets | HTTPS by default, allowlisted hosts/IP policy, DNS rebinding and redirect checks, TLS validation, bounded body | SSRF, redirect and certificate cases |
| Authentication | Least-privilege scopes, external credential provider, no token in config/snapshot/log, rotation support | Credential leakage checks |
| Distribution | Locked deps, vulnerability review, signed checksums/artifacts, SBOM, release provenance | Release checklist |

## Baseline integrity workflow

Baseline capture must be a deliberate maintainer operation on a trusted server version. Code review should inspect the diff and provenance before promoting it. CI checks against that reviewed baseline and must not regenerate it just before comparison. A passing check only states that the observed, configured behavior matches policy; it does not certify the server's security.

## Incident and vulnerability process to add before public release

Define a private reporting address, response owner, severity/triage deadlines, supported versions, patch and advisory process, and credential/data exposure response. A `SECURITY.md` at repo root may later publish that policy; this document is the engineering threat model.

References: https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html and https://modelcontextprotocol.io/specification/2026-07-28
