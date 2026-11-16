use std::sync::Mutex;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const ICON: &[u8] = include_bytes!("../../../assets/parlo-256.png");

pub struct AutostartItem(Mutex<Option<CheckMenuItem<Wry>>>);

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let dictate = MenuItem::with_id(app, "dictate", "Dictate", true, None::<&str>)?;
    let copy = MenuItem::with_id(app, "copy", "Copy dictation", true, None::<&str>)?;
    let review = MenuItem::with_id(app, "review", "Review dictation", true, None::<&str>)?;
    let live = MenuItem::with_id(app, "live", "Live", true, None::<&str>)?;
    let captions = MenuItem::with_id(app, "captions", "Captions", true, None::<&str>)?;
    let undo = MenuItem::with_id(app, "undo", "Undo last dictation", true, None::<&str>)?;
    let home = MenuItem::with_id(app, "home", "Home", true, None::<&str>)?;
    let library = MenuItem::with_id(app, "library", "Library", true, None::<&str>)?;
    let meetings = MenuItem::with_id(app, "meetings", "Meetings", true, None::<&str>)?;
    let voice = MenuItem::with_id(app, "voice", "Voice", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start at login",
        true,
        parlo_core::autostart::entry().is_some(),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Parlo", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &dictate,
            &copy,
            &review,
            &live,
            &captions,
            &undo,
            &PredefinedMenuItem::separator(app)?,
            &home,
            &library,
            &meetings,
            &voice,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &autostart,
            &quit,
        ],
    )?;

    app.manage(AutostartItem(Mutex::new(Some(autostart))));

    let icon = tauri::image::Image::from_bytes(ICON)?;
    TrayIconBuilder::with_id("presence")
        .icon(icon)
        .tooltip("Parlo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "dictate" => crate::island::show(app),
            "copy" => crate::island::copy_recovery(app),
            "review" => crate::island::review_recovery(app),
            "live" => crate::live_toggle(app),
            "captions" => crate::captions_toggle(app),
            "undo" => crate::undo_last(app),
            "home" => focus_hub(app, Some("home")),
            "library" => focus_hub(app, Some("library")),
            "meetings" => focus_hub(app, Some("meetings")),
            "voice" => focus_hub(app, Some("voice")),
            "settings" => focus_hub(app, Some("settings")),
            "autostart" => toggle_autostart(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::island::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn toggle_autostart(app: &AppHandle) {
    let outcome = if parlo_core::autostart::entry().is_some() {
        parlo_core::autostart::disable()
    } else {
        std::env::current_exe()
            .map_err(|error| error.to_string())
            .map(|exe| format!("{} --hidden", parlo_core::autostart::quote(&exe)))
            .and_then(|command| parlo_core::autostart::enable(&command))
    };
    if let Err(error) = outcome {
        crate::note(&format!("start at login: {error}"));
    }
    let now_on = parlo_core::autostart::entry().is_some();
    if let Some(state) = app.try_state::<AutostartItem>() {
        if let Ok(slot) = state.0.lock() {
            if let Some(item) = slot.as_ref() {
                let _ = item.set_checked(now_on);
            }
        }
    }
}

pub fn focus_hub(app: &AppHandle, section: Option<&str>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    if let Some(section) = section {
        if let Ok(mut url) = window.url() {
            url.set_fragment(Some(section));
            let _ = window.navigate(url);
        }
    }
}

pub fn set_status(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("presence") {
        let _ = tray.set_tooltip(Some(text));
    }
}
