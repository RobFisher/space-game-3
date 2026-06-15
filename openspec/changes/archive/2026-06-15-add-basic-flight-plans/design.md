## Context

The server already owns simulation time, ephemeris queries, and the single player ship. The player ship currently has stable identity and an orbiting motion mode; query paths such as `distance`, `distances`, `where`, `ship`, and `status` resolve the ship state at a `GameTime` through the server query service.

Flight plans should fit that existing shape. The server remains authoritative, the TUI remains a command client, and the shared protocol carries DTOs rather than ephemeris or UI types.

## Goals / Non-Goals

**Goals:**
- Add a small player-facing navigation loop: create a flight plan to a known object, inspect it, cancel it, and see ship position derive from it.
- Let the user provide acceleration at plan creation time, with a default for quick use.
- Estimate moving-object arrival by predicting the destination object's position at the calculated arrival time.
- Keep active flight resolution deterministic after a plan is registered.
- Leave room for later navigation models, ship capability limits, richer target types, and better arrival/orbit behavior.

**Non-Goals:**
- Real orbital transfer math, n-body simulation, continuous guidance, collision handling, or fuel accounting.
- Multiple queued flight plans or persisted flight history.
- Exposing raw coordinates in the default TUI output.
- Treating the player ship as a registered ephemeris object.

## Decisions

### Represent flight plans as ship-owned motion

Add flight plan motion to the player ship state model rather than creating a separate movement service. Ship position resolution already flows through `player_ship_state(at)`, so distance and location queries can naturally use flight-plan-derived positions.

Alternative considered: calculate flight plan position only in command handlers. That would leave existing distance and location paths using stale orbiting motion and create two conflicting ship position sources.

### Store the user's acceleration on each plan

The first version stores `acceleration_km_s2` directly on the flight plan. The command accepts a positive finite acceleration value, and later ship capability work can validate this value against ship stats.

Alternative considered: make acceleration a global server default only. That would be simpler but would remove the first useful gameplay tuning knob.

### Estimate moving-target intercept, then snapshot

For object targets, plan creation resolves the ship origin at departure time, calculates an initial transfer duration, resolves the target object at the predicted arrival time, and repeats a small fixed number of times until the arrival estimate stabilizes or the iteration limit is reached. The final target state and arrival time are stored on the plan.

After registration, the ship interpolates toward the snapshotted target state rather than continuously chasing the target. On completion, the ship transitions into orbit around the target object using that object's state at arrival or completion time.

Alternative considered: continuously track the object during flight. That is more physically plausible for guided navigation, but it makes active flight state depend on future target motion at every query and complicates cancellation, replanning, and deterministic status.

### Use symmetric acceleration/deceleration interpolation

For v1, duration is calculated as `2 * sqrt(distance_km / acceleration_km_s2)`, assuming acceleration for the first half of the transfer and deceleration for the second half. Position interpolation uses a deterministic ease-in/ease-out curve over normalized progress.

Alternative considered: constant velocity interpolation. It is simpler, but it does not reflect the chosen acceleration parameter and gives weaker gameplay feedback.

### Replanning replaces active flight

Creating a new flight plan resolves the ship's current state at the current authoritative simulation time and replaces any active plan. The new plan starts from that resolved state and uses the newly requested acceleration.

Alternative considered: reject overlapping plans. Replacement better matches the desired player behavior of changing acceleration and registering a new plan from the current location.

### Keep protocol responses explicit

Add a `FlightPlanDto` and `ServerToClient::FlightPlan` response rather than encoding plan status as generic output text. This gives the TUI and plain mode structured data for consistent display and future UI expansion.

Alternative considered: return only `OutputLine`. That would be faster but would make protocol tests and future UI behavior brittle.

## Risks / Trade-offs

- Intercept estimate can be imperfect for fast-moving targets or very slow ships -> Keep iteration count/tolerance deterministic and treat arrival as successful navigation handoff into target orbit.
- Replacing an active plan discards previous active-plan history in v1 -> Use observable `superseded` behavior only if history is retained; otherwise specify that the current active plan is replaced.
- Explicit future queries may show completed flight state before the authoritative clock reaches arrival -> Keep resolution pure for the requested time and only mutate stored current motion on current-time command paths.
- Tiny acceleration can produce very long duration -> Reject non-positive, non-finite values and allow practical bounds to be added later.
- Arrival orbit is gameplay shorthand rather than physical transfer insertion -> Name it clearly as a default fictional orbit and keep realistic navigation out of scope.
