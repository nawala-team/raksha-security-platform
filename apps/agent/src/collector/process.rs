use serde::Serialize;
use sysinfo::System;

/// Process monitoring data.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessMetrics {
    pub total_processes: usize,
    pub top_cpu: Vec<ProcessInfo>,
    pub top_memory: Vec<ProcessInfo>,
    pub suspicious: Vec<SuspiciousProcess>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub user: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuspiciousProcess {
    pub pid: u32,
    pub name: String,
    pub reason: String,
    pub command: String,
}

/// Known suspicious process names (simplified detection).
const SUSPICIOUS_NAMES: &[&str] = &[
    "mimikatz", "meterpreter", "cobalt", "ncat", "nc.exe",
    "powershell_ise", "psexec", "procdump", "lazagne",
    "bloodhound", "sharphound", "rubeus", "certutil",
    "wmic", "bitsadmin",
];

/// Suspicious command patterns.
const SUSPICIOUS_PATTERNS: &[&str] = &[
    "-encodedcommand", "-enc ", "bypass", "hidden",
    "invoke-expression", "downloadstring", "iex(",
    "reverse", "bind_tcp", "msfvenom",
    "/etc/shadow", "/etc/passwd",
];

pub fn collect_process_metrics(sys: &mut System) -> ProcessMetrics {
    sys.refresh_all();

    let processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, proc)| ProcessInfo {
            pid: pid.as_u32(),
            name: proc.name().to_string_lossy().to_string(),
            cpu_usage: proc.cpu_usage(),
            memory_bytes: proc.memory(),
            status: format!("{:?}", proc.status()),
            // `Uid` does not implement `Display`; it derefs to the platform uid type.
            user: proc.user_id().map(|u| format!("{}", **u)),
            command: proc.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect::<Vec<_>>().join(" "),
        })
        .collect();

    let total_processes = processes.len();

    // Top 10 by CPU
    let mut top_cpu = processes.clone();
    top_cpu.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    top_cpu.truncate(10);

    // Top 10 by memory
    let mut top_memory = processes.clone();
    top_memory.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    top_memory.truncate(10);

    // Detect suspicious processes
    let suspicious = detect_suspicious(&processes);

    ProcessMetrics {
        total_processes,
        top_cpu,
        top_memory,
        suspicious,
    }
}

fn detect_suspicious(processes: &[ProcessInfo]) -> Vec<SuspiciousProcess> {
    let mut results = Vec::new();

    for proc in processes {
        let name_lower = proc.name.to_lowercase();
        let cmd_lower = proc.command.to_lowercase();

        // Check process name
        for &suspicious_name in SUSPICIOUS_NAMES {
            if name_lower.contains(suspicious_name) {
                results.push(SuspiciousProcess {
                    pid: proc.pid,
                    name: proc.name.clone(),
                    reason: format!("Known suspicious process: {suspicious_name}"),
                    command: proc.command.clone(),
                });
                break;
            }
        }

        // Check command-line patterns
        for &pattern in SUSPICIOUS_PATTERNS {
            if cmd_lower.contains(pattern) {
                results.push(SuspiciousProcess {
                    pid: proc.pid,
                    name: proc.name.clone(),
                    reason: format!("Suspicious command pattern: {pattern}"),
                    command: proc.command.clone(),
                });
                break;
            }
        }
    }

    results
}
