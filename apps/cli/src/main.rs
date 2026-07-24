mod commands;

use clap::{Parser, Subcommand};
use commands::config::CliConfig;
use commands::ApiClient;

#[derive(Parser)]
#[command(
    name = "raksha",
    version,
    about = "Raksha Security Platform CLI",
    long_about = "Command-line interface for managing the Raksha Security Platform"
)]
struct Cli {
    /// Portal URL override
    #[arg(long, global = true)]
    url: Option<String>,

    /// Auth token override
    #[arg(long, global = true)]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show platform and agent status
    Status,

    /// Manage agents
    #[command(subcommand)]
    Agents(AgentsCmd),

    /// View and manage alerts
    #[command(subcommand)]
    Alerts(AlertsCmd),

    /// Manage CLI configuration
    #[command(subcommand)]
    Config(ConfigCmd),

    /// Trigger and manage scans
    #[command(subcommand)]
    Scan(ScanCmd),
}

#[derive(Subcommand)]
enum AgentsCmd {
    /// List all registered agents
    List,
    /// Show details for a specific agent
    Show { agent_id: String },
    /// Remove an agent
    Remove { agent_id: String },
}

#[derive(Subcommand)]
enum AlertsCmd {
    /// List alerts
    List {
        /// Filter by severity (low, medium, high, critical)
        #[arg(short, long)]
        severity: Option<String>,
        /// Filter by status (open, acknowledged, resolved)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show alert details
    Show { alert_id: String },
    /// Acknowledge an alert
    Ack { alert_id: String },
    /// Resolve an alert
    Resolve { alert_id: String },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set { key: String, value: String },
    /// Initialize default configuration
    Init,
}

#[derive(Subcommand)]
enum ScanCmd {
    /// Trigger a new scan
    Run {
        /// Scan type (full, process, filesystem, network)
        #[arg(short, long, default_value = "full")]
        scan_type: String,
        /// Target agent ID (omit for all agents)
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Check scan status
    Status { scan_id: String },
    /// List recent scans
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = CliConfig::load();

    let base_url = cli.url.unwrap_or(config.portal_url);
    let token = cli.token.unwrap_or(config.token);

    let client = ApiClient::new(base_url, token);

    let result = match cli.command {
        Commands::Status => commands::status::run(&client).await,
        Commands::Agents(cmd) => match cmd {
            AgentsCmd::List => commands::agents::list(&client).await,
            AgentsCmd::Show { agent_id } => commands::agents::show(&client, &agent_id).await,
            AgentsCmd::Remove { agent_id } => commands::agents::remove(&client, &agent_id).await,
        },
        Commands::Alerts(cmd) => match cmd {
            AlertsCmd::List { severity, status } => {
                commands::alerts::list(
                    &client,
                    severity.as_deref(),
                    status.as_deref(),
                ).await
            }
            AlertsCmd::Show { alert_id } => commands::alerts::show(&client, &alert_id).await,
            AlertsCmd::Ack { alert_id } => commands::alerts::acknowledge(&client, &alert_id).await,
            AlertsCmd::Resolve { alert_id } => commands::alerts::resolve(&client, &alert_id).await,
        },
        Commands::Config(cmd) => match cmd {
            ConfigCmd::Show => { commands::config::show(); Ok(()) }
            ConfigCmd::Set { key, value } => commands::config::set(&key, &value),
            ConfigCmd::Init => commands::config::init(),
        },
        Commands::Scan(cmd) => match cmd {
            ScanCmd::Run { scan_type, agent } => {
                commands::scan::trigger(&client, agent.as_deref(), &scan_type).await
            }
            ScanCmd::Status { scan_id } => commands::scan::status(&client, &scan_id).await,
            ScanCmd::List => commands::scan::list(&client).await,
        },
    };

    if let Err(e) = result {
        eprintln!("{}: {e}", colored::Colorize::red("Error"));
        std::process::exit(1);
    }
}
