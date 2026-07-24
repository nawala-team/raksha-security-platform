use super::ApiClient;
use colored::Colorize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PortalStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub agents_online: usize,
    pub agents_total: usize,
    pub alerts_pending: usize,
}

#[derive(Debug, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub hostname: String,
    pub status: String,
    pub last_seen: String,
    pub version: String,
}

pub async fn run(client: &ApiClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Raksha Security Platform - Status".bold());
    println!("{}", "=".repeat(40));

    // Portal status
    println!("\n{}", "Portal:".bold());
    match client.get::<PortalStatus>("/status").await {
        Ok(status) => {
            println!("  Version:        {}", status.version);
            println!("  Uptime:         {}s", status.uptime_secs);
            println!(
                "  Agents Online:  {}/{}",
                status.agents_online.to_string().green(),
                status.agents_total
            );
            println!(
                "  Pending Alerts: {}",
                if status.alerts_pending > 0 {
                    status.alerts_pending.to_string().red()
                } else {
                    status.alerts_pending.to_string().green()
                }
            );
        }
        Err(e) => {
            println!("  {} Cannot reach portal: {}", "ERROR".red(), e);
        }
    }

    // Local agent status
    println!("\n{}", "Local Agent:".bold());
    match client.get::<AgentStatus>("/agents/local").await {
        Ok(agent) => {
            let status_colored = match agent.status.as_str() {
                "online" => agent.status.green(),
                "offline" => agent.status.red(),
                _ => agent.status.yellow(),
            };
            println!("  Agent ID:  {}", agent.agent_id);
            println!("  Hostname:  {}", agent.hostname);
            println!("  Status:    {}", status_colored);
            println!("  Last Seen: {}", agent.last_seen);
            println!("  Version:   {}", agent.version);
        }
        Err(e) => {
            println!("  {} Cannot get agent status: {}", "ERROR".red(), e);
        }
    }

    Ok(())
}
