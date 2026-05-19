#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

struct AppProcesses(Mutex<Vec<Child>>);

fn wait_for_port(port: u16, retries: u32) -> bool {
    for _ in 0..retries {
        if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    false
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let resource_dir = app.path().resource_dir().unwrap();

            let node = resource_dir.join("node.exe");
            let backend_exe = resource_dir.join("mythweaver.exe");
            let frontend_dir = resource_dir.join("frontend");

            println!("Resource dir: {:?}", resource_dir);
            println!("Node path: {:?}", node);
            println!("Backend path: {:?}", backend_exe);
            println!("Frontend dir: {:?}", frontend_dir);

            let mut processes: Vec<Child> = vec![];

            // Spawn backend
            match Command::new(&backend_exe).spawn() {
                Ok(child) => {
                    println!("Backend started");
                    processes.push(child);
                }
                Err(e) => println!("Backend failed to start: {}", e),
            }

            // Spawn frontend via vite preview
            match Command::new(&node)
                .args([
                    "node_modules/vite/bin/vite.js",
                    "preview",
                    "--port",
                    "5173",
                    "--host",
                ])
                .current_dir(&frontend_dir)
                .spawn()
            {
                Ok(child) => {
                    println!("Frontend started");
                    processes.push(child);
                }
                Err(e) => println!("Frontend failed to start: {}", e),
            }

            app.manage(AppProcesses(Mutex::new(processes)));

            // Wait for both
            if !wait_for_port(3001, 33) {
                println!("Backend never became available");
            }
            if !wait_for_port(5173, 33) {
                println!("Frontend never became available");
            }

            // Open window pointing at frontend
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External("http://localhost:5173".parse().unwrap()),
            )
            .title("MythWeaver")
            .inner_size(1280.0, 800.0)
            .min_inner_size(900.0, 600.0)
            .devtools(true)
            .build()?;

            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    window_clone.hide().unwrap();
                }
            });

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
                        if let Some(state) = app.try_state::<AppProcesses>() {
                            if let Ok(mut processes) = state.0.lock() {
                                for child in processes.iter_mut() {
                                    let _ = child.kill();
                                }
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
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
            if let WindowEvent::Destroyed = event {
                if let Some(state) = window.try_state::<AppProcesses>() {
                    if let Ok(mut processes) = state.0.lock() {
                        for child in processes.iter_mut() {
                            let _ = child.kill();
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error running MythWeaver");
}