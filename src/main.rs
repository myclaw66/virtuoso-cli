use tracing_subscriber::EnvFilter;

mod client;
mod commands;
mod config;
mod error;
mod exit_codes;
mod models;
mod ocean;
mod output;
mod spectre;
mod transport;

use clap::{Parser, Subcommand, ValueEnum};
use output::{OutputFormat, print_json};

#[derive(Parser)]
#[command(
    name = "virtuoso",
    about = "Control Cadence Virtuoso from anywhere",
    long_about = "CLI tool for AI agents and humans to control Cadence Virtuoso, locally or remotely.\n\n\
        Examples:\n  \
        virtuoso tunnel start              # Start SSH tunnel\n  \
        virtuoso skill exec '1+1'          # Execute SKILL code\n  \
        virtuoso cell open --lib my --cell top\n  \
        virtuoso schema --all              # Show full command schema as JSON",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json or table (default: table in TTY, json in pipe)
    #[arg(long, global = true)]
    format: Option<FormatArg>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    quiet: bool,

    /// Enable debug logging
    #[arg(long, short, global = true)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum FormatArg {
    Json,
    Table,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .env template with default configuration
    #[command(
        long_about = "Create a .env configuration template in the current directory.\n\n\
            Examples:\n  \
            virtuoso init\n  \
            virtuoso init --if-not-exists"
    )]
    Init {
        /// Skip if .env already exists (exit 0 instead of error)
        #[arg(long)]
        if_not_exists: bool,
    },

    /// Manage SSH tunnel to remote Virtuoso host
    #[command(subcommand)]
    Tunnel(TunnelCmd),

    /// Execute SKILL code on connected Virtuoso instance
    #[command(subcommand)]
    Skill(SkillCmd),

    /// Manage cellviews in Virtuoso
    #[command(subcommand)]
    Cell(CellCmd),

    /// Circuit simulation automation via Ocean SKILL
    #[command(subcommand)]
    Sim(SimCmd),

    /// Show CLI command schema as JSON for agent introspection
    #[command(
        long_about = "Show the full command schema as JSON, useful for AI agent discovery.\n\n\
            Examples:\n  \
            virtuoso schema --all\n  \
            virtuoso schema tunnel start"
    )]
    Schema {
        /// Show full command tree
        #[arg(long)]
        all: bool,

        /// Command noun (e.g. tunnel)
        noun: Option<String>,

        /// Command verb (e.g. start)
        verb: Option<String>,
    },
}

#[derive(Subcommand)]
enum TunnelCmd {
    /// Start SSH tunnel and deploy daemon
    #[command(
        long_about = "Establish SSH tunnel to remote host and deploy the bridge daemon.\n\n\
            Examples:\n  \
            virtuoso tunnel start\n  \
            virtuoso tunnel start --timeout 60\n  \
            virtuoso tunnel start --dry-run --format json"
    )]
    Start {
        /// Connection timeout in seconds
        #[arg(long, short, default_value = "30")]
        timeout: u64,

        /// Preview without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Stop SSH tunnel and clean up remote files
    #[command(
        long_about = "Stop the running SSH tunnel and optionally clean up remote files.\n\n\
            Examples:\n  \
            virtuoso tunnel stop\n  \
            virtuoso tunnel stop --force"
    )]
    Stop {
        /// Force kill even if PID verification fails
        #[arg(long)]
        force: bool,

        /// Preview without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Restart SSH tunnel (stop + start)
    Restart {
        /// Connection timeout in seconds
        #[arg(long, short, default_value = "30")]
        timeout: u64,
    },

    /// Show tunnel, daemon, and connection status
    #[command(
        long_about = "Check the status of tunnel, daemon, and Virtuoso connection.\n\n\
            Examples:\n  \
            virtuoso tunnel status\n  \
            virtuoso tunnel status --format json"
    )]
    Status,
}

#[derive(Subcommand)]
enum SkillCmd {
    /// Execute a SKILL expression and return result
    #[command(
        long_about = "Send a SKILL expression to Virtuoso for evaluation.\n\n\
            Examples:\n  \
            virtuoso skill exec '1+1'\n  \
            virtuoso skill exec 'geGetEditCellView()' --timeout 60"
    )]
    Exec {
        /// SKILL expression to evaluate
        code: String,

        /// Execution timeout in seconds
        #[arg(long, short, default_value = "30")]
        timeout: u64,
    },

    /// Upload and load an IL script file into Virtuoso
    #[command(
        long_about = "Upload a SKILL/IL file to the remote host and load it.\n\n\
            Examples:\n  \
            virtuoso skill load my_script.il"
    )]
    Load {
        /// Path to .il file
        file: String,
    },
}

#[derive(Subcommand)]
enum CellCmd {
    /// Open a cellview for editing
    #[command(
        long_about = "Open a cellview in Virtuoso.\n\n\
            Examples:\n  \
            virtuoso cell open --lib myLib --cell myCell\n  \
            virtuoso cell open --lib myLib --cell myCell --view schematic --mode r"
    )]
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

        /// Open mode: r(ead), o(verwrite), a(ppend)
        #[arg(long, default_value = "a")]
        mode: String,

        /// Preview without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Save the current cellview
    Save,

    /// Close the current cellview without saving
    Close,

    /// Get info about the currently open cellview
    Info,
}

#[derive(Subcommand)]
enum SimCmd {
    /// Set simulator and design target
    #[command(
        long_about = "Configure simulator and design for simulation.\n\n\
            Examples:\n  \
            virtuoso sim setup --lib FT0001A_SH --cell Bandgap_LDO\n  \
            virtuoso sim setup --lib myLib --cell myCell --simulator spectre"
    )]
    Setup {
        /// Library name
        #[arg(long)]
        lib: String,

        /// Cell name
        #[arg(long)]
        cell: String,

        /// View name
        #[arg(long, default_value = "schematic")]
        view: String,

        /// Simulator engine
        #[arg(long, default_value = "spectre")]
        simulator: String,
    },

    /// Run a simulation analysis
    #[command(
        long_about = "Execute a simulation with specified analysis type.\n\n\
            Examples:\n  \
            virtuoso sim run --analysis tran --stop 10u\n  \
            virtuoso sim run --analysis dc --from 0 --to 1.2 --step 0.01\n  \
            virtuoso sim run --analysis ac --start 1 --stop 1e9 --dec 10"
    )]
    Run {
        /// Analysis type: tran, dc, ac, stb
        #[arg(long)]
        analysis: String,

        /// Stop time (tran) or stop value (dc/ac)
        #[arg(long)]
        stop: Option<String>,

        /// Start value (dc/ac)
        #[arg(long)]
        start: Option<String>,

        /// From value (dc)
        #[arg(long)]
        from: Option<String>,

        /// To value (dc)
        #[arg(long)]
        to: Option<String>,

        /// Step value (dc)
        #[arg(long)]
        step: Option<String>,

        /// Points per decade (ac)
        #[arg(long)]
        dec: Option<String>,

        /// Error preset
        #[arg(long)]
        errpreset: Option<String>,

        /// Extra key=value params
        #[arg(long, value_parser = parse_key_val)]
        param: Vec<(String, String)>,

        /// Simulation timeout in seconds
        #[arg(long, short, default_value = "300")]
        timeout: u64,
    },

    /// Extract waveform measurements from last simulation
    #[command(
        long_about = "Extract metrics from simulation results.\n\n\
            Examples:\n  \
            virtuoso sim measure --expr 'ymax(VT(\"/OUT\"))'\n  \
            virtuoso sim measure --analysis tran --expr 'cross(VT(\"/OUT\") 0.6 1 \"rising\")'"
    )]
    Measure {
        /// Measurement expression (can be repeated)
        #[arg(long, required = true)]
        expr: Vec<String>,

        /// Analysis type to select results from
        #[arg(long, default_value = "tran")]
        analysis: String,
    },

    /// Parameter sweep: vary a design variable and measure
    #[command(
        long_about = "Sweep a design variable across a range and collect measurements.\n\n\
            Examples:\n  \
            virtuoso sim sweep --var W --from 1e-6 --to 5e-6 --step 1e-6 \\\n    \
              --measure 'ymax(VT(\"/OUT\"))'"
    )]
    Sweep {
        /// Design variable to sweep
        #[arg(long)]
        var: String,

        /// Start value
        #[arg(long)]
        from: f64,

        /// End value
        #[arg(long)]
        to: f64,

        /// Step size
        #[arg(long)]
        step: f64,

        /// Measurement expression (can be repeated)
        #[arg(long, required = true)]
        measure: Vec<String>,

        /// Analysis type
        #[arg(long, default_value = "tran")]
        analysis: String,

        /// Simulation timeout in seconds
        #[arg(long, short, default_value = "600")]
        timeout: u64,
    },

    /// PVT corner analysis from JSON config
    #[command(
        long_about = "Run simulations across PVT corners defined in a JSON file.\n\n\
            Examples:\n  \
            virtuoso sim corner --file corners.json\n  \
            virtuoso sim corner --file corners.json --format table"
    )]
    Corner {
        /// Path to corner configuration JSON file
        #[arg(long)]
        file: String,

        /// Simulation timeout in seconds (per corner)
        #[arg(long, short, default_value = "600")]
        timeout: u64,
    },

    /// Show simulation results directory and contents
    Results,
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=VALUE: no '=' in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .with_target(false)
        .init();

    let format = match &cli.format {
        Some(FormatArg::Json) => OutputFormat::Json,
        Some(FormatArg::Table) => OutputFormat::Table,
        None => OutputFormat::resolve(None),
    };

    let is_status_cmd = matches!(&cli.command, Commands::Tunnel(TunnelCmd::Status));

    let result = match cli.command {
        Commands::Init { if_not_exists } => commands::init::run(if_not_exists),
        Commands::Tunnel(cmd) => match cmd {
            TunnelCmd::Start { timeout, dry_run } => {
                commands::tunnel::start(Some(timeout), dry_run)
            }
            TunnelCmd::Stop { force, dry_run } => commands::tunnel::stop(force, dry_run),
            TunnelCmd::Restart { timeout } => commands::tunnel::restart(Some(timeout)),
            TunnelCmd::Status => commands::tunnel::status(format),
        },
        Commands::Skill(cmd) => match cmd {
            SkillCmd::Exec { code, timeout } => commands::skill::exec(&code, timeout),
            SkillCmd::Load { file } => commands::skill::load(&file),
        },
        Commands::Cell(cmd) => match cmd {
            CellCmd::Open {
                lib,
                cell,
                view,
                mode,
                dry_run,
            } => commands::cell::open(&lib, &cell, &view, &mode, dry_run),
            CellCmd::Save => commands::cell::save(),
            CellCmd::Close => commands::cell::close(),
            CellCmd::Info => commands::cell::info(),
        },
        Commands::Sim(cmd) => match cmd {
            SimCmd::Setup {
                lib,
                cell,
                view,
                simulator,
            } => commands::sim::setup(&lib, &cell, &view, &simulator),
            SimCmd::Run {
                analysis,
                stop,
                start,
                from,
                to,
                step,
                dec,
                errpreset,
                param,
                timeout,
            } => {
                let mut params: std::collections::HashMap<String, String> = param.into_iter().collect();
                if let Some(v) = stop { params.insert("stop".into(), v); }
                if let Some(v) = start { params.insert("start".into(), v); }
                if let Some(v) = from { params.insert("from".into(), v); }
                if let Some(v) = to { params.insert("to".into(), v); }
                if let Some(v) = step { params.insert("step".into(), v); }
                if let Some(v) = dec { params.insert("dec".into(), v); }
                if let Some(v) = errpreset { params.insert("errpreset".into(), v); }
                commands::sim::run(&analysis, &params, timeout)
            }
            SimCmd::Measure { expr, analysis } => {
                commands::sim::measure(&analysis, &expr)
            }
            SimCmd::Sweep {
                var,
                from,
                to,
                step,
                measure,
                analysis,
                timeout,
            } => commands::sim::sweep(&var, from, to, step, &analysis, &measure, timeout),
            SimCmd::Corner { file, timeout } => commands::sim::corner(&file, timeout),
            SimCmd::Results => commands::sim::results(),
        },
        Commands::Schema { all, noun, verb } => {
            let schema = if all || noun.is_none() {
                commands::schema::show(None, None)
            } else {
                commands::schema::show(noun.as_deref(), verb.as_deref())
            };
            print_json(&schema);
            return;
        }
    };

    match result {
        Ok(value) => {
            let exit_code = if value
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                exit_codes::DRY_RUN_OK
            } else {
                exit_codes::SUCCESS
            };

            match format {
                OutputFormat::Json => print_json(&value),
                OutputFormat::Table => {
                    if !is_status_cmd {
                        output::print_value(&value, format);
                    }
                }
            }

            std::process::exit(exit_code);
        }
        Err(e) => {
            let cli_error = e.to_cli_error();
            cli_error.print(format);
            std::process::exit(e.exit_code());
        }
    }
}
