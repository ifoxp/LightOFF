//! User settings (`%APPDATA%\LightOFF\settings.json`) and the app data paths.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub const APP_DIR: &str = "LightOFF";

/// `%APPDATA%\LightOFF`, created on first use.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(APP_DIR);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn path(name: &str) -> PathBuf {
    data_dir().join(name)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // EcoFlow Developer API
    pub access_key: String,
    pub secret_key: String,
    pub sn: String,
    pub api_host: String,

    // Address on the VOE site
    pub city: String,
    pub street: String,
    pub house: String,

    // Automation
    /// Switch to the battery this many minutes before a block starts.
    pub lead_minutes: u32,
    /// Switch back to the grid this many minutes after a block starts.
    pub back_minutes: u32,
    /// Long outages are split into blocks of this length; each block repeats the switch.
    pub block_minutes: u32,
    /// Reserve for «від станції»; `None` = discharge limit + 3 %.
    pub battery_reserve: Option<u8>,
    /// Reserve for «від мережі»; `None` = hold the current charge.
    pub grid_reserve: Option<u8>,
    /// Treat «можливе відключення» hours as outages.
    pub include_maybe: bool,

    // Behaviour
    pub notifications: bool,
    /// Background schedule refresh from the VOE site, minutes; 0 = off.
    pub schedule_refresh_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            access_key: String::new(),
            secret_key: String::new(),
            sn: String::new(),
            api_host: "https://api-e.ecoflow.com".into(),
            city: String::new(),
            street: String::new(),
            house: String::new(),
            lead_minutes: 5,
            back_minutes: 15,
            block_minutes: 120,
            battery_reserve: None,
            grid_reserve: None,
            include_maybe: true,
            notifications: true,
            schedule_refresh_minutes: 60,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(path("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        write_atomic(&path("settings.json"), &text)
    }

    pub fn has_api(&self) -> bool {
        !self.access_key.trim().is_empty() && !self.secret_key.trim().is_empty() && !self.sn.trim().is_empty()
    }

    pub fn has_address(&self) -> bool {
        !self.city.trim().is_empty() && !self.street.trim().is_empty() && !self.house.trim().is_empty()
    }

    /// Clamp values the UI could send out of range.
    pub fn normalized(mut self) -> Self {
        self.lead_minutes = self.lead_minutes.clamp(1, 60);
        self.back_minutes = self.back_minutes.clamp(1, 120);
        self.block_minutes = self.block_minutes.clamp(30, 720);
        self.battery_reserve = self.battery_reserve.map(|v| v.min(100));
        self.grid_reserve = self.grid_reserve.map(|v| v.min(100));
        if self.api_host.trim().is_empty() {
            self.api_host = Settings::default().api_host;
        }
        self.api_host = self.api_host.trim().trim_end_matches('/').to_string();
        for s in [&mut self.access_key, &mut self.secret_key, &mut self.sn] {
            *s = s.trim().to_string();
        }
        self
    }

    /// Fields from the old Python LightOFF `config.json` (keys, SN, address).
    pub fn import_legacy(&mut self, file: &Path) -> anyhow::Result<()> {
        let text = std::fs::read_to_string(file).with_context(|| format!("не вдалося прочитати {}", file.display()))?;
        let v: serde_json::Value = serde_json::from_str(&text).context("файл не схожий на config.json LightOFF")?;
        let get = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.trim().to_string());
        let mut found = false;
        for (key, slot) in [
            ("access_key", &mut self.access_key),
            ("secret_key", &mut self.secret_key),
            ("sn", &mut self.sn),
            ("city", &mut self.city),
            ("street", &mut self.street),
            ("house", &mut self.house),
        ] {
            if let Some(val) = get(key).filter(|s| !s.is_empty()) {
                *slot = val;
                found = true;
            }
        }
        anyhow::ensure!(found, "у файлі немає ключів access_key / secret_key / sn");
        Ok(())
    }
}

/// Write via a temp file so a crash never leaves a half-written JSON.
pub fn write_atomic(target: &Path, text: &str) -> anyhow::Result<()> {
    let tmp = target.with_extension("tmp");
    std::fs::write(&tmp, text).with_context(|| format!("запис {}", tmp.display()))?;
    std::fs::rename(&tmp, target).with_context(|| format!("запис {}", target.display()))?;
    Ok(())
}
