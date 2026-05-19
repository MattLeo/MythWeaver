#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

struct BackendProcess(Mutex<Option<Child>>);

fn wait_for_backend(port: u16, retries: u32) -> bool {
    for _ in 0..retries {
        if reqwest::blocking::get(format!("http://localhost:{}/health", port)).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    false
}

fn spawn_backend() -> Option<Child> {
    // In release, the backend binary sits next to the desktop exe
    let backend_path = std::env::current_exe()
        .ok()?
        .parent()?
        .join("mythweaver-backend.exe");

    Command::new(backend_path)
        .spawn()
        .ok()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Spawn backend
            let child = spawn_backend();
            app.manage(BackendProcess(Mutex::new(child)));

            // Wait up to 10 seconds for it to be healthy
            if !wait_for_backend(3001, 33) {
                eprintln!("Backend failed to start");
            }

            // Build the main window
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External("http://localhost:3001".parse().unwrap()),
            )
            .title("MythWeaver")
            .inner_size(1280.0, 800.0)
            .min_inner_size(900.0, 600.0)
            .build()?;

            // Hide to tray on close instead of quitting
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    window_clone.hide().unwrap();
                }
            });

            // System tray
            let show = MenuItem::with_id(app, "show", "Open MythWeaver", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        // Kill backend before exiting
                        if let Some(state) = app.try_state::<BackendProcess>() {
                            if let Ok(mut guard) = state.0.lock() {
                                if let Some(mut child) = guard.take() {
                                    let _ = child.kill();
                                }
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Final cleanup if the process is killed directly
            if let WindowEvent::Destroyed = event {
                if let Some(state) = window.try_state::<BackendProcess>() {
                    if let Ok(mut guard) = state.0.lock() {
                        if let Some(mut child) = guard.take() {
                            let _ = child.kill();
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error running MythWeaver");
}