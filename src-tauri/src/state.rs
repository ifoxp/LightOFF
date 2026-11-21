//! Shared app state, the in-app journal («Журнал») and the persisted runtime flags.

use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::ecoflow::{EcoFlow, Status};
use crate::schedule::Schedule;
use crate::settings::{path, write_atomic, Settings};

const JOURNAL_KEEP: usize = 300;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    /// `dd.mm HH:MM:SS`
    pub ts: String,
    /// «info» | «ok» | «warn» | «error»
    pub level: String,
    pub text: String,
}

/// Survives restarts: the automation switch and which actions already ran.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Runtime {
    pub automation: bool,
    /// Action key -> unix time it ran.
    pub done: BTreeMap<String, i64>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { automation: true, done: BTreeMap::new() }
    }
}

impl Runtime {
    fn load() -> Self {
        std::fs::read_to_string(path("state.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(text) = serde_json::to_string_pretty(self) {
            if let Err(e) = write_atomic(&path("state.json"), &text) {
                tracing::warn!(error = %e, "state.json not saved");
            }
        }
    }
}

/// A VOE page load in progress.
#[derive(Clone, Debug)]
pub struct VoeJob {
    pub started: Instant,
    pub visible: bool,
    /// Last state the page script reported («challenge», «manual», …).
    pub stage: String,
}

pub struct AppState {
    pub settings: RwLock<Settings>,
    pub eco: RwLock<Option<Arc<EcoFlow>>>,
    pub status: RwLock<Option<Status>>,
    pub status_error: RwLock<Option<String>>,
    pub schedule: RwLock<Schedule>,
    pub runtime: Mutex<Runtime>,
    pub journal: Mutex<VecDeque<Entry>>,
    pub voe: Mutex<Option<VoeJob>>,
    /// Unix time of the last background schedule refresh attempt.
    pub voe_last_try: Mutex<i64>,
    /// Retry bookkeeping for failed automation actions: key -> unix time of the last try.
    pub retry: Mutex<BTreeMap<String, i64>>,
}

impl AppState {
    pub fn load() -> Arc<Self> {
        let settings = Settings::load();
        let eco = settings.has_api().then(|| EcoFlow::new(&settings).ok().map(Arc::new)).flatten();
        Arc::new(Self {
            settings: RwLock::new(settings),
            eco: RwLock::new(eco),
            status: RwLock::new(None),
            status_error: RwLock::new(None),
            schedule: RwLock::new(Schedule::load()),
            runtime: Mutex::new(Runtime::load()),
            journal: Mutex::new(load_journal()),
            voe: Mutex::new(None),
            voe_last_try: Mutex::new(0),
            retry: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn eco(&self) -> anyhow::Result<Arc<EcoFlow>> {
        self.eco.read().clone().ok_or_else(|| anyhow::anyhow!("вкажіть ключі EcoFlow і SN у «Налаштуваннях»"))
    }

    pub fn apply_settings(&self, s: Settings) -> anyhow::Result<()> {
        s.save()?;
        let eco = if s.has_api() { Some(Arc::new(EcoFlow::new(&s)?)) } else { None };
        *self.eco.write() = eco;
        *self.settings.write() = s;
        Ok(())
    }

    pub fn log(&self, level: &str, text: impl Into<String>) {
        let text = text.into();
        match level {
            "error" => tracing::error!("{text}"),
            "warn" => tracing::warn!("{text}"),
            _ => tracing::info!("{text}"),
        }
        let e = Entry { ts: chrono::Local::now().format("%d.%m %H:%M:%S").to_string(), level: level.into(), text };
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path("journal.jsonl")) {
            if let Ok(line) = serde_json::to_string(&e) {
                let _ = writeln!(f, "{line}");
            }
        }
        let mut j = self.journal.lock();
        j.push_front(e);
        j.truncate(JOURNAL_KEEP);
    }
}

/// Last entries of `journal.jsonl`, newest first; the file is trimmed when it grows.
fn load_journal() -> VecDeque<Entry> {
    let file = path("journal.jsonl");
    let Ok(text) = std::fs::read_to_string(&file) else { return VecDeque::new() };
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() > JOURNAL_KEEP * 4 {
        let keep = lines[lines.len() - JOURNAL_KEEP..].join("\n") + "\n";
        let _ = write_atomic(&file, &keep);
    }
    lines.iter().rev().take(JOURNAL_KEEP).filter_map(|l| serde_json::from_str(l).ok()).collect()
}
