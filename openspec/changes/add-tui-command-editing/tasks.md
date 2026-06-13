## 1. Protocol Completion Messages

- [x] 1.1 Add completion request, completion response, completion candidate, and candidate-kind DTOs to `space-game-protocol`.
- [x] 1.2 Include sequence number, input text, cursor byte offset, replacement byte span, insertion text, display text, and candidate kind in the protocol types.
- [x] 1.3 Add serialization round-trip tests for completion requests, completion responses, empty candidate responses, and sequence correlation.

## 2. Server Completion Logic

- [x] 2.1 Add server command metadata for server-supported command names and supported option names.
- [x] 2.2 Implement completion request handling in the WebSocket command/message path.
- [x] 2.3 Implement command-name completion for first-token input.
- [x] 2.4 Implement object-name completion for `distance <object>` argument positions using authoritative object data.
- [x] 2.5 Implement option completion for supported option positions such as `distances --s`.
- [x] 2.6 Return empty completion responses for unsupported completion contexts without treating them as command errors.
- [x] 2.7 Add server unit tests for command-name, object-name, multi-word display-name, option-name, unsupported-context, and sequence-preservation completion behavior.

## 3. TUI Input Controller

- [x] 3.1 Evaluate and add a compatible `tui-input` dependency, upgrading Ratatui/Crossterm only if needed and verified.
- [x] 3.2 Introduce a command input controller that owns editable text, cursor position, history browsing state, reverse-search state, pending completion state, and completion candidate state.
- [x] 3.3 Replace direct `String`/cursor mutation in `ClientApp` with the command input controller while preserving command submission behavior.
- [x] 3.4 Update terminal key handling so Up/Down, Ctrl-R, Tab, Esc, Enter, text input, Backspace, and cursor movement route through the controller.
- [x] 3.5 Add local-only completion candidates for TUI commands such as `quit` and `exit`.
- [x] 3.6 Add view-model tests for editable recalled commands, draft restoration, Ctrl-R search accept/cancel, local-only command completion, and normal command submission.

## 4. Persistent History

- [x] 4.1 Add a command history store with injectable path or storage abstraction for tests.
- [x] 4.2 Load saved history when starting the TUI client.
- [x] 4.3 Save non-empty submitted commands except `quit` and `exit`.
- [x] 4.4 Deduplicate adjacent repeated commands and enforce the configured maximum history length.
- [x] 4.5 Ensure non-history tests use in-memory, disabled, temporary, or injected history storage rather than the real user history file.
- [x] 4.6 Add history persistence tests that use temporary files or injected storage paths.

## 5. TUI Autocomplete UX

- [x] 5.1 Send protocol completion requests on Tab with the current input text and cursor byte offset.
- [x] 5.2 Apply single-candidate responses using the server-provided replacement span and cursor placement.
- [x] 5.3 For multi-candidate responses, apply the longest common insertion prefix when it extends the current replacement text.
- [x] 5.4 Display remaining completion candidates in a compact command-area list or popup without submitting a command.
- [x] 5.5 Show an autocomplete spinner or pending indicator only after a request has been pending for more than 0.2 seconds.
- [x] 5.6 Cancel pending completion locally on Esc and ignore stale or canceled completion responses.
- [x] 5.7 Add view-model/rendering tests for completion request creation, single-candidate application, longest-common-prefix application, multi-candidate display, spinner threshold behavior, cancellation, and stale response handling.

## 6. Integration and Verification

- [x] 6.1 Update UI rendering to show normal input, reverse-search mode, completion candidates, and delayed completion pending state.
- [x] 6.2 Update README or user-facing command documentation for history, reverse search, and completion keys.
- [x] 6.3 Run relevant `cargo test` coverage for protocol, server, and TUI crates.
- [x] 6.4 Run `openspec validate --all`.
