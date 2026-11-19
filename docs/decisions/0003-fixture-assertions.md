# 0003: Assert raw results before normalization

Status: accepted
Date: 2026-11-19

## Context

Exact baseline comparison can preserve an incorrect result if it was originally recorded incorrectly. Ignore pointers can also hide a value that a fixture still needs to validate.

## Options

Assertions after normalization would only see ignored/redacted markers. Assertions before normalization validate the original response while still keeping the saved artifact sanitized.

## Decision

Add exact JSON-value assertions keyed by non-root JSON Pointers and evaluate them before ignored fields or redaction. Reject missing paths and mismatches without exposing actual/expected values. Reject result assertions on RPC-error cases. Add optional per-case tool deadlines bounded by existing request and total-run deadlines. A timed-out call aborts capture and shuts down the transport.

## Consequences

The settings participate in fixture provenance, but absent/empty settings are omitted to preserve existing digests. Config v1 and snapshot v2 remain supported. Assertions prove only the specified checks, not general correctness. Regex, numeric tolerances, schema assertions and setup/teardown remain future work.

## Evidence

Five additional real-process tests in `tests/e2e.py` and the unchanged reviewed-baseline gate in `scripts/check.py`.

## Revisit trigger

Need for tolerance-based comparison, result schema validation, or fixture lifecycle hooks.
