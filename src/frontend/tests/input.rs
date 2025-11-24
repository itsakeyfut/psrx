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

//! Unit tests for InputHandler

use crate::core::controller::buttons;
use crate::frontend::input::{string_to_keycode, InputConfig, InputHandler};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Helper function to create an InputHandler with default config for testing.
/// This bypasses file I/O to ensure tests are deterministic regardless of local config files.
fn new_test_handler() -> InputHandler {
    let config = InputConfig::default_config();
    let mut key_mapping = HashMap::new();
    for (key_str, button) in config.key_mapping {
        if let Some(key) = string_to_keycode(&key_str) {
            key_mapping.insert(key, button);
        }
    }
    InputHandler {
        key_mapping,
        config_path: "test-config.toml".to_string(),
    }
}

#[test]
fn test_default_keyboard_mapping() {
    let handler = new_test_handler();

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
    let mut handler = new_test_handler();

    // Change mapping
    handler.set_key_mapping(KeyCode::KeyP, buttons::START);
    assert_eq!(
        handler.handle_keyboard(KeyCode::KeyP, true),
        Some((buttons::START, true))
    );
}

#[test]
fn test_detect_conflicts() {
    let handler = new_test_handler();

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
    let mut handler = new_test_handler();

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
    let handler = new_test_handler();
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
