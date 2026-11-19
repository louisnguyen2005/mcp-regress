# Protocol and agent compatibility ledger

Updated 2026-11-19. "Planned" means no interoperability claim. For each supported combination record product version/commit, agent version, OS, transport, protocol revision, installation steps, tool discovery result, example check result, limitations, and evidence link.

| Surface | Current state | Next validation |
| --- | --- | --- |
| MCP stdio client, 2025-06-18 | Implemented with bounded transport and validation; local mock evidence only, not conformance-certified | Independent servers, error/lifecycle/pagination corpus |
| MCP 2025-11-25 | Planned | Negotiation and schema behavior |
| MCP 2026-07-28 | Planned; has a different stateless request model | Version-specific lifecycle and metadata |
| Streamable HTTP client | Planned | TLS/auth/routing/redirect/security matrix |
| Claude Code | Planned skill/adapter guide | Install, discover, invoke, inspect report |
| Cursor | Planned skill/adapter guide | Same workflow; verify tool support and config syntax |
| Codex | Planned skill/adapter guide | Same workflow with current Codex docs |
| VS Code GitHub Copilot | Planned skill/adapter guide | Agent mode and `.vscode/mcp.json` workflow |
| Other agents | Unscoped | Add only after user demand and independent runs |

Compatibility is feature-specific. A client's MCP tool support does not imply support for prompts, resources, OAuth flows, or every protocol revision. Agent setup files are generated only from verified examples; never assume one configuration works across all agents.

The current MCP client feature matrix and VS Code configuration reference are useful starting points, but each integration needs a live run: https://modelcontextprotocol.io/clients and https://code.visualstudio.com/docs/agents/reference/mcp-configuration. MCP 2026-07-28 changes: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/changelog.mdx

## Evidence entry template

```text
Date:
mcp-regress commit/version:
Agent and version:
OS:
Transport and MCP revision:
Setup command/config:
Server fixture and commit:
Tool discovery result:
Check/report result:
Known limitations:
Evidence URL or local artifact:
Reviewer:
```

## Local evidence, 2026-11-19

v0.2 Linux development run: 2025-06-18 mock server initialize/list/call, pagination/error validation, repeatable snapshots, CLI exit codes, redaction, frame/run limits, Unix process-group cleanup, and baseline integrity are covered by `tests/core.rs` and `tests/e2e.py`. CI now schedules Linux/macOS/Windows jobs. Those remote jobs have not been observed in this session. No agent application interoperability run has been performed.
