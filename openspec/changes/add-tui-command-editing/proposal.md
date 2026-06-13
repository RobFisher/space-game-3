## Why

The TUI command line is currently limited to basic text entry, which makes repeated commands and discoverable command syntax awkward during normal play and testing. Adding readline-style editing, persistent history, and server-backed completion gives the client a more usable command surface while keeping authoritative command knowledge on the server.

## What Changes

- Add command history browsing with Up/Down arrows.
- Let recalled history entries be edited before submission.
- Persist command history to a local user data file so commands survive client restarts.
- Add Ctrl-R incremental reverse history search with an explicit search mode.
- Add Tab completion for command names and context-sensitive arguments.
- Add protocol messages for server-backed completion requests and responses.
- Have the server compute completion candidates for command names, object names, and supported command argument positions.
- Complete local-only TUI commands such as `quit` and `exit`.
- When multiple completion candidates share a longer prefix, insert the longest common prefix before displaying candidates.
- Show a pending completion spinner in the TUI when a completion request takes more than 0.2 seconds.
- Let Esc cancel a pending completion request locally and ignore its eventual response.
- Keep completion and history behavior testable through view-model, protocol, and server unit tests without requiring a live terminal or writing real command history outside history-specific tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tui-space-client`: Add readline-style command editing, history browsing/search, persistent command history, and asynchronous autocomplete UX.
- `network-protocol`: Add completion request/response protocol messages with sequence correlation and replacement spans.
- `authoritative-space-server`: Add server-side completion behavior for command names and object-aware argument positions.

## Impact

- Affects `crates/space-client-tui` input state, terminal event handling, rendering, local file persistence, and tests.
- Affects `crates/space-game-protocol` wire-visible message enums and serialization tests.
- Affects `crates/space-server` command metadata/completion logic and tests.
- May add `tui-input` for robust command-line editing and a small local-data path dependency for history file placement.
