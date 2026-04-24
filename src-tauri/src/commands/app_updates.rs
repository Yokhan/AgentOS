use crate::state::AppState;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

fn auto_updates_enabled() -> bool {
    !matches!(
        env::var("AGENT_OS_DISABLE_AUTO_UPDATES"),
        Ok(v) if v == "1" || v.eq_ignore_ascii_case("true")
    )
}

fn should_check_on_startup<R: Runtime>(app: &AppHandle<R>) -> bool {
    if !auto_updates_enabled() {
        return false;
    }

    let Some(state) = app.try_state::<Arc<AppState>>() else {
        return false;
    };

    let Ok(exe) = env::current_exe() else {
        return false;
    };

    // Repo/dev runs should never auto-update themselves.
    !exe.starts_with(&state.root)
}

fn notify<R: Runtime>(app: &AppHandle<R>, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn spawn_startup_update_check<R: Runtime>(app: &AppHandle<R>) {
    if !should_check_on_startup(app) {
        return;
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        if let Err(err) = check_and_install_updates(&handle).await {
            crate::log_warn!("[updater] startup auto-update failed: {}", err);
        }
    });
}

pub async fn check_and_install_updates<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    let Some(update) = update else {
        crate::log_info!("[updater] no update available");
        return Ok(());
    };

    crate::log_info!(
        "[updater] update found: current={} latest={}",
        update.current_version,
        update.version
    );
    notify(
        app,
        "Agent OS update found",
        &format!("Downloading Agent OS {}…", update.version),
    );

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;

    crate::log_info!("[updater] update installed, restarting Agent OS");
    notify(
        app,
        "Agent OS update installed",
        "Restarting to finish update.",
    );
    app.restart()
}
