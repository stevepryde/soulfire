use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

use jiff::{SignedDuration, Span, Timestamp, civil, tz::TimeZone};

#[derive(Debug, thiserror::Error)]
pub enum DateTimeError {
    #[error("parse error: {0}")]
    ParseError(String),
}

impl From<jiff::Error> for DateTimeError {
    fn from(e: jiff::Error) -> Self {
        DateTimeError::ParseError(e.to_string())
    }
}

/// A UTC instant.
///
/// Backed by [`jiff::Timestamp`]; the backend type is intentionally **not**
/// exposed (no `to_chrono`/`to_jiff` escape hatches) so the datetime library
/// stays swappable. The string form is RFC 3339 with fixed millisecond
/// precision and a `Z` suffix (e.g. `2026-06-07T12:00:00.000Z`) — a contract
/// value relied on by the persisted record formats (`specs/01-data-model.md`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub struct SpDateTime(Timestamp);

impl SpDateTime {
    pub fn now() -> Self {
        // Truncate to millisecond resolution so the in-memory value matches the
        // serialized contract form (`…000Z`, millis); otherwise a value from
        // `now()` would not survive a serialize → parse round-trip. The store is
        // the single source of truth (`DATA-23`) and persists at millis.
        Self(Self::truncate_to_millis(Timestamp::now()))
    }

    fn truncate_to_millis(ts: Timestamp) -> Timestamp {
        Timestamp::from_millisecond(ts.as_millisecond())
            .expect("millisecond timestamp is always in range")
    }

    /// Convert from UNIX timestamp (seconds since epoch).
    pub fn from_timestamp(timestamp: i64) -> Option<Self> {
        Timestamp::from_second(timestamp).ok().map(Self)
    }

    pub fn timestamp(&self) -> i64 {
        self.0.as_second()
    }

    pub fn format_friendly(&self) -> String {
        self.format("%Y-%m-%d %H:%M:%S")
    }

    /// Format the instant in UTC using `strftime`-style specifiers.
    pub fn format(&self, format: &str) -> String {
        self.0.strftime(format).to_string()
    }

    /// Format the instant in the system local timezone.
    pub fn format_local(&self, format: &str) -> String {
        self.0
            .to_zoned(TimeZone::system())
            .strftime(format)
            .to_string()
    }

    pub fn format_local_friendly(&self) -> String {
        self.format_local("%Y-%m-%d %H:%M:%S")
    }

    pub fn add_days(&self, days: i64) -> Self {
        // Absolute 24h-per-day arithmetic on the instant (matches the prior
        // chrono `Duration::days` semantics).
        Self(self.0 + SignedDuration::from_hours(24 * days))
    }

    pub fn sub_days(&self, days: i64) -> Self {
        Self(self.0 - SignedDuration::from_hours(24 * days))
    }

    /// Return this instant advanced by `seconds` (negative to go back).
    pub fn add_seconds(&self, seconds: i64) -> Self {
        Self(self.0 + SignedDuration::from_secs(seconds))
    }

    /// Return this instant advanced by `millis` (negative to go back).
    pub fn add_millis(&self, millis: i64) -> Self {
        Self(self.0 + SignedDuration::from_millis(millis))
    }

    /// Whole seconds elapsed from `earlier` to `self` (negative if `self` is
    /// before `earlier`). Useful for timeout/stale-lock checks.
    pub fn seconds_since(&self, earlier: SpDateTime) -> i64 {
        self.timestamp() - earlier.timestamp()
    }

    /// Whole milliseconds elapsed from `earlier` to `self`.
    pub fn millis_since(&self, earlier: SpDateTime) -> i64 {
        self.0.as_millisecond() - earlier.0.as_millisecond()
    }

    /// Format as YYYY-MM-DD (UTC) for HTML date inputs.
    pub fn format_date(&self) -> String {
        self.format("%Y-%m-%d")
    }

    /// Parse from YYYY-MM-DD, interpreted as midnight UTC.
    pub fn parse_date(s: &str) -> Result<Self, DateTimeError> {
        let date = civil::Date::strptime("%Y-%m-%d", s)?;
        let zoned = date.to_zoned(TimeZone::UTC)?;
        Ok(Self(zoned.timestamp()))
    }

    /// Convert to [`SpDate`] (date only, in the system local timezone).
    pub fn to_date(&self) -> SpDate {
        SpDate(self.0.to_zoned(TimeZone::system()).date())
    }
}

impl Default for SpDateTime {
    fn default() -> Self {
        Self::now()
    }
}

impl Display for SpDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Deterministic RFC 3339 with always-3 fractional digits and `Z`, so the
        // serialized form is a stable contract value regardless of subsecond
        // precision in the underlying timestamp.
        let z = self.0.to_zoned(TimeZone::UTC);
        let (d, t) = (z.date(), z.time());
        let millis = t.subsec_nanosecond() / 1_000_000;
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            d.year(),
            d.month(),
            d.day(),
            t.hour(),
            t.minute(),
            t.second(),
            millis,
        )
    }
}

impl FromStr for SpDateTime {
    type Err = DateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<Timestamp>()?))
    }
}

impl Add<SignedDuration> for SpDateTime {
    type Output = Self;

    fn add(self, rhs: SignedDuration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<SignedDuration> for SpDateTime {
    fn add_assign(&mut self, rhs: SignedDuration) {
        self.0 = self.0 + rhs;
    }
}

impl Sub<SignedDuration> for SpDateTime {
    type Output = Self;

    fn sub(self, rhs: SignedDuration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<SignedDuration> for SpDateTime {
    fn sub_assign(&mut self, rhs: SignedDuration) {
        self.0 = self.0 - rhs;
    }
}

// ===== SpDate - Date without time =====

/// A calendar date with no time-of-day, backed by [`jiff::civil::Date`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub struct SpDate(civil::Date);

impl SpDate {
    /// Get today's date in the system local timezone.
    pub fn today() -> Self {
        Self(Timestamp::now().to_zoned(TimeZone::system()).date())
    }

    /// Create from year, month, day.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self(civil::Date::new(year as i16, month as i8, day as i8).expect("valid calendar date"))
    }

    /// Convert to [`SpDateTime`] at midnight in the system local timezone.
    pub fn to_datetime(&self) -> SpDateTime {
        match self.0.to_zoned(TimeZone::system()) {
            Ok(zoned) => SpDateTime(zoned.timestamp()),
            // Fallback: treat as midnight UTC.
            Err(_) => SpDateTime(
                self.0
                    .to_zoned(TimeZone::UTC)
                    .expect("midnight UTC is always valid")
                    .timestamp(),
            ),
        }
    }

    /// Format as YYYY-MM-DD.
    pub fn format_date(&self) -> String {
        self.0.strftime("%Y-%m-%d").to_string()
    }

    /// Parse from YYYY-MM-DD format.
    pub fn parse(s: &str) -> Result<Self, DateTimeError> {
        Ok(Self(civil::Date::strptime("%Y-%m-%d", s)?))
    }

    /// Add days to the date.
    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + Span::new().days(days))
    }

    /// Subtract days from the date.
    pub fn sub_days(&self, days: i64) -> Self {
        Self(self.0 - Span::new().days(days))
    }

    /// Calculate days between this date and another.
    pub fn days_until(&self, other: SpDate) -> i64 {
        self.0
            .until(other.0)
            .map(|span| span.get_days() as i64)
            .unwrap_or(0)
    }

    /// Get year.
    pub fn year(&self) -> i32 {
        self.0.year() as i32
    }

    /// Get month (1-12).
    pub fn month(&self) -> u32 {
        self.0.month() as u32
    }

    /// Get day (1-31).
    pub fn day(&self) -> u32 {
        self.0.day() as u32
    }
}

impl Default for SpDate {
    fn default() -> Self {
        Self::today()
    }
}

impl Display for SpDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year(), self.month(), self.day())
    }
}

impl FromStr for SpDate {
    type Err = DateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<SpDateTime> for SpDate {
    fn from(dt: SpDateTime) -> Self {
        dt.to_date()
    }
}

impl From<SpDate> for SpDateTime {
    fn from(date: SpDate) -> Self {
        date.to_datetime()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let default_datetime = SpDateTime::default();
        assert_eq!(default_datetime.timestamp(), SpDateTime::now().timestamp());
    }

    #[test]
    fn test_from_str() {
        let datetime_str = "2022-01-01T00:00:00.000Z";
        let datetime = SpDateTime::from_str(datetime_str).unwrap();
        assert_eq!(datetime.to_string(), datetime_str);
    }

    #[test]
    fn test_from_str_without_millis_roundtrips_to_contract_format() {
        // RFC 3339 input without fractional seconds still serializes back with
        // the fixed `.000Z` contract form.
        let datetime = SpDateTime::from_str("2022-01-01T00:00:00Z").unwrap();
        assert_eq!(datetime.to_string(), "2022-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_add_duration() {
        let datetime = SpDateTime::from_str("2022-01-01T00:00:00Z").unwrap();
        let duration = SignedDuration::from_hours(24);
        let new_datetime = datetime + duration;
        assert_eq!(
            new_datetime.timestamp(),
            datetime.timestamp() + duration.as_secs()
        );
    }

    #[test]
    fn test_add_assign_duration() {
        let mut datetime = SpDateTime::from_str("2022-01-01T00:00:00Z").unwrap();
        datetime += SignedDuration::from_hours(24);
        assert_eq!(
            datetime.timestamp(),
            SpDateTime::from_str("2022-01-02T00:00:00Z")
                .unwrap()
                .timestamp()
        );
    }

    #[test]
    fn test_sub_duration() {
        let datetime = SpDateTime::from_str("2022-01-02T00:00:00Z").unwrap();
        let duration = SignedDuration::from_hours(24);
        let new_datetime = datetime - duration;
        assert_eq!(
            new_datetime.timestamp(),
            datetime.timestamp() - duration.as_secs()
        );
    }

    #[test]
    fn test_sub_assign_duration() {
        let mut datetime = SpDateTime::from_str("2022-01-02T00:00:00Z").unwrap();
        datetime -= SignedDuration::from_hours(24);
        assert_eq!(
            datetime.timestamp(),
            SpDateTime::from_str("2022-01-01T00:00:00Z")
                .unwrap()
                .timestamp()
        );
    }

    #[test]
    fn now_survives_serialize_parse_round_trip() {
        // now() is millisecond-resolution so it equals its persisted form.
        let t = SpDateTime::now();
        let json = serde_json::to_string(&t).unwrap();
        let back: SpDateTime = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn test_serde_roundtrip_is_contract_format() {
        let datetime = SpDateTime::from_str("2022-01-01T00:00:00.000Z").unwrap();
        let json = serde_json::to_string(&datetime).unwrap();
        assert_eq!(json, "\"2022-01-01T00:00:00.000Z\"");
        let parsed: SpDateTime = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, datetime);
    }

    // SpDate tests
    #[test]
    fn test_spdate_from_str() {
        let date = SpDate::from_str("2022-01-15").unwrap();
        assert_eq!(date.to_string(), "2022-01-15");
    }

    #[test]
    fn test_spdate_add_days() {
        let date = SpDate::from_ymd(2022, 1, 15);
        let new_date = date.add_days(10);
        assert_eq!(new_date.to_string(), "2022-01-25");
    }

    #[test]
    fn test_spdate_sub_days() {
        let date = SpDate::from_ymd(2022, 1, 15);
        let new_date = date.sub_days(5);
        assert_eq!(new_date.to_string(), "2022-01-10");
    }

    #[test]
    fn test_spdate_days_until() {
        let date1 = SpDate::from_ymd(2022, 1, 1);
        let date2 = SpDate::from_ymd(2022, 1, 11);
        assert_eq!(date1.days_until(date2), 10);
    }

    #[test]
    fn test_spdate_to_datetime() {
        let date = SpDate::from_ymd(2022, 1, 15);
        let datetime = date.to_datetime();
        assert_eq!(datetime.format_local("%Y-%m-%d"), "2022-01-15");
    }

    #[test]
    fn test_datetime_to_date() {
        let datetime = SpDateTime::parse_date("2022-01-15").unwrap();
        let date = datetime.to_date();
        assert_eq!(date.to_string(), "2022-01-15");
    }

    #[test]
    fn test_spdate_ordering() {
        let date1 = SpDate::from_ymd(2022, 1, 1);
        let date2 = SpDate::from_ymd(2022, 1, 15);
        assert!(date1 < date2);
        assert!(date2 > date1);
    }
}
