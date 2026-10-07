//! Pure business rules with zero I/O: completion, lifecycle, streaks, badges, merge.
//!
//! Only `shared` (error contract), `chrono`, and `thiserror` are allowed here.

use chrono::NaiveDate;
use std::str::FromStr;
use thiserror::Error;

// Error contract

/// Domain rule violation. Mapped to [`shared::AppError`] for transport.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

impl From<DomainError> for shared::AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::InvalidValue(msg) => shared::AppError::BadRequest(msg),
        }
    }
}

// Reading completion percentage

pub const COMPLETION_MIN: f32 = 0.0;
pub const COMPLETION_MAX: f32 = 100.0;

/// Validated reading completion percentage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage(f32);

impl Percentage {
    pub fn new(value: f32) -> Result<Self, DomainError> {
        if !value.is_finite() || !(COMPLETION_MIN..=COMPLETION_MAX).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "completion percentage {value} out of range 0.0-100.0"
            )));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f32 {
        self.0
    }

    /// A book counts as finished exactly at 100%.
    pub fn is_finished(self) -> bool {
        self.0 >= COMPLETION_MAX
    }

    /// Canonical two-decimal storage form for the `numeric` column.
    pub fn decimal_string(self) -> String {
        format!("{:.2}", self.0)
    }
}

/// Guest-to-cloud resolution: highest known progress wins.
pub fn merge_percentage(existing: Option<f32>, incoming: f32) -> f32 {
    existing.map(|prev| prev.max(incoming)).unwrap_or(incoming)
}

// Publication lifecycle

/// Legal book publication states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookStatus {
    Draft,
    Processing,
    Published,
    Archived,
}

impl FromStr for BookStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "draft" => Ok(Self::Draft),
            "processing" => Ok(Self::Processing),
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            other => Err(DomainError::InvalidValue(format!(
                "unknown book status '{other}'"
            ))),
        }
    }
}

impl BookStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Processing => "processing",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        use BookStatus::{Archived, Draft, Processing, Published};
        if self == next {
            return true;
        }
        matches!(
            (self, next),
            (Draft, Processing)
                | (Processing, Published)
                | (Draft, Archived)
                | (Processing, Archived)
                | (Published, Archived)
        )
    }
}

// Daily streaks and XP

/// A day counts toward the streak after this many reading seconds.
pub const DAILY_THRESHOLD_SECONDS: i64 = 300;
/// XP granted for every recorded heartbeat.
pub const BASE_HEARTBEAT_XP: i32 = 10;
/// Bonus XP for extending a streak to a consecutive day.
pub const STREAK_BONUS_XP: i32 = 50;
/// Bonus XP for the first counted day (new or restarted streak).
pub const DAILY_MILESTONE_BONUS_XP: i32 = 20;
/// Longest stretch of reading a single heartbeat may claim. The API admits
/// one heartbeat per minute, so this bounds XP inflation to 1.5x wall-clock
/// time even when a client reports the maximum.
pub const MAX_HEARTBEAT_CREDIT_SECONDS: i32 = 90;

/// Reading seconds actually credited for one reported heartbeat.
pub fn credited_heartbeat_seconds(reported: i32) -> i32 {
    reported.clamp(0, MAX_HEARTBEAT_CREDIT_SECONDS)
}

/// Outcome of applying one heartbeat day-evaluation to a streak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreakOutcome {
    pub current_days: i32,
    pub longest_days: i32,
    pub incremented: bool,
    /// Bonus XP only (caller adds [`BASE_HEARTBEAT_XP`]).
    pub bonus_xp: i32,
    pub last_active: Option<NaiveDate>,
}

/// Pure streak transition (no database): below threshold nothing changes,
/// otherwise count today once — consecutive day extends, gap restarts at 1.
pub fn advance_streak(
    current_days: i32,
    longest_days: i32,
    last_active: Option<NaiveDate>,
    today: NaiveDate,
    threshold_met: bool,
) -> StreakOutcome {
    if !threshold_met {
        return StreakOutcome {
            current_days,
            longest_days,
            incremented: false,
            bonus_xp: 0,
            last_active,
        };
    }

    let yesterday = today - chrono::Duration::days(1);
    let (current_days, incremented, bonus_xp) = match last_active {
        Some(date) if date == today => (current_days, false, 0),
        Some(date) if date == yesterday => (current_days + 1, true, STREAK_BONUS_XP),
        _ => (1, true, DAILY_MILESTONE_BONUS_XP),
    };

    StreakOutcome {
        current_days,
        longest_days: longest_days.max(current_days),
        incremented,
        bonus_xp,
        last_active: Some(today),
    }
}

/// Milestone badge eligibility matrix (ids must exist in `badges` seed data).
pub fn eligible_badges(streak_days: i32, total_seconds: i64) -> Vec<&'static str> {
    let mut badges = Vec::with_capacity(4);
    if total_seconds >= 60 {
        badges.push("first_step");
    }
    if streak_days >= 3 {
        badges.push("streak_3_days");
    }
    if streak_days >= 7 {
        badges.push("streak_7_days");
    }
    if total_seconds >= 3600 {
        badges.push("dedicated_reader");
    }
    badges
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    #[test]
    fn percentage_accepts_bounds() {
        assert_eq!(Percentage::new(0.0).unwrap().value(), 0.0);
        assert_eq!(Percentage::new(45.5).unwrap().value(), 45.5);
        assert_eq!(Percentage::new(100.0).unwrap().is_finished(), true);
        assert_eq!(Percentage::new(99.99).unwrap().is_finished(), false);
    }

    #[test]
    fn percentage_rejects_out_of_range_and_non_finite() {
        assert!(Percentage::new(-0.1).is_err());
        assert!(Percentage::new(100.01).is_err());
        assert!(Percentage::new(f32::NAN).is_err());
        assert!(Percentage::new(f32::INFINITY).is_err());
    }

    #[test]
    fn percentage_decimal_form_matches_storage() {
        assert_eq!(Percentage::new(45.5).unwrap().decimal_string(), "45.50");
        assert_eq!(Percentage::new(100.0).unwrap().decimal_string(), "100.00");
    }

    #[test]
    fn merge_prefers_highest_progress() {
        assert_eq!(merge_percentage(Some(75.0), 40.0), 75.0);
        assert_eq!(merge_percentage(Some(20.0), 80.0), 80.0);
        assert_eq!(merge_percentage(None, 10.0), 10.0);
    }

    #[test]
    fn book_status_round_trips() {
        for status in [
            BookStatus::Draft,
            BookStatus::Processing,
            BookStatus::Published,
            BookStatus::Archived,
        ] {
            assert_eq!(status.as_str().parse(), Ok(status));
        }
        assert!("deleted".parse::<BookStatus>().is_err());
    }

    #[test]
    fn book_status_transitions_follow_lifecycle() {
        use BookStatus::{Archived, Draft, Processing, Published};
        assert!(Draft.can_transition_to(Processing));
        assert!(Processing.can_transition_to(Published));
        assert!(Draft.can_transition_to(Archived));
        assert!(Published.can_transition_to(Archived));
        assert!(Published.can_transition_to(Published));
        assert!(!Draft.can_transition_to(Published));
        assert!(!Published.can_transition_to(Draft));
        assert!(!Archived.can_transition_to(Published));
    }

    #[test]
    fn streak_below_threshold_changes_nothing() {
        let today = day(2026, 9, 27);
        let outcome = advance_streak(5, 9, Some(today - chrono::Duration::days(1)), today, false);
        assert_eq!(
            outcome,
            StreakOutcome {
                current_days: 5,
                longest_days: 9,
                incremented: false,
                bonus_xp: 0,
                last_active: Some(today - chrono::Duration::days(1)),
            }
        );
    }

    #[test]
    fn streak_same_day_maintains_without_double_count() {
        let today = day(2026, 9, 27);
        let outcome = advance_streak(3, 5, Some(today), today, true);
        assert_eq!(outcome.current_days, 3);
        assert_eq!(outcome.incremented, false);
        assert_eq!(outcome.bonus_xp, 0);
    }

    #[test]
    fn streak_consecutive_day_increments_with_bonus() {
        let today = day(2026, 9, 27);
        let outcome = advance_streak(2, 5, Some(today - chrono::Duration::days(1)), today, true);
        assert_eq!(outcome.current_days, 3);
        assert_eq!(outcome.longest_days, 5);
        assert_eq!(outcome.incremented, true);
        assert_eq!(outcome.bonus_xp, STREAK_BONUS_XP);
        assert_eq!(outcome.last_active, Some(today));
    }

    #[test]
    fn streak_broken_restarts_at_day_one_and_extends_longest() {
        let today = day(2026, 9, 27);
        let outcome = advance_streak(4, 4, Some(today - chrono::Duration::days(5)), today, true);
        assert_eq!(outcome.current_days, 1);
        assert_eq!(outcome.longest_days, 4);
        assert_eq!(outcome.bonus_xp, DAILY_MILESTONE_BONUS_XP);
    }

    #[test]
    fn streak_first_day_counts_as_milestone() {
        let today = day(2026, 9, 27);
        let outcome = advance_streak(0, 0, None, today, true);
        assert_eq!(outcome.current_days, 1);
        assert_eq!(outcome.longest_days, 1);
        assert_eq!(outcome.incremented, true);
    }

    #[test]
    fn heartbeat_credit_is_capped_and_never_negative() {
        assert_eq!(credited_heartbeat_seconds(45), 45);
        assert_eq!(
            credited_heartbeat_seconds(3600),
            MAX_HEARTBEAT_CREDIT_SECONDS
        );
        assert_eq!(credited_heartbeat_seconds(-5), 0);
    }

    #[test]
    fn badge_matrix_matches_repository_rules() {
        assert_eq!(eligible_badges(0, 0), Vec::<&str>::new());
        assert_eq!(eligible_badges(0, 60), vec!["first_step"]);
        assert_eq!(eligible_badges(3, 61), vec!["first_step", "streak_3_days"]);
        assert_eq!(
            eligible_badges(7, 3600),
            vec![
                "first_step",
                "streak_3_days",
                "streak_7_days",
                "dedicated_reader"
            ]
        );
    }

    #[test]
    fn domain_error_maps_to_bad_request() {
        let err = Percentage::new(150.0).unwrap_err();
        let app_err: shared::AppError = err.into();
        assert!(matches!(app_err, shared::AppError::BadRequest(_)));
    }
}
