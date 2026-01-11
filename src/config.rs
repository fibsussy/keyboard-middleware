use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// QMK-inspired keycode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum KeyCode {
    // Letters
    KC_A,
    KC_B,
    KC_C,
    KC_D,
    KC_E,
    KC_F,
    KC_G,
    KC_H,
    KC_I,
    KC_J,
    KC_K,
    KC_L,
    KC_M,
    KC_N,
    KC_O,
    KC_P,
    KC_Q,
    KC_R,
    KC_S,
    KC_T,
    KC_U,
    KC_V,
    KC_W,
    KC_X,
    KC_Y,
    KC_Z,

    // Numbers
    KC_1,
    KC_2,
    KC_3,
    KC_4,
    KC_5,
    KC_6,
    KC_7,
    KC_8,
    KC_9,
    KC_0,

    // Modifiers
    KC_LCTL,
    KC_LSFT,
    KC_LALT,
    KC_LGUI,
    KC_LCMD, // KC_LCMD is alias for KC_LGUI
    KC_RCTL,
    KC_RSFT,
    KC_RALT,
    KC_RGUI,
    KC_RCMD, // KC_RCMD is alias for KC_RGUI

    // Special keys
    KC_ESC,
    KC_CAPS,
    KC_TAB,
    KC_SPC,
    KC_ENT,
    KC_BSPC,
    KC_DEL,
    KC_GRV,
    KC_MINS,
    KC_EQL,
    KC_LBRC,
    KC_RBRC,
    KC_BSLS,
    KC_SCLN,
    KC_QUOT,
    KC_COMM,
    KC_DOT,
    KC_SLSH,

    // Arrow keys
    KC_LEFT,
    KC_DOWN,
    KC_UP,
    KC_RGHT,

    // Function keys
    KC_F1,
    KC_F2,
    KC_F3,
    KC_F4,
    KC_F5,
    KC_F6,
    KC_F7,
    KC_F8,
    KC_F9,
    KC_F10,
    KC_F11,
    KC_F12,
    KC_F13,
    KC_F14,
    KC_F15,
    KC_F16,
    KC_F17,
    KC_F18,
    KC_F19,
    KC_F20,
    KC_F21,
    KC_F22,
    KC_F23,
    KC_F24,

    // Navigation keys
    KC_PGUP,
    KC_PGDN,
    KC_HOME,
    KC_END,
    KC_INS,
    KC_PSCR,

    // Numpad keys
    KC_KP_0,
    KC_KP_1,
    KC_KP_2,
    KC_KP_3,
    KC_KP_4,
    KC_KP_5,
    KC_KP_6,
    KC_KP_7,
    KC_KP_8,
    KC_KP_9,
    KC_KP_SLASH,
    KC_KP_ASTERISK,
    KC_KP_MINUS,
    KC_KP_PLUS,
    KC_KP_ENTER,
    KC_KP_DOT,
    KC_NUM_LOCK,

    // Media keys
    KC_MUTE,
    KC_VOL_UP,
    KC_VOL_DN,
    KC_MEDIA_PLAY_PAUSE,
    KC_MEDIA_STOP,
    KC_MEDIA_NEXT_TRACK,
    KC_MEDIA_PREV_TRACK,
    KC_MEDIA_SELECT,

    // System keys
    KC_PWR,
    KC_SLEP,
    KC_WAKE,
    KC_CALC,
    KC_MY_COMP,
    KC_WWW_SEARCH,
    KC_WWW_HOME,
    KC_WWW_BACK,
    KC_WWW_FORWARD,
    KC_WWW_STOP,
    KC_WWW_REFRESH,
    KC_WWW_FAVORITES,

    // Locking keys
    KC_SCRL,
    KC_PAUS,

    // Application keys
    KC_APP,
    KC_MENU,

    // Multimedia keys
    KC_BRIU,
    KC_BRID,
    KC_DISPLAY_OFF,
    KC_WLAN,
    KC_BLUETOOTH,
    KC_KEYBOARD_LAYOUT,

    // International keys
    KC_INTL_BACKSLASH,
    KC_INTL_YEN,
    KC_INTL_RO,
}

/// Layer identifier - fully generic string-based layers
/// "base" and "game_mode" are reserved layer names
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Layer(pub String);

impl Layer {
    /// Base layer (always exists)
    pub fn base() -> Self {
        Layer("base".to_string())
    }

    /// Check if this is the base layer
    pub fn is_base(&self) -> bool {
        self.0 == "base"
    }

    /// Create a new layer from string
    pub fn new(name: impl Into<String>) -> Self {
        Layer(name.into())
    }
}

/// Key action - what happens when a key is pressed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Direct key mapping
    Key(KeyCode),
    /// Homerow mod: tap for key, hold for modifier (works on ANY key)
    HR(KeyCode, KeyCode),
    /// Simple overload: tap for key, hold for modifier (no permissive hold, works on ANY key)
    OVERLOAD(KeyCode, KeyCode),
    /// Switch to layer
    TO(Layer),
    /// SOCD (Simultaneous Opposite Cardinal Direction) - fully generic
    /// Format: Socd { this_key, opposing_key }
    /// Example: Socd { this_key: KC_W, opposing_key: KC_S }
    Socd {
        this_key: KeyCode,
        opposing_key: KeyCode,
    },
    /// Type password from file (parameter is the identifier for the password file)
    /// File path: ~/.config/keyboard-middleware/password_{id}.txt
    /// Use Password("default") for ~/.config/keyboard-middleware/password_default.txt
    Password(String),
}

/// Game mode detection methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    GamescopeAppId,
    SteamAppPrefix,
    IsGameEnvVar,
    ProcessTreeWalk,
}

/// Layer configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerConfig {
    pub remaps: HashMap<KeyCode, Action>,
}

/// Game mode configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMode {
    pub remaps: HashMap<KeyCode, Action>,
}

impl GameMode {
    #[must_use]
    pub const fn auto_detect_enabled() -> bool {
        true
    }

    #[must_use]
    pub fn detection_methods() -> Vec<DetectionMethod> {
        vec![
            DetectionMethod::GamescopeAppId,
            DetectionMethod::SteamAppPrefix,
            DetectionMethod::IsGameEnvVar,
            DetectionMethod::ProcessTreeWalk,
        ]
    }

    #[must_use]
    pub const fn process_tree_depth() -> u32 {
        10
    }
}

/// Per-keyboard override configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardOverride {
    pub keymap: Option<KeymapOverride>,
    pub settings: Option<SettingsOverride>,
}

/// Keymap override - specify which layers/remaps to override
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeymapOverride {
    pub base_remaps: Option<HashMap<KeyCode, Action>>,
    pub layers: Option<HashMap<Layer, LayerConfig>>,
    pub game_mode_remaps: Option<HashMap<KeyCode, Action>>,
}

/// Settings override
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsOverride {
    pub tapping_term_ms: Option<u32>,
    pub double_tap_window_ms: Option<u32>,
}

/// Password configuration (stored separately for security)
pub struct Passwords;

impl Passwords {
    /// Load password from separate file by ID (just plain text, no RON)
    #[allow(clippy::missing_errors_doc)]
    pub fn load(id: &str) -> anyhow::Result<Option<String>> {
        let path = Self::path_for_id(id)?;

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let trimmed = content.trim();

        // If file is empty, treat as no password
        if trimmed.is_empty() {
            return Ok(None);
        }

        Ok(Some(trimmed.to_string()))
    }

    /// Get password file path for a specific ID
    #[allow(clippy::missing_errors_doc)]
    pub fn path_for_id(id: &str) -> anyhow::Result<std::path::PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Failed to get config dir"))?;
        Ok(config_dir
            .join("keyboard-middleware")
            .join(format!("password_{}.txt", id)))
    }

    /// Get default password file path (for backward compatibility)
    #[allow(clippy::missing_errors_doc)]
    pub fn default_path() -> anyhow::Result<std::path::PathBuf> {
        Self::path_for_id("default")
    }
}

/// Main configuration structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub tapping_term_ms: u32,
    pub double_tap_window_ms: Option<u32>,
    pub enabled_keyboards: Option<Vec<String>>,
    pub remaps: HashMap<KeyCode, Action>,
    pub layers: HashMap<Layer, LayerConfig>,
    pub game_mode: GameMode,
    pub keyboard_overrides: HashMap<String, KeyboardOverride>,
}

impl Config {
    /// Load config from RON file
    #[allow(clippy::missing_errors_doc)]
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = ron::from_str(&content)?;
        Ok(config)
    }

    /// Save config to RON file
    #[allow(clippy::missing_errors_doc)]
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let pretty = ron::ser::PrettyConfig::default();
        let content = ron::ser::to_string_pretty(self, pretty)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get default config path
    #[allow(clippy::missing_errors_doc)]
    pub fn default_path() -> anyhow::Result<std::path::PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Failed to get config dir"))?;
        Ok(config_dir.join("keyboard-middleware").join("config.ron"))
    }

    /// Get effective config for a specific keyboard
    #[must_use]
    pub fn for_keyboard(&self, keyboard_id: &str) -> Self {
        let mut config = self.clone();

        if let Some(override_cfg) = self.keyboard_overrides.get(keyboard_id) {
            // Apply settings overrides
            if let Some(settings) = &override_cfg.settings {
                if let Some(term) = settings.tapping_term_ms {
                    config.tapping_term_ms = term;
                }
                if let Some(window) = settings.double_tap_window_ms {
                    config.double_tap_window_ms = Some(window);
                }
            }

            // Apply keymap overrides
            if let Some(keymap) = &override_cfg.keymap {
                if let Some(base) = &keymap.base_remaps {
                    config.remaps.clone_from(base);
                }
                if let Some(layers) = &keymap.layers {
                    config.layers.clone_from(layers);
                }
                if let Some(game) = &keymap.game_mode_remaps {
                    config.game_mode.remaps.clone_from(game);
                }
            }
        }

        config
    }

    /// Save only `enabled_keyboards` field, preserving rest of file
    #[allow(clippy::missing_errors_doc)]
    pub fn save_enabled_keyboards_only(&self, path: &std::path::Path) -> anyhow::Result<()> {
        // Just use the working save() method - no need for complex text surgery
        // The original implementation had an off-by-one error that corrupted configs
        self.save(path)
    }
}
