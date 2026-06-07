use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
pub enum DateTimeError {
    #[error("parse error: {0}")]
    ParseError(#[from] chrono::ParseError),
}

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
pub struct SpDateTime(chrono::DateTime<chrono::Utc>);

impl SpDateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn to_chrono(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }

    pub fn from_chrono(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }

    /// Convert from UNIX timestamp (seconds since epoch).
    pub fn from_timestamp(timestamp: i64) -> Option<Self> {
        Some(Self(chrono::DateTime::<chrono::Utc>::from_timestamp(
            timestamp, 0,
        )?))
    }

    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn format_friendly(&self) -> String {
        self.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn format(&self, format: &str) -> String {
        self.0.format(format).to_string()
    }

    pub fn format_local(&self, format: &str) -> String {
        self.0
            .with_timezone(&chrono::Local)
            .format(format)
            .to_string()
    }

    pub fn format_local_friendly(&self) -> String {
        self.format_local("%Y-%m-%d %H:%M:%S")
    }

    pub fn weekday(&self) -> chrono::Weekday {
        use chrono::Datelike;
        self.0.weekday()
    }

    pub fn num_days_from_monday(&self) -> u32 {
        use chrono::Datelike;
        self.0.weekday().num_days_from_monday()
    }

    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + chrono::Duration::days(days))
    }

    pub fn sub_days(&self, days: i64) -> Self {
        Self(self.0 - chrono::Duration::days(days))
    }

    /// Calculate the number of days elapsed since this datetime until now
    pub fn elapsed_days(&self) -> i64 {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.0);
        duration.num_days()
    }

    /// Format as YYYY-MM-DD for HTML date inputs
    pub fn format_date(&self) -> String {
        self.format("%Y-%m-%d")
    }

    /// Parse from YYYY-MM-DD HTML date input format
    pub fn parse_date(s: &str) -> Result<Self, DateTimeError> {
        use chrono::TimeZone;
        let naive = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
        let datetime = chrono::Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap());
        Ok(Self(datetime))
    }

    /// Convert to SpDate (date only, no time)
    pub fn to_date(&self) -> SpDate {
        use chrono::Datelike;
        let local_date = self.0.with_timezone(&chrono::Local);
        SpDate::from_ymd(local_date.year(), local_date.month(), local_date.day())
    }
}

impl Default for SpDateTime {
    fn default() -> Self {
        Self::now()
    }
}

impl Display for SpDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )
    }
}

impl FromStr for SpDateTime {
    type Err = DateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            chrono::DateTime::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc),
        ))
    }
}

impl Add<chrono::Duration> for SpDateTime {
    type Output = Self;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<chrono::Duration> for SpDateTime {
    fn add_assign(&mut self, rhs: chrono::Duration) {
        self.0 = self.0 + rhs;
    }
}

impl Sub<chrono::Duration> for SpDateTime {
    type Output = Self;

    fn sub(self, rhs: chrono::Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<chrono::Duration> for SpDateTime {
    fn sub_assign(&mut self, rhs: chrono::Duration) {
        self.0 = self.0 - rhs;
    }
}

// ===== SpDate - Date without time =====

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
pub struct SpDate(chrono::NaiveDate);

impl SpDate {
    /// Get today's date in local timezone
    pub fn today() -> Self {
        use chrono::Datelike;
        let now = chrono::Local::now();
        Self(chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap())
    }

    /// Create from year, month, day
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self(chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap())
    }

    /// Get the underlying NaiveDate
    pub fn to_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }

    /// Create from NaiveDate
    pub fn from_naive_date(date: chrono::NaiveDate) -> Self {
        Self(date)
    }

    /// Convert to SpDateTime at midnight UTC
    pub fn to_datetime(&self) -> SpDateTime {
        use chrono::TimeZone;
        // Treat SpDate as local time and convert to UTC
        let datetime = chrono::Utc.from_local_datetime(&self.0.and_hms_opt(0, 0, 0).unwrap());
        SpDateTime::from_chrono(datetime.earliest().unwrap_or_else(|| {
            // Fallback: treat as UTC directly
            chrono::Utc.from_utc_datetime(&self.0.and_hms_opt(0, 0, 0).unwrap())
        }))
    }

    /// Format as YYYY-MM-DD
    pub fn format_date(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }

    /// Parse from YYYY-MM-DD format
    pub fn parse(s: &str) -> Result<Self, DateTimeError> {
        let naive = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
        Ok(Self(naive))
    }

    /// Add days to the date
    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + chrono::Duration::days(days))
    }

    /// Subtract days from the date
    pub fn sub_days(&self, days: i64) -> Self {
        Self(self.0 - chrono::Duration::days(days))
    }

    /// Get the weekday
    pub fn weekday(&self) -> chrono::Weekday {
        use chrono::Datelike;
        self.0.weekday()
    }

    /// Get number of days from Monday (0 = Monday, 6 = Sunday)
    pub fn num_days_from_monday(&self) -> u32 {
        use chrono::Datelike;
        self.0.weekday().num_days_from_monday()
    }

    /// Calculate days between this date and another
    pub fn days_until(&self, other: SpDate) -> i64 {
        (other.0 - self.0).num_days()
    }

    /// Get year
    pub fn year(&self) -> i32 {
        use chrono::Datelike;
        self.0.year()
    }

    /// Get month (1-12)
    pub fn month(&self) -> u32 {
        use chrono::Datelike;
        self.0.month()
    }

    /// Get day (1-31)
    pub fn day(&self) -> u32 {
        use chrono::Datelike;
        self.0.day()
    }
}

impl Default for SpDate {
    fn default() -> Self {
        Self::today()
    }
}

impl Display for SpDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl FromStr for SpDate {
    type Err = DateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Add<chrono::Duration> for SpDate {
    type Output = Self;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<chrono::Duration> for SpDate {
    fn add_assign(&mut self, rhs: chrono::Duration) {
        self.0 = self.0 + rhs;
    }
}

impl Sub<chrono::Duration> for SpDate {
    type Output = Self;

    fn sub(self, rhs: chrono::Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<chrono::Duration> for SpDate {
    fn sub_assign(&mut self, rhs: chrono::Duration) {
        self.0 = self.0 - rhs;
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
    fn test_add_duration() {
        let datetime = SpDateTime::from_str("2022-01-01T00:00:00Z").unwrap();
        let duration = chrono::Duration::days(1);
        let new_datetime = datetime + duration;
        assert_eq!(
            new_datetime.timestamp(),
            datetime.timestamp() + duration.num_seconds()
        );
    }

    #[test]
    fn test_add_assign_duration() {
        let mut datetime = SpDateTime::from_str("2022-01-01T00:00:00Z").unwrap();
        let duration = chrono::Duration::days(1);
        datetime += duration;
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
        let duration = chrono::Duration::days(1);
        let new_datetime = datetime - duration;
        assert_eq!(
            new_datetime.timestamp(),
            datetime.timestamp() - duration.num_seconds()
        );
    }

    #[test]
    fn test_sub_assign_duration() {
        let mut datetime = SpDateTime::from_str("2022-01-02T00:00:00Z").unwrap();
        let duration = chrono::Duration::days(1);
        datetime -= duration;
        assert_eq!(
            datetime.timestamp(),
            SpDateTime::from_str("2022-01-01T00:00:00Z")
                .unwrap()
                .timestamp()
        );
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
        assert_eq!(datetime.format_date(), "2022-01-15");
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
