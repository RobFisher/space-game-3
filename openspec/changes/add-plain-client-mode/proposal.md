## Why

The current Ratatui client is useful for humans but awkward for automated agents
and smoke tests because it requires terminal UI interaction. A plain text client
mode will let tests send existing client commands to a running server and assert
against deterministic line-oriented output.

## What Changes

- Add a non-interactive plain text mode to the existing `space-client-tui`
  binary.
- Keep the Ratatui interface as the default behavior.
- Support sending one command from a command-line argument.
- Support reading newline-delimited commands from standard input.
- Print command responses and errors as plain text lines suitable for scripted
  assertions.
- Reuse the existing WebSocket protocol and server-side command semantics.
- Document plain mode usage for local smoke tests against a running server.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `tui-space-client`: Adds a plain text mode alongside the existing Ratatui mode
  so automated agents and tests can exercise the client without terminal UI
  interaction.

## Impact

- Affected code:
  - `crates/space-client-tui/src/main.rs`
  - `crates/space-client-tui/src/net.rs`
  - `crates/space-client-tui/src/app.rs`
  - `crates/space-client-tui/src/lib.rs`
  - new plain-mode module under `crates/space-client-tui/src/`
  - `README.md`
- Existing protocol and server command handling should remain unchanged.
- The default TUI behavior must be preserved.
- A small command-line argument parser may be added manually or via a lightweight
  dependency if justified during implementation.
