use crate::state::AppState;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

const STARTUP_UPDATE_DELAY_SECS: u64 = 3;
const UPDATE_CHECK_TIMEOUT_SECS: u64 = 45;
const UPDATE_INSTALL_TIMEOUT_SECS: u64 = 600;

fn auto_updates_enabled() -> bool {
    !matches!(
        env::var("AGENT_OS_DISABLE_AUTO_UPDATES"),
        Ok(v) if v == "1" || v.eq_ignore_ascii_case("true")
    )
}

fn startup_update_skip_reason<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    if !auto_updates_enabled() {
        return Some("AGENT_OS_DISABLE_AUTO_UPDATES is set".to_string());
    }

    let Some(state) = app.try_state::<Arc<AppState>>() else {
        return Some("AppState is not available yet".to_string());
    };

    let Ok(exe) = env::current_exe() else {
        return Some("current_exe is unavailable".to_string());
    };

    // Repo/dev runs should never auto-update themselves.
    if exe.starts_with(&state.root) {
        return Some(format!(
            "current executable is inside project root: exe={:?} root={:?}",
            exe, state.root
        ));
    }

    None
}

fn notify<R: Runtime>(app: &AppHandle<R>, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn spawn_startup_update_check<R: Runtime>(app: &AppHandle<R>) {
    if let Some(reason) = startup_update_skip_reason(app) {
        crate::log_info!("[updater] startup check skipped: {}", reason);
        return;
    }

    crate::log_info!(
        "[updater] startup check scheduled in {}s",
        STARTUP_UPDATE_DELAY_SECS
    );
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(STARTUP_UPDATE_DELAY_SECS)).await;
        if let Err(err) = check_and_install_updates(&handle).await {
            crate::log_warn!("[updater] startup auto-update failed: {}", err);
        }
    });
}

pub async fn check_and_install_updates<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    crate::log_info!(
        "[updater] checking for updates (timeout={}s)",
        UPDATE_CHECK_TIMEOUT_SECS
    );
    let update = tokio::time::timeout(
        Duration::from_secs(UPDATE_CHECK_TIMEOUT_SECS),
        updater.check(),
    )
    .await
    .map_err(|_| {
        format!(
            "update check timed out after {}s",
            UPDATE_CHECK_TIMEOUT_SECS
        )
    })?
    .map_err(|e| e.to_string())?;

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
        &format!("Downloading Agent OS {}...", update.version),
    );

    crate::log_info!(
        "[updater] downloading and installing update (timeout={}s)",
        UPDATE_INSTALL_TIMEOUT_SECS
    );
    tokio::time::timeout(
        Duration::from_secs(UPDATE_INSTALL_TIMEOUT_SECS),
        update.download_and_install(|_, _| {}, || {}),
    )
    .await
    .map_err(|_| {
        format!(
            "update download/install timed out after {}s",
            UPDATE_INSTALL_TIMEOUT_SECS
        )
    })?
    .map_err(|e| e.to_string())?;

    crate::log_info!("[updater] update installed, restarting Agent OS");
    notify(
        app,
        "Agent OS update installed",
        "Restarting to finish update.",
    );
    app.restart()
}
