use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Timestamptz;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use time::error::ComponentRange;
use time::serde::rfc3339;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime};
use tracing::info;
use utoipa::ToSchema;

/// Used for timing things. Prints how long the object lived when dropped.
pub struct Timer<'a> {
    name: &'a str,
    start: std::time::Instant,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        let elapsed_time = self.start.elapsed();
        let time_in_s = elapsed_time.as_secs_f64();
        match elapsed_time.as_nanos().ilog10() {
            0..3 => info!("{} took {:.1}ns", self.name, time_in_s * 1e9),
            3..6 => info!("{} took {:.3}μs", self.name, time_in_s * 1e6),
            6..9 => info!("{} took {:.3}ms", self.name, time_in_s * 1e3),
            9..12 => info!("{} took {:.3}s", self.name, time_in_s),
            12.. => info!("{} took {:.0}s", self.name, time_in_s),
        }
    }
}

/// A wrapper for [`OffsetDateTime`] that serializes/deserializes according to RFC 3339.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema,
)]
#[diesel(sql_type = Timestamptz)]
#[schema(description = "A RFC 3339 formatted datetime string")]
pub struct DateTime(#[serde(with = "rfc3339")] OffsetDateTime);

impl DateTime {
    pub fn now() -> Self {
        OffsetDateTime::now_utc().into()
    }

    pub fn today() -> Self {
        Self::now().date().midnight().assume_utc().into()
    }

    pub fn yesterday() -> Self {
        Self::now()
            .date()
            .previous_day()
            .unwrap_or(Date::MIN)
            .midnight()
            .assume_utc()
            .into()
    }

    pub fn tomorrow() -> Self {
        Self::now()
            .date()
            .next_day()
            .unwrap_or(Date::MAX)
            .midnight()
            .assume_utc()
            .into()
    }

    pub fn from_date(year: i32, month: Month, day: u8) -> Result<Self, ComponentRange> {
        let date = Date::from_calendar_date(year, month, day)?;
        let date_time = Date::midnight(date);
        let utc_time = PrimitiveDateTime::assume_utc(date_time);
        Ok(Self(utc_time))
    }

    #[cfg(test)]
    pub const fn test_date() -> Self {
        Self(time::macros::datetime!(2008-09-15 0:00 UTC))
    }
}

impl Deref for DateTime {
    type Target = OffsetDateTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<OffsetDateTime> for DateTime {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl ToSql<Timestamptz, Pg> for DateTime {
    fn to_sql<'a>(&'a self, out: &mut Output<'a, '_, Pg>) -> serialize::Result {
        <OffsetDateTime as ToSql<Timestamptz, Pg>>::to_sql(self, out)
    }
}

impl FromSql<Timestamptz, Pg> for DateTime {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        OffsetDateTime::from_sql(value).map(DateTime)
    }
}

pub mod option_datetime {
    use super::*;
    use time::format_description::well_known::Rfc3339;
    use time::format_description::FormatItem;
    use time::macros::format_description;

    const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

    fn parse_datetime(s: &str) -> Result<OffsetDateTime, time::error::Parse> {
        if let Ok(dt) = OffsetDateTime::parse(s, &Rfc3339) {
            return Ok(dt);
        }
        let date = time::Date::parse(s, &DATE_FORMAT).map_err(|_| {
            // Return the Rfc3339 error for consistency
            OffsetDateTime::parse(s, &Rfc3339).unwrap_err()
        })?;
        Ok(date.midnight().assume_utc())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s.as_deref() {
            None | Some("") => Ok(None),
            Some(s) => parse_datetime(s)
                .map(|dt| Some(DateTime(dt)))
                .map_err(de::Error::custom),
        }
    }
}
