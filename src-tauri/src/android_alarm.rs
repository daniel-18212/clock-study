use serde::Serialize;
use std::time::Duration;
use tauri::{plugin::{Builder, PluginHandle, TauriPlugin}, AppHandle, Manager, Runtime};

type Alarm = Option<(Duration, &'static str)>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Schedule {
    deadline_millis: u64,
    message: &'static str,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("clock-alarm")
        .setup(|app, api| {
            let handle = api.register_android_plugin("com.clockstudy.desktop", "ClockAlarmPlugin")?;
            app.manage(handle);
            Ok(())
        })
        .build()
}

pub fn update(app: &AppHandle, previous: Alarm, next: Alarm, now: Duration) -> Result<(), String> {
    if previous == next {
        return Ok(());
    }
    let plugin = app.state::<PluginHandle<tauri::Wry>>();
    let result = if let Some((deadline, message)) = next {
        // Android's elapsedRealtime and CLOCK_BOOTTIME share the same time base.
        let deadline_millis = deadline.as_millis() as u64;
        plugin.run_mobile_plugin::<()>("schedule", Schedule { deadline_millis, message })
    } else if previous.is_some_and(|(deadline, _)| deadline > now) {
        plugin.run_mobile_plugin::<()>("cancel", ())
    } else {
        return Ok(());
    };
    result.map_err(|error| error.to_string())
}
