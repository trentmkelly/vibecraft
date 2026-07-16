//! Calendar helpers matching Minecraft's `SpecialDates` utility.

#![allow(dead_code)]

use chrono::{Datelike, Local};

/// A month/day pair without a year, equivalent to Java's `MonthDay`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}

impl MonthDay {
    pub const fn of(month: u32, day: u32) -> Self {
        Self { month, day }
    }
}

/// The special calendar dates used by vanilla's server UI and events.
pub struct SpecialDates;

impl SpecialDates {
    pub const HALLOWEEN: MonthDay = MonthDay::of(10, 31);
    pub const CHRISTMAS_RANGE: [MonthDay; 3] = [
        MonthDay::of(12, 24),
        MonthDay::of(12, 25),
        MonthDay::of(12, 26),
    ];
    pub const CHRISTMAS: MonthDay = MonthDay::of(12, 24);
    pub const NEW_YEAR: MonthDay = MonthDay::of(1, 1);

    /// Returns the current month/day in the host's local timezone.
    pub fn day_now() -> MonthDay {
        let now = Local::now();
        MonthDay::of(now.month(), now.day())
    }

    pub fn is_halloween() -> bool {
        Self::HALLOWEEN == Self::day_now()
    }

    pub fn is_extended_christmas() -> bool {
        Self::CHRISTMAS_RANGE.contains(&Self::day_now())
    }
}

#[cfg(test)]
mod tests {
    use super::{MonthDay, SpecialDates};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn special_dates_match_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/SpecialDates.java");
        assert_eq!(JAVA.lines().count(), 27);
        for fragment in [
            "public static final MonthDay HALLOWEEN = MonthDay.of(Month.OCTOBER, 31)",
            "MonthDay.of(Month.DECEMBER, 24), MonthDay.of(Month.DECEMBER, 25), MonthDay.of(Month.DECEMBER, 26)",
            "public static final MonthDay CHRISTMAS = MonthDay.of(Month.DECEMBER, 24)",
            "public static final MonthDay NEW_YEAR = MonthDay.of(Month.JANUARY, 1)",
            "public static MonthDay dayNow()",
            "public static boolean isHalloween()",
            "public static boolean isExtendedChristmas()",
        ] {
            assert!(JAVA.contains(fragment), "missing SpecialDates source fragment: {fragment}");
        }
    }

    #[test]
    fn special_date_constants_and_membership_match_vanilla() {
        assert_eq!(SpecialDates::HALLOWEEN, MonthDay::of(10, 31));
        assert_eq!(
            SpecialDates::CHRISTMAS_RANGE,
            [MonthDay::of(12, 24), MonthDay::of(12, 25), MonthDay::of(12, 26)]
        );
        assert_eq!(SpecialDates::CHRISTMAS, MonthDay::of(12, 24));
        assert_eq!(SpecialDates::NEW_YEAR, MonthDay::of(1, 1));
        assert!(SpecialDates::CHRISTMAS_RANGE.contains(&MonthDay::of(12, 25)));
        assert!(!SpecialDates::CHRISTMAS_RANGE.contains(&MonthDay::of(12, 27)));
    }
}
