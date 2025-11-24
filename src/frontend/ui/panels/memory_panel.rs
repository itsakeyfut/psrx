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

//! Memory inspector panel
//!
//! Provides a hex dump viewer for examining memory contents.

use crate::core::system::System;

/// Render the memory inspector panel
///
/// Shows a hex dump of memory at the specified address.
pub fn render_memory_panel(ctx: &egui::Context, system: &System, address_input: &mut String) {
    egui::Window::new("Memory Inspector")
        .default_width(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Address input
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(address_input);

                if ui.button("Go").clicked() {
                    // Force update by doing nothing (the hex dump will update automatically)
                }
            });

            ui.separator();

            // Parse address
            let address = parse_address(address_input);

            match address {
                Some(addr) => {
                    // Hex dump
                    egui::ScrollArea::vertical()
                        .id_salt("memory_hex_dump")
                        .show(ui, |ui| {
                            render_hex_dump(ui, system, addr);
                        });
                }
                None => {
                    ui.colored_label(
                        egui::Color32::RED,
                        "Invalid address format. Use hex (0x1234) or decimal.",
                    );
                }
            }
        });
}

/// Parse address from string input
///
/// Supports both hexadecimal (with 0x prefix) and decimal formats.
fn parse_address(input: &str) -> Option<u32> {
    let input = input.trim();

    if let Some(hex_str) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        // Hexadecimal
        u32::from_str_radix(hex_str, 16).ok()
    } else {
        // Try decimal
        input.parse::<u32>().ok()
    }
}

/// Render hex dump of memory
///
/// Shows 16 rows of 16 bytes each (256 bytes total).
fn render_hex_dump(ui: &mut egui::Ui, system: &System, base_addr: u32) {
    let bus = system.bus();

    // Display 16 rows of 16 bytes
    for row in 0..16 {
        let row_addr = base_addr.wrapping_add(row * 16);

        ui.horizontal(|ui| {
            // Address
            ui.label(format!("0x{:08X}:", row_addr));

            // Hex bytes
            let mut bytes = Vec::new();
            for i in 0..16 {
                let addr = row_addr.wrapping_add(i);
                match bus.read8(addr) {
                    Ok(byte) => {
                        ui.label(format!("{:02X}", byte));
                        bytes.push(byte);
                    }
                    Err(_) => {
                        ui.colored_label(egui::Color32::RED, "??");
                        bytes.push(0);
                    }
                }
            }

            // ASCII representation
            ui.separator();
            let ascii_str: String = bytes
                .iter()
                .map(|&b| {
                    if (0x20..=0x7E).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            ui.label(ascii_str);
        });
    }
}
