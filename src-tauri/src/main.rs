//! LightOFF: switches the EcoFlow STREAM between battery and grid around VOE outage blocks.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod automation;
mod autostart;
mod commands;
mod ecoflow;
mod notify;
mod planner;
mod schedule;
mod settings;
mod state;
mod tray;
mod voe;
mod window;

use std::sync::Arc;

use tauri::{Manager, WindowEvent};
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let dir = settings::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("lightoff")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&dir)
        .unwrap_or_else(|_| tracing_appender::rolling::daily(&dir, "lightoff.log"));
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).with_writer(writer).with_ansi(false).init();
    guard
}

fn main() {
    let _log_guard = init_logging();
    let start_hidden = std::env::args().any(|a| a == "--tray");
    let state = AppState::load();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), start_hidden, "start");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::show_main(app)))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::get_settings,
            commands::save_settings,
            commands::test_connection,
            commands::refresh_status,
            commands::set_source,
            commands::set_reserve,
            commands::set_ai_mode,
            commands::set_automation,
            commands::set_cells,
            commands::clear_day,
            commands::open_voe,
            commands::import_legacy,
            commands::autostart_status,
            commands::set_autostart,
            commands::test_notification,
            commands::open_data_dir,
            commands::quit_app,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            tray::build(&handle)?;
            if !start_hidden {
                window::show_main(&handle);
            }
            let st = handle.state::<Arc<AppState>>();
            st.log("info", "LightOFF запущено");
            automation::spawn(&handle);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // The main window hides to the tray; automation keeps running.
            WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Destroyed if window.label() == "voe" => {
                *window.app_handle().state::<Arc<AppState>>().voe.lock() = None;
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("tauri app")
        .run(|_app, event| {
            // Keep running in the tray when the last window is hidden or the VOE window closes.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
