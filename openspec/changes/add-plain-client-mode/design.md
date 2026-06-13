## Context

`space-client-tui` currently starts directly in Ratatui mode, enters the
alternate screen, connects to the default WebSocket endpoint, and runs one async
loop that merges terminal events, network messages, and render ticks. The client
view model in `app.rs` already owns input editing, sequence numbers, quit state,
and formatting of server messages into readable output lines.

Server command semantics already live on the server. The client sends
`ClientToServer::Command { seq, text }` and receives protocol responses with the
same sequence number. This change should keep that boundary intact and add only
enough non-interactive behavior for automated agents and smoke tests.

## Goals / Non-Goals

**Goals:**

- Add a deterministic plain text mode for `space-client-tui`.
- Keep the existing Ratatui mode as the default.
- Let agents and tests send one command or a stdin script to a running server.
- Print line-oriented output that is easy to assert against.
- Reuse the existing WebSocket protocol and server-side command parser.
- Keep the change small enough to implement and validate quickly.

**Non-Goals:**

- No new command protocol or scripting language.
- No server-side behavior changes.
- No autocomplete, reconnect policy, persistent sessions, or batch transaction
  semantics.
- No replacement of Ratatui rendering or existing human-facing TUI behavior.

## Decisions

### Add `--plain` as an alternate client mode

The `space-client-tui` binary will dispatch to plain mode when started with
`--plain`; otherwise it will keep starting the Ratatui client. `--plain` is more
specific than `--text` because it describes the absence of terminal UI behavior
rather than implying a different rendering style.

Alternative considered: create a separate binary. A second binary would work,
but a mode flag keeps the smoke-test entry point close to the existing client
and avoids another package target for a small testability improvement.

### Support both `--command` and stdin commands

Plain mode will accept a single command from `--command "<text>"` and will also
read newline-delimited commands from standard input. `--command` gives tests a
simple one-shot path; stdin supports multi-command smoke scripts without adding a
custom scripting language.

Empty stdin lines should be ignored. `quit` and `exit` should terminate plain
mode locally without sending a command to the server, matching the TUI behavior.

Alternative considered: support stdin only. That is enough for scripts, but a
single command flag is easier for agents and direct assertions.

### Reuse server command semantics and protocol messages

Plain mode will send the same `ClientToServer::Command { seq, text }` messages
as the TUI. The server remains authoritative for parsing `help`, `objects`,
`distance`, `distances`, and `status`.

Alternative considered: parse plain-mode commands locally into typed protocol
requests. That would duplicate command semantics in the client and make plain
mode diverge from the TUI behavior it is meant to test.

### Keep response completion deliberately simple

For each sent command, plain mode will read protocol messages until it observes
a command result or error for that command sequence. Current commands each
produce a command acknowledgement plus one result message, except `distances`
where the single result message contains multiple distance rows.

Unsolicited welcome/status messages may be used to update the app state, but
plain mode output should prefer command-correlated responses so smoke tests do
not depend on session IDs or startup timing.

Alternative considered: add an explicit protocol end-of-response marker. That
would make batching more general, but it expands protocol and server scope beyond
the immediate testability need.

### Share formatting where practical without changing TUI output

The existing `ClientApp::apply_server_message` formatting can be reused for
plain mode output when it produces deterministic lines. If stricter error lines
are needed for assertions, the implementation should extract small formatting
helpers rather than changing Ratatui presentation behavior.

Alternative considered: make plain mode print raw JSON protocol messages. Raw
JSON would be deterministic, but it would test the transport more than the
user-facing command interface and would be less convenient for smoke scripts.

## Risks / Trade-offs

- Response completion is implicit for now -> Limit the first implementation to
  the current command set and document that plain mode is line-oriented smoke
  test support, not a full batch protocol.
- Startup welcome messages include a random session id -> Do not require tests
  to assert exact welcome output; prefer command-correlated output.
- Reusing TUI output strings can make assertions sensitive to human-facing text
  changes -> Keep output formats simple and add tests around representative
  lines.
- Manual argument parsing can become brittle if more flags are added -> Keep the
  first parser small, or use a lightweight parser if implementation complexity
  starts to grow.
