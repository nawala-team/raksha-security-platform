//! Tests for the agent collector modules.

use sysinfo::System;

// Import the collector module from the agent crate
// Note: For these tests to compile, the collector module must be public in lib.rs
// or these tests use integration-style access via the binary's module structure.

/// Test that server metrics collection returns valid data.
#[test]
fn test_server_metrics_collection() {
    let mut sys = System::new_all();
    // Allow sysinfo to gather initial readings
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_all();

    let cpu_count = sys.cpus().len();
    assert!(cpu_count > 0, "Should detect at least one CPU");

    let total_memory = sys.total_memory();
    assert!(total_memory > 0, "Total memory should be > 0");

    let used_memory = sys.used_memory();
    assert!(used_memory <= total_memory, "Used memory should not exceed total");

    let cpu_usage = sys.global_cpu_usage();
    assert!(
        (0.0..=100.0).contains(&cpu_usage),
        "CPU usage should be 0-100%, got {}",
        cpu_usage
    );
}

/// Test memory usage percentage calculation.
#[test]
fn test_memory_usage_percent_calculation() {
    let total: u64 = 16_000_000_000; // 16 GB
    let used: u64 = 8_000_000_000; // 8 GB

    let percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    assert!(
        (percent - 50.0).abs() < 0.01,
        "Expected ~50%, got {}",
        percent
    );
}

/// Test zero total memory edge case.
#[test]
fn test_memory_usage_zero_total() {
    let total: u64 = 0;
    let used: u64 = 0;

    let percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(percent, 0.0);
}

/// Test disk usage percentage calculation.
#[test]
fn test_disk_usage_percent_calculation() {
    let total: u64 = 500_000_000_000; // 500 GB
    let available: u64 = 200_000_000_000; // 200 GB available

    let usage_percent = if total > 0 {
        ((total - available) as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    assert!(
        (usage_percent - 60.0).abs() < 0.01,
        "Expected ~60% usage, got {}",
        usage_percent
    );
}

/// Test that disk listing returns at least one disk.
#[test]
fn test_disk_enumeration() {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    // Most systems have at least one disk
    assert!(!disks.list().is_empty(), "Should detect at least one disk");
}

/// Test MetricPayload serialization.
#[test]
fn test_metric_payload_serialization() {
    use chrono::Utc;
    use serde_json::json;

    let payload = json!({
        "agent_id": "agent-001",
        "hostname": "test-server",
        "timestamp": Utc::now().to_rfc3339(),
        "category": "server",
        "data": {
            "cpu_usage_percent": 45.2,
            "memory_usage_percent": 67.8
        }
    });

    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.contains("agent-001"));
    assert!(serialized.contains("server"));

    // Verify round-trip
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized["agent_id"], "agent-001");
    assert_eq!(deserialized["category"], "server");
}

/// Test alert severity serialization matches expected format.
#[test]
fn test_alert_serialization() {
    use chrono::Utc;
    use serde_json::json;

    let alert = json!({
        "agent_id": "agent-002",
        "hostname": "prod-web-01",
        "timestamp": Utc::now().to_rfc3339(),
        "severity": "critical",
        "category": "process",
        "title": "Suspicious process detected",
        "description": "Unknown binary executing with elevated privileges",
        "metadata": {
            "pid": 12345,
            "binary": "/tmp/.hidden/payload",
            "user": "root"
        }
    });

    let serialized = serde_json::to_string(&alert).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["severity"], "critical");
    assert_eq!(parsed["metadata"]["pid"], 12345);
}

/// Test network metrics structure.
#[test]
fn test_network_metrics_structure() {
    use sysinfo::Networks;

    let networks = Networks::new_with_refreshed_list();

    // Verify we can iterate network interfaces
    for (name, data) in &networks {
        assert!(!name.is_empty(), "Network interface name should not be empty");
        // Bytes received/transmitted should be non-negative (u64)
        let _received = data.total_received();
        let _transmitted = data.total_transmitted();
    }
}

/// Test process listing.
#[test]
fn test_process_enumeration() {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let process_count = sys.processes().len();
    assert!(
        process_count > 0,
        "Should detect at least one running process"
    );
}
