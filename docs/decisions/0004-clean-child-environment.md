# 0004: Opt-in clean environment for child servers

Status: accepted
Date: 2026-11-19

## Context

The runner launches the configured server as a child process. Inherited CI and developer environment variables can include tokens unrelated to a fixture. Clearing the environment by default could break existing servers and PATH-based launch commands.

## Options

Keep inheritance only, clear by default, or offer an explicit clean mode.

## Decision

Add `server.env_clear`, false by default. When true, clear the child environment, apply only `server.env`, and require an absolute executable path. Keep inherited behavior as the compatibility default. Report errors without echoing environment values.

## Consequences

This is defense in depth against accidental credential forwarding. It is not a sandbox; the child retains filesystem, network and OS privileges. Users may need to explicitly pass legitimate runtime variables. Runtime secrets should be provided through an external config-generation/secret mechanism, not committed fixtures.

## Evidence

`tests/e2e.py` checks parent secret removal, explicit allowlisted values and relative-path rejection; the default reviewed baseline continues to pass.

## Revisit trigger

A future sandbox or secret-provider interface changes process security boundaries and requires separate design and platform testing.
