use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Timelike, Utc, offset::LocalResult};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
    pub fn nanos(&self) -> Option<i64> {
        self.0.timestamp_nanos_opt()
    }
    pub fn micros(&self) -> i64 {
        self.0.timestamp_micros()
    }
    pub fn millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
    pub fn seconds(&self) -> i64 {
        self.0.timestamp()
    }
    pub fn elapsed(&self) -> Duration {
        Utc::now()
            .signed_duration_since(self.0)
            .to_std()
            .unwrap_or_default() //: not sure about that
    }

    pub fn elapsed_since(&self, other: &Self) -> Duration {
        self.0
            .signed_duration_since(other.0)
            .to_std()
            .unwrap_or_default() //: not sure about that
    }

    pub fn next_o_clock(&self) -> anyhow::Result<Self> {
        let datetime = self.0;
        let next_hour = datetime + chrono::Duration::hours(1);
        let rounded = chrono::Utc.with_ymd_and_hms(
            next_hour.year(),
            next_hour.month(),
            next_hour.day(),
            next_hour.hour(),
            0,
            0,
        );

        match rounded {
            LocalResult::Single(rounded) => Ok(Self(rounded)),
            _ => Err(anyhow::anyhow!(
                "Failed to round to the next hour: {:?}",
                rounded
            )),
        }
    }

    pub fn future(&self, delta: TimeDelta) -> Timestamp {
        Timestamp(self.0 + delta)
    }

    pub fn to_day_string(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }
}

impl From<Timestamp> for u64 {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.0.timestamp_millis() as u64
    }
}

impl From<u64> for Timestamp {
    fn from(timestamp: u64) -> Self {
        Self(DateTime::from_timestamp_millis(timestamp as i64).unwrap())
    }
}

impl From<i64> for Timestamp {
    fn from(timestamp: i64) -> Self {
        Self(DateTime::from_timestamp_millis(timestamp).unwrap())
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}
