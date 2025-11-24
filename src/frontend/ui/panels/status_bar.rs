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

//! Status bar panel
//!
//! Provides the bottom status bar with FPS, frame time, PC, and emulation state.

use crate::core::system::System;
use crate::frontend::frame_timer::FrameTimer;

/// Render the status bar
///
/// Displays real-time performance metrics and emulation state at the bottom of the window.
pub fn render_status_bar(
    ctx: &egui::Context,
    system: &System,
    frame_timer: &FrameTimer,
    paused: bool,
) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // FPS
            ui.label(format!("FPS: {:.1}", frame_timer.fps()));
            ui.separator();

            // Frame time
            ui.label(format!("Frame: {:.2}ms", frame_timer.frame_time_ms()));
            ui.separator();

            // Program counter
            ui.label(format!("PC: 0x{:08X}", system.cpu().pc()));
            ui.separator();

            // Emulation state
            if paused {
                ui.colored_label(egui::Color32::YELLOW, "⏸ PAUSED");
            } else {
                ui.colored_label(egui::Color32::GREEN, "▶ RUNNING");
            }
        });
    });
}
