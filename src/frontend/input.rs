// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 itsakeyfut
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Input handling system for keyboard and gamepad
//!
//! This module provides mapping from keyboard (and optionally gamepad) inputs
//! to PlayStation controller buttons.

use crate::core::controller::buttons;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Input configuration that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Keyboard to button mapping
    pub key_mapping: HashMap<String, u16>,
}

impl InputConfig {
    /// Create default configuration
    pub fn default_config() -> Self {
        let mut key_mapping = HashMap::new();

        // Convert KeyCode to String for serialization
        // D-Pad
        key_mapping.insert(keycode_to_string(KeyCode::KeyW), buttons::UP);
        key_mapping.insert(keycode_to_string(KeyCode::ArrowUp), buttons::UP);
        key_mapping.insert(keycode_to_string(KeyCode::KeyS), buttons::DOWN);
        key_mapping.insert(keycode_to_string(KeyCode::ArrowDown), buttons::DOWN);
        key_mapping.insert(keycode_to_string(KeyCode::KeyA), buttons::LEFT);
        key_mapping.insert(keycode_to_string(KeyCode::ArrowLeft), buttons::LEFT);
        key_mapping.insert(keycode_to_string(KeyCode::KeyD), buttons::RIGHT);
        key_mapping.insert(keycode_to_string(KeyCode::ArrowRight), buttons::RIGHT);

        // Face buttons
        key_mapping.insert(keycode_to_string(KeyCode::KeyI), buttons::TRIANGLE);
        key_mapping.insert(keycode_to_string(KeyCode::KeyZ), buttons::TRIANGLE);
        key_mapping.insert(keycode_to_string(KeyCode::KeyL), buttons::CIRCLE);
        key_mapping.insert(keycode_to_string(KeyCode::KeyC), buttons::CIRCLE);
        key_mapping.insert(keycode_to_string(KeyCode::KeyK), buttons::CROSS);
        key_mapping.insert(keycode_to_string(KeyCode::KeyX), buttons::CROSS);
        key_mapping.insert(keycode_to_string(KeyCode::KeyJ), buttons::SQUARE);
        key_mapping.insert(keycode_to_string(KeyCode::KeyV), buttons::SQUARE);

        // Shoulder buttons
        key_mapping.insert(keycode_to_string(KeyCode::KeyQ), buttons::L1);
        key_mapping.insert(keycode_to_string(KeyCode::KeyE), buttons::R1);
        key_mapping.insert(keycode_to_string(KeyCode::Digit1), buttons::L2);
        key_mapping.insert(keycode_to_string(KeyCode::Digit3), buttons::R2);

        // Start/Select
        key_mapping.insert(keycode_to_string(KeyCode::Enter), buttons::START);
        key_mapping.insert(keycode_to_string(KeyCode::ShiftLeft), buttons::SELECT);
        key_mapping.insert(keycode_to_string(KeyCode::ShiftRight), buttons::SELECT);

        Self { key_mapping }
    }

    /// Load configuration from TOML file
    pub fn load(path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Save configuration to TOML file
    pub fn save(&self, path: &str) -> Result<(), String> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, contents).map_err(|e| format!("Failed to write config file: {}", e))
    }
}

/// Convert KeyCode to a string representation for serialization
fn keycode_to_string(key: KeyCode) -> String {
    format!("{:?}", key)
}

/// Convert string representation back to KeyCode
fn string_to_keycode(s: &str) -> Option<KeyCode> {
    // This is a bit hacky but works for now
    // We use the Debug format which matches the enum variant names
    match s {
        "KeyW" => Some(KeyCode::KeyW),
        "KeyA" => Some(KeyCode::KeyA),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyZ" => Some(KeyCode::KeyZ),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyE" => Some(KeyCode::KeyE),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit3" => Some(KeyCode::Digit3),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "Enter" => Some(KeyCode::Enter),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "Space" => Some(KeyCode::Space),
        "F5" => Some(KeyCode::F5),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}

/// Get button name for display
fn button_name(button: u16) -> &'static str {
    match button {
        buttons::UP => "UP",
        buttons::DOWN => "DOWN",
        buttons::LEFT => "LEFT",
        buttons::RIGHT => "RIGHT",
        buttons::TRIANGLE => "TRIANGLE",
        buttons::CIRCLE => "CIRCLE",
        buttons::CROSS => "CROSS",
        buttons::SQUARE => "SQUARE",
        buttons::L1 => "L1",
        buttons::R1 => "R1",
        buttons::L2 => "L2",
        buttons::R2 => "R2",
        buttons::START => "START",
        buttons::SELECT => "SELECT",
        buttons::L3 => "L3",
        buttons::R3 => "R3",
        _ => "UNKNOWN",
    }
}

/// Input handler for keyboard and gamepad
///
/// Maps keyboard (and optionally gamepad) inputs to PlayStation controller buttons.
/// Supports customizable key bindings and configuration persistence.
pub struct InputHandler {
    /// Keyboard to button mapping (KeyCode -> PSX button bit)
    key_mapping: HashMap<KeyCode, u16>,

    /// Configuration path
    config_path: String,
}

impl InputHandler {
    /// Create a new input handler with default key bindings
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::frontend::InputHandler;
    ///
    /// let handler = InputHandler::new();
    /// ```
    pub fn new() -> Self {
        Self::with_config_path("input.toml")
    }

    /// Create a new input handler with a custom config path
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    pub fn with_config_path(config_path: &str) -> Self {
        let config = InputConfig::load(config_path).unwrap_or_else(|e| {
            log::info!("Using default input config (failed to load: {})", e);
            InputConfig::default_config()
        });

        let mut key_mapping = HashMap::new();
        for (key_str, button) in config.key_mapping {
            if let Some(key) = string_to_keycode(&key_str) {
                key_mapping.insert(key, button);
            } else {
                log::warn!("Unknown key code in config: {}", key_str);
            }
        }

        Self {
            key_mapping,
            config_path: config_path.to_string(),
        }
    }

    /// Handle keyboard input
    ///
    /// Converts a keyboard event to a PlayStation button press/release.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code that was pressed/released
    /// * `pressed` - true if the key was pressed, false if released
    ///
    /// # Returns
    ///
    /// Optional tuple of (button bit, pressed state) if the key is mapped
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::frontend::InputHandler;
    /// use winit::keyboard::KeyCode;
    ///
    /// let handler = InputHandler::new();
    /// if let Some((button, pressed)) = handler.handle_keyboard(KeyCode::KeyW, true) {
    ///     // Handle button press
    /// }
    /// ```
    pub fn handle_keyboard(&self, key: KeyCode, pressed: bool) -> Option<(u16, bool)> {
        self.key_mapping.get(&key).map(|&button| (button, pressed))
    }

    /// Set a key mapping
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to map
    /// * `button` - The PlayStation button bit to map to
    pub fn set_key_mapping(&mut self, key: KeyCode, button: u16) {
        self.key_mapping.insert(key, button);
    }

    /// Remove a key mapping
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to remove
    pub fn remove_key_mapping(&mut self, key: KeyCode) {
        self.key_mapping.remove(&key);
    }

    /// Get all key mappings
    ///
    /// # Returns
    ///
    /// Reference to the key mapping HashMap
    pub fn key_mapping(&self) -> &HashMap<KeyCode, u16> {
        &self.key_mapping
    }

    /// Detect key conflicts
    ///
    /// Returns a list of buttons that are mapped to multiple keys.
    /// This is useful for warning users about potential conflicts.
    ///
    /// # Returns
    ///
    /// Vector of (button, keys) tuples where each button maps to multiple keys
    pub fn detect_conflicts(&self) -> Vec<(u16, Vec<KeyCode>)> {
        let mut button_to_keys: HashMap<u16, Vec<KeyCode>> = HashMap::new();

        for (&key, &button) in &self.key_mapping {
            button_to_keys.entry(button).or_default().push(key);
        }

        button_to_keys
            .into_iter()
            .filter(|(_, keys)| keys.len() > 1)
            .collect()
    }

    /// Save current configuration to file
    ///
    /// # Returns
    ///
    /// Result indicating success or error message
    pub fn save_config(&self) -> Result<(), String> {
        let mut key_mapping = HashMap::new();
        for (&key, &button) in &self.key_mapping {
            key_mapping.insert(keycode_to_string(key), button);
        }

        let config = InputConfig { key_mapping };
        config.save(&self.config_path)
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &str {
        &self.config_path
    }

    /// Get a list of all mapped buttons with their keys
    ///
    /// Returns a sorted list of (button name, keys) for display in UI
    pub fn get_button_mappings(&self) -> Vec<(String, Vec<KeyCode>)> {
        let mut button_to_keys: HashMap<u16, Vec<KeyCode>> = HashMap::new();

        for (&key, &button) in &self.key_mapping {
            button_to_keys.entry(button).or_default().push(key);
        }

        let mut result: Vec<_> = button_to_keys
            .into_iter()
            .map(|(button, keys)| (button_name(button).to_string(), keys))
            .collect();

        // Sort by button name for consistent display
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keyboard_mapping() {
        let handler = InputHandler::new();

        // Test D-Pad mappings
        assert_eq!(
            handler.handle_keyboard(KeyCode::KeyW, true),
            Some((buttons::UP, true))
        );
        assert_eq!(
            handler.handle_keyboard(KeyCode::ArrowUp, true),
            Some((buttons::UP, true))
        );
        assert_eq!(
            handler.handle_keyboard(KeyCode::KeyS, true),
            Some((buttons::DOWN, true))
        );
        assert_eq!(
            handler.handle_keyboard(KeyCode::ArrowDown, true),
            Some((buttons::DOWN, true))
        );

        // Test face buttons
        assert_eq!(
            handler.handle_keyboard(KeyCode::KeyX, true),
            Some((buttons::CROSS, true))
        );
        assert_eq!(
            handler.handle_keyboard(KeyCode::KeyC, true),
            Some((buttons::CIRCLE, true))
        );

        // Test unmapped key
        assert_eq!(handler.handle_keyboard(KeyCode::KeyP, true), None);
    }

    #[test]
    fn test_set_key_mapping() {
        let mut handler = InputHandler::new();

        // Change mapping
        handler.set_key_mapping(KeyCode::KeyP, buttons::START);
        assert_eq!(
            handler.handle_keyboard(KeyCode::KeyP, true),
            Some((buttons::START, true))
        );
    }

    #[test]
    fn test_detect_conflicts() {
        let handler = InputHandler::new();

        // Default config has intentional "conflicts" (multiple keys for same button)
        let conflicts = handler.detect_conflicts();

        // Should find conflicts for buttons with multiple keys
        assert!(!conflicts.is_empty());

        // Check that UP has both W and ArrowUp
        let up_conflict = conflicts.iter().find(|(button, _)| *button == buttons::UP);
        assert!(up_conflict.is_some());
        let (_, keys) = up_conflict.unwrap();
        assert!(keys.contains(&KeyCode::KeyW));
        assert!(keys.contains(&KeyCode::ArrowUp));
    }

    #[test]
    fn test_remove_key_mapping() {
        let mut handler = InputHandler::new();

        // Remove a mapping
        handler.remove_key_mapping(KeyCode::KeyW);
        assert_eq!(handler.handle_keyboard(KeyCode::KeyW, true), None);

        // Other mappings should still work
        assert_eq!(
            handler.handle_keyboard(KeyCode::ArrowUp, true),
            Some((buttons::UP, true))
        );
    }

    #[test]
    fn test_get_button_mappings() {
        let handler = InputHandler::new();
        let mappings = handler.get_button_mappings();

        // Should have mappings for all buttons
        assert!(!mappings.is_empty());

        // Find the UP button mapping
        let up_mapping = mappings.iter().find(|(name, _)| name == "UP");
        assert!(up_mapping.is_some());

        let (_, keys) = up_mapping.unwrap();
        assert!(keys.contains(&KeyCode::KeyW));
        assert!(keys.contains(&KeyCode::ArrowUp));
    }

    #[test]
    fn test_config_serialization() {
        let config = InputConfig::default_config();

        // Serialize and deserialize
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: InputConfig = toml::from_str(&toml_str).unwrap();

        // Check that key mappings are preserved
        assert_eq!(config.key_mapping.len(), deserialized.key_mapping.len());
        for (key, button) in &config.key_mapping {
            assert_eq!(deserialized.key_mapping.get(key), Some(button));
        }
    }
}
