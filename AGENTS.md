# Working on mcp-regress

Read [docs/ROADMAP.md](docs/ROADMAP.md), [docs/BACKLOG.md](docs/BACKLOG.md), and [docs/SECURITY.md](docs/SECURITY.md) before starting a milestone. The backlog is the source of truth for status and dependencies; the roadmap explains why work is ordered that way.

For each change:

1. Select a backlog item with satisfied dependencies. Keep its ID in the PR or commit description.
2. Confirm the expected user behavior and acceptance criteria. Record a design decision in `docs/decisions/` when changing the protocol model, snapshot format, security boundary, or public CLI.
3. Make the smallest coherent change. Update user documentation and migration notes alongside behavior changes.
4. Verify against the item's acceptance criteria. Do not regenerate a trusted baseline in the same CI job that checks it.
5. Update the backlog status and evidence links. Record new risks in `docs/SECURITY.md` or the relevant decision record.

Never print fixture secrets or full server responses in CI logs by default. Treat configured MCP servers, tool output, snapshots, and report paths as untrusted. Local process execution is an explicit trust decision, not a sandbox. Preserve CLI exit codes unless a documented versioned migration changes them.

Status values: `todo`, `in_progress`, `blocked`, `done`. A task is `done` only after its acceptance criteria and documentation are complete. Do not claim support for an agent or protocol version without a recorded interoperability run.
