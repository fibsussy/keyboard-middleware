/// Double-Tap (DT) processor - QMK-inspired tap dance
///
/// Implements double-tap detection with configurable timing:
/// - First tap: Wait for potential second tap (adds latency)
/// - Second tap within window: Execute double-tap action immediately
/// - Timeout: Execute single-tap action
///
/// Follows QMK tap dance behavior:
/// - Accepts latency on single-tap for reliable detection
/// - Double-tap is instant once detected
/// - Per-key tracking with fast HashMap lookups
use crate::config::KeyCode;
use std::collections::HashMap;
use std::time::Instant;

/// State of a double-tap key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtState {
    /// First press, waiting for release
    FirstPress,
    /// Released, waiting for second tap
    WaitingSecondTap,
    /// Second press detected - double-tap!
    DoubleTapDetected,
}

/// Double-tap key tracking
#[derive(Debug, Clone)]
pub struct DtKey {
    /// Physical keycode being tracked
    pub keycode: KeyCode,
    /// Tap output (KeyCode for now, will support Actions later)
    pub tap_key: KeyCode,
    /// Double-tap output (KeyCode for now, will support Actions later)
    pub double_tap_key: KeyCode,
    /// When first press occurred
    pub first_press_at: Instant,
    /// When first release occurred (if released)
    pub first_release_at: Option<Instant>,
    /// Current state
    pub state: DtState,
}

impl DtKey {
    pub fn new(keycode: KeyCode, tap_key: KeyCode, double_tap_key: KeyCode) -> Self {
        Self {
            keycode,
            tap_key,
            double_tap_key,
            first_press_at: Instant::now(),
            first_release_at: None,
            state: DtState::FirstPress,
        }
    }

    /// Time since first press
    pub fn elapsed_since_first_press(&self) -> u128 {
        self.first_press_at.elapsed().as_millis()
    }

    /// Time since first release (if released)
    pub fn elapsed_since_first_release(&self) -> Option<u128> {
        self.first_release_at.map(|t| t.elapsed().as_millis())
    }
}

/// Result of DT processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DtResolution {
    /// Emit single tap (timeout expired)
    SingleTap(KeyCode),
    /// Emit double tap (second tap detected)
    DoubleTap(KeyCode),
    /// Still undecided, waiting
    Undecided,
}

/// Double-Tap processor configuration
#[derive(Debug, Clone)]
pub struct DtConfig {
    /// Time window for double-tap detection (ms)
    pub double_tap_window_ms: u64,
}

impl Default for DtConfig {
    fn default() -> Self {
        Self {
            double_tap_window_ms: 250, // QMK-inspired default
        }
    }
}

/// Double-Tap processor - manages all DT keys
pub struct DtProcessor {
    /// Config
    config: DtConfig,

    /// Currently tracked DT keys
    tracked_keys: HashMap<KeyCode, DtKey>,
}

impl DtProcessor {
    /// Create new DT processor
    pub fn new(config: DtConfig) -> Self {
        Self {
            config,
            tracked_keys: HashMap::new(),
        }
    }

    /// Handle key press - returns resolution if available
    pub fn on_press(
        &mut self,
        keycode: KeyCode,
        tap_key: KeyCode,
        double_tap_key: KeyCode,
    ) -> DtResolution {
        if let Some(dt_key) = self.tracked_keys.get_mut(&keycode) {
            // Second press within window!
            if dt_key.state == DtState::WaitingSecondTap {
                if let Some(elapsed) = dt_key.elapsed_since_first_release() {
                    if elapsed <= self.config.double_tap_window_ms as u128 {
                        // Double-tap detected!
                        dt_key.state = DtState::DoubleTapDetected;
                        return DtResolution::DoubleTap(dt_key.double_tap_key);
                    }
                }
            }

            // Timeout expired, complete previous tap and start new one
            // (This shouldn't normally happen, but handle it gracefully)
            self.tracked_keys.remove(&keycode);
        }

        // First press - start tracking
        let dt_key = DtKey::new(keycode, tap_key, double_tap_key);
        self.tracked_keys.insert(keycode, dt_key);

        DtResolution::Undecided
    }

    /// Handle key release - returns resolution if timeout expired
    pub fn on_release(&mut self, keycode: KeyCode) -> DtResolution {
        if let Some(dt_key) = self.tracked_keys.get_mut(&keycode) {
            match dt_key.state {
                DtState::FirstPress => {
                    // First release - start waiting for second tap
                    dt_key.state = DtState::WaitingSecondTap;
                    dt_key.first_release_at = Some(Instant::now());
                    DtResolution::Undecided
                }
                DtState::DoubleTapDetected => {
                    // Double-tap already emitted, clean up
                    self.tracked_keys.remove(&keycode);
                    DtResolution::Undecided
                }
                _ => DtResolution::Undecided,
            }
        } else {
            DtResolution::Undecided
        }
    }

    /// Check for timeouts and resolve single taps
    /// Call this periodically during event processing
    pub fn check_timeouts(&mut self) -> Vec<(KeyCode, DtResolution)> {
        let mut resolutions = Vec::new();
        let window_ms = self.config.double_tap_window_ms;

        // Find expired keys
        let expired: Vec<KeyCode> = self
            .tracked_keys
            .iter()
            .filter_map(|(keycode, dt_key)| {
                match dt_key.state {
                    DtState::WaitingSecondTap => {
                        // Check if window expired
                        if let Some(elapsed) = dt_key.elapsed_since_first_release() {
                            if elapsed > window_ms as u128 {
                                Some(*keycode)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    DtState::FirstPress => {
                        // If still holding after window, treat as single tap
                        if dt_key.elapsed_since_first_press() > window_ms as u128 {
                            Some(*keycode)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect();

        // Resolve expired keys
        for keycode in expired {
            if let Some(dt_key) = self.tracked_keys.remove(&keycode) {
                resolutions.push((keycode, DtResolution::SingleTap(dt_key.tap_key)));
            }
        }

        resolutions
    }

    /// Get currently tracked keys (for debugging)
    pub fn tracked_count(&self) -> usize {
        self.tracked_keys.len()
    }
}
