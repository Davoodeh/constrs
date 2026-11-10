//! Holds WHATWG date parsers in const time without allocation.
//!
//! The date in WHATWG is a [proleptic Gregorian date] from year 1 (0 is invalid) and onwards.
//!
//! The timestamp functions only exist under module [`mod@u32`] (since the input and output years
//! are [`prim@u32`]). Other primitives do not support timestamp operations (yet).
//!
//! [proleptic Gregorian date]:
//! https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#proleptic-gregorian-date

use crate::{num, slice};

/// Parse the last 4 (Least Significant Digits) of the given number, run `f` or return default if 0.
macro_rules! last_four_digits_or {
    ($e:expr, |$v:ident $(: u16)?| $f:expr, $default:expr) => {{
        let $v = match $v.last_chunk::<4>() {
            Some(v) => v,
            None => $v,
        };
        if let Some($v @ 1..) = num::u16::decode($v, 10) {
            return { $f };
        }
        $default
    }};
}

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
    last_four_digits_or!(year, |year| u16::days_in_month(year, month), 0)
}

/// [`days_in_month`] but assumes the year is non-leap, always returns `0 | 28..=31`.
pub const fn days_in_month_non_leap(month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28,
        _ => 0,
    }
}

/// Count the number of days in a given moon count (moon is `0..12`).
///
/// This is the last day of each month accumulated (ordinal).
pub const FULL_MOON_ORDINALS: [u16; 12] = [31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

/// Return the number of day in the year from `1..=365` (0 if month not in `1..=12` or day invalid).
///
/// # Example
///
/// ```rust
/// # use constrs::date::ordinal_non_leap;
/// assert_eq!(ordinal_non_leap(00, 00),   0);
/// assert_eq!(ordinal_non_leap(00, 31),   0);
/// assert_eq!(ordinal_non_leap(01, 32),   0);
/// assert_eq!(ordinal_non_leap(01, 01),   1);
/// assert_eq!(ordinal_non_leap(01, 02),   2);
/// assert_eq!(ordinal_non_leap(01, 31),  31);
/// assert_eq!(ordinal_non_leap(02, 01),  32);
/// assert_eq!(ordinal_non_leap(02, 28),  59);
/// assert_eq!(ordinal_non_leap(03, 01),  60);
/// assert_eq!(ordinal_non_leap(03, 31),  90);
/// assert_eq!(ordinal_non_leap(04, 01),  91);
/// assert_eq!(ordinal_non_leap(04, 30), 120);
/// assert_eq!(ordinal_non_leap(05, 01), 121);
/// assert_eq!(ordinal_non_leap(05, 31), 151);
/// assert_eq!(ordinal_non_leap(06, 01), 152);
/// assert_eq!(ordinal_non_leap(06, 30), 181);
/// assert_eq!(ordinal_non_leap(07, 01), 182);
/// assert_eq!(ordinal_non_leap(07, 31), 212);
/// assert_eq!(ordinal_non_leap(08, 01), 213);
/// assert_eq!(ordinal_non_leap(08, 31), 243);
/// assert_eq!(ordinal_non_leap(09, 01), 244);
/// assert_eq!(ordinal_non_leap(09, 30), 273);
/// assert_eq!(ordinal_non_leap(10, 01), 274);
/// assert_eq!(ordinal_non_leap(10, 31), 304);
/// assert_eq!(ordinal_non_leap(11, 01), 305);
/// assert_eq!(ordinal_non_leap(11, 30), 334);
/// assert_eq!(ordinal_non_leap(12, 01), 335);
/// assert_eq!(ordinal_non_leap(12, 31), 365);
/// ```
pub const fn ordinal_non_leap(month: u8, day: u8) -> u16 {
    if month < 1 || month > 12 {
        return 0; // invalid month
    }
    let days_of_month = days_in_month_non_leap(month);
    if days_of_month < day {
        return 0; // invalid day of month
    }
    if month == 1 {
        return day as u16;
    }
    FULL_MOON_ORDINALS[(month - 2) as usize] + day as u16
}

/// Return [`ordinal_non_leap`] but leap aware.
pub const fn ordinal(year: &[u8], month: u8, day: u8) -> u16 {
    last_four_digits_or!(
        year,
        |year| if let v @ 1.. = ordinal_non_leap(month, day) {
            v + ((month > 2 && u16::is_leap(year)) as u16)
        } else {
            0
        },
        0
    )
}

/// Return [`ordinal_non_leap`] of this month with it's last day of month.
///
/// # Example
///
/// ```rust
/// # use constrs::date::{ordinal_non_leap, ordinal_last_non_leap};
/// assert_eq!(ordinal_non_leap(01, 31), ordinal_last_non_leap(01));
/// assert_eq!(ordinal_non_leap(02, 28), ordinal_last_non_leap(02));
/// assert_eq!(ordinal_non_leap(03, 31), ordinal_last_non_leap(03));
/// assert_eq!(ordinal_non_leap(04, 30), ordinal_last_non_leap(04));
/// assert_eq!(ordinal_non_leap(05, 31), ordinal_last_non_leap(05));
/// assert_eq!(ordinal_non_leap(06, 30), ordinal_last_non_leap(06));
/// assert_eq!(ordinal_non_leap(07, 31), ordinal_last_non_leap(07));
/// assert_eq!(ordinal_non_leap(08, 31), ordinal_last_non_leap(08));
/// assert_eq!(ordinal_non_leap(09, 30), ordinal_last_non_leap(09));
/// assert_eq!(ordinal_non_leap(10, 31), ordinal_last_non_leap(10));
/// assert_eq!(ordinal_non_leap(11, 30), ordinal_last_non_leap(11));
/// assert_eq!(ordinal_non_leap(12, 31), ordinal_last_non_leap(12));
/// ```
pub const fn ordinal_last_non_leap(month: u8) -> u16 {
    if month < 1 || month > 12 {
        return 0; // invalid month
    }
    FULL_MOON_ORDINALS[(month - 1) as usize]
}

/// Return [`ordinal_last_non_leap`] but leap aware.
pub const fn ordinal_last(year: &[u8], month: u8) -> u16 {
    last_four_digits_or!(
        year,
        |year| if let v @ 1.. = ordinal_last_non_leap(month) {
            v + ((month > 2 && u16::is_leap(year)) as u16)
        } else {
            0
        },
        0
    )
}

/// Is this year a leap year?
pub const fn is_leap(year: &[u8]) -> bool {
    last_four_digits_or!(year, |year| u16::is_leap(year), false)
}

/// Return the next leap year (0 means the end of the type is reached).
///
/// Only returns 0 when the value is not a valid ASCII string of numbers.
///
/// See [`u16::next_leap`] for more.
pub const fn next_leap(year: &[u8]) -> u16 {
    last_four_digits_or!(year, |year| u16::next_leap(year), 0)
}

/// Return the previous leap year (0 means the end of the type is reached).
///
/// See [`u16::previous_leap`] for more.
pub const fn previous_leap(year: &[u8]) -> u16 {
    last_four_digits_or!(year, |year| u16::previous_leap(year), 0)
}

/// Return whether the given year is a 53 week year (true) or a 52 week year (false).
///
/// See [`u16::is_weeks_53`].
pub const fn is_weeks_53(year: &[u8]) -> bool {
    last_four_digits_or!(year, |year| u16::is_weeks_53(year), false)
}

/// Errors thrown when parsing months and years (see [`year_week`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearWeekError {
    /// Expected an input of at least 4 (year) + 2 (dash then 'W', `"-W"`) + 2 (weeks) characters.
    InputLen,
    /// Expected a year of at least 4 digits and less than what the datatype can hold.
    YearInvalid,
    /// Expected a dash and then a W to separate year and week.
    DashWExpected,
    /// Expected a week value valid for the year between `1..=53`.
    WeekInvalid,
}

impl core::fmt::Display for YearWeekError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InputLen => "input is too short or long for a 'yyyy-Www' date",
            Self::YearInvalid => "expected a non-zero year as a four or more ASCII digit year",
            Self::DashWExpected => "expected a '-W' to separate 'yyyy-Www'",
            Self::WeekInvalid => {
                "expected a two ASCII digit week between 1 and 53, valid for the year"
            }
        }
        .fmt(f)
    }
}

impl core::error::Error for YearWeekError {}

/// Take a `Fn(s: &[u8]) -> (NON_ZERO_YEAR, len, s)` and parse `yyyy-Www`.
macro_rules! year_week_prefix {
    (|$s:ident| $expr:expr) => {{
        let s = $s;
        if s.len() < 8 {
            return Err(YearWeekError::InputLen);
        }

        let (non_zero_year, year_len, s) = $expr;

        // skip the dash which was checked before this
        //
        // the length was already checked
        if s[0] != b'-' {
            return Err(YearWeekError::DashWExpected);
        }
        let s = s.split_at(1).1;

        // exactly two digits for the month is required.
        if s.len() < 2 {
            return Err(YearWeekError::WeekInvalid);
        }

        // do not go till the end of the text, only the next two digits matter
        let Some(week @ 1..=53) = num::u8::decode(s.split_at(2).0, 10) else {
            return Err(YearWeekError::WeekInvalid);
        };
        if week == 53 && !is_weeks_53(non_zero_year) {
            return Err(YearWeekError::WeekInvalid);
        }

        Ok((non_zero_year, week, year_len + 1 + 3))
    }};
}

/// Return the `yyyy-Www` [valid week string] where year in `1..` and week in `1..=53` at prefix.
///
/// [valid week string]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-week-string
pub const fn year_week_prefix(s: &[u8]) -> Result<(&[u8], u8, usize), YearWeekError> {
    year_week_prefix!(|s| {
        // end == len => year is 0
        let (year_zeros_end, year_len) = num::prefix_span(s, 10);
        if year_len < 4 || year_zeros_end == year_len {
            return Err(YearWeekError::YearInvalid);
        }
        // skip zeros, then save the year and the rest
        let (year, s) = s
            .split_at(year_zeros_end)
            .1
            .split_at(year_len - year_zeros_end);
        (year, year_len, s)
    })
}

macro_rules! year_week {
    ($s:expr) => {{
        let s = $s;
        match year_week_prefix(s) {
            Ok((v1, v2, len)) if len == s.len() => Ok((v1, v2)),
            Ok(_) => Err(YearWeekError::InputLen),
            Err(e) => Err(e),
        }
    }};
}

/// Return the `yyyy-Www` [valid week string] where year in `1..` and week in `1..=53`.
///
/// [valid week string]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-month-string
pub const fn year_week(s: &[u8]) -> Result<(&[u8], u8), YearWeekError> {
    year_week!(s)
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

/// Return the `yyyy-mm` [valid month string] where year in `1..` and month in `1..=12`.
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

    /// See [`super::year_month`].
    pub const fn year_month(s: &[u8]) -> Result<(u8, u8), YearMonthError> {
        year_month!(s)
    }

    /// See [`super::days_in_month`].
    pub const fn days_in_month(year: u8, month: u8) -> u8 {
        u16::days_in_month(year as u16 + 1970, month)
    }

    /// Is this year a leap year?
    pub const fn is_leap(year: u8) -> bool {
        u16::is_leap(year as u16 + 1970)
    }

    /// See [`super::ordinal`].
    pub const fn ordinal(year: u8, month: u8, day: u8) -> u16 {
        if let v @ 1.. = ordinal_non_leap(month, day) {
            v + ((month > 2 && is_leap(year)) as u16)
        } else {
            0
        }
    }

    /// See [`super::ordinal_last`].
    pub const fn ordinal_last(year: u8, month: u8) -> u16 {
        if let v @ 1.. = ordinal_last_non_leap(month) {
            v + ((month > 2 && is_leap(year)) as u16)
        } else {
            0
        }
    }

    /// Return the next leap year (0 means the end of the type is reached).
    pub const fn next_leap(year: u8) -> u16 {
        u16::next_leap(year as u16 + 1970)
    }

    /// Return the previous leap year (0 means the end of the type is reached).
    ///
    /// See [`next_leap`] for more.
    pub const fn previous_leap(year: u8) -> u16 {
        u16::previous_leap(year as u16 + 1970)
    }

    /// Get the leap years/days count before and including the given year (offset 1970).
    ///
    /// See [`u16::leaps_inclusive_to`] for more.
    pub const fn leaps_inclusive_to(year: u8) -> u16 {
        u16::leaps_inclusive_to(year as u16 + 1970)
    }

    /// See [`super::prefix`].
    pub const fn prefix(s: &[u8]) -> Result<(u8, u8, u8, usize), DateError> {
        prefix!(s)
    }

    /// See [`super::decode`].
    pub const fn decode(s: &[u8]) -> Result<(u8, u8, u8), DateError> {
        decode!(s)
    }

    /// Return whether the given year is a 53 week year (true) or a 52 week year (false).
    ///
    /// See [`u16::is_weeks_53`] for more.
    pub const fn is_weeks_53(year: u8) -> bool {
        u16::is_weeks_53(year as u16 + 1970)
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
            (year & 3 == 0 && year % 100 != 0) || year % 400 == 0
        }

        /// See [`super::ordinal`].
        pub const fn ordinal(year: $ty, month: u8, day: u8) -> u16 {
            if let v @ 1.. = ordinal_non_leap(month, day) {
                v + ((month > 2 && is_leap(year)) as u16)
            } else {
                0
            }
        }

        /// See [`super::ordinal_last`].
        pub const fn ordinal_last(year: $ty, month: u8) -> u16 {
            if let v @ 1.. = ordinal_last_non_leap(month) {
                v + ((month > 2 && is_leap(year)) as u16)
            } else {
                0
            }
        }

        /// Return the next leap year (0 means the end of the type is reached).
        ///
        /// See [`previous_leap`] for more.
        ///
        /// # Example
        ///
        /// ```rust
        #[doc = concat!("# use constrs::date::", stringify!($ty), "::next_leap;")]
        /// assert_eq!(next_leap(0),    4);
        /// assert_eq!(next_leap(4),    8);
        /// assert_eq!(next_leap(8),    12);
        /// assert_eq!(next_leap(96),   104);
        /// assert_eq!(next_leap(1596), 1600);
        #[doc = concat!(
            "assert_eq!(next_leap(", stringify!($ty), "::MAX - 8), ", stringify!($ty), "::MAX - 7);"
        )]
        #[doc = concat!(
            "assert_eq!(next_leap(", stringify!($ty), "::MAX - 7), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!(
            "assert_eq!(next_leap(", stringify!($ty), "::MAX - 6), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!(
            "assert_eq!(next_leap(", stringify!($ty), "::MAX - 5), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!(
            "assert_eq!(next_leap(", stringify!($ty), "::MAX - 4), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!("assert_eq!(next_leap(", stringify!($ty), "::MAX - 3), 0);")]
        #[doc = concat!("assert_eq!(next_leap(", stringify!($ty), "::MAX - 2), 0);")]
        #[doc = concat!("assert_eq!(next_leap(", stringify!($ty), "::MAX - 1), 0);")]
        #[doc = concat!("assert_eq!(next_leap(", stringify!($ty), "::MAX), 0);")]
        /// ```
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
            // v & !3 => v - (v % 4) => floor_multiple_of_4
            let candidate = (year + 4) & !3;

            // check the new year to be 100 and if so, year % 4 is not enough, it must be year % 400
            if candidate % 400 != 0 && candidate % 100 == 0 {
                // if year % 400 for century leap is not valid skip it
                return candidate + 4;
            }
            candidate
        }

        /// Return the previous leap year (0 means the end of the type is reached).
        ///
        /// See [`next_leap`] for more.
        ///
        /// # Example
        ///
        /// ```rust
        #[doc = concat!("# use constrs::date::", stringify!($ty), "::previous_leap;")]
        /// assert_eq!(previous_leap(0),    0);
        /// assert_eq!(previous_leap(4),    0);
        /// assert_eq!(previous_leap(8),    4);
        /// assert_eq!(previous_leap(96),   92);
        /// assert_eq!(previous_leap(100),  96);
        /// assert_eq!(previous_leap(104),  96);
        /// assert_eq!(previous_leap(1600), 1596);
        /// assert_eq!(previous_leap(1604), 1600);
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 6), ", stringify!($ty), "::MAX - 7);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 5), ", stringify!($ty), "::MAX - 7);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 4), ", stringify!($ty), "::MAX - 7);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 3), ", stringify!($ty), "::MAX - 7);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 2), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX - 1), ", stringify!($ty), "::MAX - 3);"
        )]
        #[doc = concat!(
            "assert_eq!(previous_leap(", stringify!($ty), "::MAX), ", stringify!($ty), "::MAX - 3);"
        )]
        /// ```
        pub const fn previous_leap(year: $ty) -> $ty {
            if year <= 3 {
                return 0;
            }
            let candidate = (year - 1) & !3;
            // check the new year to be 100 and if so, year % 4 is not enough, it must be year % 400
            if candidate % 400 != 0 && candidate % 100 == 0 {
                // if year % 400 for century leap is not valid skip it
                // this will not be the case for value 3..=8 so it's safe to have -4
                return candidate - 4;
            }
            candidate
        }

        /// Get the leap years/days count before and including the given year.
        ///
        /// ```rust
        #[doc = concat!("# use constrs::date::", stringify!($ty), "::leaps_inclusive_to;")]
        /// assert_eq!(leaps_inclusive_to(0), 0);
        /// assert_eq!(leaps_inclusive_to(3), 0);
        /// assert_eq!(leaps_inclusive_to(4), 1);
        /// assert_eq!(leaps_inclusive_to(5), 1);
        /// assert_eq!(leaps_inclusive_to(1970), 477);
        /// assert_eq!(leaps_inclusive_to(1972), 478);
        /// assert_eq!(leaps_inclusive_to(2225), 539);
        /// assert_eq!(leaps_inclusive_to(65535), 15891);
        /// ```
        pub const fn leaps_inclusive_to(year: $ty) -> $ty {
            (year / 4) - (year / 100) + (year / 400)
        }

        /// Return whether the given year is a 53 week year (true) or a 52 week year (false).
        ///
        /// The week description is taken from
        /// <https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#weeks>.
        ///
        /// Generally, counting from the 1-1-1, the number of `days % 7` yields 0 for Monday, 1 for
        /// Tuesday and so forth. If a year starts with Thursday (3) or starts with Wednesday (2) on
        /// leap years, it is a 53 weeks year.
        ///
        /// Since the calendar is priodic with cycles of 146097 days (400 year cycles) and `146097 %
        /// 7` is 0 implies that `year % 400` will have the same results as `year`. All in all,
        /// everything proves this, much like leap calculations, is only dependent on the last 4
        /// digits of a year number.
        ///
        /// This returns false for year 0 which is invalid.
        pub const fn is_weeks_53(year: $ty) -> bool {
            if year == 0 {
                return false;
            }

            let days = (year - 1) * 365 + leaps_inclusive_to(year - 1);
            let weekday = days % 7; // weekday of Jan 1st

            // is Thusday or (is Wednesday on a leap year)
            weekday == 3 || (weekday == 2 && is_leap(year))
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

    /// Return the timestamp of the date in seconds since "1970-01-01T00:00:00.0Z".
    ///
    /// The year must no have any prefixed zeros. Month must be in `1..=12` and the day of the month
    /// exist in that month according to the [`days_in_month`] limit.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::date::u32::timestamp_i64;
    /// assert_eq!(timestamp_i64(0000, 01, 01), None); // invalid year
    /// assert_eq!(timestamp_i64(1970, 00, 01), None); // invalid month
    /// assert_eq!(timestamp_i64(1970, 01, 00), None); // invalid day
    /// assert_eq!(timestamp_i64(1970, 01, 32), None); // invalid day
    /// assert_eq!(timestamp_i64(1970, 02, 29), None); // invalid day (leap aware)
    /// assert_eq!(timestamp_i64(0001, 01, 01), Some(-62135596800));
    /// assert_eq!(timestamp_i64(1951, 01, 01), Some(-599616000));
    /// assert_eq!(timestamp_i64(1951, 01, 02), Some(-599529600));
    /// assert_eq!(timestamp_i64(1951, 02, 01), Some(-596937600));
    /// assert_eq!(timestamp_i64(1951, 02, 02), Some(-596851200));
    /// assert_eq!(timestamp_i64(1951, 04, 16), Some(-590544000));
    /// assert_eq!(timestamp_i64(1951, 12, 01), Some(-570758400));
    /// assert_eq!(timestamp_i64(1951, 12, 30), Some(-568252800));
    /// assert_eq!(timestamp_i64(1951, 12, 31), Some(-568166400));
    /// assert_eq!(timestamp_i64(1958, 01, 01), Some(-378691200));
    /// assert_eq!(timestamp_i64(1958, 01, 02), Some(-378604800));
    /// assert_eq!(timestamp_i64(1958, 02, 01), Some(-376012800));
    /// assert_eq!(timestamp_i64(1958, 02, 02), Some(-375926400));
    /// assert_eq!(timestamp_i64(1958, 04, 16), Some(-369619200));
    /// assert_eq!(timestamp_i64(1958, 12, 01), Some(-349833600));
    /// assert_eq!(timestamp_i64(1958, 12, 30), Some(-347328000));
    /// assert_eq!(timestamp_i64(1958, 12, 31), Some(-347241600));
    /// assert_eq!(timestamp_i64(1968, 01, 01), Some(-63158400));
    /// assert_eq!(timestamp_i64(1968, 01, 02), Some(-63072000));
    /// assert_eq!(timestamp_i64(1968, 02, 01), Some(-60480000));
    /// assert_eq!(timestamp_i64(1968, 02, 02), Some(-60393600));
    /// assert_eq!(timestamp_i64(1968, 04, 16), Some(-54000000));
    /// assert_eq!(timestamp_i64(1968, 12, 01), Some(-34214400));
    /// assert_eq!(timestamp_i64(1968, 12, 30), Some(-31708800));
    /// assert_eq!(timestamp_i64(1968, 12, 31), Some(-31622400));
    /// assert_eq!(timestamp_i64(1969, 01, 01), Some(-31536000));
    /// assert_eq!(timestamp_i64(1969, 01, 02), Some(-31449600));
    /// assert_eq!(timestamp_i64(1969, 02, 01), Some(-28857600));
    /// assert_eq!(timestamp_i64(1969, 02, 02), Some(-28771200));
    /// assert_eq!(timestamp_i64(1969, 04, 16), Some(-22464000));
    /// assert_eq!(timestamp_i64(1969, 12, 01), Some(-2678400));
    /// assert_eq!(timestamp_i64(1969, 12, 30), Some(-172800));
    /// assert_eq!(timestamp_i64(1969, 12, 31), Some(-86400));
    /// assert_eq!(timestamp_i64(1970, 01, 01), Some(0));
    /// assert_eq!(timestamp_i64(1970, 01, 02), Some(86400));
    /// assert_eq!(timestamp_i64(1970, 02, 01), Some(2678400));
    /// assert_eq!(timestamp_i64(1970, 02, 02), Some(2764800));
    /// assert_eq!(timestamp_i64(1970, 04, 16), Some(9072000));
    /// assert_eq!(timestamp_i64(1970, 12, 01), Some(28857600));
    /// assert_eq!(timestamp_i64(1970, 12, 30), Some(31363200));
    /// assert_eq!(timestamp_i64(1970, 12, 31), Some(31449600));
    /// assert_eq!(timestamp_i64(1971, 01, 01), Some(31536000));
    /// assert_eq!(timestamp_i64(1971, 01, 02), Some(31622400));
    /// assert_eq!(timestamp_i64(1971, 02, 01), Some(34214400));
    /// assert_eq!(timestamp_i64(1971, 02, 02), Some(34300800));
    /// assert_eq!(timestamp_i64(1971, 04, 16), Some(40608000));
    /// assert_eq!(timestamp_i64(1971, 12, 01), Some(60393600));
    /// assert_eq!(timestamp_i64(1971, 12, 30), Some(62899200));
    /// assert_eq!(timestamp_i64(1971, 12, 31), Some(62985600));
    /// assert_eq!(timestamp_i64(2024, 01, 01), Some(1704067200));
    /// assert_eq!(timestamp_i64(2024, 01, 02), Some(1704153600));
    /// assert_eq!(timestamp_i64(2024, 02, 01), Some(1706745600));
    /// assert_eq!(timestamp_i64(2024, 02, 02), Some(1706832000));
    /// assert_eq!(timestamp_i64(2024, 04, 16), Some(1713225600));
    /// assert_eq!(timestamp_i64(2024, 12, 01), Some(1733011200));
    /// assert_eq!(timestamp_i64(2024, 12, 30), Some(1735516800));
    /// assert_eq!(timestamp_i64(2024, 12, 31), Some(1735603200));
    /// assert_eq!(timestamp_i64(2025, 01, 01), Some(1735689600));
    /// assert_eq!(timestamp_i64(2025, 01, 02), Some(1735776000));
    /// assert_eq!(timestamp_i64(2025, 02, 01), Some(1738368000));
    /// assert_eq!(timestamp_i64(2025, 02, 02), Some(1738454400));
    /// assert_eq!(timestamp_i64(2025, 04, 16), Some(1744761600));
    /// assert_eq!(timestamp_i64(2025, 12, 01), Some(1764547200));
    /// assert_eq!(timestamp_i64(2025, 12, 30), Some(1767052800));
    /// assert_eq!(timestamp_i64(2025, 12, 31), Some(1767139200));
    /// assert_eq!(timestamp_i64(u16::MAX as _, 12, 31), Some(2005949059200));
    /// assert_eq!(timestamp_i64(u32::MAX, 12, 31), Some(135536014634198400));
    /// ```
    pub const fn timestamp_i64(year: u32, month: u8, day: u8) -> Option<i64> {
        let days_in_month @ 1.. = days_in_month(year, month) else {
            return None;
        };
        if day < 1 || day > days_in_month {
            return None;
        }

        let non_leap_days = (year as i64 - 1970) * 365;
        let leap_days: i64 =
            leaps_inclusive_to(year - 1) as i64 - const { leaps_inclusive_to(1969) } as i64;
        let month_days = ordinal_last(year, month - 1) as i64;

        let days = non_leap_days + leap_days + month_days + (day as i64 - 1);
        Some(days * 86400)
    }

    /// Return the timestamp of the date in seconds since "1970-01-01T00:00:00.0Z".
    ///
    /// The year must no have any prefixed zeros. Month must be in `1..=12` and the day of the month
    /// exist in that month according to the [`days_in_month`] limit.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::date::u32::from_timestamp_i64;
    /// assert_eq!(from_timestamp_i64(-62135596801), None);
    /// assert_eq!(from_timestamp_i64(-62135596800), Some((0001, 01, 01)));
    /// assert_eq!(from_timestamp_i64(i32::MIN as i64), Some((1901, 12, 13)));
    /// assert_eq!(from_timestamp_i64(-599616000), Some((1951, 01, 01)));
    /// assert_eq!(from_timestamp_i64(-599529600), Some((1951, 01, 02)));
    /// assert_eq!(from_timestamp_i64(-596937600), Some((1951, 02, 01)));
    /// assert_eq!(from_timestamp_i64(-596851200), Some((1951, 02, 02)));
    /// assert_eq!(from_timestamp_i64(-590544000), Some((1951, 04, 16)));
    /// assert_eq!(from_timestamp_i64(-570758400), Some((1951, 12, 01)));
    /// assert_eq!(from_timestamp_i64(-568252800), Some((1951, 12, 30)));
    /// assert_eq!(from_timestamp_i64(-568166400), Some((1951, 12, 31)));
    /// assert_eq!(from_timestamp_i64(-378691200), Some((1958, 01, 01)));
    /// assert_eq!(from_timestamp_i64(-378604800), Some((1958, 01, 02)));
    /// assert_eq!(from_timestamp_i64(-376012800), Some((1958, 02, 01)));
    /// assert_eq!(from_timestamp_i64(-375926400), Some((1958, 02, 02)));
    /// assert_eq!(from_timestamp_i64(-369619200), Some((1958, 04, 16)));
    /// assert_eq!(from_timestamp_i64(-349833600), Some((1958, 12, 01)));
    /// assert_eq!(from_timestamp_i64(-347328000), Some((1958, 12, 30)));
    /// assert_eq!(from_timestamp_i64(-347241600), Some((1958, 12, 31)));
    /// assert_eq!(from_timestamp_i64(-63158400),  Some((1968, 01, 01)));
    /// assert_eq!(from_timestamp_i64(-63072000),  Some((1968, 01, 02)));
    /// assert_eq!(from_timestamp_i64(-60480000),  Some((1968, 02, 01)));
    /// assert_eq!(from_timestamp_i64(-60393600),  Some((1968, 02, 02)));
    /// assert_eq!(from_timestamp_i64(-54000000),  Some((1968, 04, 16)));
    /// assert_eq!(from_timestamp_i64(-34214400),  Some((1968, 12, 01)));
    /// assert_eq!(from_timestamp_i64(-31708800),  Some((1968, 12, 30)));
    /// assert_eq!(from_timestamp_i64(-31622400),  Some((1968, 12, 31)));
    /// assert_eq!(from_timestamp_i64(-31536000),  Some((1969, 01, 01)));
    /// assert_eq!(from_timestamp_i64(-31449600),  Some((1969, 01, 02)));
    /// assert_eq!(from_timestamp_i64(-28857600),  Some((1969, 02, 01)));
    /// assert_eq!(from_timestamp_i64(-28771200),  Some((1969, 02, 02)));
    /// assert_eq!(from_timestamp_i64(-22464000),  Some((1969, 04, 16)));
    /// assert_eq!(from_timestamp_i64(-2678400),   Some((1969, 12, 01)));
    /// assert_eq!(from_timestamp_i64(-172800),    Some((1969, 12, 30)));
    /// assert_eq!(from_timestamp_i64(-86400),     Some((1969, 12, 31)));
    /// assert_eq!(from_timestamp_i64(0),          Some((1970, 01, 01)));
    /// assert_eq!(from_timestamp_i64(86400),      Some((1970, 01, 02)));
    /// assert_eq!(from_timestamp_i64(2678400),    Some((1970, 02, 01)));
    /// assert_eq!(from_timestamp_i64(2764800),    Some((1970, 02, 02)));
    /// assert_eq!(from_timestamp_i64(9072000),    Some((1970, 04, 16)));
    /// assert_eq!(from_timestamp_i64(28857600),   Some((1970, 12, 01)));
    /// assert_eq!(from_timestamp_i64(31363200),   Some((1970, 12, 30)));
    /// assert_eq!(from_timestamp_i64(31449600),   Some((1970, 12, 31)));
    /// assert_eq!(from_timestamp_i64(31536000),   Some((1971, 01, 01)));
    /// assert_eq!(from_timestamp_i64(31622400),   Some((1971, 01, 02)));
    /// assert_eq!(from_timestamp_i64(34214400),   Some((1971, 02, 01)));
    /// assert_eq!(from_timestamp_i64(34300800),   Some((1971, 02, 02)));
    /// assert_eq!(from_timestamp_i64(40608000),   Some((1971, 04, 16)));
    /// assert_eq!(from_timestamp_i64(60393600),   Some((1971, 12, 01)));
    /// assert_eq!(from_timestamp_i64(62899200),   Some((1971, 12, 30)));
    /// assert_eq!(from_timestamp_i64(62985600),   Some((1971, 12, 31)));
    /// assert_eq!(from_timestamp_i64(1704067200), Some((2024, 01, 01)));
    /// assert_eq!(from_timestamp_i64(1704153600), Some((2024, 01, 02)));
    /// assert_eq!(from_timestamp_i64(1706745600), Some((2024, 02, 01)));
    /// assert_eq!(from_timestamp_i64(1706832000), Some((2024, 02, 02)));
    /// assert_eq!(from_timestamp_i64(1713225600), Some((2024, 04, 16)));
    /// assert_eq!(from_timestamp_i64(1733011200), Some((2024, 12, 01)));
    /// assert_eq!(from_timestamp_i64(1735516800), Some((2024, 12, 30)));
    /// assert_eq!(from_timestamp_i64(1735603200), Some((2024, 12, 31)));
    /// assert_eq!(from_timestamp_i64(1735689600), Some((2025, 01, 01)));
    /// assert_eq!(from_timestamp_i64(1735776000), Some((2025, 01, 02)));
    /// assert_eq!(from_timestamp_i64(1738368000), Some((2025, 02, 01)));
    /// assert_eq!(from_timestamp_i64(1738454400), Some((2025, 02, 02)));
    /// assert_eq!(from_timestamp_i64(1744761600), Some((2025, 04, 16)));
    /// assert_eq!(from_timestamp_i64(1764547200), Some((2025, 12, 01)));
    /// assert_eq!(from_timestamp_i64(1767052800), Some((2025, 12, 30)));
    /// assert_eq!(from_timestamp_i64(1767139200), Some((2025, 12, 31)));
    /// assert_eq!(from_timestamp_i64(i32::MAX as i64), Some((2038, 01, 19)));
    /// assert_eq!(from_timestamp_i64(u32::MAX as i64), Some((2106, 02, 07)));
    /// assert_eq!(from_timestamp_i64(135536014634198400), Some((u32::MAX, 12, 31)));
    /// assert_eq!(from_timestamp_i64(135536014634198401), None);
    /// ```
    // TODO move to their own module (i64)
    pub const fn from_timestamp_i64(timestamp: i64) -> Option<(u32, u8, u8)> {
        if timestamp < const { u32::timestamp_i64(1, 1, 1).unwrap() }
            || timestamp > const { u32::timestamp_i64(u32::MAX, 12, 31).unwrap() }
        {
            return None;
        }

        // calculate years and ordinal
        // 146097 is a full 400 year cycle (no GCD)
        let days = timestamp.div_euclid(86400); // guarantee positive results
        let cycles = days.div_euclid(146097); // a full 400 years cycle in days
        // (assuming u32 is unsigned) since the input is limited to the timestamp of
        // u32::MAX-12-31, this will not overflow nor miss information if casted to u32.
        // Although mathematically this can get as high as u32::MAX and as low as -30, the
        // naive might cast it to larger types.  However, this is proven to work even if casted
        // to signed of u32.
        //
        // If this is u32, the y += 1 in the loop will overflow for the minimum timestamp
        // (1-1-1). So either this must remain signed of u32 or the adding be wrapping add.
        let mut year = (1970 + (cycles * 400)) as u32;
        let mut days = days.rem_euclid(146097) as u32; // remove the cycles just added to year
        let mut leapness = u32::is_leap(year);
        while let Some(new_days) = days.checked_sub(365 + leapness as u32) {
            days = new_days;
            year = year.wrapping_add(1);
            leapness = u32::is_leap(year);
        }
        let ordinal = 1 + days as u16;

        // calculate the month and day
        let mut month = 12;
        let mut day = ordinal as u8; // if lower than 31 loop never runs so it's valid
        while month > 0 {
            let last_ordinal = ordinal_last_non_leap(month - 1) + ((month > 2 && leapness) as u16);

            // if the ordinal is larger than the end of the previous month, it's definitely there.
            if let Some(v) = ordinal.checked_sub(last_ordinal) {
                day = v as u8;
                break;
            }
            month -= 1;
        }

        Some((year, month, day))
    }

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
