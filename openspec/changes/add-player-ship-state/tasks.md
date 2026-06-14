## 1. Ship State Model

- [x] 1.1 Add a server-side player ship model with stable id, display name, motion mode, frame, and quality metadata.
- [x] 1.2 Implement orbiting ship state resolution at a requested `GameTime` using an Earth-parented fictional circular orbit.
- [x] 1.3 Add runtime ship renaming with validation that rejects empty or whitespace-only names.
- [x] 1.4 Add focused unit tests for default ship creation, orbiting state resolution, time-varying position, and rename validation.

## 2. Protocol

- [ ] 2.1 Add ship state DTOs and server-to-client ship response messages to `space-game-protocol`.
- [ ] 2.2 Replace observer-oriented status DTO fields with ship-oriented status fields.
- [ ] 2.3 Add protocol serialization round-trip tests for ship responses and updated status messages.

## 3. Server Query and Commands

- [ ] 3.1 Replace fixed observer query state with player ship state in the server query service.
- [ ] 3.2 Update `status`, `where`, `distance`, and `distances` behavior to resolve from the player ship at the effective simulation time.
- [ ] 3.3 Add `ship`, `ship status`, and `ship name <name>` command handling.
- [ ] 3.4 Update command completion and help text for the new `ship` command.
- [ ] 3.5 Update server unit and WebSocket tests for ship-centered status, location, distance, naming, and invalid-name errors.

## 4. TUI and Plain Client

- [ ] 4.1 Update the TUI view model to store and apply ship-oriented status fields.
- [ ] 4.2 Render the player ship name in the TUI status pane.
- [ ] 4.3 Display ship state responses in the output log.
- [ ] 4.4 Update plain text mode status and ship response formatting.
- [ ] 4.5 Update TUI and plain mode tests for ship status/name presentation.

## 5. Documentation and Verification

- [ ] 5.1 Update README command examples and descriptions from fixed observer wording to player ship wording.
- [ ] 5.2 Run `cargo test` for the affected workspace crates.
- [ ] 5.3 Run `openspec validate --all`.
