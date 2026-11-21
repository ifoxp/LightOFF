//! Tray icon: tooltip with the charge and the next switch, quick actions in the menu.

use std::sync::Arc;

use chrono::Local;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::ecoflow::Source;
use crate::state::AppState;
use crate::{automation, planner, voe, window};

const TRAY_ID: &str = "lightoff";

/// Kept to flip the check mark when automation changes elsewhere.
pub struct TrayItems {
    pub automation: CheckMenuItem<Wry>,
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let st = app.state::<Arc<AppState>>().inner().clone();
    let open = MenuItem::with_id(app, "open", "Відкрити LightOFF", true, None::<&str>)?;
    let battery = MenuItem::with_id(app, "battery", "Від станції", true, None::<&str>)?;
    let grid = MenuItem::with_id(app, "grid", "Від мережі", true, None::<&str>)?;
    let auto = CheckMenuItem::with_id(app, "auto", "Автоматика за графіком", true, st.runtime.lock().automation, None::<&str>)?;
    let schedule = MenuItem::with_id(app, "schedule", "Оновити графік з сайту ВОЕ", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Вийти", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &sep1, &battery, &grid, &auto, &schedule, &sep2, &quit])?;
    app.manage(TrayItems { automation: auto });

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("LightOFF")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    let st = app.state::<Arc<AppState>>().inner().clone();
    match id {
        "open" => window::show_main(app),
        "battery" | "grid" => {
            let to = if id == "battery" { Source::Battery } else { Source::Grid };
            let h = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = automation::switch(&h, to, "Вручну з трею").await;
            });
        }
        "auto" => {
            let on = {
                let mut rt = st.runtime.lock();
                rt.automation = !rt.automation;
                rt.save();
                rt.automation
            };
            set_automation_check(app, on);
            st.log("info", if on { "Автоматику увімкнено" } else { "Автоматику вимкнено" });
        }
        "schedule" => {
            if let Err(e) = voe::open(app, true) {
                st.log("warn", format!("Не вдалося відкрити сайт ВОЕ: {e:#}"));
            }
        }
        "quit" => app.exit(0),
        _ => {}
    }
}

pub fn set_automation_check(app: &AppHandle, on: bool) {
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.automation.set_checked(on);
    }
}

/// Tooltip: charge, current source and the next planned switch.
pub fn update(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let st = app.state::<Arc<AppState>>().inner().clone();
    let mut lines = vec!["LightOFF".to_string()];
    match (st.status.read().as_ref(), st.status_error.read().as_ref()) {
        (Some(s), _) => {
            let src = match s.source {
                "battery" => "від станції",
                "grid" => "від мережі",
                _ => "AI-режим",
            };
            lines.push(format!("Заряд {:.0} % · {src}", s.soc));
        }
        (None, Some(_)) => lines.push("EcoFlow недоступний".into()),
        _ => {}
    }
    if st.runtime.lock().automation {
        let settings = st.settings.read().clone();
        let now = Local::now().naive_local();
        let next = planner::plan(&st.schedule.read().days, &settings).into_iter().find(|a| a.at > now);
        if let Some(a) = next {
            lines.push(format!("{} → {}", a.at.format("%H:%M"), a.to.label()));
        }
    } else {
        lines.push("Автоматику вимкнено".into());
    }
    let _ = tray.set_tooltip(Some(lines.join("\n")));
}
