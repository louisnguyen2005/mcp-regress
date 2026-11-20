# 0006: Require opt-in for side-effecting or unclassified fixture tools

Status: accepted
Date: 2026-11-20

## Context

Fixture calls can mutate databases, files, or external services. The runner executes the configured server with the user's OS privileges, and fixture arguments are intentionally flexible. The MCP server may provide a `readOnlyHint`, but that metadata is controlled by the server and cannot prove what a tool does.

## Options

1. Continue to invoke every configured case and rely on users to remember the documentation warning.
2. Deny a case call unless its tool advertises `annotations.readOnlyHint = true` or that exact case opts in with `allow_side_effects = true`.
3. Attempt to infer side effects from tool names or JSON Schemas.

## Decision

Choose option 2. The safe default is to call only tools marked read-only. A missing, false, or malformed hint requires `allow_side_effects = true` on that case. The option is part of fixture provenance so reviewers can see when permission changed. The runner does not trust the annotation as a security guarantee and does not claim to sandbox the server.

Keep config v1 and snapshot v2. Omit the default `false` value during serialization so existing fixture digests remain stable when behavior is otherwise unchanged. Existing configurations remain parseable; cases that call unannotated or non-read-only tools must opt in and review a fresh baseline.

## Consequences

- Unannotated write tools are rejected before `tools/call`; explicitly approved cases still run.
- Correctly annotated read-only server tools require no extra fixture setting.
- Servers may lie or behave unexpectedly; the policy reduces accidental calls but is not process isolation.
- Setup and teardown hooks remain necessary for repeatable side-effecting cases.

## Evidence

E2E coverage in `tests/e2e.py` verifies blocking with false and missing hints, successful explicit opt-in, and that blocked calls do not mutate the test marker file. The public filesystem-server interoperability probe in `scripts/interop_official_filesystem.py` successfully exercises two read-only tools.

## Revisit trigger

When fixture lifecycle hooks are designed, when common server annotations prove insufficient, or when maintaining config v2 migration policy.
