# Public MCP server interoperability run

## Environment

- Date: 2026-11-20 (Asia/Ho_Chi_Minh)
- Runner: mcp-regress 0.7.0, Linux x86_64
- Runtime: Node.js 26.13.0
- Server: `@modelcontextprotocol/server-filesystem@2.4.0`, from the upstream Model Context Protocol servers project
- Upstream references: [filesystem server README](https://github.com/modelcontextprotocol/servers/blob/main/src/filesystem/README.md); [pinned npm package](https://www.npmjs.com/package/@modelcontextprotocol/server-filesystem/v/2.4.0)
- Transport/profile: stdio, MCP initialize handshake `2025-06-18`
- Package lock SHA-256: `8edd9b22b2b93de9e2227cb0692b8e59a70a6fb0d0d996c2762bd457c0b577d6`
- Server package integrity from the lockfile: `sha512-BoT8Gws5MsetfAey4vDyuzy8WYQES6mGYLSShuqSNGAC0X0NBV4p78/R78KfW4RkxtWz+zxhS9jwhzJrQUTt2Q==`
- npm registry advisory check during locked install: zero known vulnerabilities reported for the installed dependency tree; this is not a security audit.

## Procedure and results

Installed the locked dependency tree with lifecycle scripts disabled. The probe started the server with `--read-only`, `env_clear = true`, and a generated temporary directory containing one synthetic text file. It discovered 10 tools, called `list_directory`, and called `read_file` with exact assertions for the content type and synthetic file contents.

Two baseline captures produced the same content digest:

```text
caced6c520a1a3ba1b395db23c954551be64c07ce919a529441929cb4130b755
```

A separate `check` capture compared equal to the reviewed temporary baseline and exited 0 with zero changes. The snapshot, reports, directory contents, and full tool results were temporary and are not committed. The reproducible script prints only sanitized metadata and status.

## Scope limits

This is one server package, one OS, and two read-only tools. It proves only that this runner version interoperated with the exercised `2025-06-18` stdio flow: initialize, tool discovery, tool call, exact result assertion, repeat capture, and compare. It does not test pagination, RPC errors, cancellation, resources/prompts, Streamable HTTP, newer protocol revisions, named AI agents, arbitrary servers, or security. The public sample does not meet the three consented fixture-project requirement in DISC-02.

## Reproduction

See [`interop/official-filesystem/README.md`](../../interop/official-filesystem/README.md) and run `python3 scripts/interop_official_filesystem.py` after installing its pinned lockfile and building the CLI.
