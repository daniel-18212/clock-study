mod timer;

use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path, sync::Mutex, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};
#[cfg(desktop)]
use tauri::{LogicalSize, PhysicalSize, WebviewWindow, WindowEvent};
use tauri_plugin_notification::NotificationExt;
use timer::{Action, Settings, Snapshot, Timer};

#[cfg(desktop)]
const MAIN_MIN: LogicalSize<f64> = LogicalSize::new(390.0, 315.0);
#[cfg(desktop)]
const FLOAT_MIN: LogicalSize<f64> = LogicalSize::new(230.0, 130.0);

#[cfg(desktop)]
#[derive(Default, serde::Deserialize, Serialize)]
struct WindowSizes {
    main: Option<(u32, u32)>,
    floating: Option<(u32, u32)>,
}

#[cfg(desktop)]
#[derive(Default)]
struct WindowLayout {
    floating: bool,
    sizes: WindowSizes,
    main_maximized: bool,
}

fn load<T: DeserializeOwned + Default>(app: &AppHandle, name: &str) -> T {
    app.path().app_config_dir().ok()
        .and_then(|dir| fs::read(dir.join(name)).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save<T: Serialize>(app: &AppHandle, name: &str, value: &T) -> Result<(), String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    write_json(&directory, name, value)
        .map_err(|error| format!("Não foi possível salvar as preferências: {error}"))
}

fn write_json<T: Serialize>(directory: &Path, name: &str, value: &T) -> std::io::Result<()> {
    fs::create_dir_all(directory)?;
    let staging = directory.join(format!("{name}.tmp"));
    fs::write(&staging, serde_json::to_vec(value).map_err(std::io::Error::other)?)?;
    fs::rename(staging, directory.join(name))
}

#[cfg(desktop)]
fn size(value: (u32, u32)) -> PhysicalSize<u32> {
    PhysicalSize::new(value.0.clamp(230, 7680), value.1.clamp(130, 4320))
}

#[cfg(desktop)]
#[tauri::command]
fn set_floating(
    window: WebviewWindow,
    state: State<'_, Mutex<WindowLayout>>,
    enabled: bool,
) -> Result<(), String> {
    let mut layout = state.lock().map_err(|error| error.to_string())?;
    if layout.floating == enabled {
        return Ok(());
    }

    let previous_size = window.inner_size().map_err(|error| error.to_string())?;
    let was_maximized = window.is_maximized().map_err(|error| error.to_string())?;
    let previous_minimum = if layout.floating { FLOAT_MIN } else { MAIN_MIN };
    let change = (|| -> tauri::Result<()> {
        if was_maximized {
            window.unmaximize()?;
        }
        if enabled {
            window.set_min_size(Some(FLOAT_MIN))?;
            if let Some(saved) = layout.sizes.floating {
                window.set_size(size(saved))?;
            } else {
                window.set_size(LogicalSize::new(305.0, 160.0))?;
            }
        } else {
            window.set_min_size(Some(MAIN_MIN))?;
            if let Some(saved) = layout.sizes.main {
                window.set_size(size(saved))?;
            } else {
                window.set_size(LogicalSize::new(740.0, 500.0))?;
            }
            if layout.main_maximized {
                window.maximize()?;
            }
        }
        window.set_always_on_top(enabled)?;
        Ok(())
    })();

    if let Err(error) = change {
        let _ = window.set_min_size(Some(previous_minimum));
        let _ = window.set_size(previous_size);
        let _ = window.set_always_on_top(layout.floating);
        if was_maximized {
            let _ = window.maximize();
        }
        return Err(error.to_string());
    }

    if enabled {
        layout.main_maximized = was_maximized;
        if !was_maximized {
            layout.sizes.main = Some((previous_size.width, previous_size.height));
        }
    } else if !was_maximized {
        layout.sizes.floating = Some((previous_size.width, previous_size.height));
    }
    layout.floating = enabled;
    Ok(())
}

#[cfg_attr(target_os = "android", tauri::command(async))]
#[cfg_attr(not(target_os = "android"), tauri::command)]
fn timer_action(
    app: AppHandle,
    timer: State<'_, Mutex<Timer>>,
    action: Option<Action>,
) -> Result<Snapshot, String> {
    let mut timer = timer.lock().map_err(|error| error.to_string())?;
    let now = timer::now();
    if let Some(action) = action {
        if let Action::Configure(ref settings) = action {
            if !settings.valid() {
                return Err("Informe tempos válidos e de 1 a 20 blocos.".into());
            }
            save(&app, "timer.json", settings)?;
        }
        #[cfg(target_os = "android")]
        let previous_alert = timer.alert_deadline();

        timer.apply(action, now)?;

        #[cfg(target_os = "android")]
        if let Err(error) = update_android_alarm(&app, previous_alert, timer.alert_deadline(), now) {
            eprintln!("Clock Study: {error}");
        }
    }
    Ok(timer.snapshot(now))
}

#[tauri::command]
fn supports_floating() -> bool {
    cfg!(desktop)
}

#[tauri::command(async)]
fn request_android_notifications(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    app.notification().request_permission().map_err(|error| error.to_string())?;
    #[cfg(not(target_os = "android"))]
    let _ = app;
    Ok(())
}

#[cfg(target_os = "android")]
fn update_android_alarm(
    app: &AppHandle,
    previous: Option<(Duration, &'static str)>,
    next: Option<(Duration, &'static str)>,
    now: Duration,
) -> tauri_plugin_notification::Result<()> {
    use tauri_plugin_notification::Schedule;

    const ALARM_ID: i32 = 1;

    if previous == next {
        return Ok(());
    }
    let notification = app.notification();
    if let Some((deadline, message)) = next {
        let date = time::OffsetDateTime::now_utc() + deadline.saturating_sub(now);
        notification.builder()
            .id(ALARM_ID)
            .title("Clock Study")
            .body(message)
            .schedule(Schedule::At { date, repeating: false, allow_while_idle: true })
            .show()?;
    } else if previous.is_some_and(|(deadline, _)| deadline > now) {
        notification.cancel(vec![ALARM_ID])?;
    }
    Ok(())
}

fn track_timer(app: AppHandle) {
    thread::spawn(move || {
        let mut previous: Option<Snapshot> = None;
        #[cfg(not(target_os = "android"))]
        let mut alarm = 0;
        loop {
            let snapshot = {
                let state = app.state::<Mutex<Timer>>();
                let Ok(mut timer) = state.lock() else { return };
                let now = timer::now();
                timer.tick(now);
                timer.snapshot(now)
            };
            #[cfg(not(target_os = "android"))]
            if snapshot.alarm != alarm {
                alarm = snapshot.alarm;
                let _ = app.notification().builder()
                    .title("Clock Study")
                    .body(snapshot.notice.unwrap_or("Tempo encerrado"))
                    .show();
            }
            if previous.as_ref() != Some(&snapshot) {
                let _ = app.emit("clock-tick", &snapshot);
                previous = Some(snapshot);
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let mut settings: Settings = load(app.handle(), "timer.json");
            if !settings.valid() {
                settings = Settings::default();
            }
            app.manage(Mutex::new(Timer::new(settings)));
            #[cfg(desktop)]
            restore_window(app.handle());
            track_timer(app.handle().clone());
            Ok(())
        });

    #[cfg(desktop)]
    let builder = builder
        .on_window_event(save_window)
        .invoke_handler(tauri::generate_handler![set_floating, timer_action, supports_floating, request_android_notifications]);
    #[cfg(mobile)]
    let builder = builder
        .invoke_handler(tauri::generate_handler![timer_action, supports_floating, request_android_notifications]);

    builder.run(tauri::generate_context!())
        .expect("não foi possível iniciar Clock Study");
}

#[cfg(desktop)]
fn restore_window(app: &AppHandle) {
    let sizes: WindowSizes = load(app, "windows.json");
    if let (Some(main), Some(saved)) = (app.get_webview_window("main"), sizes.main) {
        if let Ok(Some(monitor)) = main.current_monitor() {
            let bounds = monitor.size();
            let restored = size(saved);
            let _ = main.set_size(PhysicalSize::new(
                restored.width.min(bounds.width), restored.height.min(bounds.height),
            ));
        }
    }
    app.manage(Mutex::new(WindowLayout { sizes, ..WindowLayout::default() }));
}

#[cfg(desktop)]
fn save_window(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { .. } = event {
        let state = window.state::<Mutex<WindowLayout>>();
        if let Ok(mut layout) = state.lock() {
            if let (Ok(false), Ok(size)) = (window.is_maximized(), window.inner_size()) {
                let bounds = Some((size.width, size.height));
                if layout.floating {
                    layout.sizes.floating = bounds;
                } else {
                    layout.sizes.main = bounds;
                }
            }
            let _ = save(window.app_handle(), "windows.json", &layout.sizes);
        };
    }
}
