//! Background loops: STREAM status polling, the planned switches and schedule refreshes.

use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use tauri::{AppHandle, Manager};

use crate::ecoflow::Source;
use crate::state::AppState;
use crate::{notify, planner, tray, voe};

/// An action is still run this long after its time (sleep, restart, network hiccup).
const GRACE_MIN: i64 = 10;
/// A failed action is retried at most this often.
const RETRY_SECS: i64 = 60;

pub fn spawn(app: &AppHandle) {
    let h = app.clone();
    tauri::async_runtime::spawn(async move { status_loop(h).await });
    let h = app.clone();
    tauri::async_runtime::spawn(async move { action_loop(h).await });
    let h = app.clone();
    tauri::async_runtime::spawn(async move { schedule_loop(h).await });
}

fn state(app: &AppHandle) -> Arc<AppState> {
    app.state::<Arc<AppState>>().inner().clone()
}

/// Read the STREAM now and publish the result to the UI and the tray.
pub async fn refresh_status(app: &AppHandle) {
    let st = state(app);
    let Ok(eco) = st.eco() else {
        *st.status_error.write() = Some("вкажіть ключі EcoFlow і SN у «Налаштуваннях»".into());
        tray::update(app);
        return;
    };
    match eco.status().await {
        Ok(s) => {
            *st.status.write() = Some(s);
            *st.status_error.write() = None;
        }
        Err(e) => {
            tracing::warn!(error = %format!("{e:#}"), "status");
            *st.status_error.write() = Some(format!("{e:#}"));
        }
    }
    tray::update(app);
}

async fn status_loop(app: AppHandle) {
    loop {
        refresh_status(&app).await;
        let visible = app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
        tokio::time::sleep(Duration::from_secs(if visible { 20 } else { 60 })).await;
    }
}

/// Switch the source now (buttons, tray, automation). Logs the outcome.
pub async fn switch(app: &AppHandle, to: Source, why: &str) -> anyhow::Result<i64> {
    let st = state(app);
    let eco = st.eco()?;
    let s = st.settings.read().clone();
    match eco.switch(to, &s).await {
        Ok(reserve) => {
            st.log("ok", format!("{why}: {} (резерв {reserve} %)", to.label()));
            let h = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(4)).await;
                refresh_status(&h).await;
            });
            Ok(reserve)
        }
        Err(e) => {
            st.log("error", format!("{why}: не вдалося перемкнути {} — {e:#}", to.label()));
            Err(e)
        }
    }
}

async fn action_loop(app: AppHandle) {
    tokio::time::sleep(Duration::from_secs(5)).await;
    loop {
        run_due(&app).await;
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}

async fn run_due(app: &AppHandle) {
    let st = state(app);
    if !st.runtime.lock().automation {
        return;
    }
    let settings = st.settings.read().clone();
    let actions = planner::plan(&st.schedule.read().days, &settings);
    let now = Local::now().naive_local();
    let now_ts = Local::now().timestamp();

    // Forget runs older than two days.
    {
        let mut rt = st.runtime.lock();
        let before = rt.done.len();
        rt.done.retain(|_, t| now_ts - *t < 2 * 86_400);
        if rt.done.len() != before {
            rt.save();
        }
    }

    for a in actions {
        let late = (now - a.at).num_minutes();
        if now < a.at || late > GRACE_MIN {
            continue;
        }
        if st.runtime.lock().done.contains_key(&a.key) {
            continue;
        }
        if let Some(last) = st.retry.lock().get(&a.key) {
            if now_ts - last < RETRY_SECS {
                continue;
            }
        }
        st.retry.lock().insert(a.key.clone(), now_ts);

        let why = match a.to {
            Source::Battery => format!("Автоматика, відключення о {}", hm(a.block)),
            Source::Grid => format!("Автоматика, {} хв після {}", settings.back_minutes, hm(a.block)),
        };
        if a.to == Source::Battery {
            notify::send(
                app,
                "Скоро можливе відключення",
                &format!("О {} за графіком ({}). Перемикаю дім на станцію.", hm(a.block), a.window),
            );
        }
        if switch(app, a.to, &why).await.is_ok() {
            let mut rt = st.runtime.lock();
            rt.done.insert(a.key.clone(), now_ts);
            rt.save();
            st.retry.lock().remove(&a.key);
        }
    }
}

fn hm(t: NaiveDateTime) -> String {
    t.format("%H:%M").to_string()
}

async fn schedule_loop(app: AppHandle) {
    tokio::time::sleep(Duration::from_secs(30)).await;
    loop {
        let st = state(&app);
        let (minutes, ready) = {
            let s = st.settings.read();
            (s.schedule_refresh_minutes, s.has_address())
        };
        let now = Local::now().timestamp();
        let due = {
            let last = *st.voe_last_try.lock();
            minutes > 0 && ready && now - last >= minutes as i64 * 60
        };
        if due && !voe::is_open(&app) {
            *st.voe_last_try.lock() = now;
            tracing::info!("background schedule refresh");
            if let Err(e) = voe::open(&app, false) {
                st.log("warn", format!("Не вдалося відкрити сайт ВОЕ: {e:#}"));
            }
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
