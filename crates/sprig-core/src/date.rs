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

    /// The date `n` days later (or earlier, for negative `n`).
    pub fn add_days(self, n: i64) -> Date {
        Date::from_day_number(self.day_number() + n)
    }

    /// 0 for Sunday through 6 for Saturday.
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday.
        (self.day_number() + 4).rem_euclid(7) as u8
    }

    /// The same day next month, or that month's last day if it is shorter.
    pub fn add_month(self) -> Date {
        let (year, month) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        Date {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    // After Howard Hinnant's `civil_from_days`.
    fn from_day_number(z: i64) -> Date {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
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

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];
const WEEKDAYS: [&str; 7] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
];

/// A month or weekday named by a prefix of at least three letters (§9.3).
fn named(v: &str, names: &[&str]) -> Option<usize> {
    if v.len() < 3 || !v.bytes().all(|b| b.is_ascii_lowercase()) {
        return None;
    }
    names.iter().position(|n| n.starts_with(v))
}

/// Reads a date field's value: the stored form, or a typed shortcut resolved
/// against `today` (§9.2, §9.3).
///
/// The shortcuts are the reference parser's, with one difference: a month and
/// day that do not exist, such as `feb30`, are not a date, where the reference
/// parser rolls them into the next month.
pub fn resolve(value: &str, today: Date) -> Option<Date> {
    if let Some(d) = Date::parse_iso(value) {
        return Some(d);
    }
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "today" => return Some(today),
        "tomorrow" => return Some(today.add_days(1)),
        _ => {}
    }
    if let Some(n) = v.strip_prefix('+') {
        let (count, days) = count_and_unit(n)?;
        return Some(today.add_days(count * days));
    }
    if let [b'q', q @ b'1'..=b'4'] = v.as_bytes() {
        let q = q - b'0';
        let last = |year| {
            let month = q * 3;
            Date::new(year, month, days_in_month(year, month))
                .expect("a quarter ends on a real day")
        };
        let d = last(today.year);
        return Some(if d < today { last(today.year + 1) } else { d });
    }
    if let Some((month, day)) = month_day(&v) {
        // A bare month and day more than four months back means next year.
        // Outside a leap year feb29 has no day this year, so measure from
        // the 28th: read in the autumn, it means the coming leap day.
        let this_year = Date::new(today.year, month, day);
        let past = this_year
            .or_else(|| Date::new(today.year, month, day.checked_sub(1)?))
            .is_some_and(|d| today.days_until(d) < -120);
        if past {
            return Date::new(today.year + 1, month, day);
        }
        return this_year;
    }
    let wd = named(&v, &WEEKDAYS)? as i64;
    Some(today.add_days((wd - i64::from(today.weekday())).rem_euclid(7)))
}

/// Whether a value reads as a date on some day: the stored form or a typed
/// shortcut. Used where no `today` is known, to warn about values that never
/// will (§9.5).
pub fn readable(value: &str) -> bool {
    // A leap year, so that feb29 reads.
    let anchor = Date {
        year: 2024,
        month: 1,
        day: 1,
    };
    resolve(value, anchor).is_some()
}

/// `3d` or `2w`: a count of up to five digits, and the days in its unit.
fn count_and_unit(v: &str) -> Option<(i64, i64)> {
    let (count, days) = match (v.strip_suffix('d'), v.strip_suffix('w')) {
        (Some(c), _) => (c, 1),
        (_, Some(c)) => (c, 7),
        _ => return None,
    };
    if count.is_empty() || count.len() > 5 || !count.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((count.parse().ok()?, days))
}

// `oct3`, `oct-3`, `3oct` and `3-oct`.
fn month_day(v: &str) -> Option<(u8, u8)> {
    let (name, day) = if v.starts_with(|c: char| c.is_ascii_digit()) {
        let end = v.find(|c: char| !c.is_ascii_digit())?;
        let (day, name) = v.split_at(end);
        (name.strip_prefix('-').unwrap_or(name), day)
    } else {
        let i = v.find(|c: char| c.is_ascii_digit())?;
        let (name, day) = v.split_at(i);
        (name.strip_suffix('-').unwrap_or(name), day)
    };
    if day.is_empty() || day.len() > 2 || !day.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let month = named(name, &MONTHS)? as u8 + 1;
    Some((month, day.parse().ok()?))
}

/// Whether an `every:` value names a recurrence (§10.3).
pub fn recurs(every: &str) -> bool {
    let anchor = Date {
        year: 2024,
        month: 1,
        day: 1,
    };
    next_occurrence(None, every, anchor).is_some()
}

/// The next due date of a recurring item ticked on `today` (§10.4): the next
/// occurrence after the later of its current due date and today. For a weekday
/// rule that is strictly after; an item with no due date counts from today.
pub fn next_occurrence(due: Option<Date>, every: &str, today: Date) -> Option<Date> {
    let base = match due {
        Some(d) if d >= today => d,
        _ => today,
    };
    let every = every.trim().to_ascii_lowercase();
    match every.as_str() {
        "day" | "daily" => return Some(base.add_days(1)),
        "week" | "weekly" => return Some(base.add_days(7)),
        "month" | "monthly" => return Some(base.add_month()),
        _ => {}
    }
    if let Some((n, days)) = count_and_unit(&every) {
        return (n > 0).then(|| base.add_days(n * days));
    }
    let wd = named(&every, &WEEKDAYS)? as i64;
    let ahead = (wd - i64::from(base.weekday())).rem_euclid(7);
    Some(base.add_days(if ahead == 0 { 7 } else { ahead }))
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

    fn d(s: &str) -> Date {
        Date::parse_iso(s).unwrap()
    }

    fn on(today: &str, value: &str) -> Option<String> {
        resolve(value, d(today)).map(|x| x.to_string())
    }

    #[test]
    fn days_and_weekdays_count_from_the_epoch() {
        assert_eq!(d("2026-10-01").weekday(), 4, "a Thursday");
        assert_eq!(d("1970-01-01").weekday(), 4);
        assert_eq!(d("2026-10-04").weekday(), 0, "a Sunday");
        for s in ["1969-12-31", "2000-02-29", "2026-12-31", "2400-03-01"] {
            assert_eq!(Date::from_day_number(d(s).day_number()), d(s));
        }
        assert_eq!(d("2026-12-31").add_days(1), d("2027-01-01"));
        assert_eq!(d("2026-01-31").add_month(), d("2026-02-28"));
        assert_eq!(d("2026-12-15").add_month(), d("2027-01-15"));
    }

    #[test]
    fn today_tomorrow_and_offsets_resolve_against_today() {
        for (today, tomorrow, plus3d, plus2w) in [
            ("2026-10-01", "2026-10-02", "2026-10-04", "2026-10-15"),
            ("2026-12-31", "2027-01-01", "2027-01-03", "2027-01-14"),
            ("2028-02-28", "2028-02-29", "2028-03-02", "2028-03-13"),
        ] {
            assert_eq!(on(today, "today").as_deref(), Some(today));
            assert_eq!(on(today, "Tomorrow").as_deref(), Some(tomorrow));
            assert_eq!(on(today, "+3d").as_deref(), Some(plus3d));
            assert_eq!(on(today, "+2w").as_deref(), Some(plus2w));
        }
    }

    #[test]
    fn quarters_end_on_their_last_day_and_roll_to_next_year_once_passed() {
        assert_eq!(on("2026-10-01", "q4").as_deref(), Some("2026-12-31"));
        assert_eq!(on("2026-10-01", "q3").as_deref(), Some("2027-09-30"));
        assert_eq!(on("2026-09-30", "q3").as_deref(), Some("2026-09-30"));
        assert_eq!(on("2026-01-15", "Q1").as_deref(), Some("2026-03-31"));
        assert_eq!(on("2026-01-15", "q5"), None);
    }

    #[test]
    fn month_days_read_four_ways_and_roll_over_after_120_days() {
        for v in ["oct3", "oct-3", "3oct", "3-oct", "October3", "3octob"] {
            assert_eq!(on("2026-10-01", v).as_deref(), Some("2026-10-03"), "{v}");
        }
        assert_eq!(
            on("2026-10-01", "jun3").as_deref(),
            Some("2026-06-03"),
            "120 days back is still this year"
        );
        assert_eq!(
            on("2026-10-01", "jun2").as_deref(),
            Some("2027-06-02"),
            "121 days back is next year"
        );
        assert_eq!(on("2026-10-01", "may31").as_deref(), Some("2027-05-31"));
        // Never last year: three weeks back means the December ahead.
        assert_eq!(on("2027-01-10", "dec20").as_deref(), Some("2027-12-20"));
        assert_eq!(on("2028-02-28", "feb29").as_deref(), Some("2028-02-29"));
        assert_eq!(on("2028-02-28", "29-feb").as_deref(), Some("2028-02-29"));
        assert_eq!(
            on("2027-10-01", "feb29").as_deref(),
            Some("2028-02-29"),
            "past this year's would-be leap day, so the coming one"
        );
        assert_eq!(on("2026-10-01", "feb29"), None, "2027 has no leap day");
        assert_eq!(on("2027-01-10", "feb29"), None, "not yet past, and no day");
        assert_eq!(on("2026-10-01", "oct0"), None);
        assert_eq!(
            on("2028-02-28", "jan1").as_deref(),
            Some("2028-01-01"),
            "58 days back"
        );
        assert_eq!(on("2026-10-01", "feb30"), None);
        assert_eq!(on("2026-10-01", "oc3"), None, "a month needs three letters");
        assert_eq!(on("2026-10-01", "oct123"), None);
    }

    #[test]
    fn weekdays_are_the_next_such_day_today_included() {
        // 2026-10-01 is a Thursday.
        assert_eq!(on("2026-10-01", "thu").as_deref(), Some("2026-10-01"));
        assert_eq!(on("2026-10-01", "fri").as_deref(), Some("2026-10-02"));
        assert_eq!(on("2026-10-01", "Wednesday").as_deref(), Some("2026-10-07"));
        assert_eq!(on("2026-10-04", "sun").as_deref(), Some("2026-10-04"));
        assert_eq!(on("2026-12-30", "mon").as_deref(), Some("2027-01-04"));
        assert_eq!(on("2026-10-01", "fr"), None);
    }

    #[test]
    fn anything_else_is_not_a_date() {
        for v in [
            "soonish",
            "",
            "+d",
            "+3m",
            "+3",
            "next week",
            "2026-1-5",
            "q",
            "35oct",
            "+3é",
            "é",
        ] {
            assert_eq!(on("2026-10-01", v), None, "{v}");
            assert!(!readable(v), "{v}");
        }
        assert!(readable("feb29"));
        assert!(readable("2026-10-01"));
    }

    #[test]
    fn a_recurring_item_moves_to_its_next_occurrence() {
        let next = |due: Option<&str>, every, today| {
            next_occurrence(due.map(d), every, d(today)).map(|x| x.to_string())
        };
        // Weekly numbers every Friday, ticked on Friday 2026-10-02.
        assert_eq!(
            next(Some("2026-10-02"), "fri", "2026-10-02").as_deref(),
            Some("2026-10-09")
        );
        // Ticked late: counts from today.
        assert_eq!(
            next(Some("2026-09-25"), "fri", "2026-10-01").as_deref(),
            Some("2026-10-02")
        );
        // Ticked early: counts from the due date.
        assert_eq!(
            next(Some("2026-10-09"), "week", "2026-10-01").as_deref(),
            Some("2026-10-16")
        );
        assert_eq!(
            next(None, "3d", "2026-10-01").as_deref(),
            Some("2026-10-04")
        );
        assert_eq!(
            next(None, "2w", "2026-10-01").as_deref(),
            Some("2026-10-15")
        );
        assert_eq!(
            next(None, "daily", "2026-10-01").as_deref(),
            Some("2026-10-02")
        );
        assert_eq!(
            next(Some("2026-01-31"), "monthly", "2026-01-31").as_deref(),
            Some("2026-02-28")
        );
        assert_eq!(next(None, "sometimes", "2026-10-01"), None);
        assert_eq!(next(None, "0d", "2026-10-01"), None);
    }

    #[test]
    fn dates_order_chronologically() {
        let d = |s| Date::parse_iso(s).unwrap();
        assert!(d("2026-09-30") < d("2026-10-01"));
        assert!(d("2025-12-31") < d("2026-01-01"));
    }
}
