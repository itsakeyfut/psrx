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

//! GP1 display configuration commands
//!
//! Implements display settings including resolution, area, and video mode.

use super::super::registers::{ColorDepth, HorizontalRes, VerticalRes, VideoMode};
use super::super::GPU;

impl GPU {
    /// GP1(0x03): Display Enable
    ///
    /// Enables or disables the display output.
    ///
    /// # Arguments
    ///
    /// * `value` - Bit 0: 0=Enable, 1=Disable (inverted logic)
    pub(crate) fn gp1_display_enable(&mut self, value: u32) {
        let enabled = (value & 1) == 0;
        self.display_mode.display_disabled = !enabled;
        self.status.display_disabled = !enabled;

        log::debug!("Display {}", if enabled { "enabled" } else { "disabled" });
    }

    /// GP1(0x05): Start of Display Area
    ///
    /// Sets the top-left corner of the display area in VRAM.
    ///
    /// # Arguments
    ///
    /// * `value` - Bits 0-9: X coordinate, Bits 10-18: Y coordinate
    pub(crate) fn gp1_display_area_start(&mut self, value: u32) {
        let x = (value & 0x3FF) as u16;
        let y = ((value >> 10) & 0x1FF) as u16;

        self.display_area.x = x;
        self.display_area.y = y;

        log::debug!("Display area start: ({}, {})", x, y);
    }

    /// GP1(0x06): Horizontal Display Range
    ///
    /// Sets the horizontal display range on screen (scanline timing).
    ///
    /// # Arguments
    ///
    /// * `value` - Bits 0-11: X1 start, Bits 12-23: X2 end
    pub(crate) fn gp1_horizontal_display_range(&mut self, value: u32) {
        let x1 = (value & 0xFFF) as u16;
        let x2 = ((value >> 12) & 0xFFF) as u16;

        // Store as width
        self.display_area.width = x2.saturating_sub(x1);

        log::debug!(
            "Horizontal display range: {} to {} (width: {})",
            x1,
            x2,
            self.display_area.width
        );
    }

    /// GP1(0x07): Vertical Display Range
    ///
    /// Sets the vertical display range on screen (scanline timing).
    ///
    /// # Arguments
    ///
    /// * `value` - Bits 0-9: Y1 start, Bits 10-19: Y2 end
    pub(crate) fn gp1_vertical_display_range(&mut self, value: u32) {
        let y1 = (value & 0x3FF) as u16;
        let y2 = ((value >> 10) & 0x3FF) as u16;

        // Store as height
        self.display_area.height = y2.saturating_sub(y1);

        log::debug!(
            "Vertical display range: {} to {} (height: {})",
            y1,
            y2,
            self.display_area.height
        );
    }

    /// GP1(0x08): Display Mode
    ///
    /// Sets the display mode including resolution, video mode, and color depth.
    ///
    /// # Arguments
    ///
    /// * `value` - Display mode configuration bits:
    ///   - Bits 0-1: Horizontal resolution 1
    ///   - Bit 2: Vertical resolution (0=240, 1=480)
    ///   - Bit 3: Video mode (0=NTSC, 1=PAL)
    ///   - Bit 4: Color depth (0=15bit, 1=24bit)
    ///   - Bit 5: Interlace (0=Off, 1=On)
    ///   - Bit 6: Horizontal resolution 2
    ///   - Bit 7: Reverse flag
    pub(crate) fn gp1_display_mode(&mut self, value: u32) {
        // Horizontal resolution
        let hr1 = (value & 3) as u8;
        let hr2 = ((value >> 6) & 1) as u8;
        self.display_mode.horizontal_res = match (hr2, hr1) {
            (0, 0) => HorizontalRes::R256,
            (0, 1) => HorizontalRes::R320,
            (0, 2) => HorizontalRes::R512,
            (0, 3) => HorizontalRes::R640,
            (1, 0) => HorizontalRes::R368,
            (1, 1) => HorizontalRes::R384,
            (1, _) => HorizontalRes::R368, // Reserved combinations default to 368
            _ => HorizontalRes::R320,
        };

        // Update status register horizontal resolution bits
        self.status.horizontal_res_1 = hr1;
        self.status.horizontal_res_2 = hr2;

        // Vertical resolution
        let vres = ((value >> 2) & 1) != 0;
        self.display_mode.vertical_res = if vres {
            VerticalRes::R480
        } else {
            VerticalRes::R240
        };
        self.status.vertical_res = vres;

        // Video mode (NTSC/PAL)
        let video_mode = ((value >> 3) & 1) != 0;
        self.display_mode.video_mode = if video_mode {
            VideoMode::PAL
        } else {
            VideoMode::NTSC
        };
        self.status.video_mode = video_mode;

        // Color depth
        let color_depth = ((value >> 4) & 1) != 0;
        self.display_mode.display_area_color_depth = if color_depth {
            ColorDepth::C24Bit
        } else {
            ColorDepth::C15Bit
        };
        self.status.display_area_color_depth = color_depth;

        // Interlace
        let interlaced = ((value >> 5) & 1) != 0;
        self.display_mode.interlaced = interlaced;
        self.status.vertical_interlace = interlaced;

        // Reverse flag (rarely used)
        self.status.reverse_flag = ((value >> 7) & 1) != 0;

        log::debug!(
            "Display mode: {:?} {:?} {:?} {:?} interlaced={}",
            self.display_mode.horizontal_res,
            self.display_mode.vertical_res,
            self.display_mode.video_mode,
            self.display_mode.display_area_color_depth,
            interlaced
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gp1_display_enable() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: bit 0: 0=Enable, 1=Disable (inverted logic)

        // Enable display (value = 0)
        gpu.gp1_display_enable(0);
        assert!(!gpu.status.display_disabled);
        assert!(!gpu.display_mode.display_disabled);

        // Disable display (value = 1)
        gpu.gp1_display_enable(1);
        assert!(gpu.status.display_disabled);
        assert!(gpu.display_mode.display_disabled);
    }

    #[test]
    fn test_gp1_display_enable_masks_bit_0() {
        let mut gpu = GPU::new();

        // Test that only bit 0 is used
        gpu.gp1_display_enable(0xFFFFFFFE); // Even = enable
        assert!(!gpu.status.display_disabled);

        gpu.gp1_display_enable(0xFFFFFFFF); // Odd = disable
        assert!(gpu.status.display_disabled);
    }

    #[test]
    fn test_gp1_display_area_start() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Bits 0-9: X, Bits 10-18: Y

        // Test basic coordinates
        gpu.gp1_display_area_start(0x00000000); // X=0, Y=0
        assert_eq!(gpu.display_area.x, 0);
        assert_eq!(gpu.display_area.y, 0);

        // Test maximum valid coordinates
        // X: 10 bits = 0-1023
        // Y: 9 bits = 0-511
        gpu.gp1_display_area_start(0x0007FFFF); // X=1023, Y=511
        assert_eq!(gpu.display_area.x, 1023);
        assert_eq!(gpu.display_area.y, 511);

        // Test mid-range coordinates
        gpu.gp1_display_area_start((200 << 10) | 100); // X=100, Y=200
        assert_eq!(gpu.display_area.x, 100);
        assert_eq!(gpu.display_area.y, 200);
    }

    #[test]
    fn test_gp1_display_area_start_coordinate_masking() {
        let mut gpu = GPU::new();

        // Test that X is masked to 10 bits and Y to 9 bits
        gpu.gp1_display_area_start(0xFFFFFFFF);
        assert_eq!(gpu.display_area.x, 0x3FF); // 10 bits: 1023
        assert_eq!(gpu.display_area.y, 0x1FF); // 9 bits: 511
    }

    #[test]
    fn test_gp1_horizontal_display_range() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Bits 0-11: X1, Bits 12-23: X2
        // Typical NTSC: X1=260h, X2=260h+320×8 (for 320 width)

        let x1 = 0x260; // 608
        let x2 = 0x260 + (320 * 8); // 608 + 2560 = 3168
        gpu.gp1_horizontal_display_range(x1 | (x2 << 12));

        // Width = X2 - X1
        assert_eq!(gpu.display_area.width, (x2 - x1) as u16);
    }

    #[test]
    fn test_gp1_horizontal_display_range_zero_width() {
        let mut gpu = GPU::new();

        // Test X1 == X2 (zero width)
        gpu.gp1_horizontal_display_range(0x123123); // X1=0x123, X2=0x123
        assert_eq!(gpu.display_area.width, 0);
    }

    #[test]
    fn test_gp1_horizontal_display_range_x2_less_than_x1() {
        let mut gpu = GPU::new();

        // Test X2 < X1 (should saturate to 0)
        let x1 = 1000u32;
        let x2 = 500u32;
        gpu.gp1_horizontal_display_range(x1 | (x2 << 12));

        // Width should saturate to 0 (not wrap around)
        assert_eq!(gpu.display_area.width, 0);
    }

    #[test]
    fn test_gp1_vertical_display_range() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Bits 0-9: Y1, Bits 10-19: Y2
        // Typical NTSC: Y1=88h-120, Y2=88h+240

        let y1 = 0x88; // 136
        let y2 = 0x88 + 240; // 376
        gpu.gp1_vertical_display_range(y1 | (y2 << 10));

        // Height = Y2 - Y1
        assert_eq!(gpu.display_area.height, (y2 - y1) as u16);
        assert_eq!(gpu.display_area.height, 240);
    }

    #[test]
    fn test_gp1_vertical_display_range_coordinate_masking() {
        let mut gpu = GPU::new();

        // Test that Y1 and Y2 are masked to 10 bits each
        gpu.gp1_vertical_display_range(0xFFFFFFFF);

        // Y1: bits 0-9 = 0x3FF (1023)
        // Y2: bits 10-19 = 0x3FF (1023)
        // Height = Y2 - Y1 = 0
        assert_eq!(gpu.display_area.height, 0);
    }

    #[test]
    fn test_gp1_display_mode_horizontal_resolution() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Bits 0-1 (hr1) + Bit 6 (hr2) determine resolution

        // 256 pixels: hr2=0, hr1=0
        gpu.gp1_display_mode(0b00000000);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R256);

        // 320 pixels: hr2=0, hr1=1
        gpu.gp1_display_mode(0b00000001);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R320);

        // 512 pixels: hr2=0, hr1=2
        gpu.gp1_display_mode(0b00000010);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R512);

        // 640 pixels: hr2=0, hr1=3
        gpu.gp1_display_mode(0b00000011);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R640);

        // 368 pixels: hr2=1, hr1=0
        gpu.gp1_display_mode(0b01000000);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R368);

        // 384 pixels: hr2=1, hr1=1
        gpu.gp1_display_mode(0b01000001);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R384);
    }

    #[test]
    fn test_gp1_display_mode_vertical_resolution() {
        let mut gpu = GPU::new();

        // Bit 2: Vertical resolution
        // 0 = 240 lines, 1 = 480 lines

        gpu.gp1_display_mode(0b00000000); // 240
        assert_eq!(gpu.display_mode.vertical_res, VerticalRes::R240);
        assert!(!gpu.status.vertical_res);

        gpu.gp1_display_mode(0b00000100); // 480
        assert_eq!(gpu.display_mode.vertical_res, VerticalRes::R480);
        assert!(gpu.status.vertical_res);
    }

    #[test]
    fn test_gp1_display_mode_video_mode() {
        let mut gpu = GPU::new();

        // Bit 3: Video mode
        // 0 = NTSC (60Hz), 1 = PAL (50Hz)

        gpu.gp1_display_mode(0b00000000); // NTSC
        assert_eq!(gpu.display_mode.video_mode, VideoMode::NTSC);
        assert!(!gpu.status.video_mode);

        gpu.gp1_display_mode(0b00001000); // PAL
        assert_eq!(gpu.display_mode.video_mode, VideoMode::PAL);
        assert!(gpu.status.video_mode);
    }

    #[test]
    fn test_gp1_display_mode_color_depth() {
        let mut gpu = GPU::new();

        // Bit 4: Color depth
        // 0 = 15bit, 1 = 24bit

        gpu.gp1_display_mode(0b00000000); // 15bit
        assert_eq!(
            gpu.display_mode.display_area_color_depth,
            ColorDepth::C15Bit
        );
        assert!(!gpu.status.display_area_color_depth);

        gpu.gp1_display_mode(0b00010000); // 24bit
        assert_eq!(
            gpu.display_mode.display_area_color_depth,
            ColorDepth::C24Bit
        );
        assert!(gpu.status.display_area_color_depth);
    }

    #[test]
    fn test_gp1_display_mode_interlace() {
        let mut gpu = GPU::new();

        // Bit 5: Vertical interlace
        // 0 = Off, 1 = On

        gpu.gp1_display_mode(0b00000000); // Off
        assert!(!gpu.display_mode.interlaced);
        assert!(!gpu.status.vertical_interlace);

        gpu.gp1_display_mode(0b00100000); // On
        assert!(gpu.display_mode.interlaced);
        assert!(gpu.status.vertical_interlace);
    }

    #[test]
    fn test_gp1_display_mode_reverse_flag() {
        let mut gpu = GPU::new();

        // Bit 7: Reverse flag (rarely used)

        gpu.gp1_display_mode(0b00000000);
        assert!(!gpu.status.reverse_flag);

        gpu.gp1_display_mode(0b10000000);
        assert!(gpu.status.reverse_flag);
    }

    #[test]
    fn test_gp1_display_mode_combined() {
        let mut gpu = GPU::new();

        // Test typical display mode setting
        // 320×240, NTSC, 15bit, non-interlaced
        // hr1=1 (320), vres=0 (240), pal=0 (NTSC), depth=0 (15bit), interlace=0
        gpu.gp1_display_mode(0b00000001);

        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R320);
        assert_eq!(gpu.display_mode.vertical_res, VerticalRes::R240);
        assert_eq!(gpu.display_mode.video_mode, VideoMode::NTSC);
        assert_eq!(
            gpu.display_mode.display_area_color_depth,
            ColorDepth::C15Bit
        );
        assert!(!gpu.display_mode.interlaced);
    }

    #[test]
    fn test_gp1_display_mode_status_register_sync() {
        let mut gpu = GPU::new();

        // Verify that display mode changes are reflected in status register
        // Bit layout: [7:reverse][6:hr2][5:interlace][4:depth][3:pal][2:vres][1-0:hr1]
        let mode = 0b01111111; // hr2=1, interlace=1, depth=1, pal=1, vres=1, hr1=3

        gpu.gp1_display_mode(mode);

        assert_eq!(gpu.status.horizontal_res_1, 3);
        assert_eq!(gpu.status.horizontal_res_2, 1);
        assert!(gpu.status.vertical_res);
        assert!(gpu.status.video_mode);
        assert!(gpu.status.display_area_color_depth);
        assert!(gpu.status.vertical_interlace);
    }

    #[test]
    fn test_gp1_display_configuration_sequence() {
        let mut gpu = GPU::new();

        // Test typical display configuration sequence
        // Per PSX-SPX, this is what games typically do:

        // 1. Set display area start
        gpu.gp1_display_area_start(0x00000000); // (0, 0)

        // 2. Set horizontal range
        gpu.gp1_horizontal_display_range(0x260 | ((0x260 + 320 * 8) << 12));

        // 3. Set vertical range
        gpu.gp1_vertical_display_range(0x88 | ((0x88 + 240) << 10));

        // 4. Set display mode
        gpu.gp1_display_mode(0b00000001); // 320×240 NTSC

        // 5. Enable display
        gpu.gp1_display_enable(0);

        // Verify final state
        assert_eq!(gpu.display_area.x, 0);
        assert_eq!(gpu.display_area.y, 0);
        assert_eq!(gpu.display_mode.horizontal_res, HorizontalRes::R320);
        assert!(!gpu.display_mode.display_disabled);
    }
}
