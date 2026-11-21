//! Windows notifications (respect the «Сповіщення» setting).

use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::state::AppState;

pub fn send(app: &AppHandle, title: &str, body: &str) {
    if !app.state::<Arc<AppState>>().settings.read().notifications {
        return;
    }
    force(app, title, body);
}

/// Shown even when notifications are off (the «Перевірити сповіщення» button).
pub fn force(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        tracing::warn!(error = %e, "notification failed");
    }
}
