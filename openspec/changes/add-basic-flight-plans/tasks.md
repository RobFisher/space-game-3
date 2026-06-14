## 1. Protocol and Data Model

- [x] 1.1 Add shared flight plan DTOs and server response variants to `space-game-protocol`.
- [x] 1.2 Add protocol serialization tests for active flight plan responses and no-active-plan responses.
- [ ] 1.3 Add server-side flight plan structs, statuses, target representation, acceleration validation, and id generation.

## 2. Flight Planning and Ship Resolution

- [ ] 2.1 Implement object-target intercept estimation using fixed deterministic iterations and target-state snapshotting.
- [ ] 2.2 Implement accelerate/decelerate transfer duration and interpolation from origin state to snapshotted target state.
- [ ] 2.3 Extend player ship state resolution so active flight plans determine in-flight position and completed plans hand off to target orbit behavior.
- [ ] 2.4 Implement replacement behavior so a new flight plan starts from the ship's current resolved state and supersedes active flight motion.
- [ ] 2.5 Implement cancellation behavior and no-active-plan status handling.

## 3. Server Commands and Queries

- [ ] 3.1 Add `flight plan <object> [--accel <km_per_s2>]`, `flight status`, and `flight cancel` command parsing.
- [ ] 3.2 Return structured flight plan protocol responses from flight commands.
- [ ] 3.3 Ensure `status`, `ship`, `distance`, `distances`, and `where` resolve ship state through active flight plan motion.
- [ ] 3.4 Add server/query/command tests for flight creation, default acceleration, invalid acceleration, replacement, cancellation, arrival, and distance behavior.

## 4. TUI and Plain Output

- [ ] 4.1 Display flight plan responses in the interactive TUI output log.
- [ ] 4.2 Display flight plan responses in plain text mode.
- [ ] 4.3 Add TUI/plain formatting tests for active, cancelled, and no-active-plan responses.

## 5. Documentation and Validation

- [ ] 5.1 Update README command examples/help text for flight plan commands where appropriate.
- [ ] 5.2 Run relevant Rust tests for protocol, server, and TUI crates.
- [ ] 5.3 Run `openspec validate --all`.
