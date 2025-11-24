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

//! GPU debug panel
//!
//! Displays GPU status, display area, drawing area, and a VRAM minimap viewer.

use crate::core::system::System;

/// Render the GPU debug panel
///
/// Shows GPU status, display configuration, drawing area, and a minimap of VRAM.
pub fn render_gpu_panel(ctx: &egui::Context, system: &System) {
    egui::SidePanel::right("gpu_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("GPU");
            ui.separator();

            let gpu = system.gpu();
            let gpu_borrow = gpu.borrow();

            // GPU Status
            ui.label(format!("Status: 0x{:08X}", gpu_borrow.status()));
            ui.separator();

            // Display Area
            ui.heading("Display Area");
            let display_area = gpu_borrow.display_area();
            ui.label(format!(
                "Position: ({}, {})",
                display_area.x, display_area.y
            ));
            ui.label(format!(
                "Size: {}x{}",
                display_area.width, display_area.height
            ));
            ui.separator();

            // Drawing Area (from draw_area field)
            ui.heading("Drawing Area");
            let draw_area = &gpu_borrow.draw_area;
            ui.label(format!("Top-Left: ({}, {})", draw_area.left, draw_area.top));
            ui.label(format!(
                "Bottom-Right: ({}, {})",
                draw_area.right, draw_area.bottom
            ));
            ui.separator();

            // Drawing Offset
            ui.heading("Drawing Offset");
            let draw_offset = gpu_borrow.draw_offset;
            ui.label(format!("Offset: ({}, {})", draw_offset.0, draw_offset.1));
            ui.separator();

            // VRAM Viewer (minimap)
            ui.heading("VRAM");
            ui.label("1024x512 minimap (4:1 scale)");

            // Create VRAM texture and display it
            render_vram_minimap(ui, ctx, &gpu_borrow.vram);
        });
}

/// Render VRAM minimap
///
/// Converts VRAM from 16-bit 5-5-5 RGB to RGBA8 and displays as a scaled-down image.
fn render_vram_minimap(ui: &mut egui::Ui, ctx: &egui::Context, vram: &[u16]) {
    // VRAM is 1024x512 pixels
    const VRAM_WIDTH: usize = 1024;
    const VRAM_HEIGHT: usize = 512;

    // Scale down for minimap (4:1 downscale = 256x128 display)
    const SCALE: usize = 4;
    const MINI_WIDTH: usize = VRAM_WIDTH / SCALE;
    const MINI_HEIGHT: usize = VRAM_HEIGHT / SCALE;

    // Convert VRAM to RGBA8 with downsampling
    let mut rgba_data = Vec::with_capacity(MINI_WIDTH * MINI_HEIGHT * 4);

    for mini_y in 0..MINI_HEIGHT {
        for mini_x in 0..MINI_WIDTH {
            // Sample the center pixel of each 4x4 block
            let vram_x = mini_x * SCALE + SCALE / 2;
            let vram_y = mini_y * SCALE + SCALE / 2;
            let index = vram_y * VRAM_WIDTH + vram_x;

            let pixel = vram[index];

            // Convert from 16-bit 5-5-5 RGB to RGBA8
            let r = ((pixel & 0x1F) << 3) as u8;
            let g = (((pixel >> 5) & 0x1F) << 3) as u8;
            let b = (((pixel >> 10) & 0x1F) << 3) as u8;
            let a = 255u8;

            rgba_data.push(r);
            rgba_data.push(g);
            rgba_data.push(b);
            rgba_data.push(a);
        }
    }

    // Create egui ColorImage
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([MINI_WIDTH, MINI_HEIGHT], &rgba_data);

    // Create texture handle (cached by name)
    let texture = ctx.load_texture(
        "vram_minimap",
        color_image,
        egui::TextureOptions::NEAREST, // Use nearest neighbor for pixel-perfect display
    );

    // Display the texture
    ui.image(&texture);
}
