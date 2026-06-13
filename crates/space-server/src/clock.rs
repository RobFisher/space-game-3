use std::time::Instant;

use space_game_ephemeris::GameTime;
use space_game_protocol::{SimulationTimeDto, TimeUnit};

#[derive(Debug, Clone)]
pub struct SimulationClock {
    anchor_sim_time: GameTime,
    anchor_wall_time: Instant,
    running: bool,
    rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationClockSnapshot {
    pub current_time: GameTime,
    pub running: bool,
    pub rate: f64,
}

impl SimulationClock {
    pub fn new(initial_time: GameTime, anchor_wall_time: Instant) -> Self {
        Self {
            anchor_sim_time: initial_time,
            anchor_wall_time,
            running: true,
            rate: 1.0,
        }
    }

    pub fn snapshot(&self, now: Instant) -> SimulationClockSnapshot {
        SimulationClockSnapshot {
            current_time: self.current_time(now),
            running: self.running,
            rate: self.rate,
        }
    }

    pub fn advance(
        &mut self,
        amount: i64,
        unit: TimeUnit,
        now: Instant,
    ) -> SimulationClockSnapshot {
        let current_time = self.current_time(now);
        self.anchor_sim_time = current_time.add_seconds(unit_seconds(unit) * amount as f64);
        self.anchor_wall_time = now;
        self.snapshot(now)
    }

    fn current_time(&self, now: Instant) -> GameTime {
        if !self.running {
            return self.anchor_sim_time.clone();
        }

        let elapsed_seconds = if now >= self.anchor_wall_time {
            now.duration_since(self.anchor_wall_time).as_secs_f64()
        } else {
            -self.anchor_wall_time.duration_since(now).as_secs_f64()
        };
        self.anchor_sim_time
            .add_seconds(elapsed_seconds * self.rate)
    }
}

impl SimulationClockSnapshot {
    pub fn to_dto(&self) -> SimulationTimeDto {
        SimulationTimeDto {
            current_time: self.current_time.to_string(),
            running: self.running,
            rate: self.rate,
        }
    }
}

fn unit_seconds(unit: TimeUnit) -> f64 {
    match unit {
        TimeUnit::Seconds => 1.0,
        TimeUnit::Minutes => 60.0,
        TimeUnit::Hours => 3_600.0,
        TimeUnit::Days => 86_400.0,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::DEFAULT_GAME_TIME;

    fn epoch() -> GameTime {
        GameTime::from_utc_iso8601(DEFAULT_GAME_TIME).unwrap()
    }

    #[test]
    fn initializes_at_default_time() {
        let anchor = Instant::now();
        let clock = SimulationClock::new(epoch(), anchor);

        assert_eq!(
            clock.snapshot(anchor).current_time.to_string(),
            DEFAULT_GAME_TIME
        );
    }

    #[test]
    fn computes_running_time_from_controlled_instant() {
        let anchor = Instant::now();
        let clock = SimulationClock::new(epoch(), anchor);

        assert_eq!(
            clock
                .snapshot(anchor + Duration::from_secs(1))
                .current_time
                .to_string(),
            "2097-01-01T00:00:01Z"
        );
    }

    #[test]
    fn advances_by_one_day_from_current_time() {
        let anchor = Instant::now();
        let mut clock = SimulationClock::new(epoch(), anchor);

        let snapshot = clock.advance(1, TimeUnit::Days, anchor + Duration::from_secs(1));

        assert_eq!(snapshot.current_time.to_string(), "2097-01-02T00:00:01Z");
    }

    #[test]
    fn advances_by_larger_units() {
        let anchor = Instant::now();
        let mut clock = SimulationClock::new(epoch(), anchor);

        let snapshot = clock.advance(2, TimeUnit::Hours, anchor);

        assert_eq!(snapshot.current_time.to_string(), "2097-01-01T02:00:00Z");
    }

    #[test]
    fn advances_by_supported_units() {
        let anchor = Instant::now();
        let mut clock = SimulationClock::new(epoch(), anchor);

        assert_eq!(
            clock
                .advance(30, TimeUnit::Seconds, anchor)
                .current_time
                .to_string(),
            "2097-01-01T00:00:30Z"
        );
        assert_eq!(
            clock
                .advance(10, TimeUnit::Minutes, anchor)
                .current_time
                .to_string(),
            "2097-01-01T00:10:30Z"
        );
        assert_eq!(
            clock
                .advance(1, TimeUnit::Days, anchor)
                .current_time
                .to_string(),
            "2097-01-02T00:10:30Z"
        );
    }
}
