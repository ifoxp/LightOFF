//! Schedule from the VOE site (disconnection.voe.com.ua).
//!
//! The site sits behind a Cloudflare check, so the app opens it in its own browser window
//! (WebView2) like a person would: the check is passed in that window when it asks, and the
//! page script (`voe.js`) searches the saved address and reads the table. Background refreshes
//! use a hidden window; when the site wants a check, the user is asked to open it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

use crate::notify;
use crate::schedule;
use crate::state::{AppState, VoeJob};

pub const VOE_URL: &str = "https://disconnection.voe.com.ua/";
const WINDOW: &str = "voe";
const REPORT_HOST: &str = "lightoff.invalid";
const PAGE_JS: &str = include_str!("voe.js");
/// A hidden refresh that has not finished by then is abandoned.
const HIDDEN_TIMEOUT: Duration = Duration::from_secs(120);

/// Open the VOE window (shown or hidden). A window that is already open is reused.
pub fn open(app: &AppHandle, visible: bool) -> anyhow::Result<()> {
    let st = app.state::<Arc<AppState>>().inner().clone();
    if let Some(w) = app.get_webview_window(WINDOW) {
        if visible {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.set_focus();
            if let Some(job) = st.voe.lock().as_mut() {
                job.visible = true;
            }
        }
        return Ok(());
    }

    let s = st.settings.read().clone();
    let cfg = json!({ "city": s.city, "street": s.street, "house": s.house, "auto": s.has_address() });
    let script = PAGE_JS.replace("__LIGHTOFF_CFG__", &cfg.to_string());
    let handle = app.clone();
    let url: Url = VOE_URL.parse()?;

    *st.voe.lock() = Some(VoeJob { started: Instant::now(), visible, stage: "loading".into() });
    let built = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::External(url))
        .title("LightOFF · графік ВОЕ")
        .inner_size(1120.0, 820.0)
        .center()
        .visible(visible)
        .initialization_script(&script)
        .on_navigation(move |url| {
            if url.host_str() == Some(REPORT_HOST) {
                let (h, u) = (handle.clone(), url.clone());
                tauri::async_runtime::spawn(async move { on_report(&h, &u) });
                return false;
            }
            true
        })
        .build();
    if let Err(e) = built {
        *st.voe.lock() = None;
        return Err(e.into());
    }

    if !visible {
        let h = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(HIDDEN_TIMEOUT).await;
            let st = h.state::<Arc<AppState>>().inner().clone();
            let job = st.voe.lock().clone();
            if let Some(job) = job.filter(|j| !j.visible && j.started.elapsed() >= HIDDEN_TIMEOUT) {
                let why = match job.stage.as_str() {
                    "challenge" => "сайт ВОЕ просить перевірку «я не робот»",
                    "loading" => "сайт ВОЕ не відкрився",
                    _ => "сайт ВОЕ не віддав графік",
                };
                close(&h);
                needs_user(&h, why);
            }
        });
    }
    Ok(())
}

pub fn close(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(WINDOW) {
        let _ = w.destroy();
    }
    *app.state::<Arc<AppState>>().voe.lock() = None;
}

pub fn is_open(app: &AppHandle) -> bool {
    app.get_webview_window(WINDOW).is_some()
}

/// Background refresh failed in a way only the user can fix.
fn needs_user(app: &AppHandle, why: &str) {
    let st = app.state::<Arc<AppState>>();
    st.log("warn", format!("Графік не оновився у фоні: {why}. Відкрийте «Графік» → «Оновити з сайту ВОЕ»."));
    notify::send(app, "Графік не оновився", &format!("{why}. Відкрийте LightOFF → «Графік» → «Оновити з сайту ВОЕ»."));
}

fn on_report(app: &AppHandle, url: &Url) {
    let st = app.state::<Arc<AppState>>().inner().clone();
    let kind = url.path().trim_matches('/').to_string();
    let payload: Value = url
        .fragment()
        .and_then(|f| urlencoding::decode(f).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);

    match kind.as_str() {
        "log" => {
            let text = payload.get("text").and_then(Value::as_str).unwrap_or_default();
            st.log("info", format!("ВОЕ: {text}"));
        }
        "status" => {
            let stage = payload.get("stage").and_then(Value::as_str).unwrap_or("?").to_string();
            let visible = {
                let mut job = st.voe.lock();
                if let Some(j) = job.as_mut() {
                    j.stage = stage.clone();
                }
                job.as_ref().is_some_and(|j| j.visible)
            };
            tracing::info!(stage, visible, "voe page");
            if stage == "manual" {
                let err = payload.get("error").and_then(Value::as_str).unwrap_or("");
                if visible {
                    if !err.is_empty() {
                        st.log("warn", format!("ВОЕ: автопошук не спрацював ({err}). Знайдіть адресу у вікні сайту вручну."));
                    }
                } else {
                    close(app);
                    let why = if err.is_empty() { "вкажіть адресу в «Налаштуваннях»".to_string() } else { format!("автопошук адреси: {err}") };
                    needs_user(app, &why);
                }
            }
        }
        "schedule" => {
            if let Some(classes) = payload.get("classes") {
                tracing::info!(%classes, how = ?payload.get("how"), "voe table classes");
            }
            match schedule::days_from_voe(&payload, chrono::Local::now().date_naive()) {
                Ok((days, queue)) => {
                    let n = days.len();
                    {
                        let mut s = st.schedule.write();
                        s.merge_voe(days, queue.clone());
                        if let Err(e) = s.save() {
                            st.log("error", format!("Графік не збережено: {e:#}"));
                        }
                    }
                    let q = queue.map(|q| format!(", черга {q}")).unwrap_or_default();
                    st.log("ok", format!("Графік оновлено з сайту ВОЕ: днів {n}{q}"));
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(1200)).await;
                        close(&h);
                    });
                }
                Err(e) => st.log("warn", format!("ВОЕ: {e:#}")),
            }
        }
        other => tracing::debug!(other, "unknown voe report"),
    }
}
