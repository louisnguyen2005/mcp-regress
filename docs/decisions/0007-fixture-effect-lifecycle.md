# 0007: Classify fixture effects and run cleanup hooks

Status: accepted
Date: 2026-11-20

## Context

The v0.7 per-case opt-in prevented accidental calls to tools without a read-only hint, but it did not tell reviewers what effect a fixture intended or provide deterministic setup and cleanup. A mutating test can leave persistent state behind. The server and its annotations remain untrusted, and the runner is not a sandbox.

## Options

1. Keep one boolean opt-in and rely on operator-managed cleanup outside the runner.
2. Add explicit effect classifications and MCP setup/cleanup actions to the fixture config.
3. Run arbitrary shell cleanup commands after each case.

## Decision

Choose option 2. Config v1 adds `effect = "auto" | "read_only" | "mutating" | "destructive"`, optional `setup` and `cleanup` MCP tool actions, and retains the per-case `allow_side_effects` approval.

- `auto` is the default and permits a case only when the server reports `annotations.readOnlyHint = true`.
- `read_only` is an explicit, trusted maintainer classification that can override a missing/false server hint; it cannot include lifecycle actions or a side-effect opt-in.
- `mutating` and `destructive` require `allow_side_effects = true` and a cleanup action. Setup is optional.
- Setup runs before the case. Cleanup is attempted after setup or case failure. A cleanup failure aborts capture and no baseline is stored. Lifecycle responses are validated but are not included in snapshots.
- Lifecycle actions are MCP calls only. Arbitrary local commands are out of scope. Each action uses the case deadline and remains subject to the server and total-run budgets.

Default values are omitted from fixture serialization so existing read-only fixture digests remain stable. Explicit effect classifications and lifecycle actions participate in provenance. The v0.7 form `allow_side_effects = true` without an effect classification and cleanup is rejected; the contract documents this migration.

## Consequences

- Reviewers can see whether a case is intended to be read-only, mutating, or destructive, and which cleanup call should run.
- Ordinary setup and case failures still trigger cleanup. Cleanup failures prevent capture from succeeding.
- Cleanup is not a rollback guarantee. A server can mislabel a tool, cleanup can itself fail, external actions can be irreversible, and process termination or the total-run deadline can prevent cleanup.
- Operators still need disposable test data and credentials. The configured server still runs with the runner's privileges.

## Evidence

`tests/core.rs` validates required classification/approval/cleanup combinations. Real-process E2E tests exercise successful lifecycle completion, cleanup following setup and case failures, invalid configurations, and cleanup failure preventing baseline creation.

## Revisit trigger

Revisit if a fixture needs dynamic values from setup output, durable cleanup retries, or cleanup guarantees across process termination. Those features require a separate data-flow and recovery design.
