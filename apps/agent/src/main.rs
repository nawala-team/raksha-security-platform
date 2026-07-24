mod config;
mod collector;
mod deception;
mod reporter;
mod updater;

use config::AgentConfig;
use collector::{MetricCategory, MetricPayload};
use reporter::Reporter;
use updater::Updater;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("raksha_agent=info".parse().unwrap()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("run");

    match command {
        "install" => install_service(),
        "uninstall" => uninstall_service(),
        "start" => start_service(),
        "stop" => stop_service(),
        "run" => run_agent().await,
        "init" => init_config(),
        "version" => println!("raksha-agent v{VERSION}"),
        "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Raksha Security Agent v{VERSION}");
    println!();
    println!("Usage: raksha-agent <command>");
    println!();
    println!("Commands:");
    println!("  run        Run the agent in foreground");
    println!("  install    Install as a system service");
    println!("  uninstall  Remove the system service");
    println!("  start      Start the system service");
    println!("  stop       Stop the system service");
    println!("  init       Generate default configuration");
    println!("  version    Show version");
}

fn init_config() {
    match AgentConfig::write_default(None) {
        Ok(path) => println!("Config written to: {}", path.display()),
        Err(e) => {
            eprintln!("Failed to write config: {e}");
            std::process::exit(1);
        }
    }
}


async fn run_agent() {
    info!("Starting Raksha Security Agent v{VERSION}");

    let config = match AgentConfig::load(None) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {e}");
            eprintln!("Error: {e}");
            eprintln!("Run 'raksha-agent init' to generate default configuration.");
            std::process::exit(1);
        }
    };

    info!("Agent ID: {}", config.agent_id);
    info!("Hostname: {}", config.hostname);
    info!("Portal:   {}", config.portal_url);

    let buffer_path = if config.buffer.file_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(&config.buffer.file_path))
    };

    let reporter = Arc::new(Reporter::new(
        config.portal_url.clone(),
        config.auth_token.clone(),
        config.buffer.max_items,
        buffer_path,
    ));

    let sys = Arc::new(RwLock::new(System::new_all()));
    let config = Arc::new(config);
    let mut handles = Vec::new();

    // Heartbeat task
    let r = reporter.clone();
    let c = config.clone();
    handles.push(tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(c.intervals.report_secs),
        );
        loop {
            interval.tick().await;
            r.heartbeat(&c.agent_id, &c.hostname).await;
        }
    }));

    // Server metrics
    if config.modules.server_metrics {
        let r = reporter.clone();
        let c = config.clone();
        let s = sys.clone();
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(c.intervals.metrics_secs),
            );
            loop {
                interval.tick().await;
                let metrics = {
                    let mut sys = s.write().await;
                    collector::server::collect_server_metrics(&mut sys)
                };
                let payload = MetricPayload {
                    agent_id: c.agent_id.clone(),
                    hostname: c.hostname.clone(),
                    timestamp: chrono::Utc::now(),
                    category: MetricCategory::Server,
                    data: serde_json::to_value(&metrics).unwrap_or_default(),
                };
                r.send_metrics(&payload).await;
            }
        }));
    }

    // Network metrics
    if config.modules.network_metrics {
        let r = reporter.clone();
        let c = config.clone();
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(c.intervals.metrics_secs),
            );
            loop {
                interval.tick().await;
                let metrics = collector::network::collect_network_metrics();
                let payload = MetricPayload {
                    agent_id: c.agent_id.clone(),
                    hostname: c.hostname.clone(),
                    timestamp: chrono::Utc::now(),
                    category: MetricCategory::Network,
                    data: serde_json::to_value(&metrics).unwrap_or_default(),
                };
                r.send_metrics(&payload).await;
            }
        }));
    }

    // Process monitoring
    if config.modules.process_monitor {
        let r = reporter.clone();
        let c = config.clone();
        let s = sys.clone();
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(c.intervals.process_secs),
            );
            loop {
                interval.tick().await;
                let metrics = {
                    let mut sys = s.write().await;
                    collector::process::collect_process_metrics(&mut sys)
                };
                for proc in &metrics.suspicious {
                    let alert = collector::Alert {
                        agent_id: c.agent_id.clone(),
                        hostname: c.hostname.clone(),
                        timestamp: chrono::Utc::now(),
                        severity: collector::AlertSeverity::High,
                        category: MetricCategory::Process,
                        title: format!("Suspicious process: {}", proc.name),
                        description: proc.reason.clone(),
                        metadata: serde_json::json!({
                            "pid": proc.pid,
                            "command": proc.command,
                        }),
                    };
                    r.send_alert(&alert).await;
                }
                let payload = MetricPayload {
                    agent_id: c.agent_id.clone(),
                    hostname: c.hostname.clone(),
                    timestamp: chrono::Utc::now(),
                    category: MetricCategory::Process,
                    data: serde_json::to_value(&metrics).unwrap_or_default(),
                };
                r.send_metrics(&payload).await;
            }
        }));
    }

    // File integrity monitoring
    if config.modules.file_integrity {
        let r = reporter.clone();
        let c = config.clone();
        handles.push(tokio::spawn(async move {
            let db_path = if cfg!(windows) {
                PathBuf::from(r"C:\ProgramData\Raksha\fim.json")
            } else {
                PathBuf::from("/var/lib/raksha/fim.json")
            };
            let mut db = collector::filesystem::HashDatabase::load(&db_path);
            let paths = collector::filesystem::default_monitored_paths();
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(c.intervals.file_integrity_secs),
            );
            loop {
                interval.tick().await;
                let metrics = collector::filesystem::check_integrity(&mut db, &paths);
                if let Err(e) = db.save(&db_path) {
                    error!("Failed to save FIM database: {e}");
                }
                for change in &metrics.changes_detected {
                    let alert = collector::Alert {
                        agent_id: c.agent_id.clone(),
                        hostname: c.hostname.clone(),
                        timestamp: chrono::Utc::now(),
                        severity: collector::AlertSeverity::Medium,
                        category: MetricCategory::FileIntegrity,
                        title: format!("File {:?}: {}", change.change_type, change.path),
                        description: format!(
                            "File integrity change: {:?}", change.change_type
                        ),
                        metadata: serde_json::to_value(change).unwrap_or_default(),
                    };
                    r.send_alert(&alert).await;
                }
                let payload = MetricPayload {
                    agent_id: c.agent_id.clone(),
                    hostname: c.hostname.clone(),
                    timestamp: chrono::Utc::now(),
                    category: MetricCategory::FileIntegrity,
                    data: serde_json::to_value(&metrics).unwrap_or_default(),
                };
                r.send_metrics(&payload).await;
            }
        }));
    }

    // Updater task
    if config.updater.enabled && !config.updater.update_url.is_empty() {
        let c = config.clone();
        handles.push(tokio::spawn(async move {
            let upd = Updater::new(c.updater.update_url.clone());
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(c.intervals.update_check_secs),
            );
            loop {
                interval.tick().await;
                if let Some(info) = upd.check_update().await {
                    if let Err(e) = upd.apply_update(&info).await {
                        error!("Update failed: {e}");
                    }
                }
            }
        }));
    }

    // Deception (Honeypot) module
    let deception_config = match deception::HoneypotManager::load_default_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to load deception config, module disabled: {e}");
            deception::DeceptionConfig {
                enabled: false,
                honeypots: Vec::new(),
            }
        }
    };

    let mut honeypot_manager = deception::HoneypotManager::new(
        deception_config,
        reporter.clone(),
        config.agent_id.clone(),
        config.hostname.clone(),
    );
    honeypot_manager.start().await;

    info!("All collectors started. Agent is running.");
    shutdown_signal().await;
    info!("Shutdown signal received. Stopping agent...");

    honeypot_manager.stop().await;

    for handle in handles {
        handle.abort();
    }
}

/// Platform-aware shutdown signal handler.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
    }
    #[cfg(windows)]
    {
        tokio::signal::ctrl_c().await.expect("Ctrl+C handler");
    }
}

// --- Service management (platform-specific) ---

#[cfg(target_os = "linux")]
fn install_service() {
    let exe_path = std::env::current_exe().expect("Cannot determine executable path");
    let unit = format!(
        r#"[Unit]
Description=Raksha Security Agent
After=network.target

[Service]
Type=simple
ExecStart={} run
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"#,
        exe_path.display()
    );

    let service_path = "/etc/systemd/system/raksha-agent.service";
    if let Err(e) = std::fs::write(service_path, unit) {
        eprintln!("Failed to write service file: {e}");
        eprintln!("Try running with sudo.");
        std::process::exit(1);
    }

    let _ = std::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["enable", "raksha-agent"])
        .status();

    println!("Service installed: {service_path}");
}

#[cfg(target_os = "linux")]
fn uninstall_service() {
    let _ = std::process::Command::new("systemctl")
        .args(["stop", "raksha-agent"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["disable", "raksha-agent"])
        .status();
    let _ = std::fs::remove_file("/etc/systemd/system/raksha-agent.service");
    let _ = std::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status();
    println!("Service uninstalled.");
}

#[cfg(target_os = "linux")]
fn start_service() {
    let status = std::process::Command::new("systemctl")
        .args(["start", "raksha-agent"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service started."),
        _ => eprintln!("Failed to start service."),
    }
}

#[cfg(target_os = "linux")]
fn stop_service() {
    let status = std::process::Command::new("systemctl")
        .args(["stop", "raksha-agent"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service stopped."),
        _ => eprintln!("Failed to stop service."),
    }
}

#[cfg(target_os = "macos")]
fn install_service() {
    let exe_path = std::env::current_exe().expect("Cannot determine executable path");
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.raksha.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>"#,
        exe_path.display()
    );

    let plist_path = "/Library/LaunchDaemons/com.raksha.agent.plist";
    if let Err(e) = std::fs::write(plist_path, plist) {
        eprintln!("Failed to write plist: {e}");
        std::process::exit(1);
    }
    println!("Service installed: {plist_path}");
}

#[cfg(target_os = "macos")]
fn uninstall_service() {
    let _ = std::process::Command::new("launchctl")
        .args(["unload", "/Library/LaunchDaemons/com.raksha.agent.plist"])
        .status();
    let _ = std::fs::remove_file("/Library/LaunchDaemons/com.raksha.agent.plist");
    println!("Service uninstalled.");
}

#[cfg(target_os = "macos")]
fn start_service() {
    let status = std::process::Command::new("launchctl")
        .args(["load", "/Library/LaunchDaemons/com.raksha.agent.plist"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service started."),
        _ => eprintln!("Failed to start service."),
    }
}

#[cfg(target_os = "macos")]
fn stop_service() {
    let status = std::process::Command::new("launchctl")
        .args(["unload", "/Library/LaunchDaemons/com.raksha.agent.plist"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service stopped."),
        _ => eprintln!("Failed to stop service."),
    }
}

#[cfg(windows)]
fn install_service() {
    let exe_path = std::env::current_exe().expect("Cannot determine executable path");
    let status = std::process::Command::new("sc")
        .args([
            "create", "RakshaAgent",
            &format!("binPath= \"{}\" run", exe_path.display()),
            "start=", "auto",
            "DisplayName=", "Raksha Security Agent",
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service installed."),
        _ => {
            eprintln!("Failed to install service. Run as Administrator.");
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn uninstall_service() {
    let _ = std::process::Command::new("sc")
        .args(["stop", "RakshaAgent"])
        .status();
    let status = std::process::Command::new("sc")
        .args(["delete", "RakshaAgent"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service uninstalled."),
        _ => eprintln!("Failed to uninstall service."),
    }
}

#[cfg(windows)]
fn start_service() {
    let status = std::process::Command::new("sc")
        .args(["start", "RakshaAgent"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service started."),
        _ => eprintln!("Failed to start service."),
    }
}

#[cfg(windows)]
fn stop_service() {
    let status = std::process::Command::new("sc")
        .args(["stop", "RakshaAgent"])
        .status();
    match status {
        Ok(s) if s.success() => println!("Service stopped."),
        _ => eprintln!("Failed to stop service."),
    }
}

