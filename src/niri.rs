use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug)]
pub struct WindowInfo {
    pub app_id: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug)]
pub enum NiriEvent {
    WindowFocusChanged(WindowInfo),
}

/// Get the currently focused window's app ID and PID
fn get_focused_window_info() -> WindowInfo {
    let Ok(output) = Command::new("niri")
        .args(["msg", "focused-window"])
        .output() else {
            return WindowInfo { app_id: None, pid: None };
        };

    if !output.status.success() {
        return WindowInfo { app_id: None, pid: None };
    }

    let Ok(text) = String::from_utf8(output.stdout) else {
        return WindowInfo { app_id: None, pid: None };
    };

    let mut app_id = None;
    let mut pid = None;

    // Parse the output looking for: App ID: "something" and PID: 12345
    for line in text.lines() {
        let trimmed = line.trim();

        if let Some(app_id_part) = trimmed.strip_prefix("App ID:") {
            // Extract string between quotes
            let id = app_id_part.trim().trim_matches('"');
            app_id = Some(id.to_string());
        } else if let Some(pid_part) = trimmed.strip_prefix("PID:") {
            // Extract PID number
            if let Ok(pid_num) = pid_part.trim().parse::<u32>() {
                pid = Some(pid_num);
            }
        }
    }

    WindowInfo { app_id, pid }
}

/// Start monitoring niri window focus events
/// Returns immediately after spawning the monitor thread
pub fn start_niri_monitor(tx: Sender<NiriEvent>) {
    thread::spawn(move || {
        loop {
            info!("Starting niri event stream monitor...");
            info!("Watching for gamescope windows...");

            let mut child = match Command::new("niri")
                .args(["msg", "event-stream"])
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    error!("Failed to spawn niri: {}", e);
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };

            let Some(stdout) = child.stdout.take() else {
                error!("Failed to capture niri stdout");
                thread::sleep(Duration::from_secs(5));
                continue;
            };

            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if line.starts_with("Window focus changed:") {
                            let window_info = get_focused_window_info();
                            if let Some(ref app) = window_info.app_id {
                                info!("Focus changed → app_id: {}, pid: {:?}", app, window_info.pid);
                            }
                            if tx.send(NiriEvent::WindowFocusChanged(window_info)).is_err() {
                                error!("Niri monitor: channel closed, exiting");
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading niri event: {}", e);
                        break;
                    }
                }
            }

            error!("Niri event stream ended, restarting in 5 seconds...");
            thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Check if a process has `IS_GAME=1` in its environment
fn check_is_game_env(pid: u32) -> bool {
    let env_path = format!("/proc/{pid}/environ");
    if let Ok(contents) = fs::read(&env_path) {
        // Environment variables are null-separated
        let env_str = String::from_utf8_lossy(&contents);
        for var in env_str.split('\0') {
            if var == "IS_GAME=1" {
                return true;
            }
        }
    }
    false
}

/// Check if a process is running through gamescope, gamemode, or custom-gamescope
/// by examining its command line and parent process chain
fn check_process_tree(process_id: u32) -> (bool, bool) {
    let mut has_gamescope = false;
    let mut has_gamemode = false;
    let mut current_pid = process_id;

    // Walk up the process tree (max 10 levels to avoid infinite loops)
    for _ in 0..10 {
        // Check the command line
        let cmdline_path = format!("/proc/{current_pid}/cmdline");
        if let Ok(contents) = fs::read(&cmdline_path) {
            let cmdline = String::from_utf8_lossy(&contents);
            let cmd_lower = cmdline.to_lowercase();

            // Check for gamescope or custom-gamescope wrapper
            if cmd_lower.contains("gamescope") || cmd_lower.contains("custom-gamescope") {
                has_gamescope = true;
            }
            if cmd_lower.contains("gamemode") {
                has_gamemode = true;
            }
        }

        // Get parent PID
        let stat_path = format!("/proc/{current_pid}/stat");
        let parent_pid = fs::read_to_string(&stat_path)
            .ok()
            .and_then(|stat| {
                // stat format: pid (comm) state ppid ...
                // Find the last ')' to handle process names with spaces/parens
                let parts: Vec<&str> = stat.rsplitn(2, ')').collect();
                if parts.len() == 2 {
                    parts[0].split_whitespace().nth(1)?.parse::<u32>().ok()
                } else {
                    None
                }
            });

        match parent_pid {
            Some(parent) if parent > 1 => current_pid = parent,
            _ => break, // Reached init or invalid PID
        }
    }

    (has_gamescope, has_gamemode)
}

/// Handle niri window change and return whether game mode should be active
/// Checks multiple indicators:
/// 1. App ID is "gamescope"
/// 2. Process has `IS_GAME=1` environment variable
/// 3. Process is running through gamescope, gamemode, or custom-gamescope
pub fn should_enable_gamemode(window_info: &WindowInfo) -> bool {
    // Check app ID first (fastest check)
    if window_info.app_id.as_deref() == Some("gamescope") {
        return true;
    }

    // TODO: Add app-specific game detection here
    // Example: Some("org.vinegarhq.Sober") => return true,

    // If we have a PID, check environment and process tree
    if let Some(pid) = window_info.pid {
        // Check for IS_GAME=1 environment variable
        if check_is_game_env(pid) {
            return true;
        }

        // Check if running through gamescope or gamemode
        let (has_gamescope, has_gamemode) = check_process_tree(pid);
        if has_gamescope || has_gamemode {
            return true;
        }
    }

    false
}
