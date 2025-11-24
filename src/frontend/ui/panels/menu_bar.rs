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

//! Menu bar panel
//!
//! Provides the top menu bar with File, View, Debug, and Help menus.

use crate::frontend::ui::{UiAction, UiState};

/// Render the menu bar
///
/// This renders a top menu bar with File, View, Debug, and Help menus.
/// Returns a UiAction if the user triggered an action.
#[allow(deprecated)] // egui::menu::bar is deprecated but still functional
pub fn render_menu_bar(ctx: &egui::Context, ui_state: &mut UiState, paused: bool) -> UiAction {
    let mut action = UiAction::None;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("Load BIOS...").clicked() {
                    action = UiAction::LoadBios;
                    ui.close();
                }
                if ui.button("Load Disc...").clicked() {
                    action = UiAction::LoadDisc;
                    ui.close();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    action = UiAction::Exit;
                    ui.close();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut ui_state.show_cpu_panel, "CPU Panel");
                ui.checkbox(&mut ui_state.show_gpu_panel, "GPU Panel");
                ui.checkbox(&mut ui_state.show_memory_panel, "Memory Inspector");
                ui.separator();
                if ui.button("Toggle Fullscreen (F11)").clicked() {
                    action = UiAction::ToggleFullscreen;
                    ui.close();
                }
            });

            // Debug menu
            ui.menu_button("Debug", |ui| {
                let pause_text = if paused {
                    "Resume (Space)"
                } else {
                    "Pause (Space)"
                };
                if ui.button(pause_text).clicked() {
                    action = UiAction::TogglePause;
                    ui.close();
                }

                ui.add_enabled_ui(paused, |ui| {
                    if ui.button("Step Frame (F10)").clicked() {
                        action = UiAction::StepFrame;
                        ui.close();
                    }
                });

                if ui.button("Reset (F5)").clicked() {
                    action = UiAction::Reset;
                    ui.close();
                }

                ui.separator();

                if ui.button("Enable CPU Tracing").clicked() {
                    action = UiAction::EnableCpuTracing;
                    ui.close();
                }

                if ui.button("Dump VRAM to File").clicked() {
                    action = UiAction::DumpVram;
                    ui.close();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("Key Bindings").clicked() {
                    ui_state.show_key_bindings = true;
                    ui.close();
                }
                if ui.button("About").clicked() {
                    ui_state.show_about = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Documentation (README)").clicked() {
                    // Open README in browser or external viewer
                    if let Err(e) = open::that("https://github.com/itsakeyfut/psrx") {
                        log::error!("Failed to open documentation: {}", e);
                    }
                    ui.close();
                }
            });
        });
    });

    action
}
