//! [`ToSql`] and [`FromSql`] implementation for [`time::OffsetDateTime`].
use crate::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use crate::{Error, Result};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

const PRIMITIVE_DATE_TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
const PRIMITIVE_DATE_TIME_FORMAT_Z: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]Z");
const PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");
const PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS_Z: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]Z");
const PRIMITIVE_DATE_TIME_FORMAT_T: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
const PRIMITIVE_DATE_TIME_FORMAT_T_Z: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
const PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]");
const PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z");

const OFFSET_DATE_TIME_FORMAT: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
);
const OFFSET_DATE_TIME_FORMAT_SUBSECONDS: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);
const OFFSET_DATE_TIME_FORMAT_T: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
);
const OFFSET_DATE_TIME_FORMAT_T_SUBSECONDS: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);
const LEGACY_DATE_TIME_FORMAT: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second]:[subsecond] [offset_hour sign:mandatory]:[offset_minute]"
);
const DATE_FORMAT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");
const TIME_FORMAT: &[FormatItem<'_>] = format_description!("[hour]:[minute]");
const TIME_FORMAT_SECONDS: &[FormatItem<'_>] = format_description!("[hour]:[minute]:[second]");
const TIME_FORMAT_SECONDS_SUBSECONDS: &[FormatItem<'_>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

/// Date and time with time zone => ISO 8601 timestamp ("YYYY-MM-DD HH:MM:SS.SSS[+-]HH:MM").
impl ToSql for OffsetDateTime {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let time_string = self
            .format(OFFSET_DATE_TIME_FORMAT)
            .map_err(|err| Error::ToSqlConversionFailure(err.into()))?;
        Ok(ToSqlOutput::from(time_string))
    }
}

/// Parse a `OffsetDateTime` in one of the following formats:
/// YYYY-MM-DD HH:MM:SS.SSS[+-]HH:MM
/// YYYY-MM-DDTHH:MM:SS.SSS[+-]HH:MM
/// YYYY-MM-DD HH:MM:SS [+-]HH:MM
/// YYYY-MM-DD HH:MM:SS[+-]HH:MM
/// YYYY-MM-DDTHH:MM:SS[+-]HH:MM
/// YYYY-MM-DD HH:MM:SS.SSSZ
/// YYYY-MM-DDTHH:MM:SS.SSSZ
/// YYYY-MM-DD HH:MM:SS.SSS
/// YYYY-MM-DDTHH:MM:SS.SSS
/// YYYY-MM-DD HH:MM:SSZ
/// YYYY-MM-DDTHH:MM:SSZ
/// YYYY-MM-DD HH:MM:SS
/// YYYY-MM-DDTHH:MM:SS
impl FromSql for OffsetDateTime {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| {
            let has_t = Some('T') == s.chars().nth(10);
            let has_z = Some('Z') == s.chars().last();
            let is_primitive = s.len() < 25;

            let fmt = match (s.len(), has_t, has_z) {
                // YYYY-MM-DD HH:MM:SS.SSS[+-]HH:MM
                (29, false, false) => Ok(OFFSET_DATE_TIME_FORMAT_SUBSECONDS),

                // YYYY-MM-DDTHH:MM:SS.SSS[+-]HH:MM
                (29, true, false) => Ok(OFFSET_DATE_TIME_FORMAT_T_SUBSECONDS),

                // YYYY-MM-DD HH:MM:SS [+-]HH:MM
                (26, false, false) => Ok(LEGACY_DATE_TIME_FORMAT),

                // YYYY-MM-DD HH:MM:SS[+-]HH:MM
                (25, false, false) => Ok(OFFSET_DATE_TIME_FORMAT),

                // YYYY-MM-DDTHH:MM:SS[+-]HH:MM
                (25, true, false) => Ok(OFFSET_DATE_TIME_FORMAT_T),

                // YYYY-MM-DD HH:MM:SS.SSSZ
                (24, false, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS_Z),

                // YYYY-MM-DDTHH:MM:SS.SSSZ
                (24, true, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z),

                // YYYY-MM-DDTHH:MM:SS.SSSZ
                (24, true, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS),

                // YYYY-MM-DD HH:MM:SS.SSS
                (23, false, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS),

                // YYYY-MM-DDTHH:MM:SS.SSS
                (23, true, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS),

                // YYYY-MM-DD HH:MM:SSZ
                (20, false, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_Z),

                // YYYY-MM-DDTHH:MM:SSZ
                (20, true, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_Z),

                // YYYY-MM-DD HH:MM:SS
                (19, false, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT),

                // YYYY-MM-DDTHH:MM:SS
                (19, true, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T),
                _ => Err(FromSqlError::Other(
                    format!("Unknown date format: {}", s).into(),
                )),
            }?;

            if is_primitive {
                PrimitiveDateTime::parse(s, fmt).map(|date| date.assume_utc())
            } else {
                OffsetDateTime::parse(s, fmt)
            }
            .map_err(|err| FromSqlError::Other(err.into()))
        })
    }
}

/// ISO 8601 calendar date without timezone => "YYYY-MM-DD"
impl ToSql for Date {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let date_str = self
            .format(DATE_FORMAT)
            .map_err(|err| Error::ToSqlConversionFailure(err.into()))?;
        Ok(ToSqlOutput::from(date_str))
    }
}

/// "YYYY-MM-DD" => ISO 8601 calendar date without timezone.
impl FromSql for Date {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| match Date::parse(s, &DATE_FORMAT) {
                Ok(date) => Ok(date),
                Err(err) => Err(FromSqlError::Other(err.into())),
            })
    }
}

/// ISO 8601 time without timezone => "HH:MM:SS.SSS"
impl ToSql for Time {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let time_str = self
            .format(&TIME_FORMAT_SECONDS_SUBSECONDS)
            .map_err(|err| Error::ToSqlConversionFailure(err.into()))?;
        Ok(ToSqlOutput::from(time_str))
    }
}

/// "HH:MM"/"HH:MM:SS"/"HH:MM:SS.SSS" => ISO 8601 time without timezone.
impl FromSql for Time {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| {
            let fmt = match s.len() {
                5 => Ok(TIME_FORMAT),
                8 => Ok(TIME_FORMAT_SECONDS),
                10 | 11 | 12 => Ok(TIME_FORMAT_SECONDS_SUBSECONDS),
                _ => Err(FromSqlError::Other(
                    format!("Unknown time format: {}", s).into(),
                )),
            }?;

            Time::parse(s, fmt).map_err(|err| FromSqlError::Other(err.into()))
        })
    }
}

/// ISO 8601 combined date and time without timezone =>
/// "YYYY-MM-DD HH:MM:SS.SSS"
impl ToSql for PrimitiveDateTime {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let date_time_str = self
            .format(PRIMITIVE_DATE_TIME_FORMAT)
            .map_err(|err| Error::ToSqlConversionFailure(err.into()))?;
        Ok(ToSqlOutput::from(date_time_str))
    }
}

/// Parse a `PrimitiveDateTime` in one of the following formats:
/// YYYY-MM-DD HH:MM:SS.SSS[Z]
/// YYYY-MM-DDTHH:MM:SS.SSS[Z]
/// YYYY-MM-DD HH:MM:SS[Z]
/// YYYY-MM-DDTHH:MM:SS[Z]
impl FromSql for PrimitiveDateTime {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| {
            let has_t = Some('T') == s.chars().nth(10);
            let has_z = Some('Z') == s.chars().last();

            let fmt = match (s.len(), has_t, has_z) {
                (20, true, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_Z),
                (19, true, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T),
                (20, false, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_Z),
                (19, false, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT),
                (24, true, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z),
                (23, true, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS),
                (24, false, true) => Ok(PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS_Z),
                (23, false, false) => Ok(PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS),
                _ => Err(FromSqlError::Other(
                    format!("Unknown date format: {}", s).into(),
                )),
            }?;

            PrimitiveDateTime::parse(s, fmt).map_err(|err| FromSqlError::Other(err.into()))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::types::time::TIME_FORMAT;
    use crate::{Connection, Result};
    use time::format_description::well_known::Rfc3339;
    use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

    use super::{
        DATE_FORMAT, PRIMITIVE_DATE_TIME_FORMAT, PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS,
        PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS_Z, PRIMITIVE_DATE_TIME_FORMAT_T,
        PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS, PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z,
        PRIMITIVE_DATE_TIME_FORMAT_T_Z, PRIMITIVE_DATE_TIME_FORMAT_Z, TIME_FORMAT_SECONDS,
        TIME_FORMAT_SECONDS_SUBSECONDS,
    };

    #[test]
    fn test_offset_date_time() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.execute_batch("CREATE TABLE foo (t TEXT, i INTEGER, f FLOAT)")?;

        let mut ts_vec = vec![];

        let make_datetime = |secs: i128, nanos: i128| {
            OffsetDateTime::from_unix_timestamp_nanos(1_000_000_000 * secs + nanos).unwrap()
        };

        ts_vec.push(make_datetime(10_000, 0)); //January 1, 1970 2:46:40 AM
        ts_vec.push(make_datetime(10_000, 1000)); //January 1, 1970 2:46:40 AM (and one microsecond)
        ts_vec.push(make_datetime(1_500_391_124, 1_000_000)); //July 18, 2017
        ts_vec.push(make_datetime(2_000_000_000, 2_000_000)); //May 18, 2033
        ts_vec.push(make_datetime(3_000_000_000, 999_999_999)); //January 24, 2065
        ts_vec.push(make_datetime(10_000_000_000, 0)); //November 20, 2286

        for ts in ts_vec {
            db.execute("INSERT INTO foo(t) VALUES (?1)", [ts])?;

            let from: OffsetDateTime = db.one_column("SELECT t FROM foo")?;

            db.execute("DELETE FROM foo", [])?;

            assert_eq!(from, ts);
        }
        Ok(())
    }

    #[test]
    fn test_offset_date_time_parsing() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let tests = vec![
            (
                "2013-10-07 08:23:19",
                OffsetDateTime::parse("2013-10-07T08:23:19Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07 08:23:19Z",
                OffsetDateTime::parse("2013-10-07T08:23:19Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07T08:23:19Z",
                OffsetDateTime::parse("2013-10-07T08:23:19Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07 08:23:19.120",
                OffsetDateTime::parse("2013-10-07T08:23:19.120Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07 08:23:19.120Z",
                OffsetDateTime::parse("2013-10-07T08:23:19.120Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07T08:23:19.120Z",
                OffsetDateTime::parse("2013-10-07T08:23:19.120Z", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07 04:23:19-04:00",
                OffsetDateTime::parse("2013-10-07T04:23:19-04:00", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07 04:23:19.120-04:00",
                OffsetDateTime::parse("2013-10-07T04:23:19.120-04:00", &Rfc3339).unwrap(),
            ),
            (
                "2013-10-07T04:23:19.120-04:00",
                OffsetDateTime::parse("2013-10-07T04:23:19.120-04:00", &Rfc3339).unwrap(),
            ),
        ];

        for (s, t) in tests {
            let result: OffsetDateTime = db.query_row("SELECT ?1", [s], |r| r.get(0))?;
            assert_eq!(result, t);
        }
        Ok(())
    }

    #[test]
    fn test_sqlite_functions_offset_date_time() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.one_column::<OffsetDateTime>("SELECT CURRENT_TIMESTAMP")?;
        Ok(())
    }

    #[test]
    fn test_sqlite_functions_time() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.one_column::<Time>("SELECT CURRENT_TIME")?;
        Ok(())
    }

    #[test]
    fn test_sqlite_functions_date() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.one_column::<Date>("SELECT CURRENT_DATE")?;
        Ok(())
    }

    #[test]
    fn test_param_offset_date_time() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let result: bool = db.query_row(
            "SELECT 1 WHERE ?1 BETWEEN datetime('now', '-1 minute') AND datetime('now', '+1 minute')",
        [OffsetDateTime::now_utc()], |r| r.get(0)
        )?;
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_param_date() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let result: bool = db.query_row(
            "SELECT 1 WHERE ?1 BETWEEN date('now', '-1 day') AND date('now', '+1 day')",
            [OffsetDateTime::now_utc()],
            |r| r.get(0),
        )?;
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_date_parsing() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let result: Date = db.query_row("SELECT ?1", ["2013-10-07"], |r| r.get(0))?;
        assert_eq!(result, Date::parse("2013-10-07", DATE_FORMAT).unwrap());
        Ok(())
    }

    #[test]
    fn test_time_parsing() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let tests = vec![
            ("08:23", Time::parse("08:23", &TIME_FORMAT).unwrap()),
            (
                "08:23:19",
                Time::parse("08:23:19", &TIME_FORMAT_SECONDS).unwrap(),
            ),
            (
                "08:23:19.111",
                Time::parse("08:23:19.111", &TIME_FORMAT_SECONDS_SUBSECONDS).unwrap(),
            ),
        ];

        for (s, t) in tests {
            let result: Time = db.query_row("SELECT ?1", [s], |r| r.get(0))?;
            assert_eq!(result, t);
        }
        Ok(())
    }

    #[test]
    fn test_primitive_date_time_parsing() -> Result<()> {
        let db = Connection::open_in_memory()?;

        let tests = vec![
            (
                "2013-10-07T08:23:19",
                PrimitiveDateTime::parse("2013-10-07T08:23:19", &PRIMITIVE_DATE_TIME_FORMAT_T)
                    .unwrap(),
            ),
            (
                "2013-10-07T08:23:19Z",
                PrimitiveDateTime::parse("2013-10-07T08:23:19Z", &PRIMITIVE_DATE_TIME_FORMAT_T_Z)
                    .unwrap(),
            ),
            (
                "2013-10-07T08:23:19.111",
                PrimitiveDateTime::parse(
                    "2013-10-07T08:23:19.111",
                    &PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS,
                )
                .unwrap(),
            ),
            (
                "2013-10-07T08:23:19.111Z",
                PrimitiveDateTime::parse(
                    "2013-10-07T08:23:19.111Z",
                    &PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z,
                )
                .unwrap(),
            ),
            (
                "2013-10-07 08:23:19",
                PrimitiveDateTime::parse("2013-10-07 08:23:19", &PRIMITIVE_DATE_TIME_FORMAT)
                    .unwrap(),
            ),
            (
                "2013-10-07 08:23:19Z",
                PrimitiveDateTime::parse("2013-10-07 08:23:19Z", &PRIMITIVE_DATE_TIME_FORMAT_Z)
                    .unwrap(),
            ),
            (
                "2013-10-07 08:23:19.111",
                PrimitiveDateTime::parse(
                    "2013-10-07 08:23:19.111",
                    &PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS,
                )
                .unwrap(),
            ),
            (
                "2013-10-07 08:23:19.111Z",
                PrimitiveDateTime::parse(
                    "2013-10-07 08:23:19.111Z",
                    &PRIMITIVE_DATE_TIME_FORMAT_SUBSECONDS_Z,
                )
                .unwrap(),
            ),
            (
                "2013-10-07T08:23:19.111Z",
                PrimitiveDateTime::parse(
                    "2013-10-07T08:23:19.111Z",
                    &PRIMITIVE_DATE_TIME_FORMAT_T_SUBSECONDS_Z,
                )
                .unwrap(),
            ),
        ];

        for (s, t) in tests {
            let result: PrimitiveDateTime = db.query_row("SELECT ?1", [s], |r| r.get(0))?;
            assert_eq!(result, t);
        }
        Ok(())
    }
}
