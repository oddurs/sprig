//! Civil dates, hand-rolled.
//!
//! Sprig needs a calendar date, comparison and a difference in days. That is
//! small enough to own, and owning it keeps the core free of any crate that
//! could reach for the clock.

use std::fmt;

/// A day in the proleptic Gregorian calendar. Ordering is chronological.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// A date from its parts, or `None` if no such day exists.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Date> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// Reads the stored form, `YYYY-MM-DD` (§9.2), and nothing else.
    pub fn parse_iso(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let digits = |r: std::ops::Range<usize>| -> Option<u32> {
            let part = &s[r];
            if part.bytes().all(|c| c.is_ascii_digit()) {
                part.parse().ok()
            } else {
                None
            }
        };
        let year = digits(0..4)? as i32;
        let month = digits(5..7)? as u8;
        let day = digits(8..10)? as u8;
        Date::new(year, month, day)
    }

    pub fn year(self) -> i32 {
        self.year
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn day(self) -> u8 {
        self.day
    }

    /// Days from `self` to `other`: positive when `other` is later.
    pub fn days_until(self, other: Date) -> i64 {
        other.day_number() - self.day_number()
    }

    // Days since 1970-01-01, after Howard Hinnant's `days_from_civil`.
    fn day_number(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stored_form_round_trips() {
        let d = Date::parse_iso("2026-10-01").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (2026, 10, 1));
        assert_eq!(d.to_string(), "2026-10-01");
    }

    #[test]
    fn days_that_do_not_exist_are_not_dates() {
        for bad in [
            "2026-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-1-01",
            "26-01-01",
            "2026-01-01x",
            "soonish",
            "+2026-1-1",
        ] {
            assert_eq!(Date::parse_iso(bad), None, "{bad}");
        }
        assert!(Date::parse_iso("2024-02-29").is_some());
        assert!(Date::parse_iso("2000-02-29").is_some());
        assert!(Date::parse_iso("1900-02-29").is_none());
    }

    #[test]
    fn day_differences_cross_months_years_and_leap_days() {
        let d = |s| Date::parse_iso(s).unwrap();
        assert_eq!(d("2026-10-01").days_until(d("2026-10-07")), 6);
        assert_eq!(d("2026-10-07").days_until(d("2026-10-01")), -6);
        assert_eq!(d("2026-12-31").days_until(d("2027-01-01")), 1);
        assert_eq!(d("2024-02-28").days_until(d("2024-03-01")), 2);
        assert_eq!(d("1970-01-01").day_number(), 0);
        assert_eq!(d("2000-03-01").day_number(), 11_017);
    }

    #[test]
    fn dates_order_chronologically() {
        let d = |s| Date::parse_iso(s).unwrap();
        assert!(d("2026-09-30") < d("2026-10-01"));
        assert!(d("2025-12-31") < d("2026-01-01"));
    }
}
