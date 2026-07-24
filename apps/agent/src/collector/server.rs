use serde::Serialize;
use sysinfo::System;

/// Server metrics: CPU, RAM, Disk, Uptime.
#[derive(Debug, Clone, Serialize)]
pub struct ServerMetrics {
    pub cpu_usage_percent: f32,
    pub cpu_count: usize,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_usage_percent: f64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub disks: Vec<DiskMetrics>,
    pub uptime_secs: u64,
    pub load_average: LoadAverage,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub filesystem: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

pub fn collect_server_metrics(sys: &mut System) -> ServerMetrics {
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_count = sys.cpus().len();
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();

    let memory_usage_percent = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64) * 100.0
    } else {
        0.0
    };

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disk_metrics: Vec<DiskMetrics> = disks
        .iter()
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let usage = if total > 0 {
                ((total - available) as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            DiskMetrics {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                available_bytes: available,
                usage_percent: usage,
                filesystem: d.file_system().to_string_lossy().to_string(),
            }
        })
        .collect();

    let load_avg = System::load_average();
    let load_average = LoadAverage {
        one: load_avg.one,
        five: load_avg.five,
        fifteen: load_avg.fifteen,
    };

    ServerMetrics {
        cpu_usage_percent: cpu_usage,
        cpu_count,
        memory_total_bytes: memory_total,
        memory_used_bytes: memory_used,
        memory_usage_percent,
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_used,
        disks: disk_metrics,
        uptime_secs: System::uptime(),
        load_average,
    }
}
