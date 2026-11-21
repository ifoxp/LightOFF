//! Outage schedule: half-hour cells per day, stored in `schedule.json`.
//!
//! Filled from the VOE site (see `voe.rs`) or edited by hand in «Графік».

use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::settings::{path, write_atomic};

pub const CELLS: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cell {
    On,
    Off,
    Maybe,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Day {
    /// `YYYY-MM-DD`
    pub date: String,
    pub cells: Vec<Cell>,
}

impl Day {
    pub fn empty(date: NaiveDate) -> Self {
        Self { date: date.format("%Y-%m-%d").to_string(), cells: vec![Cell::On; CELLS] }
    }

    pub fn naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Schedule {
    pub days: Vec<Day>,
    /// Queue printed by VOE, e.g. «3.1».
    pub queue: Option<String>,
    /// «voe» | «manual»
    pub source: String,
    /// `dd.mm HH:MM` of the last change.
    pub updated_at: Option<String>,
}

impl Schedule {
    pub fn load() -> Self {
        let mut s: Schedule = std::fs::read_to_string(path("schedule.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        s.prune();
        s
    }

    pub fn save(&self) -> anyhow::Result<()> {
        write_atomic(&path("schedule.json"), &serde_json::to_string_pretty(self)?)
    }

    /// Keep yesterday onwards (yesterday matters for an outage crossing midnight), sorted.
    fn prune(&mut self) {
        let from = Local::now().date_naive() - Duration::days(1);
        self.days.retain(|d| d.naive().is_some_and(|n| n >= from) && d.cells.len() == CELLS);
        self.days.sort_by(|a, b| a.date.cmp(&b.date));
        self.days.dedup_by(|a, b| a.date == b.date);
    }

    fn touch(&mut self, source: &str) {
        self.source = source.to_string();
        self.updated_at = Some(Local::now().format("%d.%m %H:%M").to_string());
    }

    /// Today and tomorrow for the UI; missing days are shown as «світло є».
    pub fn view_days(&self) -> Vec<Day> {
        let today = Local::now().date_naive();
        (0..2)
            .map(|i| {
                let d = today + Duration::days(i);
                self.day(d).cloned().unwrap_or_else(|| Day::empty(d))
            })
            .collect()
    }

    pub fn day(&self, date: NaiveDate) -> Option<&Day> {
        let key = date.format("%Y-%m-%d").to_string();
        self.days.iter().find(|d| d.date == key)
    }

    pub fn set_cell(&mut self, date: NaiveDate, index: usize, cell: Cell) -> anyhow::Result<()> {
        anyhow::ensure!(index < CELLS, "немає такої половини години");
        let key = date.format("%Y-%m-%d").to_string();
        if !self.days.iter().any(|d| d.date == key) {
            self.days.push(Day::empty(date));
        }
        let day = self.days.iter_mut().find(|d| d.date == key).expect("inserted above");
        day.cells[index] = cell;
        self.touch("manual");
        self.prune();
        Ok(())
    }

    pub fn clear_day(&mut self, date: NaiveDate) {
        let key = date.format("%Y-%m-%d").to_string();
        self.days.retain(|d| d.date != key);
        self.touch("manual");
    }

    /// Replace the days VOE returned; other days stay as they are.
    pub fn merge_voe(&mut self, days: Vec<Day>, queue: Option<String>) {
        for day in days {
            self.days.retain(|d| d.date != day.date);
            self.days.push(day);
        }
        if queue.is_some() {
            self.queue = queue;
        }
        self.touch("voe");
        self.prune();
    }
}

/// Days from the page script payload: `{days:[{label:"04.11", cells:[["off","on"], …24]}], queue}`.
pub fn days_from_voe(payload: &Value, today: NaiveDate) -> anyhow::Result<(Vec<Day>, Option<String>)> {
    let mut out = Vec::new();
    let days = payload.get("days").and_then(Value::as_array).ok_or_else(|| anyhow::anyhow!("немає днів у відповіді"))?;
    for (i, d) in days.iter().enumerate() {
        let label = d.get("label").and_then(Value::as_str).unwrap_or_default();
        // Without a readable date assume consecutive days starting today.
        let date = date_from_label(label, today).unwrap_or(today + Duration::days(i as i64));
        let hours = d.get("cells").and_then(Value::as_array).cloned().unwrap_or_default();
        if hours.len() < 24 {
            continue;
        }
        let mut cells = Vec::with_capacity(CELLS);
        for h in hours.iter().take(24) {
            let halves = h.as_array().cloned().unwrap_or_default();
            for k in 0..2 {
                let s = halves.get(k).and_then(Value::as_str).unwrap_or("on");
                cells.push(match s {
                    "off" => Cell::Off,
                    "maybe" => Cell::Maybe,
                    _ => Cell::On,
                });
            }
        }
        out.push(Day { date: date.format("%Y-%m-%d").to_string(), cells });
    }
    anyhow::ensure!(!out.is_empty(), "графік порожній або не розпізнано таблицю");
    let queue = payload.get("queue").and_then(Value::as_str).map(str::to_string);
    Ok((out, queue))
}

/// «04.11», «Ср, 04.11» → the nearby calendar date (the site prints no year).
fn date_from_label(label: &str, today: NaiveDate) -> Option<NaiveDate> {
    let c: Vec<char> = label.chars().collect();
    for i in 0..c.len().saturating_sub(4) {
        let digit = |k: usize| c[i + k].to_digit(10);
        let (Some(d1), Some(d2), '.', Some(m1), Some(m2)) = (digit(0), digit(1), c[i + 2], digit(3), digit(4)) else {
            continue;
        };
        let (day, month) = (d1 * 10 + d2, m1 * 10 + m2);
        for year in [today.year() - 1, today.year(), today.year() + 1] {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                if (date - today).num_days().abs() <= 7 {
                    return Some(date);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn labels() {
        let t = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
        assert_eq!(date_from_label("04.11", t), Some(t));
        assert_eq!(date_from_label("Чт, 05.11", t), NaiveDate::from_ymd_opt(2026, 11, 5));
        let ny = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert_eq!(date_from_label("01.01", ny), NaiveDate::from_ymd_opt(2027, 1, 1));
        assert_eq!(date_from_label("сьогодні", t), None);
    }

    #[test]
    fn voe_payload() {
        let t = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
        let mut cells: Vec<Value> = (0..24).map(|_| json!(["on", "on"])).collect();
        cells[10] = json!(["off", "off"]);
        cells[11] = json!(["maybe", "off"]);
        let p = json!({"days":[{"label":"04.11","cells":cells}],"queue":"3.1"});
        let (days, q) = days_from_voe(&p, t).unwrap();
        assert_eq!(q.as_deref(), Some("3.1"));
        assert_eq!(days[0].date, "2026-11-04");
        assert_eq!(days[0].cells[20], Cell::Off);
        assert_eq!(days[0].cells[22], Cell::Maybe);
        assert_eq!(days[0].cells[23], Cell::Off);
        assert_eq!(days[0].cells[24], Cell::On);
    }
}
