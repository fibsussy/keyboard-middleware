use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::Password;
use std::path::PathBuf;

use crate::config::Config;
use crate::daemon::Daemon;
use crate::ipc::{self, IpcRequest, IpcResponse};

#[derive(Parser)]
#[command(name = "keyboard-middleware")]
#[command(about = "Keyboard middleware with home row mods, SOCD, and game mode")]
pub struct Cli {
    /// Hidden daemon flag for systemd service (use 'start' command instead)
    #[arg(long, hide = true)]
    pub daemon: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Check daemon status
    Status,
    /// List all keyboards
    List,
    /// Enable/disable keyboards interactively
    Toggle,
    /// Set password for nav+backspace password typer
    SetPassword,
}

pub fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("keyboard-middleware").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

pub fn handle_set_password() -> Result<()> {
    use console::style;

    println!("Setting password for nav+backspace password typer\n");

    let password = Password::new()
        .with_prompt("Enter password")
        .with_confirmation("Confirm password", "Passwords don't match")
        .interact()?;

    // Load existing config or create default
    let config_path = get_config_path();
    let mut config = Config::load_or_default(&config_path);

    // Update password
    config.password = Some(password);

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    config.save(&config_path)?;

    println!("\n✓ Password saved to {}", config_path.display());
    println!("\n{}", style("Note: The daemon must be restarted for this change to take effect.").yellow());

    Ok(())
}

pub fn handle_start() -> Result<()> {
    let config_path = get_config_path();
    let config = Config::load_or_default(&config_path);

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut daemon = Daemon::new(config, config_path)?;
    daemon.run()?;

    Ok(())
}

pub fn handle_stop() -> Result<()> {
    ipc::send_request(&IpcRequest::Shutdown)?;
    println!("Daemon stopped");
    Ok(())
}

pub fn handle_status() -> Result<()> {
    use console::style;

    match ipc::send_request(&IpcRequest::Ping) {
        Ok(_) => {
            println!("{}", style("✓ Daemon is running").green());
            Ok(())
        }
        Err(_) => {
            println!("{}", style("✗ Daemon is not running").red());
            std::process::exit(1);
        }
    }
}

pub fn handle_list() -> Result<()> {
    use console::style;

    let response = ipc::send_request(&IpcRequest::ListKeyboards)?;
    let keyboards = match response {
        IpcResponse::KeyboardList(kbds) => kbds,
        _ => return Err(anyhow::anyhow!("Unexpected response")),
    };

    if keyboards.is_empty() {
        println!("No keyboards detected");
        return Ok(());
    }

    println!("\n{}", style("Detected Keyboards:").bold());
    println!("{}", style("─".repeat(60)).dim());

    for (i, kbd) in keyboards.iter().enumerate() {
        let status = if kbd.enabled {
            style("enabled").green()
        } else {
            style("disabled").red()
        };

        let connection = if kbd.connected {
            style("connected").green()
        } else {
            style("disconnected").dim()
        };

        println!(
            "\n{}. {} {} [{}]",
            i + 1,
            style(&kbd.name).bold(),
            status,
            connection
        );
        println!("   ID: {}", kbd.hardware_id);
        println!("   Path: {}", kbd.device_path);
    }

    println!();
    Ok(())
}

pub fn handle_toggle() -> Result<()> {
    use console::style;
    use dialoguer::MultiSelect;

    let response = ipc::send_request(&IpcRequest::ListKeyboards)?;
    let keyboards = match response {
        IpcResponse::KeyboardList(kbds) => kbds,
        _ => return Err(anyhow::anyhow!("Unexpected response")),
    };

    if keyboards.is_empty() {
        println!("No keyboards detected");
        return Ok(());
    }

    // Show current state
    println!("\n{}", style("━━━ Keyboard Configuration ━━━").bold());
    println!("\nCurrent state:");
    for kbd in &keyboards {
        if kbd.enabled {
            println!("  {} {}", style("●").green(), kbd.name);
        } else {
            println!("  {} {}", style("○").color256(8), kbd.name);
        }
    }

    // Show interactive multi-select
    println!("\n{} (Space=toggle, Enter=confirm):\n", style("Select keyboards to enable").bold());

    let items: Vec<String> = keyboards
        .iter()
        .map(|kb| format!("  {}", kb.name))
        .collect();

    let defaults: Vec<bool> = keyboards.iter().map(|kb| kb.enabled).collect();

    let selections = MultiSelect::new()
        .items(&items)
        .defaults(&defaults)
        .interact()?;

    // Apply changes
    println!();
    let mut changes_made = false;

    for (i, kbd) in keyboards.iter().enumerate() {
        let should_enable = selections.contains(&i);
        let currently_enabled = kbd.enabled;

        if should_enable && !currently_enabled {
            // Enabling a keyboard
            print!("  {} Enabling {}... ", style("●").green(), kbd.name);
            std::io::Write::flush(&mut std::io::stdout())?;
            let response = ipc::send_request(&IpcRequest::EnableKeyboard(kbd.hardware_id.clone()))?;
            match response {
                IpcResponse::Ok => {
                    println!("{}", style("✓").green());
                    changes_made = true;
                }
                _ => println!("{}", style("✗ Failed").red()),
            }
        } else if !should_enable && currently_enabled {
            // Disabling a keyboard
            print!("  {} Disabling {}... ", style("○").color256(8), kbd.name);
            std::io::Write::flush(&mut std::io::stdout())?;
            let response = ipc::send_request(&IpcRequest::DisableKeyboard(kbd.hardware_id.clone()))?;
            match response {
                IpcResponse::Ok => {
                    println!("{}", style("✓").color256(8));
                    changes_made = true;
                }
                _ => println!("{}", style("✗ Failed").red()),
            }
        }
    }

    if changes_made {
        println!("\n{}", style("✓ Configuration updated successfully").green());
    } else {
        println!("\n{}", style("No changes made").color256(8));
    }
    println!();

    Ok(())
}
