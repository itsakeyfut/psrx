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

//! CPU debug panel
//!
//! Displays CPU registers, PC, HI/LO, and disassembly of next instructions.

use crate::core::system::System;

/// Register names for MIPS R3000A
const REGISTER_NAMES: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
    "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp",
    "ra",
];

/// Render the CPU debug panel
///
/// Shows all CPU registers, PC, HI/LO, and disassembly of upcoming instructions.
pub fn render_cpu_panel(ctx: &egui::Context, system: &System) {
    egui::SidePanel::left("cpu_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("CPU");
            ui.separator();

            // Program counter
            ui.label(format!("PC: 0x{:08X}", system.cpu().pc()));
            ui.separator();

            // General purpose registers (in two columns)
            ui.heading("Registers");
            egui::Grid::new("cpu_registers")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    for i in 0..16 {
                        // Left column (r0-r15)
                        let reg_val = system.cpu().reg(i);
                        ui.label(format!("${:2} ({:4})", i, REGISTER_NAMES[i as usize]));
                        ui.label(format!("0x{:08X}", reg_val));

                        // Right column (r16-r31)
                        let reg_val2 = system.cpu().reg(i + 16);
                        ui.label(format!(
                            "${:2} ({:4})",
                            i + 16,
                            REGISTER_NAMES[(i + 16) as usize]
                        ));
                        ui.label(format!("0x{:08X}", reg_val2));

                        ui.end_row();
                    }
                });

            ui.separator();

            // HI/LO registers
            ui.heading("Multiply/Divide");
            ui.label(format!("HI: 0x{:08X}", system.cpu().hi()));
            ui.label(format!("LO: 0x{:08X}", system.cpu().lo()));

            ui.separator();

            // Disassembly section
            ui.heading("Disassembly");
            ui.label("(Next 10 instructions)");

            egui::ScrollArea::vertical()
                .id_salt("cpu_disasm")
                .show(ui, |ui| {
                    render_disassembly(ui, system);
                });
        });
}

/// Render disassembly of next instructions
///
/// Shows the next 10 instructions starting from the current PC.
/// This is a simplified disassembly showing raw instruction words.
fn render_disassembly(ui: &mut egui::Ui, system: &System) {
    let pc = system.cpu().pc();
    let bus = system.bus();

    // Show next 10 instructions
    for i in 0..10 {
        let addr = pc.wrapping_add(i * 4);

        // Read instruction from memory
        match bus.read32(addr) {
            Ok(instruction) => {
                // Highlight current PC
                if i == 0 {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("→ 0x{:08X}: 0x{:08X}", addr, instruction),
                    );
                } else {
                    ui.label(format!("  0x{:08X}: 0x{:08X}", addr, instruction));
                }

                // TODO: Add actual disassembly decoding in future phase
                // For now, just show the raw instruction word
            }
            Err(_) => {
                ui.colored_label(egui::Color32::RED, format!("  0x{:08X}: <invalid>", addr));
            }
        }
    }
}
