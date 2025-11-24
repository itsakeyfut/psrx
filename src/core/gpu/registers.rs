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

//! GPU register type definitions
//!
//! This module contains GPU register-related types including drawing modes,
//! display settings, drawing areas, texture windows, and GPU status flags.

/// Drawing mode configuration
///
/// Specifies how primitives are rendered, including texture mapping settings,
/// transparency mode, and dithering.
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawMode {
    /// Texture page base X coordinate (in units of 64 pixels)
    pub texture_page_x_base: u16,

    /// Texture page base Y coordinate (0 or 256)
    pub texture_page_y_base: u16,

    /// Semi-transparency mode (0-3)
    ///
    /// - 0: 0.5×Back + 0.5×Front (average)
    /// - 1: 1.0×Back + 1.0×Front (additive)
    /// - 2: 1.0×Back - 1.0×Front (subtractive)
    /// - 3: 1.0×Back + 0.25×Front (additive with quarter)
    pub semi_transparency: u8,

    /// Texture color depth (0=4bit, 1=8bit, 2=15bit)
    pub texture_depth: u8,

    /// Dithering enabled
    ///
    /// When enabled, dithers 24-bit colors down to 15-bit for display.
    pub dithering: bool,

    /// Drawing to display area allowed
    pub draw_to_display: bool,

    /// Texture disable (draw solid colors instead of textured)
    pub texture_disable: bool,

    /// Textured rectangle X-flip
    pub texture_x_flip: bool,

    /// Textured rectangle Y-flip
    pub texture_y_flip: bool,
}

/// Drawing area (clipping rectangle)
///
/// Defines the rectangular region in VRAM where drawing operations are allowed.
/// Primitives are clipped to this region.
#[derive(Debug, Clone, Copy)]
pub struct DrawingArea {
    /// Left edge X coordinate (inclusive)
    pub left: u16,

    /// Top edge Y coordinate (inclusive)
    pub top: u16,

    /// Right edge X coordinate (inclusive)
    pub right: u16,

    /// Bottom edge Y coordinate (inclusive)
    pub bottom: u16,
}

impl Default for DrawingArea {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 1023,
            bottom: 511,
        }
    }
}

/// Texture window settings
///
/// Controls texture coordinate wrapping and masking within a specified window.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureWindow {
    /// Texture window mask X (in 8-pixel steps)
    pub mask_x: u8,

    /// Texture window mask Y (in 8-pixel steps)
    pub mask_y: u8,

    /// Texture window offset X (in 8-pixel steps)
    pub offset_x: u8,

    /// Texture window offset Y (in 8-pixel steps)
    pub offset_y: u8,
}

/// Display area configuration
///
/// Defines the region of VRAM that is output to the display.
#[derive(Debug, Clone, Copy)]
pub struct DisplayArea {
    /// Display area X coordinate in VRAM
    pub x: u16,

    /// Display area Y coordinate in VRAM
    pub y: u16,

    /// Display width in pixels
    pub width: u16,

    /// Display height in pixels
    pub height: u16,
}

impl Default for DisplayArea {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 320,
            height: 240,
        }
    }
}

/// Display mode settings
///
/// Controls the display output format including resolution, video mode, and color depth.
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    /// Horizontal resolution
    pub horizontal_res: HorizontalRes,

    /// Vertical resolution
    pub vertical_res: VerticalRes,

    /// Video mode (NTSC/PAL)
    pub video_mode: VideoMode,

    /// Display area color depth
    pub display_area_color_depth: ColorDepth,

    /// Interlaced mode enabled
    pub interlaced: bool,

    /// Display disabled
    pub display_disabled: bool,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self {
            horizontal_res: HorizontalRes::R320,
            vertical_res: VerticalRes::R240,
            video_mode: VideoMode::NTSC,
            display_area_color_depth: ColorDepth::C15Bit,
            interlaced: false,
            display_disabled: true,
        }
    }
}

/// Horizontal resolution modes
///
/// The GPU supports several horizontal resolutions for display output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalRes {
    /// 256 pixels wide
    R256,

    /// 320 pixels wide (most common)
    R320,

    /// 512 pixels wide
    R512,

    /// 640 pixels wide
    R640,

    /// 368 pixels wide (rarely used)
    R368,

    /// 384 pixels wide
    R384,
}

/// Vertical resolution modes
///
/// The GPU supports two vertical resolutions, with different values for NTSC and PAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalRes {
    /// 240 lines (NTSC) or 256 lines (PAL)
    R240,

    /// 480 lines (NTSC interlaced) or 512 lines (PAL interlaced)
    R480,
}

/// Video mode (refresh rate)
///
/// Determines the video timing: NTSC (60Hz) or PAL (50Hz).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    /// NTSC mode: 60Hz refresh rate
    NTSC,

    /// PAL mode: 50Hz refresh rate
    PAL,
}

/// Display color depth
///
/// Specifies the color depth used for display output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// 15-bit color (5-5-5 RGB)
    C15Bit,

    /// 24-bit color (8-8-8 RGB)
    C24Bit,
}

/// GPU status register flags
///
/// Represents all the status bits returned by the GPUSTAT register (0x1F801814).
#[derive(Debug, Clone, Copy)]
pub struct GPUStatus {
    /// Texture page X base (N×64)
    pub texture_page_x_base: u8,

    /// Texture page Y base (0=0, 1=256)
    pub texture_page_y_base: u8,

    /// Semi-transparency mode (0-3)
    pub semi_transparency: u8,

    /// Texture color depth (0=4bit, 1=8bit, 2=15bit)
    pub texture_depth: u8,

    /// Dithering enabled
    pub dithering: bool,

    /// Drawing to display area allowed
    pub draw_to_display: bool,

    /// Set mask bit when drawing
    pub set_mask_bit: bool,

    /// Check mask bit before drawing (don't draw if set)
    pub draw_pixels: bool,

    /// Interlace field (even/odd)
    pub interlace_field: bool,

    /// Reverse flag (used for debugging)
    pub reverse_flag: bool,

    /// Texture disable
    pub texture_disable: bool,

    /// Horizontal resolution 2 (368 mode bit)
    pub horizontal_res_2: u8,

    /// Horizontal resolution 1 (256/320/512/640)
    pub horizontal_res_1: u8,

    /// Vertical resolution (0=240, 1=480)
    pub vertical_res: bool,

    /// Video mode (0=NTSC, 1=PAL)
    pub video_mode: bool,

    /// Display area color depth (0=15bit, 1=24bit)
    pub display_area_color_depth: bool,

    /// Vertical interlace enabled
    pub vertical_interlace: bool,

    /// Display disabled
    pub display_disabled: bool,

    /// Interrupt request
    pub interrupt_request: bool,

    /// DMA request
    pub dma_request: bool,

    /// Ready to receive command
    pub ready_to_receive_cmd: bool,

    /// Ready to send VRAM to CPU
    pub ready_to_send_vram: bool,

    /// Ready to receive DMA block
    pub ready_to_receive_dma: bool,

    /// DMA direction (0=Off, 1=?, 2=CPUtoGP0, 3=GPUREADtoCPU)
    pub dma_direction: u8,

    /// Drawing even/odd lines in interlaced mode
    pub drawing_odd_line: bool,
}

impl Default for GPUStatus {
    fn default() -> Self {
        Self {
            texture_page_x_base: 0,
            texture_page_y_base: 0,
            semi_transparency: 0,
            texture_depth: 0,
            dithering: false,
            draw_to_display: false,
            set_mask_bit: false,
            draw_pixels: false,
            interlace_field: false,
            reverse_flag: false,
            texture_disable: false,
            horizontal_res_2: 0,
            horizontal_res_1: 0,
            vertical_res: false,
            video_mode: false,
            display_area_color_depth: false,
            vertical_interlace: false,
            display_disabled: true,
            interrupt_request: false,
            dma_request: false,
            ready_to_receive_cmd: true,
            ready_to_send_vram: true,
            ready_to_receive_dma: true,
            dma_direction: 0,
            drawing_odd_line: false,
        }
    }
}

/// VRAM transfer direction
///
/// Indicates whether a VRAM transfer is uploading from CPU to VRAM or downloading from VRAM to CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VRAMTransferDirection {
    /// CPU→VRAM transfer (GP0 0xA0)
    CpuToVram,
    /// VRAM→CPU transfer (GP0 0xC0)
    VramToCpu,
}

/// VRAM transfer state
///
/// Tracks the progress of a VRAM transfer operation (CPU-to-VRAM or VRAM-to-CPU).
#[derive(Debug, Clone)]
pub struct VRAMTransfer {
    /// Transfer start X coordinate
    pub x: u16,

    /// Transfer start Y coordinate
    pub y: u16,

    /// Transfer width in pixels
    pub width: u16,

    /// Transfer height in pixels
    pub height: u16,

    /// Current X position in transfer
    pub current_x: u16,

    /// Current Y position in transfer
    pub current_y: u16,

    /// Transfer direction
    pub direction: VRAMTransferDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawing_area_default() {
        let area = DrawingArea::default();
        assert_eq!(area.left, 0);
        assert_eq!(area.top, 0);
        assert_eq!(area.right, 1023);
        assert_eq!(area.bottom, 511);
    }

    #[test]
    fn test_drawing_area_custom() {
        let area = DrawingArea {
            left: 100,
            top: 50,
            right: 640,
            bottom: 480,
        };
        assert_eq!(area.left, 100);
        assert_eq!(area.top, 50);
        assert_eq!(area.right, 640);
        assert_eq!(area.bottom, 480);
    }

    #[test]
    fn test_texture_window_default() {
        let window = TextureWindow::default();
        assert_eq!(window.mask_x, 0);
        assert_eq!(window.mask_y, 0);
        assert_eq!(window.offset_x, 0);
        assert_eq!(window.offset_y, 0);
    }

    #[test]
    fn test_texture_window_masking_formula() {
        // Per PSX-SPX: Texcoord = (Texcoord AND (NOT (Mask*8))) OR ((Offset AND Mask)*8)
        let window = TextureWindow {
            mask_x: 3,      // 3 * 8 = 24 pixel mask
            mask_y: 7,      // 7 * 8 = 56 pixel mask
            offset_x: 2,    // 2 * 8 = 16 pixel offset
            offset_y: 4,    // 4 * 8 = 32 pixel offset
        };

        // Verify mask values are in valid range (0-31 = 5 bits)
        assert!(window.mask_x < 32);
        assert!(window.mask_y < 32);
        assert!(window.offset_x < 32);
        assert!(window.offset_y < 32);
    }

    #[test]
    fn test_display_area_default() {
        let area = DisplayArea::default();
        assert_eq!(area.x, 0);
        assert_eq!(area.y, 0);
        assert_eq!(area.width, 320);
        assert_eq!(area.height, 240);
    }

    #[test]
    fn test_display_mode_default() {
        let mode = DisplayMode::default();
        assert_eq!(mode.horizontal_res, HorizontalRes::R320);
        assert_eq!(mode.vertical_res, VerticalRes::R240);
        assert_eq!(mode.video_mode, VideoMode::NTSC);
        assert_eq!(mode.display_area_color_depth, ColorDepth::C15Bit);
        assert!(!mode.interlaced);
        assert!(mode.display_disabled);
    }

    #[test]
    fn test_draw_mode_default() {
        let mode = DrawMode::default();
        assert_eq!(mode.texture_page_x_base, 0);
        assert_eq!(mode.texture_page_y_base, 0);
        assert_eq!(mode.semi_transparency, 0);
        assert_eq!(mode.texture_depth, 0);
        assert!(!mode.dithering);
        assert!(!mode.draw_to_display);
        assert!(!mode.texture_disable);
        assert!(!mode.texture_x_flip);
        assert!(!mode.texture_y_flip);
    }

    #[test]
    fn test_draw_mode_semi_transparency_modes() {
        // Test that all 4 semi-transparency modes are valid (0-3)
        for mode in 0..4 {
            let draw_mode = DrawMode {
                semi_transparency: mode,
                ..Default::default()
            };
            assert!(draw_mode.semi_transparency < 4);
        }
    }

    #[test]
    fn test_draw_mode_texture_depth_modes() {
        // Test texture depth modes: 0=4bit, 1=8bit, 2/3=15bit
        let mode_4bit = DrawMode {
            texture_depth: 0,
            ..Default::default()
        };
        let mode_8bit = DrawMode {
            texture_depth: 1,
            ..Default::default()
        };
        let mode_15bit = DrawMode {
            texture_depth: 2,
            ..Default::default()
        };

        assert_eq!(mode_4bit.texture_depth, 0);
        assert_eq!(mode_8bit.texture_depth, 1);
        assert_eq!(mode_15bit.texture_depth, 2);
    }

    #[test]
    fn test_draw_mode_texture_page_coordinates() {
        // Texture page X base is in units of 64 pixels
        // Texture page Y base is 0 or 256
        let mode = DrawMode {
            texture_page_x_base: 128, // 2 * 64
            texture_page_y_base: 256, // 1 * 256
            ..Default::default()
        };

        assert_eq!(mode.texture_page_x_base, 128);
        assert_eq!(mode.texture_page_y_base, 256);

        // Valid X bases: 0, 64, 128, 192, 256, 320, 384, 448, 512, 576, 640, 704, 768, 832, 896, 960
        // Valid Y bases: 0, 256
    }

    #[test]
    fn test_gpu_status_default() {
        let status = GPUStatus::default();

        // Verify default ready states
        assert!(status.ready_to_receive_cmd);
        assert!(status.ready_to_send_vram);
        assert!(status.ready_to_receive_dma);

        // Verify display is initially disabled
        assert!(status.display_disabled);

        // Verify no interrupt/DMA requests initially
        assert!(!status.interrupt_request);
        assert!(!status.dma_request);

        // Verify initial DMA direction is off
        assert_eq!(status.dma_direction, 0);
    }

    #[test]
    fn test_gpu_status_dma_direction() {
        // Per PSX-SPX: 0=Off, 1=?, 2=CPUtoGP0, 3=GPUREADtoCPU
        let status_off = GPUStatus {
            dma_direction: 0,
            ..Default::default()
        };
        let status_cpu_to_gp0 = GPUStatus {
            dma_direction: 2,
            ..Default::default()
        };
        let status_gpuread_to_cpu = GPUStatus {
            dma_direction: 3,
            ..Default::default()
        };

        assert_eq!(status_off.dma_direction, 0);
        assert_eq!(status_cpu_to_gp0.dma_direction, 2);
        assert_eq!(status_gpuread_to_cpu.dma_direction, 3);
    }

    #[test]
    fn test_vram_transfer_direction() {
        let cpu_to_vram = VRAMTransferDirection::CpuToVram;
        let vram_to_cpu = VRAMTransferDirection::VramToCpu;

        assert_ne!(cpu_to_vram, vram_to_cpu);
    }

    #[test]
    fn test_vram_transfer_state() {
        let transfer = VRAMTransfer {
            x: 100,
            y: 200,
            width: 64,
            height: 32,
            current_x: 0,
            current_y: 0,
            direction: VRAMTransferDirection::CpuToVram,
        };

        assert_eq!(transfer.x, 100);
        assert_eq!(transfer.y, 200);
        assert_eq!(transfer.width, 64);
        assert_eq!(transfer.height, 32);
        assert_eq!(transfer.current_x, 0);
        assert_eq!(transfer.current_y, 0);
        assert_eq!(transfer.direction, VRAMTransferDirection::CpuToVram);
    }

    #[test]
    fn test_horizontal_res_values() {
        // Test that all horizontal resolution values are distinct
        let resolutions = vec![
            HorizontalRes::R256,
            HorizontalRes::R320,
            HorizontalRes::R512,
            HorizontalRes::R640,
            HorizontalRes::R368,
            HorizontalRes::R384,
        ];

        // Verify each resolution is unique
        for (i, res1) in resolutions.iter().enumerate() {
            for (j, res2) in resolutions.iter().enumerate() {
                if i != j {
                    assert_ne!(res1, res2);
                }
            }
        }
    }

    #[test]
    fn test_vertical_res_values() {
        let r240 = VerticalRes::R240;
        let r480 = VerticalRes::R480;

        assert_ne!(r240, r480);
    }

    #[test]
    fn test_video_mode_values() {
        let ntsc = VideoMode::NTSC;
        let pal = VideoMode::PAL;

        assert_ne!(ntsc, pal);
    }

    #[test]
    fn test_color_depth_values() {
        let c15bit = ColorDepth::C15Bit;
        let c24bit = ColorDepth::C24Bit;

        assert_ne!(c15bit, c24bit);
    }
}
