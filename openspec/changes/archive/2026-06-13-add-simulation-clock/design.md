## Context

The ephemeris crate already exposes a `GameTime` value object and the server query layer already accepts `GameTime` for distance calculations. The server command and WebSocket layers currently recreate `DEFAULT_GAME_TIME` for status and distance requests, so time is effectively fixed even though protocol DTOs already expose game time fields.

The TUI redraws on a short render interval and can display changing state without requiring the server to emit a message every frame. That makes a server-authoritative clock plus client-side display projection a small change with clear test boundaries.

## Goals / Non-Goals

**Goals:**
- Make simulation time a server-owned concept initialized from the configured default game time.
- Let clients query the current simulation time.
- Let clients manually advance simulation time by seconds, minutes, hours, or days.
- Keep distance calculations authoritative by taking a server clock snapshot unless an explicit timestamp is supplied.
- Show an advancing simulation clock in the TUI without requiring once-per-second server broadcasts.
- Keep the clock deterministic and testable by injecting or passing a wall-clock instant into clock calculations.

**Non-Goals:**
- No pause/resume UI or time-rate controls beyond the initial running rate.
- No persistent clock storage across server restarts.
- No multiplayer consensus model beyond the current single authoritative server state.
- No calendar units with ambiguous lengths, such as months or years.

## Decisions

### Use an anchor-based running clock

Represent the simulation clock with an anchor simulation timestamp, an anchor wall-clock instant, a running flag, and a fixed rate. The current simulation timestamp is computed from elapsed wall time rather than updated by a per-second loop.

Alternative considered: mutate the clock every second in a Tokio task. That would make the visible behavior obvious but would add lifecycle and test complexity without improving authoritative query semantics.

### Keep ephemeris queries explicit

Keep `SolarSystemQueryService` focused on object lookup and ephemeris math. Command and WebSocket handling should resolve the effective query time from either an explicit request timestamp or the current simulation clock snapshot, then pass a `GameTime` into existing query methods.

Alternative considered: embed the clock inside `SolarSystemQueryService`. That would reduce parameters in callers but would mix mutable server runtime state into the otherwise deterministic query boundary.

### Add typed protocol messages for time

Add protocol messages for requesting simulation time and advancing simulation time. Command text can remain as the TUI's user-facing input surface, but typed messages keep non-command clients from parsing text.

Alternative considered: only add `time` and `advance` command strings. That would be fastest for the current TUI but would make first-class simulation time less reusable for future clients.

### Let the client project display time locally

When the client receives a simulation time or status sample, it stores the simulation timestamp plus the local instant when it was received. During normal render ticks, it displays the projected timestamp using the server-provided running flag and rate. The client should periodically resync by requesting simulation time, and it should update immediately after explicit time responses.

Alternative considered: have the server broadcast status or time every second. That is simpler mentally but creates unnecessary protocol traffic and couples UI smoothness to server broadcast cadence.

### Use explicit timestamp overrides for distance requests

Distance requests without `at_game_time` use the current server simulation time. Requests with `at_game_time` use that timestamp for the calculation and do not mutate the clock.

Alternative considered: setting the simulation clock before every historical query. That would make one-off inspection commands surprising and would affect other connected clients.

## Risks / Trade-offs

- Client-side display projection can drift between syncs if the client is paused or overloaded. Mitigation: periodically request server time and treat server samples as authoritative.
- Concurrent advance and distance requests need consistent snapshots. Mitigation: take the clock write/read lock only long enough to compute or advance the timestamp, then perform ephemeris work outside the lock.
- Floating-point rates can introduce rounding differences. Mitigation: keep the initial rate fixed at `1.0` and convert elapsed durations through the existing `GameTime::add_seconds` behavior.
- Optional protocol fields must remain backward compatible. Mitigation: use `Option<String>` timestamp fields and preserve existing message variants where possible.

## Migration Plan

This is an in-repo protocol expansion with no data migration. Existing command clients continue to send `objects`, `distance`, `distances`, and `status`; those commands will begin using the running server clock by default. Rollback is a code revert to the previous fixed default timestamp behavior.

## Open Questions

- Should the first implementation expose `pause`, `resume`, or `rate` commands, or leave those to a later change?
- What client resync interval is acceptable for the TUI clock display: every second, every five seconds, or only on server responses?
