//! Container runtime collector for Raksha Security Agent.
//!
//! Detects container runtimes (Docker, containerd, podman), enumerates
//! running containers, checks for privileged containers, monitors exec
//! events, and reports container inventory to the portal.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Supported container runtimes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerRuntime {
    Docker,
    Containerd,
    Podman,
    Unknown,
}

/// Container state information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub state: String,
    pub pid: u32,
    pub privileged: bool,
    pub read_only_rootfs: bool,
    pub run_as_root: bool,
    pub capabilities: Vec<String>,
    pub mounts: Vec<MountInfo>,
    pub labels: HashMap<String, String>,
    pub created_at: u64,
    pub runtime: ContainerRuntime,
}

/// Mount point information for a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

/// Exec event detected in a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecEvent {
    pub container_id: String,
    pub container_name: String,
    pub command: Vec<String>,
    pub pid: u32,
    pub user: String,
    pub timestamp: u64,
}

/// Container inventory report sent to the portal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInventoryReport {
    pub hostname: String,
    pub runtime: ContainerRuntime,
    pub runtime_version: String,
    pub containers: Vec<ContainerInfo>,
    pub security_findings: Vec<SecurityFinding>,
    pub collected_at: u64,
}

/// A security finding related to container configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub category: String,
    pub container_id: String,
    pub container_name: String,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Errors from the container collector.
#[derive(Debug)]
pub enum CollectorError {
    NoRuntime,
    CommandFailed(String),
    ParseError(String),
    ReportError(String),
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRuntime => write!(f, "no container runtime detected"),
            Self::CommandFailed(e) => write!(f, "runtime command failed: {}", e),
            Self::ParseError(e) => write!(f, "parse error: {}", e),
            Self::ReportError(e) => write!(f, "report error: {}", e),
        }
    }
}

impl std::error::Error for CollectorError {}

/// Main container collector that interfaces with the detected runtime.
pub struct ContainerCollector {
    runtime: ContainerRuntime,
    socket_path: String,
    portal_endpoint: String,
}

impl ContainerCollector {
    /// Create a new ContainerCollector by auto-detecting the runtime.
    pub fn new(portal_endpoint: &str) -> Self {
        let (runtime, socket_path) = Self::detect_runtime();
        Self {
            runtime,
            socket_path,
            portal_endpoint: portal_endpoint.to_string(),
        }
    }

    /// Detect which container runtime is available on the host.
    pub fn detect_runtime() -> (ContainerRuntime, String) {
        let docker_paths = ["/var/run/docker.sock", "/run/docker.sock"];
        for path in &docker_paths {
            if Path::new(path).exists() {
                return (ContainerRuntime::Docker, path.to_string());
            }
        }

        let containerd_paths = [
            "/run/containerd/containerd.sock",
            "/var/run/containerd/containerd.sock",
        ];
        for path in &containerd_paths {
            if Path::new(path).exists() {
                return (ContainerRuntime::Containerd, path.to_string());
            }
        }

        let podman_paths = ["/run/podman/podman.sock", "/var/run/podman/podman.sock"];
        for path in &podman_paths {
            if Path::new(path).exists() {
                return (ContainerRuntime::Podman, path.to_string());
            }
        }

        // Fallback: CLI detection
        if Self::command_exists("docker") {
            return (ContainerRuntime::Docker, "/var/run/docker.sock".into());
        }
        if Self::command_exists("ctr") {
            return (ContainerRuntime::Containerd, "/run/containerd/containerd.sock".into());
        }
        if Self::command_exists("podman") {
            return (ContainerRuntime::Podman, String::new());
        }

        (ContainerRuntime::Unknown, String::new())
    }

    /// List all running containers from the detected runtime.
    pub fn list_containers(&self) -> Result<Vec<ContainerInfo>, CollectorError> {
        match self.runtime {
            ContainerRuntime::Docker => self.list_docker_containers(),
            ContainerRuntime::Containerd => self.list_containerd_containers(),
            ContainerRuntime::Podman => self.list_podman_containers(),
            ContainerRuntime::Unknown => Err(CollectorError::NoRuntime),
        }
    }


    /// Check all running containers for privileged mode and other issues.
    pub fn check_privileged_containers(&self) -> Result<Vec<SecurityFinding>, CollectorError> {
        let containers = self.list_containers()?;
        let mut findings = Vec::new();

        for container in &containers {
            if container.privileged {
                findings.push(SecurityFinding {
                    severity: Severity::Critical,
                    category: "privileged-container".into(),
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    description: format!(
                        "Container '{}' is running in privileged mode",
                        container.name
                    ),
                    remediation: "Remove privileged flag and use specific capabilities".into(),
                });
            }

            if container.run_as_root {
                findings.push(SecurityFinding {
                    severity: Severity::High,
                    category: "run-as-root".into(),
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    description: format!(
                        "Container '{}' is running as root (UID=0)",
                        container.name
                    ),
                    remediation: "Set runAsNonRoot: true in security context".into(),
                });
            }

            if !container.read_only_rootfs {
                findings.push(SecurityFinding {
                    severity: Severity::Medium,
                    category: "writable-rootfs".into(),
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    description: format!(
                        "Container '{}' has a writable root filesystem",
                        container.name
                    ),
                    remediation: "Set readOnlyRootFilesystem: true".into(),
                });
            }

            for mount in &container.mounts {
                if is_sensitive_mount(&mount.source) {
                    findings.push(SecurityFinding {
                        severity: Severity::Critical,
                        category: "sensitive-mount".into(),
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                        description: format!(
                            "Container '{}' mounts sensitive path: {}",
                            container.name, mount.source
                        ),
                        remediation: "Remove hostPath mount to sensitive paths".into(),
                    });
                }
            }
        }

        Ok(findings)
    }

    /// Monitor exec events in containers (Docker events stream).
    pub fn monitor_exec_events<F>(&self, callback: F) -> Result<(), CollectorError>
    where
        F: Fn(ExecEvent) + Send + 'static,
    {
        match self.runtime {
            ContainerRuntime::Docker | ContainerRuntime::Podman => {
                let cmd = if self.runtime == ContainerRuntime::Docker {
                    "docker"
                } else {
                    "podman"
                };

                let child = Command::new(cmd)
                    .args(["events", "--filter", "type=container", "--filter", "event=exec_start", "--format", "{{json .}}"])
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

                if let Some(stdout) = child.stdout {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(event) = Self::parse_exec_event(&line) {
                                callback(event);
                            }
                        }
                    }
                }
                Ok(())
            }
            ContainerRuntime::Containerd => {
                // containerd uses ctr events
                let child = Command::new("ctr")
                    .args(["events"])
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

                if let Some(stdout) = child.stdout {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if line.contains("exec") {
                                if let Ok(event) = Self::parse_exec_event(&line) {
                                    callback(event);
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            ContainerRuntime::Unknown => Err(CollectorError::NoRuntime),
        }
    }

    /// Report container inventory to the portal API.
    pub fn report_inventory(&self) -> Result<(), CollectorError> {
        let containers = self.list_containers()?;
        let findings = self.check_privileged_containers()?;

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let report = ContainerInventoryReport {
            hostname,
            runtime: self.runtime.clone(),
            runtime_version: self.get_runtime_version(),
            containers,
            security_findings: findings,
            collected_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        };

        let url = format!("{}/api/v1/agents/container-inventory", self.portal_endpoint);
        let body = serde_json::to_vec(&report)
            .map_err(|e| CollectorError::ReportError(e.to_string()))?;

        // Use ureq or reqwest for HTTP POST (simplified here)
        let status = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_bytes(&body)
            .map_err(|e| CollectorError::ReportError(e.to_string()))?
            .status();

        if status != 200 && status != 201 {
            return Err(CollectorError::ReportError(
                format!("portal returned status {}", status),
            ));
        }

        Ok(())
    }

    // --- Private helpers ---

    fn list_docker_containers(&self) -> Result<Vec<ContainerInfo>, CollectorError> {
        let output = Command::new("docker")
            .args(["inspect", "--format", "{{json .}}", "$(docker ps -q)"])
            .output()
            .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            // Fallback: use docker ps with JSON format
            let output = Command::new("docker")
                .args(["ps", "--no-trunc", "--format", "{{json .}}"])
                .output()
                .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            return self.parse_docker_ps_output(&stdout);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_docker_inspect_output(&stdout)
    }

    fn list_containerd_containers(&self) -> Result<Vec<ContainerInfo>, CollectorError> {
        let output = Command::new("ctr")
            .args(["-a", &self.socket_path, "containers", "list", "-q"])
            .output()
            .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(CollectorError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();

        for id in stdout.lines().filter(|l| !l.is_empty()) {
            if let Ok(info) = self.inspect_containerd_container(id.trim()) {
                containers.push(info);
            }
        }

        Ok(containers)
    }

    fn list_podman_containers(&self) -> Result<Vec<ContainerInfo>, CollectorError> {
        let output = Command::new("podman")
            .args(["ps", "--format", "json"])
            .output()
            .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(CollectorError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_podman_output(&stdout)
    }

    fn parse_docker_ps_output(&self, output: &str) -> Result<Vec<ContainerInfo>, CollectorError> {
        let mut containers = Vec::new();
        for line in output.lines().filter(|l| !l.is_empty()) {
            let v: serde_json::Value = serde_json::from_str(line)
                .map_err(|e| CollectorError::ParseError(e.to_string()))?;
            containers.push(ContainerInfo {
                id: v["ID"].as_str().unwrap_or_default().to_string(),
                name: v["Names"].as_str().unwrap_or_default().to_string(),
                image: v["Image"].as_str().unwrap_or_default().to_string(),
                image_id: String::new(),
                state: v["State"].as_str().unwrap_or("running").to_string(),
                pid: 0,
                privileged: false,
                read_only_rootfs: false,
                run_as_root: false,
                capabilities: Vec::new(),
                mounts: Vec::new(),
                labels: HashMap::new(),
                created_at: 0,
                runtime: ContainerRuntime::Docker,
            });
        }
        Ok(containers)
    }

    fn parse_docker_inspect_output(&self, output: &str) -> Result<Vec<ContainerInfo>, CollectorError> {
        let items: Vec<serde_json::Value> = serde_json::from_str(output)
            .map_err(|e| CollectorError::ParseError(e.to_string()))?;

        let mut containers = Vec::new();
        for v in items {
            let host_config = &v["HostConfig"];
            let state = &v["State"];

            let privileged = host_config["Privileged"].as_bool().unwrap_or(false);
            let read_only = host_config["ReadonlyRootfs"].as_bool().unwrap_or(false);
            let pid = state["Pid"].as_u64().unwrap_or(0) as u32;

            let mounts = v["Mounts"].as_array().map(|arr| {
                arr.iter().map(|m| MountInfo {
                    source: m["Source"].as_str().unwrap_or_default().to_string(),
                    destination: m["Destination"].as_str().unwrap_or_default().to_string(),
                    mode: m["Mode"].as_str().unwrap_or_default().to_string(),
                    rw: m["RW"].as_bool().unwrap_or(false),
                }).collect()
            }).unwrap_or_default();

            containers.push(ContainerInfo {
                id: v["Id"].as_str().unwrap_or_default().to_string(),
                name: v["Name"].as_str().unwrap_or_default().trim_start_matches('/').to_string(),
                image: v["Config"]["Image"].as_str().unwrap_or_default().to_string(),
                image_id: v["Image"].as_str().unwrap_or_default().to_string(),
                state: state["Status"].as_str().unwrap_or("unknown").to_string(),
                pid,
                privileged,
                read_only_rootfs: read_only,
                run_as_root: pid > 0 && Self::is_process_root(pid),
                capabilities: Vec::new(),
                mounts,
                labels: HashMap::new(),
                created_at: 0,
                runtime: ContainerRuntime::Docker,
            });
        }
        Ok(containers)
    }

    fn parse_podman_output(&self, output: &str) -> Result<Vec<ContainerInfo>, CollectorError> {
        let items: Vec<serde_json::Value> = serde_json::from_str(output)
            .map_err(|e| CollectorError::ParseError(e.to_string()))?;

        let mut containers = Vec::new();
        for v in items {
            containers.push(ContainerInfo {
                id: v["Id"].as_str().or(v["id"].as_str()).unwrap_or_default().to_string(),
                name: v["Names"].as_array()
                    .and_then(|a| a.first())
                    .and_then(|n| n.as_str())
                    .unwrap_or_default().to_string(),
                image: v["Image"].as_str().unwrap_or_default().to_string(),
                image_id: v["ImageID"].as_str().unwrap_or_default().to_string(),
                state: v["State"].as_str().unwrap_or("running").to_string(),
                pid: v["Pid"].as_u64().unwrap_or(0) as u32,
                privileged: false,
                read_only_rootfs: false,
                run_as_root: false,
                capabilities: Vec::new(),
                mounts: Vec::new(),
                labels: HashMap::new(),
                created_at: 0,
                runtime: ContainerRuntime::Podman,
            });
        }
        Ok(containers)
    }

    fn inspect_containerd_container(&self, id: &str) -> Result<ContainerInfo, CollectorError> {
        let output = Command::new("ctr")
            .args(["-a", &self.socket_path, "containers", "info", id])
            .output()
            .map_err(|e| CollectorError::CommandFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Simplified parse - in production use containerd gRPC client
        Ok(ContainerInfo {
            id: id.to_string(),
            name: id.to_string(),
            image: stdout.lines()
                .find(|l| l.contains("Image"))
                .unwrap_or_default()
                .split_whitespace()
                .last()
                .unwrap_or_default()
                .to_string(),
            image_id: String::new(),
            state: "running".to_string(),
            pid: 0,
            privileged: false,
            read_only_rootfs: false,
            run_as_root: false,
            capabilities: Vec::new(),
            mounts: Vec::new(),
            labels: HashMap::new(),
            created_at: 0,
            runtime: ContainerRuntime::Containerd,
        })
    }

    fn get_runtime_version(&self) -> String {
        let cmd = match self.runtime {
            ContainerRuntime::Docker => "docker",
            ContainerRuntime::Containerd => "ctr",
            ContainerRuntime::Podman => "podman",
            ContainerRuntime::Unknown => return "unknown".to_string(),
        };

        Command::new(cmd)
            .arg("--version")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn parse_exec_event(line: &str) -> Result<ExecEvent, CollectorError> {
        let v: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| CollectorError::ParseError(e.to_string()))?;

        Ok(ExecEvent {
            container_id: v["Actor"]["ID"].as_str().unwrap_or_default().to_string(),
            container_name: v["Actor"]["Attributes"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            command: v["Actor"]["Attributes"]["execID"]
                .as_str()
                .map(|s| vec![s.to_string()])
                .unwrap_or_default(),
            pid: 0,
            user: String::new(),
            timestamp: v["time"].as_u64().unwrap_or(0),
        })
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_process_root(pid: u32) -> bool {
        let status_path = format!("/proc/{}/status", pid);
        if let Ok(contents) = std::fs::read_to_string(&status_path) {
            for line in contents.lines() {
                if line.starts_with("Uid:") {
                    let uid = line.split_whitespace().nth(1).unwrap_or("1000");
                    return uid == "0";
                }
            }
        }
        false
    }
}

/// Check if a mount path is considered sensitive.
fn is_sensitive_mount(path: &str) -> bool {
    const SENSITIVE: &[&str] = &[
        "/etc/shadow",
        "/etc/passwd",
        "/root",
        "/var/run/docker.sock",
        "/var/run/containerd",
        "/proc",
        "/sys",
        "/dev",
    ];
    SENSITIVE.iter().any(|s| path.starts_with(s))
}


