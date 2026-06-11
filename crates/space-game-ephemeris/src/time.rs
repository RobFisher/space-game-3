use chrono::{DateTime, SecondsFormat, Utc};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::time::Duration;

use crate::EphemerisError;

/// Game-facing UTC timestamp used for ephemeris queries.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameTime(DateTime<Utc>);

impl GameTime {
    pub fn from_utc_iso8601(s: &str) -> Result<Self, EphemerisError> {
        let parsed = DateTime::parse_from_rfc3339(s).map_err(|err| {
            EphemerisError::InvalidObjectDefinition(format!("invalid UTC timestamp '{s}': {err}"))
        })?;
        Ok(Self(parsed.with_timezone(&Utc)))
    }

    pub fn now_utc() -> Self {
        Self(Utc::now())
    }

    pub fn seconds_since(&self, other: &Self) -> f64 {
        let delta = self.0.signed_duration_since(other.0);
        if delta >= chrono::Duration::zero() {
            duration_seconds(delta.to_std().expect("non-negative duration"))
        } else {
            -duration_seconds((-delta).to_std().expect("positive duration"))
        }
    }

    pub fn add_seconds(&self, seconds: f64) -> Self {
        let nanos = (seconds * 1_000_000_000.0).round() as i64;
        Self(self.0 + chrono::Duration::nanoseconds(nanos))
    }

    pub fn as_utc(&self) -> DateTime<Utc> {
        self.0
    }
}

fn duration_seconds(duration: Duration) -> f64 {
    duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0
}

impl fmt::Display for GameTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339_opts(SecondsFormat::Secs, true))
    }
}

impl Serialize for GameTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GameTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_utc_iso8601(&value).map_err(D::Error::custom)
    }
}
