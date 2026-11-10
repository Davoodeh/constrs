//! Holds WHATWG date parsers in const time without allocation.
//!
//! The date in WHATWG is a [proleptic Gregorian date] from year 1 (0 is invalid) and onwards.
//!
//! [proleptic Gregorian date]:
//! https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#proleptic-gregorian-date

use crate::{num, slice};

/// Maximum number of days in a given month (0 is invalid, guaranteed to be in `0 | 28..=31`).
///
/// This is an implementation of [number of days in month algorithm]. The interesting part about
/// the algorithm is that it only needs to have the last 4 least significant decimal digits of the
/// year to calculate it meaning "12024" and "2024" will behave the same.
///
/// # Examples
///
/// ```rust
/// # use constrs::date::{days_in_month, YearMonthError::*};
/// assert_eq!(days_in_month(b"0",    00), 00);
/// assert_eq!(days_in_month(b"00",   00), 00);
/// assert_eq!(days_in_month(b"0",    01), 00);
/// assert_eq!(days_in_month(b"0",    13), 00);
/// assert_eq!(days_in_month(b"1",    13), 00);
/// assert_eq!(days_in_month(b"01",   13), 00);
/// assert_eq!(days_in_month(b"1",    01), 31);
/// assert_eq!(days_in_month(b"1",    03), 31);
/// assert_eq!(days_in_month(b"1",    04), 30);
/// assert_eq!(days_in_month(b"1",    05), 31);
/// assert_eq!(days_in_month(b"1",    06), 30);
/// assert_eq!(days_in_month(b"1",    07), 31);
/// assert_eq!(days_in_month(b"1",    08), 31);
/// assert_eq!(days_in_month(b"1",    09), 30);
/// assert_eq!(days_in_month(b"1",    10), 31);
/// assert_eq!(days_in_month(b"1",    11), 30);
/// assert_eq!(days_in_month(b"1",    12), 31);
///
/// assert_eq!(days_in_month(b"1",    02), 28);
/// assert_eq!(days_in_month(b"4",    02), 29);
/// assert_eq!(days_in_month(b"5",    02), 28);
/// assert_eq!(days_in_month(b"8",    02), 29);
/// assert_eq!(days_in_month(b"9",    02), 28);
/// assert_eq!(days_in_month(b"80",   02), 29);
/// assert_eq!(days_in_month(b"81",   02), 28);
/// assert_eq!(days_in_month(b"104",  02), 29);
/// assert_eq!(days_in_month(b"1300", 02), 28);
/// assert_eq!(days_in_month(b"1900", 02), 28);
/// assert_eq!(days_in_month(b"2000", 02), 29);
/// assert_eq!(days_in_month(b"2024", 02), 29);
///
/// assert_eq!(days_in_month(b"91300", 02), 28);
/// assert_eq!(days_in_month(b"92000", 02), 29);
/// assert_eq!(days_in_month(b"92024", 02), 29);
/// ```
///
/// [number of days in month algorithm]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#number-of-days-in-month-month-of-year-year
pub const fn days_in_month(year: &[u8], month: u8) -> u8 {
    let year = match year.last_chunk::<4>() {
        Some(v) => v,
        None => year,
    };
    if let Some(year) = num::u16::decode(year, 10) {
        // the following will check for year == 0 and return 0 so no need to check above
        return u16::days_in_month(year, month);
    }
    0
}

/// Is this year a leap year?
pub const fn is_leap(year: &[u8]) -> bool {
    let year = match year.last_chunk::<4>() {
        Some(v) => v,
        None => year,
    };
    if let Some(year) = num::u16::decode(year, 10) {
        // the following will check for year == 0 and return 0 so no need to check above
        return u16::is_leap(year);
    }
    false
}

/// Return the next leap year (0 means the end of the type is reached).
///
/// Only returns 0 when the value is not a valid ASCII string of numbers.
pub const fn next_leap(year: &[u8]) -> u16 {
    let year = match year.last_chunk::<4>() {
        Some(v) => v,
        None => year,
    };
    if let Some(year) = num::u16::decode(year, 10) {
        // the following will check for year == 0 and return 0 so no need to check above
        return u16::next_leap(year);
    }
    0
}

/// Errors thrown when parsing months and years (see [`year_month`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearMonthError {
    /// Expected an input of at least 4 (year) + 1 (dash, `'-'`) + 2 (month) characters.
    InputLen,
    /// Expected a year of at least 4 digits and less than what the datatype can hold.
    YearInvalid,
    /// Expected a dash to separate year and month.
    DashExpected,
    /// Expected a month in `1..=12`
    MonthInvalid,
}

impl core::fmt::Display for YearMonthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InputLen => "input is too short or long for a 'yyyy-mm' date",
            Self::YearInvalid => "expected a non-zero year as a four or more ASCII digit year",
            Self::DashExpected => "expected a '-' to separate 'yyyy-mm'",
            Self::MonthInvalid => "expected a two ASCII digit month between 1 and 12",
        }
        .fmt(f)
    }
}

impl core::error::Error for YearMonthError {}

/// Take a `Fn(s: &[u8]) -> (NON_ZERO_YEAR, len, s)` and parse `yyyy-mm`.
macro_rules! year_month_prefix {
    (|$s:ident| $expr:expr) => {{
        let s = $s;
        if s.len() < 7 {
            return Err(YearMonthError::InputLen);
        }

        let (non_zero_year, year_len, s) = $expr;

        // skip the dash which was checked before this
        //
        // the length was already checked
        if s[0] != b'-' {
            return Err(YearMonthError::DashExpected);
        }
        let s = s.split_at(1).1;

        // exactly two digits for the month is required.
        if s.len() < 2 {
            return Err(YearMonthError::MonthInvalid);
        }

        // do not go till the end of the text, only the next two digits matter
        let Some(month @ 1..=12) = num::u8::decode(s.split_at(2).0, 10) else {
            return Err(YearMonthError::MonthInvalid);
        };

        Ok((non_zero_year, month, year_len + 1 + 2))
    }};
}

/// Return the `yyyy-mm` [valid month string] where year in `1..` and month in `1..=12` at prefix.
///
/// # Examples
///
/// ```rust
/// # use constrs::date::{year_month_prefix, YearMonthError::*};
/// assert_eq!(year_month_prefix(b""),         Err(InputLen));
/// assert_eq!(year_month_prefix(b"-"),        Err(InputLen));
/// assert_eq!(year_month_prefix(b"0001"),     Err(InputLen));
/// assert_eq!(year_month_prefix(b"0001-"),    Err(InputLen));
/// assert_eq!(year_month_prefix(b"001-06"),   Err(InputLen));
/// assert_eq!(year_month_prefix(b"0001-1"),   Err(InputLen));
/// assert_eq!(year_month_prefix(b"001-006"),  Err(YearInvalid));
/// assert_eq!(year_month_prefix(b"0000-06"),  Err(YearInvalid));
/// assert_eq!(year_month_prefix(b"00001-6"),  Err(MonthInvalid));
/// assert_eq!(year_month_prefix(b"0001-006"), Err(MonthInvalid));
/// assert_eq!(year_month_prefix(b"0001-00"),  Err(MonthInvalid));
/// assert_eq!(year_month_prefix(b"0001-13"),  Err(MonthInvalid));
/// assert_eq!(year_month_prefix(b"0001-01"),  Ok((b"1".as_slice(), 01, 7)));
/// assert_eq!(year_month_prefix(b"0001-06"),  Ok((b"1".as_slice(), 06, 7)));
/// assert_eq!(year_month_prefix(b"0001-12"),  Ok((b"1".as_slice(), 12, 7)));
/// assert_eq!(year_month_prefix(b"000000000002-12"), Ok((b"2".as_slice(), 12, 15)));
/// assert_eq!(year_month_prefix(b"0001-12INVALID"),  Ok((b"1".as_slice(), 12, 7)));
/// ```
///
/// [valid month string]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-month-string
pub const fn year_month_prefix(s: &[u8]) -> Result<(&[u8], u8, usize), YearMonthError> {
    year_month_prefix!(|s| {
        // end == len => year is 0
        let (year_zeros_end, year_len) = num::prefix_span(s, 10);
        if year_len < 4 || year_zeros_end == year_len {
            return Err(YearMonthError::YearInvalid);
        }
        // skip zeros, then save the year and the rest
        let (year, s) = s
            .split_at(year_zeros_end)
            .1
            .split_at(year_len - year_zeros_end);
        (year, year_len, s)
    })
}

macro_rules! year_month {
    ($s:expr) => {{
        let s = $s;
        match year_month_prefix(s) {
            Ok((v1, v2, len)) if len == s.len() => Ok((v1, v2)),
            Ok(_) => Err(YearMonthError::InputLen),
            Err(e) => Err(e),
        }
    }};
}

macro_rules! decode {
    ($s:expr) => {{
        let s = $s;
        match prefix(s) {
            Ok((v1, v2, v3, len)) if len == s.len() => Ok((v1, v2, v3)),
            Ok(_) => Err(DateError::YearMonth(YearMonthError::InputLen)),
            Err(e) => Err(e),
        }
    }};
}

/// Return the `yyyy-mm` [valid month string] where year in `1..` and month in `1..=12` at prefix.
///
/// # Examples
///
/// ```rust
/// # use constrs::date::{year_month, YearMonthError::*};
/// assert_eq!(year_month(b""),         Err(InputLen));
/// assert_eq!(year_month(b"-"),        Err(InputLen));
/// assert_eq!(year_month(b"0001"),     Err(InputLen));
/// assert_eq!(year_month(b"0001-"),    Err(InputLen));
/// assert_eq!(year_month(b"001-06"),   Err(InputLen));
/// assert_eq!(year_month(b"0001-1"),   Err(InputLen));
/// assert_eq!(year_month(b"001-006"),  Err(YearInvalid));
/// assert_eq!(year_month(b"0000-06"),  Err(YearInvalid));
/// assert_eq!(year_month(b"00001-6"),  Err(MonthInvalid));
/// assert_eq!(year_month(b"0001-006"), Err(MonthInvalid));
/// assert_eq!(year_month(b"0001-00"),  Err(MonthInvalid));
/// assert_eq!(year_month(b"0001-13"),  Err(MonthInvalid));
/// assert_eq!(year_month(b"0001-01"),  Ok((b"1".as_slice(), 01)));
/// assert_eq!(year_month(b"0001-06"),  Ok((b"1".as_slice(), 06)));
/// assert_eq!(year_month(b"0001-12"),  Ok((b"1".as_slice(), 12)));
/// assert_eq!(year_month(b"000000000002-12"), Ok((b"2".as_slice(), 12)));
/// assert_eq!(year_month(b"0001-12INVALID"),  Err(InputLen));
/// ```
///
/// [valid month string]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-month-string
pub const fn year_month(s: &[u8]) -> Result<(&[u8], u8), YearMonthError> {
    year_month!(s)
}

/// Errors thrown when parsing of a date fails (see [`decode`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    YearMonth(YearMonthError),
    /// Expected a dash to separate year and month.
    DashExpected,
    /// The day number is invalid and must be exactly 2 digits fitting the [`days_in_month`].
    InvalidDay,
}

impl core::fmt::Display for DateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::YearMonth(e) => write!(f, "year-month error: {e}"),
            Self::DashExpected => "expected a '-' to separate the 'dd' in 'yyyy-mm-dd'".fmt(f),
            Self::InvalidDay => {
                "expected a two ASCII digit day between 1 and 31 which is a valid day of the month"
                    .fmt(f)
            }
        }
    }
}

impl From<YearMonthError> for DateError {
    fn from(value: YearMonthError) -> Self {
        Self::YearMonth(value)
    }
}

macro_rules! prefix {
    ($s:expr) => {{
        let s = $s;
        let (year, month, len) = match year_month_prefix(s) {
            Ok(v) => v,
            Err(e) => return Err(DateError::YearMonth(e)),
        };

        if len + 1 > s.len() || s[len] != b'-' {
            return Err(DateError::DashExpected);
        }

        // for the split
        if len + 3 > s.len() {
            return Err(DateError::InvalidDay);
        }

        match num::u8::decode(slice::range_get(s, len + 1, len + 3), 10) {
            Some(day) if day <= days_in_month(year, month) => Ok((year, month, day, len + 3)),
            _ => Err(DateError::InvalidDay),
        }
    }};
}

/// [Parse a date component] in `yyyy-mm-dd` format with validated day count at the start.
///
/// # Examples
///
/// ```rust
/// # use constrs::date::{prefix, DateError::*};
/// assert_eq!(prefix(b"0001-01-01"),          Ok((b"1".as_slice(), 1, 1,  10)));
/// assert_eq!(prefix(b"0001-01-01INVALID"),   Ok((b"1".as_slice(), 1, 1,  10)));
/// assert_eq!(prefix(b"000001-01-01INVALID"), Ok((b"1".as_slice(), 1, 1,  12)));
/// assert_eq!(prefix(b"000001-01-31INVALID"), Ok((b"1".as_slice(), 1, 31, 12)));
/// assert_eq!(prefix(b"0001-01"),             Err(DashExpected));
/// assert_eq!(prefix(b"0001-01-"),            Err(InvalidDay));
/// assert_eq!(prefix(b"0001-01-1"),           Err(InvalidDay));
/// assert_eq!(prefix(b"0001-01-40"),          Err(InvalidDay));
/// assert_eq!(prefix(b"2023-02-28"),          Ok((b"2023".as_slice(), 2, 28, 10))); // not leap
/// assert_eq!(prefix(b"2023-02-29"),          Err(InvalidDay)); // not leap
/// assert_eq!(prefix(b"2024-02-29"),          Ok((b"2024".as_slice(), 2, 29, 10))); // leap
/// ```
///
/// [Parse a date component]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#parse-a-date-component
pub const fn prefix(s: &[u8]) -> Result<(&[u8], u8, u8, usize), DateError> {
    prefix!(s)
}

/// [Parse a date component] in `yyyy-mm-dd` format with validated day count.
///
/// # Examples
///
/// ```rust
/// # use constrs::date::{decode, DateError::*, YearMonthError};
/// assert_eq!(decode(b"0001-01-01"),          Ok((b"1".as_slice(), 1, 1)));
/// assert_eq!(decode(b"0001-01-01INVALID"),   Err(YearMonth(YearMonthError::InputLen)));
/// assert_eq!(decode(b"000001-01-01INVALID"), Err(YearMonth(YearMonthError::InputLen)));
/// assert_eq!(decode(b"000001-01-31INVALID"), Err(YearMonth(YearMonthError::InputLen)));
/// assert_eq!(decode(b"0001-01"),             Err(DashExpected));
/// assert_eq!(decode(b"0001-01-"),            Err(InvalidDay));
/// assert_eq!(decode(b"0001-01-1"),           Err(InvalidDay));
/// assert_eq!(decode(b"0001-01-40"),          Err(InvalidDay));
/// assert_eq!(decode(b"2023-02-28"),          Ok((b"2023".as_slice(), 2, 28))); // not leap
/// assert_eq!(decode(b"2023-02-29"),          Err(InvalidDay)); // not leap
/// assert_eq!(decode(b"2024-02-29"),          Ok((b"2024".as_slice(), 2, 29))); // leap
/// ```
///
/// [Parse a date component]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#parse-a-date-component
pub const fn decode(s: &[u8]) -> Result<(&[u8], u8, u8), DateError> {
    decode!(s)
}

/// Helpers for parsing values in a space that only returns [`prim@u8`].
///
/// This module assumes year 0 is 1970.
pub mod u8 {
    use super::*;

    /// Parse a year only if it fits [`prim@u8`] (0 is 1970) (see [`super::year_month_prefix`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::date::{u8::year_month_prefix, YearMonthError::*};
    /// assert_eq!(year_month_prefix(b""),         Err(InputLen));
    /// assert_eq!(year_month_prefix(b"-"),        Err(InputLen));
    /// assert_eq!(year_month_prefix(b"0001"),     Err(InputLen));
    /// assert_eq!(year_month_prefix(b"0001-"),    Err(InputLen));
    /// assert_eq!(year_month_prefix(b"001-06"),   Err(InputLen));
    /// assert_eq!(year_month_prefix(b"0001-1"),   Err(InputLen));
    /// assert_eq!(year_month_prefix(b"001-006"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"0000-06"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"1969-01"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"1969-06"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"1969-12"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"2226-01"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"2226-06"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"2226-12"),  Err(YearInvalid));
    /// assert_eq!(year_month_prefix(b"01971-6"),  Err(MonthInvalid));
    /// assert_eq!(year_month_prefix(b"1971-006"), Err(MonthInvalid));
    /// assert_eq!(year_month_prefix(b"1971-00"),  Err(MonthInvalid));
    /// assert_eq!(year_month_prefix(b"1971-13"),  Err(MonthInvalid));
    /// assert_eq!(year_month_prefix(b"1970-01"),  Ok((0, 01, 7)));
    /// assert_eq!(year_month_prefix(b"1970-06"),  Ok((0, 06, 7)));
    /// assert_eq!(year_month_prefix(b"1970-12"),  Ok((0, 12, 7)));
    /// assert_eq!(year_month_prefix(b"2225-01"),  Ok((255, 01, 7)));
    /// assert_eq!(year_month_prefix(b"2225-06"),  Ok((255, 06, 7)));
    /// assert_eq!(year_month_prefix(b"2225-12"),  Ok((255, 12, 7)));
    /// assert_eq!(year_month_prefix(b"000000001972-12"), Ok((2, 12, 15)));
    /// assert_eq!(year_month_prefix(b"1971-12INVALID"),  Ok((1, 12, 7)));
    /// ```
    pub const fn year_month_prefix(s: &[u8]) -> Result<(u8, u8, usize), YearMonthError> {
        year_month_prefix!(|s| {
            // end == len => year is 0
            let (year, year_len) = num::u16::prefix(s, 10);
            // 2225 is the max: 1970 + u8::MAX
            if year_len < 4 || year < 1970 || year > 2225 {
                return Err(YearMonthError::YearInvalid);
            }
            let year_offset = (year - 1970) as u8;

            (year_offset, year_len, s.split_at(year_len).1)
        })
    }

    /// See [`super::days_in_month`].
    pub const fn days_in_month(year: u8, month: u8) -> u8 {
        u16::days_in_month(year as u16 + 1970, month)
    }

    /// Is this year a leap year?
    pub const fn is_leap(year: u8) -> bool {
        u16::is_leap(year as u16 + 1970)
    }

    /// Return the next leap year (0 means the end of the type is reached).
    pub const fn next_leap(year: u8) -> u16 {
        u16::next_leap(year as u16 + 1970)
    }

    /// See [`super::prefix`].
    pub const fn prefix(s: &[u8]) -> Result<(u8, u8, u8, usize), DateError> {
        prefix!(s)
    }

    /// See [`super::decode`].
    pub const fn decode(s: &[u8]) -> Result<(u8, u8, u8), DateError> {
        decode!(s)
    }

    /// See [`super::year_month`].
    pub const fn year_month(s: &[u8]) -> Result<(u8, u8), YearMonthError> {
        year_month!(s)
    }
}

/// "offset" refers to the +255 behavior in [`mod@u8`].
macro_rules! date_fns_no_offset {
    ($ty:ident) => {
        /// See [`super::prefix`].
        pub const fn prefix(s: &[u8]) -> Result<($ty, u8, u8, usize), DateError> {
            prefix!(s)
        }

        /// See [`super::decode`].
        pub const fn decode(s: &[u8]) -> Result<($ty, u8, u8), DateError> {
            decode!(s)
        }

        /// See [`super::year_month_prefix`].
        pub const fn year_month_prefix(s: &[u8]) -> Result<($ty, u8, usize), YearMonthError> {
            year_month_prefix!(|s| {
                // end == len => year is 0
                let (year, year_len) = num::$ty::prefix(s, 10);
                if year_len < 4 || year == 0 {
                    return Err(YearMonthError::YearInvalid);
                }
                (year, year_len, s.split_at(year_len).1)
            })
        }

        /// See [`super::year_month`].
        pub const fn year_month(s: &[u8]) -> Result<($ty, u8), YearMonthError> {
            year_month!(s)
        }

        /// See [`super::days_in_month`].
        pub const fn days_in_month(year: $ty, month: u8) -> u8 {
            if year == 0 {
                return 0;
            }
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if is_leap(year) => 29,
                2 => 28,
                _ => 0,
            }
        }

        /// Is this year a leap year?
        pub const fn is_leap(year: $ty) -> bool {
            year % 400 == 0 || (year % 4 == 0 && year % 100 != 0)
        }

        /// Return the next leap year (0 means the end of the type is reached).
        pub const fn next_leap(year: $ty) -> $ty {
            // for all the powers of two minus one in {8, 16, 32, 64, 128}, the last leap year is
            // simply -3 of that very value, if more than that, it's not possible to calculate.
            if year >= <$ty>::MAX - 3 {
                return 0;
            }
            // now that the value is limited to be under the max, no check is needed to add or
            // remove.  for values between MAX-8, MAX-4, the clause which adds 8 is never called.
            // Meaning for all the powers of two minus 8 in {8, 16, 32, 64, 128}, no value hits 00$
            // ending.

            // go to the next 4 as the candidate
            // v & !3 => v % 4
            let candidate = (year + 4) & !3;

            // check the new year to be 100 and if so, year % 4 is not enough, it must be year % 400
            if candidate % 100 == 0 && candidate % 400 != 0 {
                // if year % 400 for century leap is not valid skip it
                return candidate + 4;
            }
            candidate
        }
    };
}

/// Helpers for parsing values in a space that only returns [`prim@usize`].
pub mod usize {
    use super::*;

    date_fns_no_offset!(usize);
}

/// Helpers for parsing values in a space that only returns [`prim@u16`].
pub mod u16 {
    use super::*;

    date_fns_no_offset!(u16);
}

/// Helpers for parsing values in a space that only returns [`prim@u32`].
pub mod u32 {
    use super::*;

    date_fns_no_offset!(u32);
}

/// Helpers for parsing values in a space that only returns [`prim@u64`].
pub mod u64 {
    use super::*;

    date_fns_no_offset!(u64);
}

/// Helpers for parsing values in a space that only returns [`prim@u128`].
pub mod u128 {
    use super::*;

    date_fns_no_offset!(u128);
}
