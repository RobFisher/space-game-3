## ADDED Requirements

### Requirement: Readline-style command history

The TUI client SHALL maintain command history that can be browsed with Up and Down arrows, and recalled history entries SHALL remain editable before submission.

#### Scenario: Browse previous command

- **WHEN** the user has submitted `objects` and then presses Up from an empty command input
- **THEN** the command input contains `objects` with the cursor positioned for editing

#### Scenario: Edit recalled command

- **WHEN** the user recalls `distance mars`, edits it to `distance luna`, and presses Enter
- **THEN** the client sends a command message containing `distance luna`

#### Scenario: Restore draft while browsing history

- **WHEN** the user has typed an unsent draft, presses Up to browse history, and presses Down back past the newest history entry
- **THEN** the command input restores the unsent draft

### Requirement: Persistent command history

The TUI client SHALL persist submitted command history to a local user data file and load that history when a later TUI session starts.

#### Scenario: Save submitted command

- **WHEN** the user submits a non-empty command other than `quit` or `exit`
- **THEN** the command is saved to the local command history store

#### Scenario: Load command history

- **WHEN** the TUI client starts after a previous session saved `status`
- **THEN** pressing Up can recall `status`

#### Scenario: Bound history size

- **WHEN** saved command history exceeds the configured maximum history length
- **THEN** the client retains only the newest commands up to that limit

#### Scenario: Isolate non-history tests

- **WHEN** tests exercise command input behavior other than history persistence
- **THEN** the client uses injected, temporary, in-memory, or disabled history storage instead of the real user history file

### Requirement: Reverse history search

The TUI client SHALL provide a Ctrl-R reverse history search mode that incrementally searches previously submitted commands.

#### Scenario: Open reverse search

- **WHEN** the user presses Ctrl-R while editing command input
- **THEN** the client enters reverse history search mode without submitting a command

#### Scenario: Incremental search updates match

- **WHEN** the user types search text while in reverse history search mode
- **THEN** the client displays the newest history entry that contains the search text

#### Scenario: Accept reverse search match

- **WHEN** reverse history search has a current match and the user presses Enter
- **THEN** the client places that match into normal command input for editing or submission

#### Scenario: Cancel reverse search

- **WHEN** the user presses Esc while in reverse history search mode
- **THEN** the client exits reverse history search and restores the command draft from before search began

### Requirement: Server-backed command completion UX

The TUI client SHALL request autocomplete candidates from the server when the user presses Tab and SHALL apply or display the server-provided completion response without requiring cached object lists.

#### Scenario: Request completion

- **WHEN** the user presses Tab with command input `distance ma`
- **THEN** the client sends a completion request containing the full input text and current cursor position

#### Scenario: Complete local-only command

- **WHEN** the user presses Tab with command input `qu`
- **THEN** the client can complete the local-only command `quit` without requiring the server to execute that command

#### Scenario: Apply single completion

- **WHEN** the client receives one completion candidate with a replacement span for the active completion request
- **THEN** the client replaces that span with the candidate insertion text and moves the cursor to the end of the inserted text

#### Scenario: Apply longest common prefix

- **WHEN** the client receives multiple completion candidates that share a longer insertion prefix than the current replacement text
- **THEN** the client replaces the active span with the longest common prefix before displaying the remaining candidates

#### Scenario: Display multiple completions

- **WHEN** the client receives multiple completion candidates for the active completion request
- **THEN** the TUI displays the candidates without submitting a command

#### Scenario: Show delayed completion spinner

- **WHEN** a completion request has been pending for more than 0.2 seconds
- **THEN** the TUI displays a spinner or pending indicator for autocomplete

#### Scenario: Suppress fast completion spinner

- **WHEN** a completion request completes before it has been pending for 0.2 seconds
- **THEN** the TUI does not display the autocomplete spinner

#### Scenario: Cancel pending completion

- **WHEN** a completion request is pending and the user presses Esc
- **THEN** the client cancels the pending completion locally and ignores any later response for that request
