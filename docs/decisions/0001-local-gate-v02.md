# 0001: Harden the local gate before remote expansion

Status: accepted for this development milestone
Date: 2026-11-19

## Context

The v0.1 prototype could allocate an unbounded line, persist transport failures as normal fixture outcomes, emit raw server output, and regenerate its CI baseline before checking. The roadmap also contains market interviews and live agent compatibility that cannot be substituted by local implementation.

## Options

1. Build HTTP and agent adapters immediately over existing behavior.
2. Stabilize the library and local capture/report contract first.

## Decision

Choose option 2. Extract a generic message transport boundary, keep the implemented 2025-06-18 profile explicit, introduce snapshot/report v2 with v1 read compatibility, and preserve exit codes. Unexpected transport/RPC/tool failures abort capture. Normalization uses replacement markers. Treat schema changes outside a conservative supported proof as inconclusive and blocking. Private JSON retains evidence; console and Markdown omit full tool values.

## Consequences

Some v0.1 baseline files need reviewed recapture, especially legacy error results and ignored fields. Metadata changes become warnings. Output schema changes may require review even when harmless. No new protocol version or agent support is claimed without evidence. Unix process groups improve cleanup but Windows still needs Job Objects. Checksums provide integrity checking, not authentication.

## Evidence

`tests/core.rs`, `tests/e2e.py`, `fixtures/baseline.json`, and `.github/workflows/ci.yml`; local results are recorded in `docs/PROGRESS.md`.

## Revisit trigger

Independent server fixtures, confirmed remote transport demand, or a false compatibility classification. Record a new ADR before widening supported schema or protocol semantics.
