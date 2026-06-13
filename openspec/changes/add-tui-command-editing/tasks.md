## 1. Protocol Completion Messages

- [ ] 1.1 Add completion request, completion response, completion candidate, and candidate-kind DTOs to `space-game-protocol`.
- [ ] 1.2 Include sequence number, input text, cursor byte offset, replacement byte span, insertion text, display text, and candidate kind in the protocol types.
- [ ] 1.3 Add serialization round-trip tests for completion requests, completion responses, empty candidate responses, and sequence correlation.

## 2. Server Completion Logic

- [ ] 2.1 Add server command metadata for server-supported command names and supported option names.
- [ ] 2.2 Implement completion request handling in the WebSocket command/message path.
- [ ] 2.3 Implement command-name completion for first-token input.
- [ ] 2.4 Implement object-name completion for `distance <object>` argument positions using authoritative object data.
- [ ] 2.5 Implement option completion for supported option positions such as `distances --s`.
- [ ] 2.6 Return empty completion responses for unsupported completion contexts without treating them as command errors.
- [ ] 2.7 Add server unit tests for command-name, object-name, multi-word display-name, option-name, unsupported-context, and sequence-preservation completion behavior.

## 3. TUI Input Controller

- [ ] 3.1 Evaluate and add a compatible `tui-input` dependency, upgrading Ratatui/Crossterm only if needed and verified.
- [ ] 3.2 Introduce a command input controller that owns editable text, cursor position, history browsing state, reverse-search state, pending completion state, and completion candidate state.
- [ ] 3.3 Replace direct `String`/cursor mutation in `ClientApp` with the command input controller while preserving command submission behavior.
- [ ] 3.4 Update terminal key handling so Up/Down, Ctrl-R, Tab, Esc, Enter, text input, Backspace, and cursor movement route through the controller.
- [ ] 3.5 Add local-only completion candidates for TUI commands such as `quit` and `exit`.
- [ ] 3.6 Add view-model tests for editable recalled commands, draft restoration, Ctrl-R search accept/cancel, local-only command completion, and normal command submission.

## 4. Persistent History

- [ ] 4.1 Add a command history store with injectable path or storage abstraction for tests.
- [ ] 4.2 Load saved history when starting the TUI client.
- [ ] 4.3 Save non-empty submitted commands except `quit` and `exit`.
- [ ] 4.4 Deduplicate adjacent repeated commands and enforce the configured maximum history length.
- [ ] 4.5 Ensure non-history tests use in-memory, disabled, temporary, or injected history storage rather than the real user history file.
- [ ] 4.6 Add history persistence tests that use temporary files or injected storage paths.

## 5. TUI Autocomplete UX

- [ ] 5.1 Send protocol completion requests on Tab with the current input text and cursor byte offset.
- [ ] 5.2 Apply single-candidate responses using the server-provided replacement span and cursor placement.
- [ ] 5.3 For multi-candidate responses, apply the longest common insertion prefix when it extends the current replacement text.
- [ ] 5.4 Display remaining completion candidates in a compact command-area list or popup without submitting a command.
- [ ] 5.5 Show an autocomplete spinner or pending indicator only after a request has been pending for more than 0.2 seconds.
- [ ] 5.6 Cancel pending completion locally on Esc and ignore stale or canceled completion responses.
- [ ] 5.7 Add view-model/rendering tests for completion request creation, single-candidate application, longest-common-prefix application, multi-candidate display, spinner threshold behavior, cancellation, and stale response handling.

## 6. Integration and Verification

- [ ] 6.1 Update UI rendering to show normal input, reverse-search mode, completion candidates, and delayed completion pending state.
- [ ] 6.2 Update README or user-facing command documentation for history, reverse search, and completion keys.
- [ ] 6.3 Run relevant `cargo test` coverage for protocol, server, and TUI crates.
- [ ] 6.4 Run `openspec validate --all`.
