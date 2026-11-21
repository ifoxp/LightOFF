//! Turns the schedule into timed actions.
//!
//! Every outage window is split into blocks (2 h by default): VOE may announce 4–6 hours but
//! cut only one of the 2-hour parts, so each block start gets its own pair of switches —
//! «від станції» `lead` minutes before it and «від мережі» `back` minutes after it.

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;

use crate::ecoflow::Source;
use crate::schedule::{Cell, Day, CELLS};
use crate::settings::Settings;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Action {
    /// Stable id: the same schedule always yields the same keys.
    pub key: String,
    #[serde(serialize_with = "ser_time")]
    pub at: NaiveDateTime,
    pub to: Source,
    /// Start of the block this action belongs to.
    #[serde(serialize_with = "ser_time")]
    pub block: NaiveDateTime,
    /// Whole outage window, `HH:MM–HH:MM`.
    pub window: String,
}

fn ser_time<S: serde::Serializer>(t: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&t.format("%Y-%m-%dT%H:%M").to_string())
}

fn cell_time(date: NaiveDate, index: usize) -> NaiveDateTime {
    date.and_time(NaiveTime::MIN) + Duration::minutes(30 * index as i64)
}

/// Contiguous outage runs; runs continue across midnight when the next day follows directly.
pub fn windows(days: &[Day], include_maybe: bool) -> Vec<Window> {
    let mut days: Vec<(NaiveDate, &Day)> = days.iter().filter_map(|d| d.naive().map(|n| (n, d))).collect();
    days.sort_by_key(|(n, _)| *n);

    let mut out = Vec::new();
    let mut open: Option<NaiveDateTime> = None;
    let mut last_end: Option<NaiveDateTime> = None;
    for (date, day) in days {
        // A gap between days closes a run at the end of the earlier day.
        if let (Some(s), Some(e)) = (open, last_end) {
            if e != cell_time(date, 0) {
                out.push(Window { start: s, end: e });
                open = None;
            }
        }
        for (i, c) in day.cells.iter().enumerate().take(CELLS) {
            let off = *c == Cell::Off || (include_maybe && *c == Cell::Maybe);
            let t = cell_time(date, i);
            match (off, open) {
                (true, None) => open = Some(t),
                (false, Some(s)) => {
                    out.push(Window { start: s, end: t });
                    open = None;
                }
                _ => {}
            }
        }
        last_end = Some(cell_time(date, 0) + Duration::days(1));
    }
    if let (Some(s), Some(e)) = (open, last_end) {
        out.push(Window { start: s, end: e });
    }
    out
}

/// All actions for the schedule, sorted by time.
pub fn plan(days: &[Day], s: &Settings) -> Vec<Action> {
    let lead = Duration::minutes(s.lead_minutes as i64);
    let back = Duration::minutes(s.back_minutes as i64);
    let block = Duration::minutes(s.block_minutes.max(30) as i64);

    let mut out = Vec::new();
    for w in windows(days, s.include_maybe) {
        let label = format!("{}–{}", w.start.format("%H:%M"), w.end.format("%H:%M"));
        let mut b = w.start;
        while b < w.end {
            for (to, at) in [(Source::Battery, b - lead), (Source::Grid, b + back)] {
                out.push(Action {
                    key: format!("{}@{}", if to == Source::Battery { "bat" } else { "grid" }, at.format("%Y%m%d%H%M")),
                    at,
                    to,
                    block: b,
                    window: label.clone(),
                });
            }
            b += block;
        }
    }
    out.sort_by_key(|a| a.at);
    out.dedup_by(|a, b| a.key == b.key);
    out
}

/// Where the schedule says we are right now: (outage now, next change time).
pub fn state_at(days: &[Day], include_maybe: bool, now: NaiveDateTime) -> (bool, Option<NaiveDateTime>) {
    let ws = windows(days, include_maybe);
    if let Some(w) = ws.iter().find(|w| w.start <= now && now < w.end) {
        return (true, Some(w.end));
    }
    (false, ws.iter().map(|w| w.start).filter(|s| *s > now).min())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str, off: &[(usize, usize)]) -> Day {
        let mut cells = vec![Cell::On; CELLS];
        for &(from, to) in off {
            for c in cells.iter_mut().take(to).skip(from) {
                *c = Cell::Off;
            }
        }
        Day { date: date.into(), cells }
    }

    fn t(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn two_hour_outage() {
        // 10:00–12:00
        let days = vec![day("2026-11-04", &[(20, 24)])];
        let p = plan(&days, &Settings::default());
        assert_eq!(p.len(), 2);
        assert_eq!((p[0].at, p[0].to), (t("2026-11-04 09:55"), Source::Battery));
        assert_eq!((p[1].at, p[1].to), (t("2026-11-04 10:15"), Source::Grid));
        assert_eq!(p[0].window, "10:00–12:00");
    }

    #[test]
    fn six_hours_repeat_every_two() {
        // 14:00–20:00 -> blocks at 14, 16, 18
        let days = vec![day("2026-11-04", &[(28, 40)])];
        let p = plan(&days, &Settings::default());
        let times: Vec<_> = p.iter().map(|a| (a.at.format("%H:%M").to_string(), a.to)).collect();
        assert_eq!(
            times,
            vec![
                ("13:55".into(), Source::Battery),
                ("14:15".into(), Source::Grid),
                ("15:55".into(), Source::Battery),
                ("16:15".into(), Source::Grid),
                ("17:55".into(), Source::Battery),
                ("18:15".into(), Source::Grid),
            ]
        );
    }

    #[test]
    fn across_midnight_is_one_window() {
        // 22:00–02:00 over two days -> blocks at 22:00 and 00:00
        let days = vec![day("2026-11-04", &[(44, 48)]), day("2026-11-05", &[(0, 4)])];
        let w = windows(&days, true);
        assert_eq!(w, vec![Window { start: t("2026-11-04 22:00"), end: t("2026-11-05 02:00") }]);
        let p = plan(&days, &Settings::default());
        assert_eq!(p.len(), 4);
        assert_eq!(p[2].at, t("2026-11-04 23:55"));
    }

    #[test]
    fn maybe_cells_follow_the_setting() {
        let mut d = day("2026-11-04", &[]);
        d.cells[16] = Cell::Maybe;
        d.cells[17] = Cell::Maybe;
        assert_eq!(windows(std::slice::from_ref(&d), true).len(), 1);
        assert!(windows(&[d], false).is_empty());
    }

    #[test]
    fn half_hour_start() {
        // 07:30–09:00
        let days = vec![day("2026-11-04", &[(15, 18)])];
        let p = plan(&days, &Settings::default());
        assert_eq!(p[0].at, t("2026-11-04 07:25"));
        assert_eq!(p[1].at, t("2026-11-04 07:45"));
    }

    #[test]
    fn state_now() {
        let days = vec![day("2026-11-04", &[(20, 24)])];
        assert_eq!(state_at(&days, true, t("2026-11-04 09:00")), (false, Some(t("2026-11-04 10:00"))));
        assert_eq!(state_at(&days, true, t("2026-11-04 11:00")), (true, Some(t("2026-11-04 12:00"))));
        assert_eq!(state_at(&days, true, t("2026-11-04 13:00")), (false, None));
    }
}
