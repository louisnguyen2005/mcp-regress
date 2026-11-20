# 0005: Contain Windows MCP server process trees

Status: accepted
Date: 2026-11-20

## Context

Tokio's child cleanup terminates the direct server process. The existing Unix process-group path covers ordinary descendants on Linux and macOS, but Windows had no equivalent; a server could leave child processes running after timeout, cancellation, or transport close. Assigning a process to a Windows Job Object after it starts can race with that server spawning descendants.

## Options

- Keep direct-child termination on Windows and document the orphan risk.
- Call `taskkill /T`, which depends on a separate system utility and process-tree enumeration.
- Use a Windows Job Object and start the server suspended until it is assigned.

## Decision

Use the `processkit` Job Object implementation on Windows only. Create the Job Object first, create the server suspended, assign it to the job, then resume it. Closing the process group terminates the server and descendants that remain in the job. Leave the current Unix process-group behavior unchanged. Fail server startup if the job cannot be created or assigned; do not silently fall back to direct-child cleanup.

The Windows implementation is behind a target-specific dependency and does not change config, CLI arguments, protocol behavior, or exit codes. This dependency requires Rust 1.88 or later. Job Object containment is process cleanup, not a filesystem, network, or CPU sandbox.

## Consequences

- Timeout, cancellation, transport close, and normal transport drop can stop the full Windows child tree, including grandchildren.
- Windows hosts that prevent Job Object assignment will fail the server launch with a generic diagnostic.
- Unix descendants that deliberately start a new session can still escape process-group termination; this remains an open risk.
- No resource limits are configured in this decision.

## Evidence

`tests/e2e.py` includes a Windows-only timeout test that starts a child and grandchild, records their PIDs, and verifies both exit. The shared hosted CI matrix is required before advertising this Windows behavior. Microsoft documents automatic child membership and kill-on-close for Job Objects: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects. The crate's spawn path assigns the process before resuming it: https://docs.rs/processkit/3.3.4/processkit/struct.ProcessGroup.html.

## Revisit trigger

Revisit if processkit changes its Windows containment guarantees, the supported Windows baseline is lowered below nested Job Object support, or the project adds explicit CPU/memory limits or a general sandbox.
