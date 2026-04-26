use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WindowEvent,
};

mod api_server;
mod commands;
pub mod logger;
mod scanner;
mod state;

/// Find the project root.
/// Priority: AGENT_OS_ROOT env → bootstrap file → walk up from exe →
/// walk up from cwd → Documents/AgentOS.
fn bootstrap_state_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Agent OS").join("bootstrap.json"))
}

fn load_bootstrap_root() -> Option<PathBuf> {
    let path = bootstrap_state_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let root = value.get("project_root").and_then(|v| v.as_str())?;
    let path = PathBuf::from(root);
    if path.join("CLAUDE.md").exists() {
        Some(path)
    } else {
        None
    }
}

fn persist_bootstrap_root(root: &PathBuf) {
    let Some(path) = bootstrap_state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let payload = serde_json::json!({
        "project_root": root.to_string_lossy(),
    });
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&payload).unwrap_or_default(),
    );
}

fn project_root() -> PathBuf {
    if let Ok(root) = std::env::var("AGENT_OS_ROOT") {
        let p = PathBuf::from(&root);
        if p.join("CLAUDE.md").exists() {
            return p;
        }
        eprintln!(
            "AGENT_OS_ROOT={} but CLAUDE.md not found there, falling back",
            root
        );
    }

    if let Some(root) = load_bootstrap_root() {
        return root;
    }

    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors().skip(1) {
            if ancestor.join("CLAUDE.md").exists() {
                return ancestor.to_path_buf();
            }
        }
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        if ancestor.join("CLAUDE.md").exists() {
            return ancestor.to_path_buf();
        }
    }

    if let Some(documents) = dirs::document_dir() {
        let candidate = documents.join("AgentOS");
        if candidate.join("CLAUDE.md").exists() {
            return candidate;
        }
    }

    eprintln!("Warning: could not find project root. Using cwd: {:?}", cwd);
    cwd
}

pub fn run() {
    let root = project_root();
    persist_bootstrap_root(&root);
    logger::init(&root);
    log_info!(
        "Agent OS v{} starting - root: {:?}",
        env!("CARGO_PKG_VERSION"),
        root
    );

    // Single shared AppState — both Tauri commands and HTTP API use the same instance
    let shared = Arc::new(state::AppState::new(root.clone()));
    let api_state = Arc::clone(&shared);
    let auto_state = Arc::clone(&shared);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            tokio::spawn(api_server::start(api_state, 3333));
            commands::auto_approve::auto_approve_loop(auto_state).await;
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![
            // Agents
            commands::agents::get_agents,
            commands::agents::get_segments,
            // Feed & health
            commands::feed::get_feed,
            commands::feed::get_activity,
            commands::feed::get_health,
            commands::feed::get_plan,
            commands::feed::get_digest,
            commands::feed::get_project_plan,
            // Chat
            commands::chat::get_chats,
            commands::chat::get_chat_history,
            commands::chat::send_chat,
            commands::chat::export_chat,
            commands::chat_stream::stream_chat,
            commands::chat_stream_poll::poll_stream,
            commands::chat_stream_poll::stop_chat,
            commands::chat_stream_poll::is_chat_running,
            // Delegation
            commands::delegation::get_delegations,
            commands::delegation_cmds::approve_delegation,
            commands::delegation_cmds::reject_delegation,
            commands::delegation_cmds::schedule_delegation,
            commands::delegation_cmds::cancel_delegation,
            commands::delegation_analytics::get_analytics,
            commands::delegation_stream::poll_delegation_stream,
            commands::auto_approve::get_auto_approve_rules,
            commands::auto_approve::set_auto_approve_rules,
            commands::usage::get_usage_summary,
            commands::cron::get_cron_jobs,
            commands::graph::get_overview_graph,
            commands::graph::get_project_graph,
            commands::graph::get_overview_graph_ops,
            commands::graph::export_graph_mermaid,
            commands::graph::verify_project,
            commands::graph::check_graph_changes,
            commands::graph::graph_diff,
            commands::graph::get_subgraph,
            commands::graph::get_graph_at_commit,
            // Config
            commands::config::get_permissions,
            commands::config::set_permission,
            commands::config::get_health_history,
            commands::config::get_impact,
            commands::config::run_action,
            commands::config::get_modules,
            commands::config::get_config,
            commands::config::set_config,
            commands::config::get_api_token,
            commands::provider_runner::get_provider_status,
            commands::provider_runner::codex_acp_authenticate,
            // Operations
            commands::ops::deploy_template,
            commands::ops::health_check,
            commands::ops::create_project,
            commands::ops::get_queue,
            commands::ops::add_to_queue,
            commands::ops::save_attachment,
            commands::ops::send_telegram,
            // Strategy
            commands::strategy::get_goals,
            commands::strategy::save_goal,
            commands::strategy::get_strategies,
            commands::strategy::generate_strategy,
            commands::strategy::approve_strategy_steps,
            commands::strategy::execute_strategy_step,
            // Signals
            commands::signals::get_signals,
            commands::signals::ack_signal,
            // Inbox
            commands::inbox::get_inbox,
            commands::inbox::clear_inbox,
            commands::inbox::process_inbox,
            // Plans
            commands::plans::get_plans,
            commands::plans::create_plan,
            commands::plans::update_plan_step,
            commands::scope::get_active_scope,
            commands::scope::get_orchestration_map,
            commands::timeline::get_execution_timeline,
            // Multi-agent sessions
            commands::multi_agent::create_multi_agent_session,
            commands::multi_agent::list_multi_agent_sessions,
            commands::multi_agent::get_multi_agent_session,
            commands::multi_agent::get_session_agent_history,
            commands::multi_agent::set_session_writer,
            commands::multi_agent::set_session_orchestrator,
            commands::multi_agent::revoke_session_writer,
            commands::multi_agent::acquire_work_item_lease_manual,
            commands::multi_agent::release_file_lease,
            commands::multi_agent::run_session_agent,
            commands::multi_agent::run_session_round,
            commands::multi_agent::run_session_room_action,
            commands::multi_agent::create_project_session,
            commands::multi_agent::create_work_item,
            commands::multi_agent::create_plan_step_work_item,
            commands::multi_agent::queue_session_delegation,
            commands::multi_agent::queue_work_item_execution,
            commands::multi_agent::queue_parallel_work_items,
            commands::multi_agent::queue_provider_parallel_round,
            commands::multi_agent::complete_user_work_item,
            // Proxy
            commands::proxy::proxy_webhook,
        ])
        .setup(move |app| {
            commands::app_updates::spawn_startup_update_check(&app.handle());

            // === System Tray ===
            let open_i = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
            let status_i = MenuItem::with_id(app, "status", "Status", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit Agent OS", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&open_i, &status_i, &sep, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Agent OS — Command Center")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "status" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building Agent OS")
        .run(|app_handle, event| match event {
            RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" => {
                api.prevent_close();
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            RunEvent::ExitRequested { .. } => {
                // Kill all tracked child processes on exit (prevents zombies)
                if let Some(state) = app_handle.try_state::<Arc<state::AppState>>() {
                    commands::process_manager::kill_all_tracked(&state);
                }
            }
            _ => {}
        });
}
