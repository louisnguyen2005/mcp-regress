# 0002: Explicit policies and input-safe artifacts

Status: accepted
Date: 2026-11-19

## Context

Teams need reviewed exceptions without replacing a trusted baseline. Artifact flags previously allowed a report to overwrite that baseline. Snapshot hashing also copied complete captured responses into a new JSON value.

## Options

Global suppressions are simple but can hide unrelated changes. Name-scoped, expiring exceptions preserve the evidence and limit the approved scope. Regenerating a baseline for an exception would discard the distinction between observed and accepted changes.

## Decision

Add version 1 TOML policies to `check` and `diff`, stable rule IDs, and report format 3. Explicit severity overrides change a rule's policy classification. Exceptions match a rule and exact tool/case name, require a reason, and expire at the end of a UTC date. Expired exceptions fail before capture, even with `--fail-on none`. Suppressed changes remain in reports. `validate` checks config/policy without starting a server.

Resolve artifact paths, including existing symlink ancestors, before execution and reject aliases to config, policy, or input snapshots as well as duplicate report destinations. The parent directory must still be trusted; this is not protection against a concurrent attacker replacing path components.

Hash borrowed snapshot fields in canonical key order rather than copying full response values. Preserve snapshot v2 hashes exactly; verify against the existing checked-in baseline. Narrow Tokio features to those used by the runner.

## Consequences

JSON report consumers must handle format 3 and exception metadata. No report finding disappears due to an exception. `--fail-on any` still blocks warnings but honors explicit exceptions. The snapshot format remains v2. Relative path resolution and symlink aliases are checked conservatively; invalid output paths fail before a tool call.

## Evidence

`tests/policy.rs`, `tests/core.rs`, `tests/e2e.py`, and the unchanged `fixtures/baseline.json`. `python3 scripts/check.py` is the shared local/CI validation entry point.

## Revisit trigger

Need for wildcard exceptions, additional rules, signed approvals, or hostile multiuser artifact directories. Those require another design decision and tests rather than silently broadening exception scope.
