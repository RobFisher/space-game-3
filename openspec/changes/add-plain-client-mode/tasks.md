## 1. Workflow Setup

- [x] 1.1 Review the approved OpenSpec artifacts and current client structure before editing implementation files.
- [ ] 1.2 Make a git commit for the pre-change state before applying implementation changes.

## 2. Client Mode Selection

- [ ] 2.1 Add minimal command-line parsing for `--plain`, optional `--command <text>`, and optional server URL override.
- [ ] 2.2 Preserve the current default startup path so running without `--plain` still enters the Ratatui client.
- [ ] 2.3 Add unit coverage for argument parsing behavior.

## 3. Plain Mode Runner

- [ ] 3.1 Add a plain-mode module that connects to the configured WebSocket endpoint without entering terminal raw mode or the alternate screen.
- [ ] 3.2 Send the existing hello message and update local client state from startup server messages without requiring tests to assert random session values.
- [ ] 3.3 Implement single-command execution from `--command <text>`.
- [ ] 3.4 Implement newline-delimited command execution from standard input, ignoring empty lines.
- [ ] 3.5 Handle `quit` and `exit` locally without sending them to the server.

## 4. Plain Output

- [ ] 4.1 Print command-correlated object, distance, distances, status, output-line, and error responses as deterministic plain text lines.
- [ ] 4.2 Keep Ratatui output formatting and behavior unchanged.
- [ ] 4.3 Add tests for representative plain output formatting, including protocol errors.

## 5. Smoke Test Support

- [ ] 5.1 Add an automated test or documented manual smoke command that runs plain mode against a running or spawned server.
- [ ] 5.2 Update `README.md` with examples for `--plain --command` and stdin-driven plain mode.
- [ ] 5.3 Run relevant checks, including `cargo test` and `openspec validate --all`.
- [ ] 5.4 Commit the implementation work in focused chunks, including a commit after tests/specs/docs are updated.
