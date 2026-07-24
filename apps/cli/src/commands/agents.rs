use super::ApiClient;
use colored::Colorize;
use serde::Deserialize;
use tabled::{Table, Tabled};

#[derive(Debug, Deserialize, Tabled)]
pub struct AgentInfo {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Hostname")]
    pub hostname: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "IP")]
    pub ip_address: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Last Seen")]
    pub last_seen: String,
}

pub async fn list(client: &ApiClient) -> Result<(), Box<dyn std::error::Error>> {
    let agents: Vec<AgentInfo> = client.get("/agents").await?;

    if agents.is_empty() {
        println!("{}", "No agents registered.".yellow());
        return Ok(());
    }

    println!("{} ({} total)\n", "Registered Agents".bold(), agents.len());
    let table = Table::new(&agents).to_string();
    println!("{table}");
    Ok(())
}

pub async fn show(client: &ApiClient, agent_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let agent: AgentInfo = client.get(&format!("/agents/{agent_id}")).await?;

    println!("{}", "Agent Details".bold());
    println!("{}", "-".repeat(30));
    println!("  ID:        {}", agent.id);
    println!("  Hostname:  {}", agent.hostname);
    println!("  Status:    {}", agent.status);
    println!("  IP:        {}", agent.ip_address);
    println!("  Version:   {}", agent.version);
    println!("  Last Seen: {}", agent.last_seen);
    Ok(())
}

pub async fn remove(client: &ApiClient, agent_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete(&format!("/agents/{agent_id}")).await?;
    println!("{} Agent {} removed.", "OK".green(), agent_id);
    Ok(())
}
