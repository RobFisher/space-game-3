## Context

The TUI client currently stores command input as a `String` with a byte cursor and handles Crossterm key events directly in the network event loop. That is enough for basic character entry, backspace, cursor movement, and Enter submission, but it does not provide durable command history, reverse search, or context-aware completion.

The server already owns command semantics, object resolution, and the object registry. Completion for object names therefore needs to ask the server instead of relying on a client-side cache. Command editing and history are local TUI concerns, while completion candidates are a protocol concern.

## Goals / Non-Goals

**Goals:**

- Provide readline-style command editing behavior in the TUI: history navigation, editable recalled commands, Ctrl-R search, and Tab completion.
- Persist TUI command history across client sessions in a local user data file.
- Add server-backed completion messages so the server can complete command names and object names from authoritative state.
- Keep input handling, history, completion state transitions, and server completion logic testable without a live terminal.
- Preserve plain text mode behavior unless explicitly extended later.

**Non-Goals:**

- Do not add client-side caching for server completion candidates.
- Do not add a full shell parser or quoting language beyond what supported commands need for completion replacement spans.
- Do not make completion metadata a stable public plugin API.
- Do not add server-side cancellation for completion requests in the first implementation.
- Do not require plain text mode to expose interactive history or completion.

## Decisions

### Use a dedicated command input controller

Introduce a TUI command input controller that owns editable input, cursor position, history browsing, reverse search mode, pending completion state, and completion result application. The Crossterm event handler should translate key events into controller actions and optionally produce protocol messages.

This keeps terminal-specific code thin and makes tests independent of a live terminal. `tui-input` is a good candidate for the editable buffer because it already supports Ratatui-style text input and cursor handling. If the latest `tui-input` version requires newer Ratatui/Crossterm versions, the implementation should either select a compatible `tui-input` version or upgrade Ratatui/Crossterm in the same change after verifying existing rendering/tests.

Alternative considered: extend the existing `String` plus byte cursor logic. That is smaller initially, but it would keep growing bespoke editing behavior and increase the risk of Unicode cursor bugs.

### Server-backed completion protocol

Add client-to-server completion requests containing sequence number, full input text, and cursor position. Add server-to-client completion responses containing the same sequence number, the replacement span, and a list of candidates with insertion text, display text, and candidate kind.

The server should compute completions from current command metadata and authoritative object data. The client should not maintain an object-name cache for completion in this change. The replacement span should be server-provided so the client does not need to duplicate token-boundary or future quoting rules.

Local-only TUI commands such as `quit` and `exit` should be completed by the client because the server does not execute them. For first-token completion, the input controller can combine local-only candidates with server-provided candidates, or satisfy the request locally when the prefix only matches local-only commands. This is part of the committed completion behavior for this change, even though those commands are not server-executed.

Alternative considered: request object lists and complete locally. That would be responsive after the first object list, but it creates cache invalidation questions and moves object-aware command knowledge into the client.

### Local cancellation for completion requests

Esc cancels a pending completion request in the client by clearing pending state and recording that the sequence should be ignored if a response arrives later. The first implementation does not send a cancel message to the server.

This matches the expected cost of command/object completion and avoids adding protocol complexity before it is needed.

Alternative considered: add explicit `CancelRequest { seq }`. That is useful for expensive future completions, but it is unnecessary for current command metadata and in-memory object lists.

### Completion pending UX

When Tab starts a completion request, the command area should remain usable enough to show that completion is pending. If the request has been pending for more than 0.2 seconds, the TUI should show a spinner or compact pending indicator near the command input. Responses before that threshold should not flash a spinner.

The pending threshold should be represented in app state so tests can assert behavior by advancing/injecting time rather than sleeping.

### Completion result application

If the response has exactly one candidate, Tab should apply it directly by replacing the provided span and placing the cursor at the end of inserted text. If the response has multiple candidates and those candidates share a longer common insertion prefix than the current replacement text, the TUI should first insert that longest common prefix. The TUI should then present the remaining candidates in a compact list near the command area, using any suitable `tui-input`/Ratatui affordance if one exists or a small custom popup/list otherwise. Repeated Tab or arrow selection can then choose a candidate. This longest-common-prefix behavior is the selected first implementation. The first implementation should keep this interaction simple and deterministic.

Stale responses whose sequence no longer matches the active pending completion state must be ignored.

### Persistent command history

Store command history under a per-user application data path such as `space-game/client-tui/history`. The history store should load on TUI startup and save after accepted command submissions. Tests should inject a path or storage abstraction so they never touch a developer's real history file. Tests that are not specifically exercising history persistence should use in-memory or disabled history storage; persistence tests should write only to temporary/injected paths. No non-history test should write command history.

History should exclude empty commands and local exit commands (`quit`, `exit`), deduplicate adjacent repeats, and keep a bounded number of entries. A limit of 1,000 entries is enough for the first implementation.

Alternative considered: keep history only in memory. That satisfies Up/Down in one session, but it misses the user's requirement to remember commands between sessions.

### Ctrl-R reverse search mode

Ctrl-R enters reverse history search mode. Typed characters update the search query incrementally, Backspace edits the query, Ctrl-R repeats to the previous match, Enter accepts the current match into normal editable input, and Esc cancels the search and restores the pre-search draft.

This should be modelled as explicit state instead of overloading normal input so tests can assert transitions and rendering can show the mode clearly.

## Risks / Trade-offs

- Protocol expansion can drift from command parsing behavior -> Keep command metadata/completion tests next to server command handling and include object-name scenarios.
- `tui-input` version compatibility may require dependency upgrades -> Verify the crate versions during implementation and keep dependency changes scoped to `space-client-tui`.
- Server-backed completion adds round-trip latency -> Use the 0.2-second spinner threshold and local cancellation; defer caching until there is evidence it is needed.
- Persisted history can leak sensitive input -> Store only local TUI command text, keep the file path conventional and user-local, and exclude empty/exit commands. Future sensitive commands may need opt-out handling.
- Cursor spans can be ambiguous with multi-byte characters -> Define cursor and replacement spans as UTF-8 byte offsets matching Rust strings, and validate spans before applying them.

## Migration Plan

This is an additive protocol and client behavior change. Existing command messages and plain text mode continue to work. Rollback is a code revert that removes completion messages and returns the TUI to non-persistent basic input behavior. Existing history files can be left in place because the application can ignore them after rollback.
