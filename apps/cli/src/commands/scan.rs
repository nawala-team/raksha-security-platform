use super::ApiClient;
use colored::Colorize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub findings: usize,
}

pub async fn trigger(
    client: &ApiClient,
    agent_id: Option<&str>,
    scan_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "agent_id": agent_id,
        "scan_type": scan_type,
    });

    let result: ScanResult = client.post("/scans", &body).await?;

    println!("{} Scan triggered.", "OK".green());
    println!("  Scan ID:  {}", result.scan_id);
    println!("  Type:     {}", scan_type);
    println!("  Status:   {}", result.status);
    println!("  Started:  {}", result.started_at);
    Ok(())
}

pub async fn status(
    client: &ApiClient,
    scan_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let result: ScanResult = client.get(&format!("/scans/{scan_id}")).await?;

    let status_colored = match result.status.as_str() {
        "completed" => result.status.green(),
        "running" => result.status.yellow(),
        "failed" => result.status.red(),
        _ => result.status.normal(),
    };

    println!("{}", "Scan Status".bold());
    println!("{}", "-".repeat(30));
    println!("  Scan ID:    {}", result.scan_id);
    println!("  Status:     {}", status_colored);
    println!("  Started:    {}", result.started_at);
    println!("  Completed:  {}", result.completed_at.unwrap_or_else(|| "-".to_string()));
    println!("  Findings:   {}", result.findings);
    Ok(())
}

pub async fn list(client: &ApiClient) -> Result<(), Box<dyn std::error::Error>> {
    let scans: Vec<ScanResult> = client.get("/scans").await?;

    if scans.is_empty() {
        println!("{}", "No scans found.".yellow());
        return Ok(());
    }

    println!("{} ({} total)\n", "Scans".bold(), scans.len());
    for scan in &scans {
        let status_colored = match scan.status.as_str() {
            "completed" => scan.status.green(),
            "running" => scan.status.yellow(),
            "failed" => scan.status.red(),
            _ => scan.status.normal(),
        };
        println!(
            "  {} | {} | {} | findings: {}",
            scan.scan_id, status_colored, scan.started_at, scan.findings
        );
    }
    Ok(())
}
