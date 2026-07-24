use super::ApiClient;
use colored::Colorize;
use serde::Deserialize;
use tabled::{Table, Tabled};

#[derive(Debug, Deserialize, Tabled)]
pub struct AlertInfo {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Severity")]
    pub severity: String,
    #[tabled(rename = "Agent")]
    pub hostname: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Time")]
    pub created_at: String,
}

pub async fn list(
    client: &ApiClient,
    severity: Option<&str>,
    status: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = "/alerts?".to_string();
    if let Some(sev) = severity {
        path.push_str(&format!("severity={sev}&"));
    }
    if let Some(st) = status {
        path.push_str(&format!("status={st}&"));
    }

    let alerts: Vec<AlertInfo> = client.get(&path).await?;

    if alerts.is_empty() {
        println!("{}", "No alerts found.".green());
        return Ok(());
    }

    println!("{} ({} total)\n", "Alerts".bold(), alerts.len());
    let table = Table::new(&alerts).to_string();
    println!("{table}");
    Ok(())
}

pub async fn acknowledge(
    client: &ApiClient,
    alert_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({ "status": "acknowledged" });
    client
        .post::<serde_json::Value>(&format!("/alerts/{alert_id}/status"), &body)
        .await?;
    println!("{} Alert {} acknowledged.", "OK".green(), alert_id);
    Ok(())
}

pub async fn resolve(
    client: &ApiClient,
    alert_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({ "status": "resolved" });
    client
        .post::<serde_json::Value>(&format!("/alerts/{alert_id}/status"), &body)
        .await?;
    println!("{} Alert {} resolved.", "OK".green(), alert_id);
    Ok(())
}

pub async fn show(
    client: &ApiClient,
    alert_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug, Deserialize)]
    struct AlertDetail {
        id: String,
        severity: String,
        hostname: String,
        title: String,
        description: String,
        status: String,
        created_at: String,
        metadata: serde_json::Value,
    }

    let alert: AlertDetail = client.get(&format!("/alerts/{alert_id}")).await?;

    let sev_colored = match alert.severity.as_str() {
        "critical" => alert.severity.red().bold(),
        "high" => alert.severity.red(),
        "medium" => alert.severity.yellow(),
        _ => alert.severity.normal(),
    };

    println!("{}", "Alert Detail".bold());
    println!("{}", "-".repeat(40));
    println!("  ID:          {}", alert.id);
    println!("  Severity:    {}", sev_colored);
    println!("  Agent:       {}", alert.hostname);
    println!("  Title:       {}", alert.title);
    println!("  Description: {}", alert.description);
    println!("  Status:      {}", alert.status);
    println!("  Created:     {}", alert.created_at);
    if !alert.metadata.is_null() {
        println!("  Metadata:    {}", serde_json::to_string_pretty(&alert.metadata)?);
    }
    Ok(())
}
