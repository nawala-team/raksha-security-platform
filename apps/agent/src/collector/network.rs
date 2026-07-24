use serde::Serialize;
use sysinfo::Networks;

/// Network interface metrics.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<InterfaceMetrics>,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub connection_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceMetrics {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub mac_address: String,
}

pub fn collect_network_metrics() -> NetworkMetrics {
    let networks = Networks::new_with_refreshed_list();

    let mut interfaces = Vec::new();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    for (name, data) in &networks {
        let rx = data.total_received();
        let tx = data.total_transmitted();
        total_rx += rx;
        total_tx += tx;

        interfaces.push(InterfaceMetrics {
            name: name.clone(),
            rx_bytes: rx,
            tx_bytes: tx,
            rx_packets: data.total_packets_received(),
            tx_packets: data.total_packets_transmitted(),
            rx_errors: data.total_errors_on_received(),
            tx_errors: data.total_errors_on_transmitted(),
            mac_address: data.mac_address().to_string(),
        });
    }

    let connection_count = get_connection_count();

    NetworkMetrics {
        interfaces,
        total_rx_bytes: total_rx,
        total_tx_bytes: total_tx,
        connection_count,
    }
}

/// Get approximate TCP connection count (platform-specific).
fn get_connection_count() -> usize {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/net/tcp and /proc/net/tcp6
        let count = std::fs::read_to_string("/proc/net/tcp")
            .map(|c| c.lines().count().saturating_sub(1))
            .unwrap_or(0);
        let count6 = std::fs::read_to_string("/proc/net/tcp6")
            .map(|c| c.lines().count().saturating_sub(1))
            .unwrap_or(0);
        count + count6
    }
    #[cfg(target_os = "windows")]
    {
        // Use netstat parsing as fallback
        std::process::Command::new("netstat")
            .args(["-n", "-p", "TCP"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| l.contains("ESTABLISHED") || l.contains("TIME_WAIT"))
                    .count()
            })
            .unwrap_or(0)
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("netstat")
            .args(["-n", "-p", "tcp"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| l.contains("ESTABLISHED") || l.contains("TIME_WAIT"))
                    .count()
            })
            .unwrap_or(0)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        0
    }
}
