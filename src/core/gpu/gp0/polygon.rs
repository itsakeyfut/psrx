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

//! GP0 polygon drawing commands
//!
//! Implements parsing for triangle and quadrilateral rendering commands,
//! including both flat-shaded, Gouraud-shaded, and textured primitives.

use super::super::primitives::{Color, TexCoord, TextureInfo, Vertex};
use super::super::GPU;

impl GPU {
    /// GP0(0x20): Monochrome Triangle (Opaque)
    ///
    /// Renders a flat-shaded triangle with a single color.
    /// Requires 4 words: command+color, vertex1, vertex2, vertex3
    pub(crate) fn parse_monochrome_triangle_opaque(&mut self) {
        if self.command_fifo.len() < 4 {
            return; // Need more words
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];

        self.render_monochrome_triangle(&vertices, &color, false);
    }

    /// GP0(0x22): Monochrome Triangle (Semi-Transparent)
    ///
    /// Renders a flat-shaded triangle with semi-transparency enabled.
    /// Requires 4 words: command+color, vertex1, vertex2, vertex3
    pub(crate) fn parse_monochrome_triangle_semi_transparent(&mut self) {
        if self.command_fifo.len() < 4 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];

        self.render_monochrome_triangle(&vertices, &color, true);
    }

    /// GP0(0x28): Monochrome Quad (Opaque)
    ///
    /// Renders a flat-shaded quadrilateral with a single color.
    /// Requires 5 words: command+color, vertex1, vertex2, vertex3, vertex4
    pub(crate) fn parse_monochrome_quad_opaque(&mut self) {
        if self.command_fifo.len() < 5 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let v4 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
            Vertex::from_u32(v4),
        ];

        self.render_monochrome_quad(&vertices, &color, false);
    }

    /// GP0(0x2A): Monochrome Quad (Semi-Transparent)
    ///
    /// Renders a flat-shaded quadrilateral with semi-transparency enabled.
    /// Requires 5 words: command+color, vertex1, vertex2, vertex3, vertex4
    pub(crate) fn parse_monochrome_quad_semi_transparent(&mut self) {
        if self.command_fifo.len() < 5 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let v4 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
            Vertex::from_u32(v4),
        ];

        self.render_monochrome_quad(&vertices, &color, true);
    }

    /// GP0(0x30): Gouraud-Shaded Triangle (Opaque)
    ///
    /// Renders a triangle with per-vertex colors (Gouraud shading).
    /// Requires 6 words: (color1, vertex1, color2, vertex2, color3, vertex3)
    ///
    /// # Command Format
    ///
    /// ```text
    /// Word 0: 0x30RRGGBB - Command (0x30) + Color1 (RGB)
    /// Word 1: YYYYXXXX - Vertex1 (X, Y)
    /// Word 2: 0x00RRGGBB - Color2 (RGB)
    /// Word 3: YYYYXXXX - Vertex2 (X, Y)
    /// Word 4: 0x00RRGGBB - Color3 (RGB)
    /// Word 5: YYYYXXXX - Vertex3 (X, Y)
    /// ```
    ///
    /// # References
    ///
    /// - [PSX-SPX: GPU Polygon Commands](http://problemkaputt.de/psx-spx.htm#gpurenderpolygoncommands)
    pub(crate) fn parse_shaded_triangle_opaque(&mut self) {
        if self.command_fifo.len() < 6 {
            return;
        }

        let c0v0 = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let c1v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let c2v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();

        let colors = [
            Color::from_u32(c0v0),
            Color::from_u32(c1v1),
            Color::from_u32(c2v2),
        ];
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];

        self.render_gradient_triangle(&vertices, &colors, false);
    }

    /// GP0(0x32): Gouraud-Shaded Triangle (Semi-Transparent)
    ///
    /// Renders a triangle with per-vertex colors and semi-transparency enabled.
    /// Requires 6 words: (color1, vertex1, color2, vertex2, color3, vertex3)
    pub(crate) fn parse_shaded_triangle_semi_transparent(&mut self) {
        if self.command_fifo.len() < 6 {
            return;
        }

        let c0v0 = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let c1v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let c2v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();

        let colors = [
            Color::from_u32(c0v0),
            Color::from_u32(c1v1),
            Color::from_u32(c2v2),
        ];
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];

        self.render_gradient_triangle(&vertices, &colors, true);
    }

    /// GP0(0x38): Gouraud-Shaded Quad (Opaque)
    ///
    /// Renders a quadrilateral with per-vertex colors (Gouraud shading).
    /// Requires 8 words: (color1, vertex1, color2, vertex2, color3, vertex3, color4, vertex4)
    ///
    /// # Command Format
    ///
    /// ```text
    /// Word 0: 0x38RRGGBB - Command (0x38) + Color1 (RGB)
    /// Word 1: YYYYXXXX - Vertex1 (X, Y)
    /// Word 2: 0x00RRGGBB - Color2 (RGB)
    /// Word 3: YYYYXXXX - Vertex2 (X, Y)
    /// Word 4: 0x00RRGGBB - Color3 (RGB)
    /// Word 5: YYYYXXXX - Vertex3 (X, Y)
    /// Word 6: 0x00RRGGBB - Color4 (RGB)
    /// Word 7: YYYYXXXX - Vertex4 (X, Y)
    /// ```
    pub(crate) fn parse_shaded_quad_opaque(&mut self) {
        if self.command_fifo.len() < 8 {
            return;
        }

        let c0v0 = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let c1v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let c2v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let c3v3 = self.command_fifo.pop_front().unwrap();
        let v4 = self.command_fifo.pop_front().unwrap();

        let colors = [
            Color::from_u32(c0v0),
            Color::from_u32(c1v1),
            Color::from_u32(c2v2),
            Color::from_u32(c3v3),
        ];
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
            Vertex::from_u32(v4),
        ];

        self.render_gradient_quad(&vertices, &colors, false);
    }

    /// GP0(0x3A): Gouraud-Shaded Quad (Semi-Transparent)
    ///
    /// Renders a quadrilateral with per-vertex colors and semi-transparency enabled.
    /// Requires 8 words: (color1, vertex1, color2, vertex2, color3, vertex3, color4, vertex4)
    pub(crate) fn parse_shaded_quad_semi_transparent(&mut self) {
        if self.command_fifo.len() < 8 {
            return;
        }

        let c0v0 = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let c1v1 = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let c2v2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let c3v3 = self.command_fifo.pop_front().unwrap();
        let v4 = self.command_fifo.pop_front().unwrap();

        let colors = [
            Color::from_u32(c0v0),
            Color::from_u32(c1v1),
            Color::from_u32(c2v2),
            Color::from_u32(c3v3),
        ];
        let vertices = [
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
            Vertex::from_u32(v4),
        ];

        self.render_gradient_quad(&vertices, &colors, true);
    }

    /// GP0(0x24): Textured Triangle (Opaque)
    ///
    /// Renders a textured triangle with texture mapping.
    /// Requires 7 words: command+color, vertex1+texcoord1, clut, vertex2+texcoord2, tpage, vertex3+texcoord3
    ///
    /// # Command Format
    ///
    /// ```text
    /// Word 0: 0x24RRGGBB - Command (0x24) + Color (RGB tint)
    /// Word 1: YYYYXXXX - Vertex1 (X, Y)
    /// Word 2: CLUTVVUU - CLUT info (bits 16-31) + TexCoord1 (U, V)
    /// Word 3: YYYYXXXX - Vertex2 (X, Y)
    /// Word 4: PAGEVVUU - Texture Page (bits 16-31) + TexCoord2 (U, V)
    /// Word 5: YYYYXXXX - Vertex3 (X, Y)
    /// Word 6: ----VVUU - TexCoord3 (U, V)
    /// ```
    ///
    /// # CLUT and Texture Page Encoding
    ///
    /// Word 2 (CLUT):
    /// - Bits 16-21: CLUT X coordinate / 16 (multiply by 16 to get actual X)
    /// - Bits 22-30: CLUT Y coordinate
    ///
    /// Word 4 (Texture Page):
    /// - Bits 16-19: Texture page X base (N×64)
    /// - Bit 20: Texture page Y base (0=Y0-255, 1=Y256-511)
    /// - Bits 21-22: Semi-transparency mode (ignored for opaque)
    /// - Bits 23-24: Texture depth (0=4bit, 1=8bit, 2=15bit)
    ///
    /// # References
    ///
    /// - [PSX-SPX: GPU Texture Commands](http://problemkaputt.de/psx-spx.htm#gputextureattributes)
    pub(crate) fn parse_textured_triangle_opaque(&mut self) {
        if self.command_fifo.len() < 7 {
            return; // Need more words
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v0 = self.command_fifo.pop_front().unwrap();
        let t0clut = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let t1page = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let t2 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v0),
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
        ];
        let texcoords = [
            TexCoord::from_u32(t0clut),
            TexCoord::from_u32(t1page),
            TexCoord::from_u32(t2),
        ];

        // Extract CLUT coordinates from word 2
        let clut_x = ((t0clut >> 16) & 0x3F) * 16;
        let clut_y = (t0clut >> 22) & 0x1FF;

        // Extract texture page information from word 4
        let page_x = ((t1page >> 16) & 0xF) * 64;
        let page_y = ((t1page >> 20) & 1) * 256;
        let tex_depth = ((t1page >> 23) & 0x3) as u8;

        let texture_info = TextureInfo {
            page_x: page_x as u16,
            page_y: page_y as u16,
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            depth: tex_depth.into(),
        };

        self.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);
    }

    /// GP0(0x26): Textured Triangle (Semi-Transparent)
    ///
    /// Renders a textured triangle with semi-transparency enabled.
    /// Same format as 0x24, but with semi-transparency blending applied.
    pub(crate) fn parse_textured_triangle_semi_transparent(&mut self) {
        if self.command_fifo.len() < 7 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v0 = self.command_fifo.pop_front().unwrap();
        let t0clut = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let t1page = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let t2 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v0),
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
        ];
        let texcoords = [
            TexCoord::from_u32(t0clut),
            TexCoord::from_u32(t1page),
            TexCoord::from_u32(t2),
        ];

        let clut_x = ((t0clut >> 16) & 0x3F) * 16;
        let clut_y = (t0clut >> 22) & 0x1FF;
        let page_x = ((t1page >> 16) & 0xF) * 64;
        let page_y = ((t1page >> 20) & 1) * 256;
        let tex_depth = ((t1page >> 23) & 0x3) as u8;

        let texture_info = TextureInfo {
            page_x: page_x as u16,
            page_y: page_y as u16,
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            depth: tex_depth.into(),
        };

        self.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, true);
    }

    /// GP0(0x2C): Textured Quadrilateral (Opaque)
    ///
    /// Renders a textured quadrilateral with texture mapping.
    /// Requires 9 words: command+color, 4×(vertex+texcoord), with CLUT and texture page info
    ///
    /// # Command Format
    ///
    /// ```text
    /// Word 0: 0x2CRRGGBB - Command (0x2C) + Color (RGB tint)
    /// Word 1: YYYYXXXX - Vertex1 (X, Y)
    /// Word 2: CLUTVVUU - CLUT info + TexCoord1 (U, V)
    /// Word 3: YYYYXXXX - Vertex2 (X, Y)
    /// Word 4: PAGEVVUU - Texture Page + TexCoord2 (U, V)
    /// Word 5: YYYYXXXX - Vertex3 (X, Y)
    /// Word 6: ----VVUU - TexCoord3 (U, V)
    /// Word 7: YYYYXXXX - Vertex4 (X, Y)
    /// Word 8: ----VVUU - TexCoord4 (U, V)
    /// ```
    pub(crate) fn parse_textured_quad_opaque(&mut self) {
        if self.command_fifo.len() < 9 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v0 = self.command_fifo.pop_front().unwrap();
        let t0clut = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let t1page = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let t2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let t3 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v0),
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];
        let texcoords = [
            TexCoord::from_u32(t0clut),
            TexCoord::from_u32(t1page),
            TexCoord::from_u32(t2),
            TexCoord::from_u32(t3),
        ];

        let clut_x = ((t0clut >> 16) & 0x3F) * 16;
        let clut_y = (t0clut >> 22) & 0x1FF;
        let page_x = ((t1page >> 16) & 0xF) * 64;
        let page_y = ((t1page >> 20) & 1) * 256;
        let tex_depth = ((t1page >> 23) & 0x3) as u8;

        let texture_info = TextureInfo {
            page_x: page_x as u16,
            page_y: page_y as u16,
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            depth: tex_depth.into(),
        };

        self.render_textured_quad(&vertices, &texcoords, &texture_info, &color, false);
    }

    /// GP0(0x2E): Textured Quadrilateral (Semi-Transparent)
    ///
    /// Renders a textured quadrilateral with semi-transparency enabled.
    /// Same format as 0x2C, but with semi-transparency blending applied.
    pub(crate) fn parse_textured_quad_semi_transparent(&mut self) {
        if self.command_fifo.len() < 9 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let v0 = self.command_fifo.pop_front().unwrap();
        let t0clut = self.command_fifo.pop_front().unwrap();
        let v1 = self.command_fifo.pop_front().unwrap();
        let t1page = self.command_fifo.pop_front().unwrap();
        let v2 = self.command_fifo.pop_front().unwrap();
        let t2 = self.command_fifo.pop_front().unwrap();
        let v3 = self.command_fifo.pop_front().unwrap();
        let t3 = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let vertices = [
            Vertex::from_u32(v0),
            Vertex::from_u32(v1),
            Vertex::from_u32(v2),
            Vertex::from_u32(v3),
        ];
        let texcoords = [
            TexCoord::from_u32(t0clut),
            TexCoord::from_u32(t1page),
            TexCoord::from_u32(t2),
            TexCoord::from_u32(t3),
        ];

        let clut_x = ((t0clut >> 16) & 0x3F) * 16;
        let clut_y = (t0clut >> 22) & 0x1FF;
        let page_x = ((t1page >> 16) & 0xF) * 64;
        let page_y = ((t1page >> 20) & 1) * 256;
        let tex_depth = ((t1page >> 23) & 0x3) as u8;

        let texture_info = TextureInfo {
            page_x: page_x as u16,
            page_y: page_y as u16,
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            depth: tex_depth.into(),
        };

        self.render_textured_quad(&vertices, &texcoords, &texture_info, &color, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monochrome_triangle_parsing() {
        let mut gpu = GPU::new();

        // GP0(0x20): Monochrome Triangle Opaque
        // Color: Red (0xFF0000)
        // Vertices: (10,20), (30,40), (50,60)
        gpu.write_gp0(0x200000FF); // Command + Red
        gpu.write_gp0(0x0014000A); // V1: Y=20, X=10
        gpu.write_gp0(0x0028001E); // V2: Y=40, X=30
        gpu.write_gp0(0x003C0032); // V3: Y=60, X=50

        // Verify command was processed (FIFO should be empty)
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_monochrome_quad_parsing() {
        let mut gpu = GPU::new();

        // GP0(0x28): Monochrome Quad Opaque
        // Color: Green (0x00FF00)
        // Vertices: (0,0), (100,0), (100,100), (0,100)
        gpu.write_gp0(0x2800FF00); // Command + Green
        gpu.write_gp0(0x00000000); // V1: (0,0)
        gpu.write_gp0(0x00000064); // V2: (100,0)
        gpu.write_gp0(0x00640064); // V3: (100,100)
        gpu.write_gp0(0x00640000); // V4: (0,100)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_gouraud_triangle_parsing() {
        let mut gpu = GPU::new();

        // GP0(0x30): Gouraud-Shaded Triangle Opaque
        // Per PSX-SPX format: (color1, vertex1, color2, vertex2, color3, vertex3)
        gpu.write_gp0(0x30FF0000); // Command + Color1 (Red)
        gpu.write_gp0(0x00000000); // V1: (0,0)
        gpu.write_gp0(0x0000FF00); // Color2 (Green)
        gpu.write_gp0(0x00640000); // V2: (100,0)
        gpu.write_gp0(0x000000FF); // Color3 (Blue)
        gpu.write_gp0(0x00320032); // V3: (50,50)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_gouraud_quad_parsing() {
        let mut gpu = GPU::new();

        // GP0(0x38): Gouraud-Shaded Quad Opaque
        gpu.write_gp0(0x38FF0000); // Command + Color1 (Red)
        gpu.write_gp0(0x00000000); // V1: (0,0)
        gpu.write_gp0(0x0000FF00); // Color2 (Green)
        gpu.write_gp0(0x00000064); // V2: (100,0)
        gpu.write_gp0(0x000000FF); // Color3 (Blue)
        gpu.write_gp0(0x00640064); // V3: (100,100)
        gpu.write_gp0(0x00FFFF00); // Color4 (Yellow)
        gpu.write_gp0(0x00640000); // V4: (0,100)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_triangle_clut_extraction() {
        let mut gpu = GPU::new();

        // GP0(0x24): Textured Triangle Opaque
        // Test CLUT coordinate extraction per PSX-SPX:
        // Bits 0-5: X coordinate / 16
        // Bits 6-14: Y coordinate (0-511)

        // CLUT at X=320 (320/16 = 20 = 0x14), Y=100 (0x64)
        let clut_word = (100 << 22) | (20 << 16) | 0x0000; // Y=100, X/16=20, U=0, V=0

        gpu.write_gp0(0x24808080); // Command + Color
        gpu.write_gp0(0x00000000); // V1: (0,0)
        gpu.write_gp0(clut_word); // CLUT + TexCoord1
        gpu.write_gp0(0x00000064); // V2: (100,0)
        gpu.write_gp0(0x00000000); // Page + TexCoord2
        gpu.write_gp0(0x00640064); // V3: (100,100)
        gpu.write_gp0(0x00000000); // TexCoord3

        assert!(gpu.command_fifo.is_empty());
        // Note: Actual CLUT values would need to be verified in rendering function
    }

    #[test]
    fn test_textured_triangle_page_extraction() {
        let mut gpu = GPU::new();

        // Test Texture Page extraction per PSX-SPX:
        // Bits 16-19: Texture page X base (N×64)
        // Bit 20: Texture page Y base (0=Y0-255, 1=Y256-511)
        // Bits 23-24: Texture depth (0=4bit, 1=8bit, 2=15bit)

        // Page at X=192 (3×64, bits 16-19 = 3), Y=256 (bit 20 = 1), 8-bit depth (bits 23-24 = 1)
        let page_word = (1 << 23) | (1 << 20) | (3 << 16) | 0x0000; // depth=8bit, Y=256, X=192, U=0, V=0

        gpu.write_gp0(0x24808080); // Command + Color
        gpu.write_gp0(0x00000000); // V1
        gpu.write_gp0(0x00000000); // CLUT + TexCoord1
        gpu.write_gp0(0x00000064); // V2
        gpu.write_gp0(page_word); // Page + TexCoord2
        gpu.write_gp0(0x00640064); // V3
        gpu.write_gp0(0x00000000); // TexCoord3

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_quad_parsing() {
        let mut gpu = GPU::new();

        // GP0(0x2C): Textured Quad Opaque
        // 9 words total
        gpu.write_gp0(0x2C808080); // Command + Color
        gpu.write_gp0(0x00000000); // V1: (0,0)
        gpu.write_gp0(0x00000000); // CLUT + TexCoord1
        gpu.write_gp0(0x00000064); // V2: (100,0)
        gpu.write_gp0(0x00000040); // Page + TexCoord2: U=64, V=0
        gpu.write_gp0(0x00640064); // V3: (100,100)
        gpu.write_gp0(0x00004040); // TexCoord3: U=64, V=64
        gpu.write_gp0(0x00640000); // V4: (0,100)
        gpu.write_gp0(0x00000040); // TexCoord4: U=0, V=64

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_vertex_coordinate_range() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: signed 16-bit coordinates, range -1024 to +1023
        // Test maximum positive coordinates
        gpu.write_gp0(0x20FFFFFF); // Monochrome triangle
        gpu.write_gp0(0x03FF03FF); // V1: (1023, 1023)
        gpu.write_gp0(0x03FF0000); // V2: (0, 1023)
        gpu.write_gp0(0x000003FF); // V3: (1023, 0)

        assert!(gpu.command_fifo.is_empty());

        // Test minimum negative coordinates (-1024)
        gpu.write_gp0(0x20FFFFFF);
        gpu.write_gp0(0xFC00FC00); // V1: (-1024, -1024)
        gpu.write_gp0(0xFC000000); // V2: (0, -1024)
        gpu.write_gp0(0x0000FC00); // V3: (-1024, 0)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_semi_transparent_flags() {
        let mut gpu = GPU::new();

        // Test opaque triangle (bit 25 = 0)
        gpu.write_gp0(0x20FFFFFF); // GP0(0x20): Opaque
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00640000);
        gpu.write_gp0(0x00320032);
        assert!(gpu.command_fifo.is_empty());

        // Test semi-transparent triangle (bit 25 = 1)
        gpu.write_gp0(0x22FFFFFF); // GP0(0x22): Semi-transparent
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00640000);
        gpu.write_gp0(0x00320032);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_clut_coordinate_bit_layout() {
        // Verify CLUT extraction formula per PSX-SPX
        // Bits 0-5: X coordinate / 16 (multiply by 16 to get actual X)
        // Bits 6-14: Y coordinate

        // Example: CLUT at VRAM (512, 256)
        // X = 512 / 16 = 32 (0x20)
        // Y = 256 (0x100)
        let clut_word = (256 << 22) | (32 << 16);

        let clut_x = ((clut_word >> 16) & 0x3F) * 16;
        let clut_y = (clut_word >> 22) & 0x1FF;

        assert_eq!(clut_x, 512);
        assert_eq!(clut_y, 256);
    }

    #[test]
    fn test_clut_coordinate_range() {
        // CLUT X: 0-5 bits = 0-63, multiply by 16 = 0-1008
        // CLUT Y: 9 bits = 0-511

        // Maximum CLUT coordinates
        let max_clut_x = 0x3F; // 63
        let max_clut_y = 0x1FF; // 511

        let clut_word = (max_clut_y << 22) | (max_clut_x << 16);
        let extracted_x = ((clut_word >> 16) & 0x3F) * 16;
        let extracted_y = (clut_word >> 22) & 0x1FF;

        assert_eq!(extracted_x, 63 * 16); // 1008
        assert_eq!(extracted_y, 511);
    }

    #[test]
    fn test_texture_page_bit_layout() {
        // Verify texture page extraction per PSX-SPX
        // Bits 16-19: Texture page X base (N×64)
        // Bit 20: Texture page Y base (0=0-255, 1=256-511)
        // Bits 23-24: Texture depth (0=4bit, 1=8bit, 2=15bit)

        // Example: Page at X=256 (4×64), Y=256, 15-bit depth
        // X base = 4 (bits 16-19)
        // Y base = 1 (bit 20)
        // Depth = 2 (bits 23-24)
        let page_word = (2 << 23) | (1 << 20) | (4 << 16);

        let page_x = ((page_word >> 16) & 0xF) * 64;
        let page_y = ((page_word >> 20) & 1) * 256;
        let tex_depth = ((page_word >> 23) & 0x3) as u8;

        assert_eq!(page_x, 256);
        assert_eq!(page_y, 256);
        assert_eq!(tex_depth, 2); // 15-bit
    }

    #[test]
    fn test_texture_depth_values() {
        // Test all three texture depth modes
        let depth_4bit = 0 << 23;
        let depth_8bit = 1 << 23;
        let depth_15bit = 2 << 23;

        assert_eq!((depth_4bit >> 23) & 0x3, 0);
        assert_eq!((depth_8bit >> 23) & 0x3, 1);
        assert_eq!((depth_15bit >> 23) & 0x3, 2);
    }

    #[test]
    fn test_partial_command_buffering() {
        let mut gpu = GPU::new();

        // Send partial triangle command (need 4 words, send only 2)
        gpu.write_gp0(0x20FFFFFF);
        gpu.write_gp0(0x00000000);

        // Command should remain in FIFO, waiting for more data
        assert_eq!(gpu.command_fifo.len(), 2);

        // Complete the command
        gpu.write_gp0(0x00640000);
        gpu.write_gp0(0x00320032);

        // Now FIFO should be empty
        assert!(gpu.command_fifo.is_empty());
    }
}
