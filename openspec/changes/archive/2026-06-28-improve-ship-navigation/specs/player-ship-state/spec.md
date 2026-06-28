## MODIFIED Requirements

### Requirement: Flight plan-derived ship state
The system SHALL resolve the player ship state from active flight plan navigation phases when a flight plan is active, while preserving orbiting motion for ships without an active flight plan.

#### Scenario: Active flight plan determines ship state
- **WHEN** the player ship has an active flight plan and server code requests ship state at a time during the transfer
- **THEN** the returned state contains position, velocity, frame, epoch, quality, ship id, ship name, and flight plan motion mode derived from that flight plan

#### Scenario: Orbit entry determines ship state
- **WHEN** the player ship has an active flight plan and server code requests ship state during the orbit-entry phase
- **THEN** the returned state contains position, velocity, frame, epoch, quality, ship id, ship name, and entering-orbit motion mode derived from that flight plan

#### Scenario: Orbiting motion remains default
- **WHEN** the player ship has no active flight plan
- **THEN** ship state resolution continues to use the existing orbiting motion behavior

#### Scenario: Completed flight hands off to orbiting motion
- **WHEN** the active flight plan has completed orbit entry
- **THEN** subsequent current-time ship state can be represented as orbiting motion around the flight plan target object using the configured arrival orbit
