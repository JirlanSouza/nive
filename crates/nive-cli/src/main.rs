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
    },

    /// Manage Lucide icons in the current app
    Icons {
        #[command(subcommand)]
        command: commands::icons::IconsCommands,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, dashboard } => commands::new::run(&name, dashboard),
        Commands::Icons { command } => commands::icons::run(command),
    }
}
