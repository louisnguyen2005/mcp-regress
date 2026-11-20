#!/usr/bin/env python3
"""Run the optional, read-only interoperability probe against a pinned MCP server."""

from __future__ import annotations

import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = ROOT / "interop" / "official-filesystem"
SERVER_PACKAGE = "@modelcontextprotocol/server-filesystem"
SERVER_VERSION = "2.4.0"
PROBE_TEXT = "mcp-regress-interoperability-probe\n"


def fail(message: str) -> int:
    print(f"interop probe failed: {message}", file=sys.stderr)
    return 1


def run(command: list[str], *, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
    )


def toml_string(value: str) -> str:
    # JSON quoted strings are valid TOML basic strings for these generated paths.
    return json.dumps(value, ensure_ascii=False)


def main() -> int:
    installed_package = PACKAGE_ROOT / "node_modules" / SERVER_PACKAGE / "package.json"
    if not installed_package.is_file():
        return fail(
            "pinned server is not installed; run `npm ci --ignore-scripts "
            "--prefix interop/official-filesystem` first"
        )

    package = json.loads(installed_package.read_text(encoding="utf-8"))
    if package.get("version") != SERVER_VERSION:
        return fail(f"expected pinned server package version {SERVER_VERSION}")

    node = shutil.which("node")
    if node is None:
        return fail("Node.js 24 or later is required")
    node_version = run([node, "--version"])
    version_match = re.fullmatch(r"v(\d+)\..*\n?", node_version.stdout.strip())
    if node_version.returncode != 0 or version_match is None or int(version_match[1]) < 24:
        return fail("Node.js 24 or later is required")

    binary = ROOT / "target" / "debug" / ("mcp-regress.exe" if os.name == "nt" else "mcp-regress")
    if not binary.is_file():
        return fail("mcp-regress binary is missing; run `cargo build --locked --bin mcp-regress`")
    version_result = run([str(binary), "--version"])
    if version_result.returncode != 0:
        return fail("could not read mcp-regress version")
    runner_version = version_result.stdout.strip().split()[-1]

    server_entry = PACKAGE_ROOT / "node_modules" / SERVER_PACKAGE / "dist" / "index.js"
    if not server_entry.is_file():
        return fail("pinned MCP server entry point is missing")

    with tempfile.TemporaryDirectory(prefix="mcp-regress-interop-") as temporary:
        work = Path(temporary)
        fixture_dir = work / "fixture"
        fixture_dir.mkdir()
        fixture_file = fixture_dir / "probe.txt"
        fixture_file.write_text(PROBE_TEXT, encoding="utf-8")
        config = work / "filesystem.toml"
        config.write_text(
            "\n".join(
                [
                    "version = 1",
                    "",
                    "[server]",
                    f"command = {toml_string(str(Path(node).resolve()))}",
                    "args = ["
                    + ", ".join(
                        toml_string(item)
                        for item in (str(server_entry.resolve()), "--read-only", str(fixture_dir))
                    )
                    + "]",
                    "timeout_ms = 10000",
                    "run_timeout_ms = 30000",
                    "env_clear = true",
                    "",
                    "[[cases]]",
                    'id = "read-only-list-directory"',
                    'tool = "list_directory"',
                    f"arguments = {{ path = {toml_string(str(fixture_dir))} }}",
                    "",
                    "[[cases]]",
                    'id = "read-only-read-file"',
                    'tool = "read_file"',
                    f"arguments = {{ path = {toml_string(str(fixture_file))} }}",
                    'assertions = { "/content/0/type" = "text", '
                    f'"/content/0/text" = {toml_string(PROBE_TEXT)} }}',
                    "",
                ]
            ),
            encoding="utf-8",
        )

        snapshots: list[dict[str, object]] = []
        for label in ("baseline", "repeat"):
            output = work / f"{label}.json"
            result = run([str(binary), "baseline", "--config", str(config), "--output", str(output)])
            if result.returncode != 0:
                return fail(f"{label} capture failed (exit {result.returncode}); CLI output omitted")
            snapshots.append(json.loads(output.read_text(encoding="utf-8")))

        first, repeated = snapshots
        first_provenance = first.get("provenance", {})
        repeated_provenance = repeated.get("provenance", {})
        if first_provenance.get("protocol_version") != "2025-06-18":
            return fail("capture did not record the expected protocol profile")
        if first_provenance.get("content_digest") != repeated_provenance.get("content_digest"):
            return fail("repeated capture produced different content digests")
        expected_cases = {"read-only-list-directory", "read-only-read-file"}
        if {case.get("id") for case in first.get("cases", [])} != expected_cases:
            return fail("capture did not record both interoperability cases")
        if any(case.get("outcome", {}).get("status") != "ok" for case in first.get("cases", [])):
            return fail("one or more read-only interoperability calls failed")

        checked = run(
            [
                str(binary),
                "check",
                "--config",
                str(config),
                "--baseline",
                str(work / "baseline.json"),
                "--fail-on",
                "breaking",
            ]
        )
        if checked.returncode != 0:
            return fail(f"baseline comparison failed (exit {checked.returncode}); CLI output omitted")

        names = sorted(tool["name"] for tool in first.get("tools", []) if isinstance(tool, dict))
        print(f"mcp-regress: {runner_version}")
        print(f"server: {SERVER_PACKAGE}@{SERVER_VERSION}")
        print(f"host: {platform.system()} {platform.machine()}; Node.js {node_version.stdout.strip()}")
        print(f"protocol: {first_provenance['protocol_version']}")
        print(f"tool count: {len(names)}")
        print("cases: 2 passed (read-only directory listing and file read with exact content assertion)")
        print("repeatability: identical content digest across two captures")
        print(f"content digest: {first_provenance['content_digest']}")
        print("baseline check: passed (0 changes)")
        print("server responses and temporary snapshots were not printed or retained")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
