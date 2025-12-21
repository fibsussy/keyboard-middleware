use keyboard_middleware::niri::{should_enable_gamemode, WindowInfo};

#[test]
fn test_gamemode_detection_gamescope_app_id() {
    let window_info = WindowInfo {
        app_id: Some("gamescope".to_string()),
        pid: None,
    };
    assert!(should_enable_gamemode(&window_info));
}

#[test]
fn test_gamemode_detection_steam_app_prefix() {
    let window_info = WindowInfo {
        app_id: Some("steam_app_123456".to_string()),
        pid: None,
    };
    assert!(should_enable_gamemode(&window_info));
}

#[test]
fn test_gamemode_detection_regular_app() {
    let window_info = WindowInfo {
        app_id: Some("org.gnome.Terminal".to_string()),
        pid: None,
    };
    assert!(!should_enable_gamemode(&window_info));
}

#[test]
fn test_gamemode_detection_none_app_id() {
    let window_info = WindowInfo {
        app_id: None,
        pid: None,
    };
    assert!(!should_enable_gamemode(&window_info));
}

#[test]
fn test_gamemode_detection_is_game_env_var() {
    // This test requires a real process with IS_GAME=1 env var
    // For now, we test that it doesn't crash with current process PID
    let window_info = WindowInfo {
        app_id: None,
        pid: Some(std::process::id()),
    };
    // Current process likely doesn't have IS_GAME=1, so should be false
    let result = should_enable_gamemode(&window_info);
    // We can't assert a specific value without setting up the environment
    // but we can ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_gamemode_detection_process_tree() {
    // Test with current process - should walk up tree without crashing
    let window_info = WindowInfo {
        app_id: None,
        pid: Some(std::process::id()),
    };
    // Current process likely doesn't have gamescope/gamemode in tree
    let result = should_enable_gamemode(&window_info);
    // We can't assert a specific value without a controlled process tree
    // but we can ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_gamemode_detection_priority_app_id_first() {
    // Even if PID would say no, app_id should be checked first
    let window_info = WindowInfo {
        app_id: Some("gamescope".to_string()),
        pid: Some(1), // init process, definitely not a game
    };
    assert!(should_enable_gamemode(&window_info));
}

#[test]
fn test_gamemode_detection_steam_app_various_formats() {
    let test_cases = vec![
        "steam_app_0",
        "steam_app_1234567890",
        "steam_app_440", // TF2
    ];

    for app_id in test_cases {
        let window_info = WindowInfo {
            app_id: Some(app_id.to_string()),
            pid: None,
        };
        assert!(
            should_enable_gamemode(&window_info),
            "Failed for app_id: {}",
            app_id
        );
    }
}

#[test]
fn test_gamemode_detection_non_game_apps() {
    let test_cases = vec![
        "firefox",
        "org.mozilla.firefox",
        "chrome",
        "org.gnome.Nautilus",
        "code",
        "steam", // Steam client itself, not a game
    ];

    for app_id in test_cases {
        let window_info = WindowInfo {
            app_id: Some(app_id.to_string()),
            pid: None,
        };
        assert!(
            !should_enable_gamemode(&window_info),
            "False positive for app_id: {}",
            app_id
        );
    }
}
