## 1. Protocol Contract

- [x] 1.1 Add a location summary DTO with observer label, frame, game time, nearest object id/name, distance kilometers, distance astronomical units, and optional quality.
- [x] 1.2 Add a server-to-client location summary response variant with request sequence correlation.
- [x] 1.3 Add protocol serialization round-trip tests for location summary responses and verify the DTO has no raw coordinate fields.

## 2. Server Spatial State Model

- [x] 2.1 Refactor the server observer model so it resolves to a `StateVector` at a requested `GameTime`.
- [x] 2.2 Add frame compatibility checks for observer-target distance calculations.
- [x] 2.3 Refactor single-object distance calculation to derive distance from observer and target state vectors.
- [x] 2.4 Refactor multi-object distance calculation and sorting to use the same state-derived distance path.
- [x] 2.5 Add server query tests covering observer state distance math, current demo distance compatibility, and incompatible-frame error behavior.

## 3. Location Summary

- [x] 3.1 Add a query-service method that builds a location summary for the observer at a requested `GameTime`.
- [x] 3.2 Calculate the nearest known object by resolving known object states and comparing Euclidean distance to the observer state.
- [x] 3.3 Ensure the location summary uses the current simulation clock when no explicit timestamp is supplied.
- [x] 3.4 Add query-service tests for nearest-object selection, frame/time fields, quality propagation, and no raw coordinate output.

## 4. Server Commands

- [x] 4.1 Add `where` to server command parsing and help text.
- [x] 4.2 Add `where` to command autocomplete candidates.
- [x] 4.3 Add WebSocket and command-handler tests for `where`, completion for `wh`, and sequence correlation.

## 5. TUI And Plain Text Client

- [ ] 5.1 Update the TUI view model to handle location summary responses.
- [ ] 5.2 Display location summaries in the TUI output log as readable landmark-based text without raw coordinates.
- [ ] 5.3 Update plain text mode to print deterministic location summary output for `where`.
- [ ] 5.4 Add TUI app and plain text formatting tests for location summary display.

## 6. Documentation And Validation

- [ ] 6.1 Update README command examples or help text documentation to mention `where`.
- [ ] 6.2 Run relevant Rust tests for protocol, server, and TUI client changes.
- [ ] 6.3 Run `openspec validate --all`.
