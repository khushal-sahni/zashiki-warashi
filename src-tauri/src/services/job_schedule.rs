use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike};

use crate::domain::{JobSchedule, RunTrigger};
use crate::error::AppError;

/// A fire later than this after its slot is labelled a catch-up run.
pub const CATCH_UP_SLACK_SECONDS: i64 = 120;
const SEARCH_WINDOW_MINUTES: i64 = 8 * 24 * 60;

/// One launchd `StartCalendarInterval` dict. `None` means "every".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarSlot {
    pub weekday: Option<u8>,
    pub hour: Option<u8>,
    pub minute: u8,
}

pub fn validate(schedule: &JobSchedule) -> Result<(), AppError> {
    match schedule {
        JobSchedule::Interval { minutes } => validate_interval(*minutes),
        JobSchedule::Daily { hour, minute } => validate_time(*hour, *minute),
        JobSchedule::Weekly { weekdays, hour, minute } => {
            if weekdays.is_empty() || weekdays.iter().any(|day| *day > 6) {
                return Err(AppError::invalid("weekly schedules need weekdays between 0 (Sun) and 6 (Sat)"));
            }
            validate_time(*hour, *minute)
        }
    }
}

fn validate_interval(minutes: u32) -> Result<(), AppError> {
    let fits_hour = minutes >= 1 && minutes < 60 && 60 % minutes == 0;
    let fits_day = minutes >= 60 && minutes % 60 == 0 && 24 % (minutes / 60) == 0;
    if fits_hour || fits_day {
        return Ok(());
    }
    Err(AppError::invalid(
        "interval must divide an hour (5, 10, 15, 20, 30 min) or a day (1, 2, 3, 4, 6, 8, 12, 24 h)",
    ))
}

fn validate_time(hour: u8, minute: u8) -> Result<(), AppError> {
    if hour > 23 || minute > 59 {
        return Err(AppError::invalid("time must be between 00:00 and 23:59"));
    }
    Ok(())
}

pub fn calendar_slots(schedule: &JobSchedule) -> Vec<CalendarSlot> {
    match schedule {
        JobSchedule::Interval { minutes } if *minutes < 60 => (0..60)
            .step_by(*minutes as usize)
            .map(|minute| CalendarSlot { weekday: None, hour: None, minute: minute as u8 })
            .collect(),
        JobSchedule::Interval { minutes } => (0..24)
            .step_by((*minutes / 60) as usize)
            .map(|hour| CalendarSlot { weekday: None, hour: Some(hour as u8), minute: 0 })
            .collect(),
        JobSchedule::Daily { hour, minute } => {
            vec![CalendarSlot { weekday: None, hour: Some(*hour), minute: *minute }]
        }
        JobSchedule::Weekly { weekdays, hour, minute } => weekdays
            .iter()
            .map(|day| CalendarSlot { weekday: Some(*day), hour: Some(*hour), minute: *minute })
            .collect(),
    }
}

fn slot_matches<Tz: TimeZone>(slot: &CalendarSlot, at: &DateTime<Tz>) -> bool {
    let weekday = at.weekday().num_days_from_sunday() as u8;
    slot.minute as u32 == at.minute()
        && slot.hour.map_or(true, |hour| hour as u32 == at.hour())
        && slot.weekday.map_or(true, |day| day == weekday)
}

fn matches_any<Tz: TimeZone>(slots: &[CalendarSlot], at: &DateTime<Tz>) -> bool {
    slots.iter().any(|slot| slot_matches(slot, at))
}

fn floor_minute<Tz: TimeZone>(at: &DateTime<Tz>) -> DateTime<Tz> {
    at.clone() - Duration::seconds(at.second() as i64) - Duration::nanoseconds(at.nanosecond() as i64)
}

/// First slot strictly after `after`.
pub fn next_fire<Tz: TimeZone>(schedule: &JobSchedule, after: &DateTime<Tz>) -> Option<DateTime<Tz>> {
    let slots = calendar_slots(schedule);
    let mut cursor = floor_minute(after) + Duration::minutes(1);
    for _ in 0..SEARCH_WINDOW_MINUTES {
        if matches_any(&slots, &cursor) {
            return Some(cursor);
        }
        cursor += Duration::minutes(1);
    }
    None
}

/// Latest slot at or before `at`.
pub fn previous_fire<Tz: TimeZone>(schedule: &JobSchedule, at: &DateTime<Tz>) -> Option<DateTime<Tz>> {
    let slots = calendar_slots(schedule);
    let mut cursor = floor_minute(at);
    for _ in 0..SEARCH_WINDOW_MINUTES {
        if matches_any(&slots, &cursor) {
            return Some(cursor);
        }
        cursor -= Duration::minutes(1);
    }
    None
}

/// Label a launchd-started run and report which slot it belongs to.
pub fn classify_scheduled<Tz: TimeZone>(
    schedule: &JobSchedule,
    now: &DateTime<Tz>,
) -> (RunTrigger, Option<i64>) {
    let Some(slot) = previous_fire(schedule, now) else {
        return (RunTrigger::CatchUp, None);
    };
    let late_by = now.timestamp() - slot.timestamp();
    let trigger = if late_by <= CATCH_UP_SLACK_SECONDS {
        RunTrigger::Calendar
    } else {
        RunTrigger::CatchUp
    };
    (trigger, Some(slot.timestamp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(5 * 3600 + 1800)
            .expect("offset")
            .with_ymd_and_hms(y, mo, d, h, mi, s)
            .single()
            .expect("time")
    }

    #[test]
    fn rejects_intervals_that_do_not_tile() {
        assert!(validate(&JobSchedule::Interval { minutes: 15 }).is_ok());
        assert!(validate(&JobSchedule::Interval { minutes: 120 }).is_ok());
        assert!(validate(&JobSchedule::Interval { minutes: 7 }).is_err());
        assert!(validate(&JobSchedule::Interval { minutes: 300 }).is_err());
        assert!(validate(&JobSchedule::Interval { minutes: 0 }).is_err());
    }

    #[test]
    fn rejects_bad_weekly() {
        let empty = JobSchedule::Weekly { weekdays: vec![], hour: 1, minute: 0 };
        let bad_day = JobSchedule::Weekly { weekdays: vec![7], hour: 1, minute: 0 };
        assert!(validate(&empty).is_err());
        assert!(validate(&bad_day).is_err());
    }

    #[test]
    fn interval_expands_to_launchd_slots() {
        let quarter = calendar_slots(&JobSchedule::Interval { minutes: 15 });
        assert_eq!(quarter.iter().map(|s| s.minute).collect::<Vec<_>>(), [0, 15, 30, 45]);
        let six_hours = calendar_slots(&JobSchedule::Interval { minutes: 360 });
        assert_eq!(six_hours.iter().map(|s| s.hour).collect::<Vec<_>>(), [Some(0), Some(6), Some(12), Some(18)]);
    }

    #[test]
    fn next_daily_fire_rolls_to_tomorrow() {
        let schedule = JobSchedule::Daily { hour: 3, minute: 0 };
        let next = next_fire(&schedule, &at(2026, 10, 7, 3, 0, 0)).expect("next");
        assert_eq!(next, at(2026, 10, 8, 3, 0, 0));
        let same_day = next_fire(&schedule, &at(2026, 10, 7, 2, 59, 30)).expect("next");
        assert_eq!(same_day, at(2026, 10, 7, 3, 0, 0));
    }

    #[test]
    fn next_weekly_fire_skips_to_listed_day() {
        // 2026-10-07 is a Wednesday (3). Next Monday (1) is 2026-10-12.
        let schedule = JobSchedule::Weekly { weekdays: vec![1], hour: 9, minute: 30 };
        let next = next_fire(&schedule, &at(2026, 10, 7, 10, 0, 0)).expect("next");
        assert_eq!(next, at(2026, 10, 12, 9, 30, 0));
    }

    #[test]
    fn classifies_on_time_and_catch_up() {
        let schedule = JobSchedule::Daily { hour: 3, minute: 0 };
        let (on_time, slot) = classify_scheduled(&schedule, &at(2026, 10, 7, 3, 1, 0));
        assert_eq!(on_time, RunTrigger::Calendar);
        assert_eq!(slot, Some(at(2026, 10, 7, 3, 0, 0).timestamp()));

        let (late, _) = classify_scheduled(&schedule, &at(2026, 10, 7, 8, 15, 0));
        assert_eq!(late, RunTrigger::CatchUp);
    }
}
