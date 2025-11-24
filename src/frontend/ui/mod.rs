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

//! UI module
//!
//! This module provides the debug UI with egui: menu bar, debug panels, status bar, and file dialogs.

pub mod panels;

use crate::core::system::System;
use crate::frontend::frame_timer::FrameTimer;

/// UI state management
///
/// Manages visibility and state for all debug UI panels.
pub struct UiState {
    /// Show CPU debug panel
    pub show_cpu_panel: bool,
    /// Show GPU debug panel
    pub show_gpu_panel: bool,
    /// Show memory inspector
    pub show_memory_panel: bool,
    /// Show input configuration
    pub show_input_config: bool,
    /// Memory address input (hex string)
    pub memory_address_input: String,
    /// Show about dialog
    pub show_about: bool,
    /// Show key bindings dialog
    pub show_key_bindings: bool,
}

impl UiState {
    /// Create a new UiState with default values
    pub fn new() -> Self {
        Self {
            show_cpu_panel: false,
            show_gpu_panel: false,
            show_memory_panel: false,
            show_input_config: false,
            memory_address_input: String::from("0x00000000"),
            show_about: false,
            show_key_bindings: false,
        }
    }

    /// Render the complete UI
    ///
    /// This renders the menu bar, status bar, and all enabled debug panels.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        system: &System,
        frame_timer: &FrameTimer,
        paused: bool,
    ) -> UiAction {
        let mut action = UiAction::None;

        // Render menu bar
        action = action.merge(panels::menu_bar::render_menu_bar(ctx, self, paused));

        // Render status bar
        panels::status_bar::render_status_bar(ctx, system, frame_timer, paused);

        // Render CPU panel if enabled
        if self.show_cpu_panel {
            panels::cpu_panel::render_cpu_panel(ctx, system);
        }

        // Render GPU panel if enabled
        if self.show_gpu_panel {
            panels::gpu_panel::render_gpu_panel(ctx, system);
        }

        // Render memory inspector if enabled
        if self.show_memory_panel {
            panels::memory_panel::render_memory_panel(ctx, system, &mut self.memory_address_input);
        }

        // Render about dialog if enabled
        if self.show_about {
            self.render_about_dialog(ctx);
        }

        // Render key bindings dialog if enabled
        if self.show_key_bindings {
            self.render_key_bindings_dialog(ctx);
        }

        // Transparent central panel to show PSX display
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |_ui| {
                // Empty - PSX display is behind egui
            });

        action
    }

    /// Render the about dialog
    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("About PSRX")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("PSRX - PlayStation Emulator");
                ui.separator();
                ui.label("Version 0.1.0");
                ui.label("Copyright 2025 itsakeyfut");
                ui.separator();
                ui.label("A PlayStation (PSX) emulator written in Rust");
                ui.label("implementing the Sony PlayStation hardware.");
                ui.separator();
                ui.label("Licensed under the Apache License, Version 2.0");
                ui.separator();
                if ui.button("Close").clicked() {
                    self.show_about = false;
                }
            });
    }

    /// Render the key bindings dialog
    fn render_key_bindings_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Key Bindings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Emulator Controls");
                ui.separator();

                ui.label("Space:  Pause/Resume");
                ui.label("F5:     Reset");
                ui.label("F10:    Step Frame (when paused)");
                ui.label("F11:    Toggle Fullscreen");
                ui.label("F12:    Screenshot (not yet implemented)");

                ui.separator();
                ui.heading("Debug Panels");
                ui.label("Use the View menu to toggle debug panels");

                ui.separator();
                if ui.button("Close").clicked() {
                    self.show_key_bindings = false;
                }
            });
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    /// No action
    None,
    /// Toggle pause/resume
    TogglePause,
    /// Step one frame
    StepFrame,
    /// Reset the system
    Reset,
    /// Toggle fullscreen
    ToggleFullscreen,
    /// Load BIOS file
    LoadBios,
    /// Load disc image
    LoadDisc,
    /// Exit the application
    Exit,
    /// Enable CPU tracing
    EnableCpuTracing,
    /// Dump VRAM to file
    DumpVram,
    /// Toggle input configuration
    ToggleInputConfig,
}

impl UiAction {
    /// Merge two actions, preferring non-None actions
    pub fn merge(self, other: UiAction) -> UiAction {
        match (self, other) {
            (UiAction::None, action) => action,
            (action, UiAction::None) => action,
            (action, _) => action, // First action takes precedence
        }
    }
}
