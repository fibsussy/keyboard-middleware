use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod daemon;
mod ipc;
mod keyboard_id;
mod keyboard_state;
mod keyboard_thread;
mod niri;
mod process_event_new;
mod socd;
mod uinput;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start => cli::handle_start(),
        Commands::Stop => cli::handle_stop(),
        Commands::Status => cli::handle_status(),
        Commands::List => cli::handle_list(),
        Commands::Toggle => cli::handle_toggle(),
        Commands::SetPassword => cli::handle_set_password(),
    }
}
