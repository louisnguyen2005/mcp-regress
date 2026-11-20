# Official filesystem server interoperability probe

This optional probe checks the implemented `2025-06-18` stdio profile against a pinned release of the public MCP filesystem server. It uses a generated temporary directory, starts the server with `--read-only`, and never stores server results in the repository.

The server process still runs with the local user's OS privileges. `--read-only` limits the MCP tool surface; it is not an operating-system sandbox. Review the pinned package and lockfile before running it in a sensitive environment.

From the repository root:

```sh
npm ci --ignore-scripts --prefix interop/official-filesystem
cargo build --locked --bin mcp-regress
python3 scripts/interop_official_filesystem.py
```

The run captures a temporary baseline twice, checks that both content digests match, and compares a third capture against the first. Cases list the synthetic directory and read one synthetic text file with exact assertions. The check does not call a write-capable MCP tool.

The run is deliberately separate from the default offline CI gate. It downloads and executes an external package, so operators should run it only when they accept that trust decision. See the sanitized [recorded run](../../docs/evidence/official-filesystem-interop-2026-11-20.md).
