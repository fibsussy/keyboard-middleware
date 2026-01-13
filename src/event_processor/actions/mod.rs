//! Action processors for keyboard events
//!
//! This module contains all the specialized processors for different action types:
//! - MT (Mod-Tap): Tap/hold dual-function keys
//! - DT (Double-Tap): Tap dance with single/double-tap detection
//! - OSM (OneShot Modifier): One-shot modifiers that auto-release
//! - SOCD (future): Simultaneous Opposite Cardinal Direction handling

pub mod doubletap;
pub mod modtap;
pub mod oneshot;

// Re-export commonly used types
pub use doubletap::{DtConfig, DtProcessor, DtResolution};
pub use modtap::{MtAction, MtConfig, MtProcessor, MtResolution, RollingStats};
pub use oneshot::{OsmConfig, OsmProcessor, OsmResolution};
