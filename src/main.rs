use tracing_subscriber::EnvFilter;

mod client;
mod commands;
mod config;
mod error;
mod models;
mod spectre;
mod transport;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "virtuoso", about = "Control Cadence Virtuoso from anywhere")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .env template
    Init,
    /// Start SSH tunnel + deploy daemon
    Start,
    /// Stop tunnel
    Stop,
    /// Restart tunnel + daemon
    Restart,
    /// Check connection status
    Status,
    /// Execute SKILL code
    Exec {
        /// SKILL expression to execute
        code: String,
        /// Connection timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },
    /// Open a cellview
    Open {
        /// Library name
        #[arg(long)]
        lib: String,
        /// Cell name
        #[arg(long)]
        cell: String,
        /// View name
        #[arg(long, default_value = "layout")]
        view: String,
        /// Open mode (r/o/a)
        #[arg(long, default_value = "a")]
        mode: String,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Start => commands::start::run(),
        Commands::Stop => commands::stop::run(),
        Commands::Restart => commands::restart::run(),
        Commands::Status => commands::status::run(),
        Commands::Exec { code, timeout } => commands::exec::run(&code, timeout),
        Commands::Open {
            lib,
            cell,
            view,
            mode,
        } => commands::open::run(&lib, &cell, &view, &mode),
    };

    if let Err(e) = result {
        tracing::error!("{}", e);
        std::process::exit(1);
    }
}
