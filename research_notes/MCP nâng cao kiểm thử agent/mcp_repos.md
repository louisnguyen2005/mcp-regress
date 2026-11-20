# GitHub repositories and official tools for production-grade MCP regression testing

Research snapshot: 2026-11-20. Repository activity and project-reported conformance numbers below are observations of upstream pages on this date, not independent audits. The repositories below were selected for direct relevance to protocol conformance, SDK interoperability, realistic test fixtures, packaging/CI, and JSON Schema validation.

## Key Question 1: What should mcp-regress use as its protocol correctness reference?

### Takeaway

Treat the official MCP Conformance Test Framework as the protocol-level oracle, while retaining mcp-regress's deterministic trusted-baseline comparison as a distinct regression check. Pin an explicit conformance release and dated spec requirement set; show required, optional/extension, pending, and unmeasured results separately. Keep expected-failure baselines narrow, reviewable, and stale-sensitive.

### Cited Findings

- The official [`modelcontextprotocol/conformance` README](https://github.com/modelcontextprotocol/conformance/blob/main/README.md) defines both client and server testing: client scenarios start a test server, run the client, capture protocol traffic, and check it against the spec; server scenarios connect as a client and inspect server behavior. It documents JSON output, wire-schema checks, an SDK runner, and a composite GitHub Action. At this snapshot its CI example pins `@modelcontextprotocol/conformance@v0.1.11`.
- The framework's [requirements documentation](https://github.com/modelcontextprotocol/conformance/blob/main/README.md) distinguishes a release-frozen requirement set from the live-growing scenario suite. Its examples explicitly separate `not_scored` cases such as extensions, scenarios added after a spec release, and pending tests from scored requirements. The same README says expected-failure baselines catch both regressions and stale entries; a previously passing item that begins failing, or a baselined item that now passes, fails CI.
- The official [SDK integration guide](https://github.com/modelcontextprotocol/conformance/blob/main/SDK_INTEGRATION.md) makes client/server responsibilities explicit: clients accept the test server URL, `MCP_CONFORMANCE_SCENARIO`, and scenario context; maintainers own server start, health check, and cleanup. JSON reports and expected-failure files are supported.
- The official [`known-sdks.ts` runner config](https://github.com/modelcontextprotocol/conformance/blob/main/src/sdk-runner/known-sdks.ts) keeps build/run commands outside SDK repositories, supports per-SDK and per-spec overrides, and documents how the Rust SDK's same endpoint serves 2025-11-25 stateful and 2026-07-28 stateless behavior. The comments explain that forcing stateless mode for one leg had caused false failures in input-required scenarios; this is evidence that version-specific lifecycle details must match the implementation's real configuration.
- The official [Rust SDK roadmap](https://github.com/modelcontextprotocol/rust-sdk/blob/main/ROADMAP.md) reports 30/30 server scenarios and 39/39 scored client scenarios for its dated 2025-11-25 and 2026-07-28 suites. This is the SDK maintainers' self-reported CI evidence, not proof that all implementations or extensions are conformant. The [SDK's workflow](https://github.com/modelcontextprotocol/rust-sdk/blob/main/.github/workflows/conformance.yml) pins a specific conformance package version, runs separate spec-version client and server suites, runs extensions as informational, sets timeouts, and uploads result artifacts.

### Inferences

- Add an optional `mcp-regress conformance` integration that invokes the official runner (or imports its machine-readable reports) instead of reimplementing protocol assertions. Preserve the existing snapshot diff as a separate result category because it answers a different question: “did this server's advertised tool surface change from my approved baseline?”
- In reports, make status explicit: `required_pass`, `required_fail`, `not_scored`, `not_measured`, and `baseline_expected_failure`. Never roll an unrun test into a passing percentage. Surface the exact spec revision and runner version in report metadata.
- Support dual-revision operation with independent expected-failure baseline files when behavior/lifecycle differs across revisions. Do not silently regenerate trusted baselines alongside conformance verification.
- Offer a CI preset that starts the configured server, waits on an explicit readiness check, invokes pinned conformance, always terminates the child process, and uploads sanitized JSON evidence. This follows upstream's lifecycle pattern but should maintain mcp-regress's own safe log policy.

### Gaps

- A passing official suite proves only the scenarios and requirements that were run. Upstream classifies optional extensions and pending tests separately; it does not prove semantic correctness of tool business logic or real-agent task success.
- The SDK roadmap's pass counts are maintainer-reported and scoped to the dated suites. They should not be restated as an independently certified ecosystem-wide pass rate.
- The official conformance runner accepts executable client commands and SDK checkout paths. Local execution of those commands is code execution, not a sandbox. mcp-regress should clearly label that trust boundary and avoid running unreviewed downloaded code by default.
- The upstream requirement set is frozen per revision, while the general scenario suite evolves. Pin the package and requirements explicitly so CI results remain reproducible.

## Key Question 2: Which test-fixture and CI patterns improve realism and operational confidence?

### Takeaway

Use realistic protocol servers over real stdio and HTTP transports, and keep unit tests, process-level integration tests, end-to-end smoke tests, and packaged-artifact checks distinct. Exercise failure and restart behavior with dedicated subprocess fixtures rather than mocks alone.

### Cited Findings

- The official [MCP Inspector test-server catalogue](https://github.com/modelcontextprotocol/inspector/blob/main/docs/test-servers.md) describes composable fixtures with both in-process and subprocess modes. The subprocess fixtures run through real stdio and can deliberately crash mid-session; that crash tool is excluded from in-process presets because it exits the entire test runner. The catalogue also documents declarative JSON configs for choosing presets.
- The official TypeScript SDK's [`everythingServer.ts` conformance fixture](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/test/conformance/src/everythingServer.ts) contains scenario-specific tools for malformed capability cases, progress notifications, logging, reconnect behavior, sampling, elicitation, JSON Schema 2020-12, and newer multi-round input-required flows. The companion [`everythingClient.ts`](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/test/conformance/src/everythingClient.ts) is used as the shared client fixture by upstream's SDK runner.
- The official servers repository has a [root quality gate in `package.json`](https://github.com/modelcontextprotocol/servers/blob/main/package.json) that combines format/lint checks, coverage-scope checks, dependency lockstep checks, action pin checks, script tests, install verification, coverage, and smoke tests. Its [memory server package](https://github.com/modelcontextprotocol/servers/blob/main/src/memory/package.json) combines format, lint, typecheck, build, test, and coverage commands.
- The [official Inspector roadmap](https://github.com/modelcontextprotocol/inspector/blob/main/docs/inspector-roadmap-2026-h2.md) describes a ≥90% per-file coverage gate plus smoke/e2e checks aimed at catching packaging failures. It proposes recording and replaying protocol sessions, using recorded sessions as regression fixtures, and adding protocol/capability diffing and CI assertions.
- The official servers' [November 11, 2026 release notes](https://github.com/modelcontextprotocol/servers/releases) document concrete security regressions fixed in filesystem, git, and fetch servers: symlink/time-of-check-to-time-of-use races, special files, untrusted Git configuration executing programs, repository confinement gaps, and IPv6 private-address filtering. The same release notes show current versioning has moved from date versions to semver for some packages, a migration trap for consumers comparing package versions.

### Inferences

- Build a small, versioned fixture catalog for mcp-regress covering: clean baseline; changed tool descriptions/schema; invalid JSON-RPC; pagination; long/large output; timeout; server crash and restart; malformed/partial SSE; HTTP status/header mismatches; auth failures; and resource exhaustion. Test both subprocess stdio and Streamable HTTP paths where supported.
- Keep process-exit, timeout, network error, and protocol error outcomes distinguishable in result JSON. A mock that returns a prepared response cannot verify shutdown, stdout framing, subprocess leaks, or real transport lifecycle.
- Record/replay sanitized protocol transcripts as fixtures, but compare protocol shape, stable metadata, and explicitly selected fields instead of treating arbitrary tool result prose as deterministic.
- Add a release-validation command that installs/runs the built artifact from a clean temporary directory, verifies its CLI help/version and representative server run, and checks that reports/fixtures are actually included in the package. CI action dependencies should be pinned to immutable SHAs where practical.
- Derive a security regression corpus from upstream advisories and test cases: hostile output framing, URL/IP allowlist edge cases, symlink/fifo handling in any filesystem fixture, and Git config execution hazards. These are adversarial protocol/server fixture cases; they do not imply mcp-regress itself should execute or modify filesystem tools.

### Gaps

- The Inspector's roadmap is a maintainer plan, not an MCP requirement or commitment. Its feature priorities are useful product signals, not compatibility guarantees.
- The official test servers are still examples, not an exhaustive production threat model. In particular, tests for one OS or transport do not establish behavior on all supported platforms.
- A server can pass conformance while exposing unsafe business tools. mcp-regress should make protocol correctness and security-policy checks separate report groups.

## Key Question 3: How should MCP tool schemas be validated without creating a new security boundary?

### Takeaway

Validate advertised tool schemas and optionally validate tool inputs against their schemas, but use a pinned JSON Schema draft/profile and an offline resolver. Never allow a schema from an untrusted server to fetch arbitrary network or local-file `$ref` targets during validation.

### Cited Findings

- The current [MCP tools specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/server/tools.mdx) says tool `inputSchema` and `outputSchema` default to JSON Schema 2020-12 when `$schema` is absent, while its examples include an explicit draft-07 schema. The Rust SDK README also describes draft 2020-12 as the default dialect. Its [`ProtocolVersion` source](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/model.rs) gates protocol behavior by dated revision. The TypeScript SDK conformance fixture includes a [draft-2020-12 schema with `$defs`, `$anchor`, and `$ref`](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/test/conformance/src/everythingServer.ts).
- The Rust [`jsonschema` validator repository](https://github.com/Stranger6667/jsonschema) advertises support for popular drafts, meta-schema validation, structured validation outputs, and a CLI. Its [Cargo manifest](https://github.com/Stranger6667/jsonschema/blob/master/Cargo.toml) declares MSRV 1.85 and MIT license. Its README's feature table says HTTP and filesystem reference resolution are enabled by default (`resolve-http`, `resolve-file`); it also documents disabling defaults and opting into specific resolver/TLS features.
- The validator's [October 27, 2026 release notes](https://github.com/Stranger6667/jsonschema/releases) report a stack-overflow fix in 0.60.2 for cyclic `$ref` schemas across `$id` resources, and an earlier 0.60.1 fix for pathological large-exponent numeric validation performance. Its [migration guide](https://github.com/Stranger6667/jsonschema/blob/master/MIGRATION.md) documents API/behavior changes across releases.

### Inferences

- A JSON Schema integration is a strong candidate for `mcp-regress`: first meta-validate server-provided `inputSchema`/`outputSchema`, then optionally validate chosen arguments/results. Default to 2020-12 when `$schema` is absent, honor an explicit dialect when supported, and report unsupported dialects instead of silently interpreting them as 2020-12. Keep validation opt-in initially and expose schema dialect, validator version, and limits in the report.
- Build the validator with default features disabled and use an in-memory/local-only reference map. Reject or mark unsupported any remote `$ref`; do not let arbitrary server schemas create outbound HTTP requests or read filesystem paths. Bound schema byte size, depth, reference count, recursion, and validation time; test cyclic references and adversarial numbers.
- Prefer structured errors (JSON Pointer/instance path, keyword, schema path) over dumping the entire server response, so CI can identify failures without leaking tool data.
- Pin validator versions and run the upstream JSON Schema Test Suite or a carefully scoped subset against supported keywords/dialect. Treat `unknown` or unsupported formats as a stated limitation, not as valid by assumption.

### Gaps

- “Supports popular drafts” does not itself prove exact MCP 2020-12 dialect compliance for every keyword or format. Validate MCP-required schema semantics and version rules separately.
- Schema validation can detect malformed declarations and invalid payloads; it cannot determine whether a schema is safe, honest, useful to an agent, or semantically aligned with tool behavior.
- Upstream release notes show validator edge-case bugs have existed. A dependency update should include regression corpus runs, and security advisories should be monitored.

## Key Question 4: What product-level evidence would make mcp-regress useful to teams shipping MCP servers?

### Takeaway

Make the result a reviewable, replayable, CI-friendly compatibility artifact: frozen baseline plus server fingerprint, spec-versioned conformance, sanitized transcript, structured diff, reproducible command/environment metadata, and explicit limitations. Cross-agent tests are useful as a separate optional layer and need recorded interoperability runs before support is claimed.

### Cited Findings

- The official conformance runner's [SDK matrix](https://github.com/modelcontextprotocol/conformance/blob/main/src/sdk-runner/known-sdks.ts) separates SDK-specific commands from the test harness and overrides those commands for different spec revisions. This supports a data-driven interoperability matrix rather than hard-coding all cases into one execution path.
- The official Inspector roadmap's [record/replay and diff sections](https://github.com/modelcontextprotocol/inspector/blob/main/docs/inspector-roadmap-2026-h2.md) argue for saving protocol logs, negotiated capabilities, and server config in one artifact; replaying a saved session as a fixture; and comparing capabilities, sessions, and payloads.
- MCPJam's [open-source Inspector README](https://github.com/MCPJam/inspector) advertises cross-client/model evals, a CLI, SDK, OAuth checks, and CI. Those are vendor-authored feature claims, not independently verified accuracy or coverage measurements; they still indicate market interest in behavior evaluation beyond protocol conformance.
- The official MCP Inspector repository has ongoing 2026 issue/release activity, with its [issue tracker](https://github.com/modelcontextprotocol/inspector/issues) showing work opened in November 2026. The official servers repository's [release history](https://github.com/modelcontextprotocol/servers/releases) also shows active releases and security fixes in November 2026.

### Inferences

- Suggested artifact fields: tool name and normalized description/schema hash; server name/version; transport; negotiated protocol revision; mcp-regress version; conformance runner/version; timestamps; relevant non-secret configuration hash; check IDs/results; redaction counts; and stable pointers to sanitized transcripts.
- Support three distinct comparisons: (1) capability/schema diff for advertised inventory; (2) protocol conformance outcome against a frozen spec requirement set; and (3) optional agent behavior evals against task cases. A tool list diff is not an agent quality score.
- Add `doctor`/preflight output for endpoint reachability, transport mode, TLS/auth setup, server identity, and protocol negotiation. Keep secrets out of report files and default logs; redact authorization headers, OAuth tokens, and configurable JSON paths in tool results.
- Provide stable JSON and JUnit output, a human Markdown/HTML summary, and artifact retention guidance. Any shareable report should be self-contained enough to reproduce a failure, with secrets removed and local file paths normalized.
- Introduce a client/agent compatibility matrix only after recording reproducible runs against named versions/configurations. Describe observations precisely (for example, protocol handshake, tool discovery, schema handling, and a specific sample task); do not make broad “works with Claude/Cursor/etc.” claims from SDK conformance alone.

### Gaps

- Agent/model behavior is stochastic and can vary with prompts, model versions, client policy, and server data. Such evaluation needs a separate harness, fixed/recorded conditions, repeated runs, and confidence intervals where scores are used.
- Public cross-agent claims become stale quickly. Require a dated interoperability report and re-run workflow before marking support as current.
- Transcript portability creates privacy risk. Redaction needs a test suite of its own, and recording should be opt-in or clearly controlled when tool outputs may contain sensitive data.

## Key Question 5: What repository and license caveats matter before reusing implementation code?

### Takeaway

Prefer reproducing architectural patterns over copying source. Confirm the license of the exact file/package and its contribution date before reuse; the MCP project is in a mixed MIT/Apache-2.0 transition, while the JSON Schema validator is MIT. Pin dependencies and review their security/upgrade policies.

### Cited Findings

- The official MCP project's [conformance LICENSE](https://github.com/modelcontextprotocol/conformance/blob/main/LICENSE) states that new code/spec contributions are Apache-2.0, documentation except specifications is CC-BY-4.0, and some contributions without relicense consent remain MIT. The same central transition notice appears in the [Rust SDK license](https://github.com/modelcontextprotocol/rust-sdk/blob/main/LICENSE) and [Inspector license](https://github.com/modelcontextprotocol/inspector/blob/main/LICENSE); do not assume every historical file has one uniform license.
- The `jsonschema` validator's [Cargo manifest](https://github.com/Stranger6667/jsonschema/blob/master/Cargo.toml) declares MIT. The official servers' package metadata says `SEE LICENSE IN LICENSE`, and the repo uses multiple packages/languages; inspect the specific package/file license before copying any implementation.
- The official MCP server release notes show that production repositories continue to receive security fixes and sometimes versioning changes. The Rust SDK publishes versioning and dependency policies in its [roadmap](https://github.com/modelcontextprotocol/rust-sdk/blob/main/ROADMAP.md) and linked governance documents.

### Inferences

- For mcp-regress, dependency reuse is preferable to copying implementation: pin crate versions in `Cargo.lock`, disable unneeded schema resolver features, and record dependency/security refresh cadence.
- If copying code or fixtures, maintain a small third-party notices file with upstream repository, exact source path/commit, applicable license, and any modifications. For MCP project content, inspect the exact file and contribution provenance rather than applying one blanket license label.
- Treat upstream SDK and fixture snapshots as inputs that change. Make upgrades explicit review items with changelog/advisory checks, especially for transports, auth, schema validation, and report serialization.

### Gaps

- The MCP project license transition is file/contribution-sensitive; the root license notice alone is not legal advice about a specific copied file. Confirm provenance and obligations before redistributing copied material.
- Current `main` branch pages are mutable. For long-lived research, replace moving links with commit permalinks and archive the exact upstream source revision used when implementation begins.
