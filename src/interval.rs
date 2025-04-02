use std::time::Duration;

// Supported Aggregation Intervals
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum Interval {
    #[cfg_attr(feature = "serde", serde(rename = "100ms"))]
    OneHundredMillisecond,
    #[cfg_attr(feature = "serde", serde(rename = "1s"))]
    OneSecond,
    #[cfg_attr(feature = "serde", serde(rename = "5s"))]
    FiveSeconds,
    #[cfg_attr(feature = "serde", serde(rename = "10s"))]
    TenSeconds,
    #[cfg_attr(feature = "serde", serde(rename = "1min"))]
    OneMinute,
    #[cfg_attr(feature = "serde", serde(rename = "5min"))]
    FiveMinutes,
    #[cfg_attr(feature = "serde", serde(rename = "15min"))]
    FifteenMinutes,
    #[cfg_attr(feature = "serde", serde(rename = "1h"))]
    OneHour,
    #[cfg_attr(feature = "serde", serde(rename = "2h"))]
    #[default]
    TwoHours,
    #[cfg_attr(feature = "serde", serde(rename = "1d"))]
    OneDay,
    #[cfg_attr(feature = "serde", serde(rename = "1w"))]
    OneWeek,
}

impl Interval {
    pub const fn to_minutes(&self) -> i64 {
        match self {
            Self::OneHundredMillisecond
            | Self::OneSecond
            | Self::FiveSeconds
            | Self::TenSeconds => 0,
            Self::OneMinute => 1,
            Self::FiveMinutes => 5,
            Self::FifteenMinutes => 15,
            Self::OneHour => 60,
            Self::TwoHours => 120,
            Self::OneDay => 1400,
            Self::OneWeek => 10080,
        }
    }

    pub const fn to_seconds(&self) -> i64 {
        if matches!(self, Self::OneHundredMillisecond) {
            return 0;
        }
        if matches!(self, Self::OneSecond) {
            return 1;
        }
        if matches!(self, Self::FiveSeconds) {
            return 5;
        }
        if matches!(self, Self::TenSeconds) {
            return 10;
        }
        self.to_minutes() * 60
    }

    pub const fn to_millis(&self) -> u64 {
        if matches!(self, Self::OneHundredMillisecond) {
            return 100;
        }

        (self.to_seconds() * 1000) as u64
    }
}

impl From<Interval> for Duration {
    fn from(interval: Interval) -> Self {
        Self::from_millis(interval.to_millis())
    }
}
