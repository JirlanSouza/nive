mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nive")]
#[command(about = "CLI for scaffolding and icon management in Nive apps")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new Nive app
    New {
        /// Name of the new app
        name: String,

        /// Scaffold the dashboard template instead of basic
        #[arg(long, default_value_t = false)]
        dashboard: bool,

        /// Git URL for the nive dependency (pre-crates.io alpha)
        #[arg(long)]
        git: Option<String>,

        /// Git tag to use with --git (e.g. v0.1.0-alpha.1)
        #[arg(long)]
        tag: Option<String>,

        /// Exact git revision to use with --git
        #[arg(long)]
        rev: Option<String>,

        /// Git branch to use with --git
        #[arg(long)]
        branch: Option<String>,
    },

    /// Add Nive to the Cargo package in the current directory
    Init {
        /// Use the dashboard template instead of basic
        #[arg(long, default_value_t = false)]
        dashboard: bool,

        /// Git URL for the nive dependency (pre-crates.io alpha)
        #[arg(long)]
        git: Option<String>,

        /// Git tag to use with --git (e.g. v0.1.0-alpha.1)
        #[arg(long)]
        tag: Option<String>,

        /// Exact git revision to use with --git
        #[arg(long)]
        rev: Option<String>,

        /// Git branch to use with --git
        #[arg(long)]
        branch: Option<String>,
    },

    /// Manage app icon manifests, generated modules, and provider discovery
    Icons {
        #[command(subcommand)]
        command: commands::icons::IconsCommands,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            dashboard,
            git,
            tag,
            rev,
            branch,
        } => commands::new::run(
            &name,
            dashboard,
            git.as_deref(),
            tag.as_deref(),
            rev.as_deref(),
            branch.as_deref(),
        ),
        Commands::Init {
            dashboard,
            git,
            tag,
            rev,
            branch,
        } => commands::init::run(
            dashboard,
            git.as_deref(),
            tag.as_deref(),
            rev.as_deref(),
            branch.as_deref(),
        ),
        Commands::Icons { command } => commands::icons::run(command),
    }
}
