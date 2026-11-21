//! Commands for the UI. Errors go back as Ukrainian text.

use std::sync::Arc;

use chrono::{Local, NaiveDate};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::ecoflow::{Source, Status};
use crate::planner::{self, Action};
use crate::schedule::{Cell, Day};
use crate::settings::Settings;
use crate::state::{AppState, Entry};
use crate::{automation, autostart, notify, settings, tray, voe};

type Res<T> = Result<T, String>;
type St<'a> = State<'a, Arc<AppState>>;

fn err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

#[derive(Serialize)]
pub struct Snapshot {
    pub status: Option<Status>,
    pub status_error: Option<String>,
    pub days: Vec<Day>,
    pub queue: Option<String>,
    pub schedule_source: String,
    pub schedule_updated: Option<String>,
    pub plan: Vec<PlanItem>,
    pub automation: bool,
    /// Outage by the schedule right now.
    pub outage_now: bool,
    /// Next schedule change, `HH:MM` (with the date when it is not today).
    pub next_change: Option<String>,
    pub voe_open: bool,
    pub voe_stage: Option<String>,
    pub voe_visible: bool,
    pub has_api: bool,
    pub has_address: bool,
    pub journal: Vec<Entry>,
}

#[derive(Serialize)]
pub struct PlanItem {
    #[serde(flatten)]
    pub action: Action,
    pub done: bool,
    pub past: bool,
}

#[tauri::command]
pub async fn snapshot(app: AppHandle, state: St<'_>) -> Res<Snapshot> {
    let settings = state.settings.read().clone();
    let sched = state.schedule.read().clone();
    let now = Local::now().naive_local();
    let rt = state.runtime.lock().clone();
    let voe_job = state.voe.lock().clone();
    let from = now - chrono::Duration::hours(3);
    let to = now + chrono::Duration::hours(36);
    let plan = planner::plan(&sched.days, &settings)
        .into_iter()
        .filter(|a| a.at >= from && a.at <= to)
        .map(|a| PlanItem { done: rt.done.contains_key(&a.key), past: a.at <= now, action: a })
        .collect();
    let (outage_now, next) = planner::state_at(&sched.days, settings.include_maybe, now);
    let next_change = next.map(|t| {
        if t.date() == now.date() {
            t.format("%H:%M").to_string()
        } else {
            t.format("%d.%m %H:%M").to_string()
        }
    });
    Ok(Snapshot {
        status: state.status.read().clone(),
        status_error: state.status_error.read().clone(),
        days: sched.view_days(),
        queue: sched.queue.clone(),
        schedule_source: sched.source.clone(),
        schedule_updated: sched.updated_at.clone(),
        plan,
        automation: rt.automation,
        outage_now,
        next_change,
        voe_open: voe::is_open(&app),
        voe_stage: voe_job.as_ref().map(|j| j.stage.clone()),
        voe_visible: voe_job.as_ref().is_some_and(|j| j.visible),
        has_api: settings.has_api(),
        has_address: settings.has_address(),
        journal: state.journal.lock().iter().take(150).cloned().collect(),
    })
}

#[tauri::command]
pub async fn get_settings(state: St<'_>) -> Res<Settings> {
    Ok(state.settings.read().clone())
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, state: St<'_>, settings: Settings) -> Res<()> {
    state.apply_settings(settings.normalized()).map_err(err)?;
    state.log("info", "Налаштування збережено");
    tray::update(&app);
    let h = app.clone();
    tauri::async_runtime::spawn(async move { automation::refresh_status(&h).await });
    Ok(())
}

/// Read the STREAM with the given (unsaved) settings.
#[tauri::command]
pub async fn test_connection(settings: Settings) -> Res<String> {
    let s = settings.normalized();
    if !s.has_api() {
        return Err("заповніть Access Key, Secret Key і SN".into());
    }
    let eco = crate::ecoflow::EcoFlow::new(&s).map_err(err)?;
    let st = eco.status().await.map_err(err)?;
    let main = eco.main_sn().await.map_err(err)?;
    Ok(format!("Зв'язок є: головний пристрій {main}, заряд {:.0} %, резерв {} %", st.soc, st.reserve))
}

#[tauri::command]
pub async fn refresh_status(app: AppHandle) -> Res<()> {
    automation::refresh_status(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn set_source(app: AppHandle, to: Source) -> Res<i64> {
    automation::switch(&app, to, "Вручну").await.map_err(err)
}

#[tauri::command]
pub async fn set_reserve(app: AppHandle, state: St<'_>, value: i64) -> Res<i64> {
    let eco = state.eco().map_err(err)?;
    let st = eco.status().await.map_err(err)?;
    let lo = st.min_dsg + 3;
    let v = value.clamp(lo, st.max_chg.max(lo));
    if !st.self_powered {
        eco.set_self_powered().await.map_err(err)?;
    }
    eco.set_reserve(v).await.map_err(err)?;
    state.log("ok", format!("Вручну: резерв {v} %"));
    automation::refresh_status(&app).await;
    Ok(v)
}

#[tauri::command]
pub async fn set_ai_mode(app: AppHandle, state: St<'_>) -> Res<()> {
    state.eco().map_err(err)?.set_ai_mode().await.map_err(err)?;
    state.log("ok", "Вручну: увімкнено AI Mode EcoFlow");
    automation::refresh_status(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn set_automation(app: AppHandle, state: St<'_>, enabled: bool) -> Res<()> {
    {
        let mut rt = state.runtime.lock();
        rt.automation = enabled;
        rt.save();
    }
    tray::set_automation_check(&app, enabled);
    tray::update(&app);
    state.log("info", if enabled { "Автоматику увімкнено" } else { "Автоматику вимкнено" });
    Ok(())
}

fn parse_date(date: &str) -> Res<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| format!("дата «{date}» не розпізнана"))
}

#[tauri::command]
pub async fn set_cells(app: AppHandle, state: St<'_>, date: String, from: usize, to: usize, cell: Cell) -> Res<()> {
    let d = parse_date(&date)?;
    {
        let mut s = state.schedule.write();
        for i in from.min(to)..=from.max(to) {
            s.set_cell(d, i, cell).map_err(err)?;
        }
        s.save().map_err(err)?;
    }
    tray::update(&app);
    Ok(())
}

#[tauri::command]
pub async fn clear_day(app: AppHandle, state: St<'_>, date: String) -> Res<()> {
    let d = parse_date(&date)?;
    {
        let mut s = state.schedule.write();
        s.clear_day(d);
        s.save().map_err(err)?;
    }
    state.log("info", format!("Графік на {} очищено вручну", d.format("%d.%m")));
    tray::update(&app);
    Ok(())
}

#[tauri::command]
pub async fn open_voe(app: AppHandle, state: St<'_>) -> Res<()> {
    *state.voe_last_try.lock() = Local::now().timestamp();
    voe::open(&app, true).map_err(err)
}

#[tauri::command]
pub async fn import_legacy(app: AppHandle, state: St<'_>) -> Res<Option<Settings>> {
    let picked = app
        .dialog()
        .file()
        .set_title("config.json старого LightOFF")
        .add_filter("config.json", &["json"])
        .blocking_pick_file();
    let Some(file) = picked else { return Ok(None) };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let mut s = state.settings.read().clone();
    s.import_legacy(&path).map_err(err)?;
    Ok(Some(s))
}

#[tauri::command]
pub async fn autostart_status() -> Res<bool> {
    Ok(autostart::enabled())
}

#[tauri::command]
pub async fn set_autostart(state: St<'_>, enabled: bool) -> Res<()> {
    autostart::set(enabled).map_err(err)?;
    state.log("info", if enabled { "Автозапуск з Windows увімкнено" } else { "Автозапуск з Windows вимкнено" });
    Ok(())
}

#[tauri::command]
pub async fn test_notification(app: AppHandle) -> Res<()> {
    notify::force(&app, "LightOFF", "Так виглядатиме попередження перед відключенням.");
    Ok(())
}

#[tauri::command]
pub async fn open_data_dir() -> Res<()> {
    std::process::Command::new("explorer").arg(settings::data_dir()).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn quit_app(app: AppHandle) -> Res<()> {
    app.exit(0);
    Ok(())
}
