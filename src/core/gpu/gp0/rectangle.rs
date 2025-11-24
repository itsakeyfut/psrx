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

//! GP0 rectangle drawing commands
//!
//! Implements parsing and rendering for rectangle primitives:
//! - Monochrome rectangles (solid color)
//! - Textured rectangles (sprite rendering)
//! - Variable size and fixed size (1×1, 8×8, 16×16)

use super::super::primitives::{Color, TexCoord, TextureInfo, Vertex};
use super::super::GPU;

impl GPU {
    // =========================================================================
    // Monochrome (Solid Color) Rectangles
    // =========================================================================

    /// GP0(0x60): Monochrome Rectangle (Variable Size, Opaque)
    ///
    /// Renders a solid-color rectangle of variable dimensions.
    /// Requires 3 words: command+color, vertex, width+height
    pub(crate) fn parse_monochrome_rect_variable_opaque(&mut self) {
        if self.command_fifo.len() < 3 {
            return; // Need more words
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        self.render_monochrome_rect(pos.x, pos.y, width, height, &color, false);
    }

    /// GP0(0x62): Monochrome Rectangle (Variable Size, Semi-Transparent)
    ///
    /// Renders a solid-color rectangle with semi-transparency enabled.
    /// Requires 3 words: command+color, vertex, width+height
    pub(crate) fn parse_monochrome_rect_variable_semi_transparent(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        self.render_monochrome_rect(pos.x, pos.y, width, height, &color, true);
    }

    /// GP0(0x68): Monochrome Rectangle (1×1, Opaque)
    ///
    /// Renders a single pixel in solid color.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_1x1_opaque(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 1, 1, &color, false);
    }

    /// GP0(0x6A): Monochrome Rectangle (1×1, Semi-Transparent)
    ///
    /// Renders a single pixel with semi-transparency.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_1x1_semi_transparent(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 1, 1, &color, true);
    }

    /// GP0(0x70): Monochrome Rectangle (8×8, Opaque)
    ///
    /// Renders an 8×8 pixel rectangle in solid color.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_8x8_opaque(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 8, 8, &color, false);
    }

    /// GP0(0x72): Monochrome Rectangle (8×8, Semi-Transparent)
    ///
    /// Renders an 8×8 pixel rectangle with semi-transparency.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_8x8_semi_transparent(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 8, 8, &color, true);
    }

    /// GP0(0x78): Monochrome Rectangle (16×16, Opaque)
    ///
    /// Renders a 16×16 pixel rectangle in solid color.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_16x16_opaque(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 16, 16, &color, false);
    }

    /// GP0(0x7A): Monochrome Rectangle (16×16, Semi-Transparent)
    ///
    /// Renders a 16×16 pixel rectangle with semi-transparency.
    /// Requires 2 words: command+color, vertex
    pub(crate) fn parse_monochrome_rect_16x16_semi_transparent(&mut self) {
        if self.command_fifo.len() < 2 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);

        self.render_monochrome_rect(pos.x, pos.y, 16, 16, &color, true);
    }

    // =========================================================================
    // Textured Rectangles
    // =========================================================================

    /// GP0(0x64): Textured Rectangle (Variable Size, Opaque, Raw Texture)
    ///
    /// Renders a textured rectangle with raw texture colors (no modulation).
    /// Requires 4 words: command+color, vertex, texcoord+clut, width+height
    pub(crate) fn parse_textured_rect_variable_opaque(&mut self) {
        if self.command_fifo.len() < 4 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        log::info!(
            "GP0(0x64) Textured Rect: pos=({}, {}), size={}x{}, texcoord=({}, {}), clut=({}, {})",
            pos.x,
            pos.y,
            width,
            height,
            texcoord.u,
            texcoord.v,
            clut_x,
            clut_y
        );

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            width,
            height,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            false,
        );
    }

    /// GP0(0x65): Textured Rectangle (Variable Size, Opaque, Modulated)
    ///
    /// Renders a textured rectangle with color modulation.
    /// Requires 4 words: command+color, vertex, texcoord+clut, width+height
    pub(crate) fn parse_textured_rect_variable_opaque_modulated(&mut self) {
        if self.command_fifo.len() < 4 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            width,
            height,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            true,
        );
    }

    /// GP0(0x66): Textured Rectangle (Variable Size, Semi-Transparent, Raw Texture)
    ///
    /// Renders a textured rectangle with semi-transparency, no modulation.
    /// Requires 4 words: command+color, vertex, texcoord+clut, width+height
    pub(crate) fn parse_textured_rect_variable_semi_transparent(&mut self) {
        if self.command_fifo.len() < 4 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            width,
            height,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            false,
        );
    }

    /// GP0(0x67): Textured Rectangle (Variable Size, Semi-Transparent, Modulated)
    ///
    /// Renders a textured rectangle with semi-transparency and color modulation.
    /// Requires 4 words: command+color, vertex, texcoord+clut, width+height
    pub(crate) fn parse_textured_rect_variable_semi_transparent_modulated(&mut self) {
        if self.command_fifo.len() < 4 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();
        let size = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;
        let width = (size >> 16) as u16;
        let height = (size & 0xFFFF) as u16;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            width,
            height,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            true,
        );
    }

    /// GP0(0x6C): Textured Rectangle (1×1, Opaque, Raw Texture)
    ///
    /// Renders a 1×1 textured rectangle (single texel).
    /// Requires 3 words: command+color, vertex, texcoord+clut
    pub(crate) fn parse_textured_rect_1x1_opaque(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            1,
            1,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            false,
        );
    }

    /// GP0(0x6D): Textured Rectangle (1×1, Opaque, Modulated)
    pub(crate) fn parse_textured_rect_1x1_opaque_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            1,
            1,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            true,
        );
    }

    /// GP0(0x6E): Textured Rectangle (1×1, Semi-Transparent, Raw Texture)
    pub(crate) fn parse_textured_rect_1x1_semi_transparent(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            1,
            1,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            false,
        );
    }

    /// GP0(0x6F): Textured Rectangle (1×1, Semi-Transparent, Modulated)
    pub(crate) fn parse_textured_rect_1x1_semi_transparent_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            1,
            1,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            true,
        );
    }

    /// GP0(0x74): Textured Rectangle (8×8, Opaque, Raw Texture)
    pub(crate) fn parse_textured_rect_8x8_opaque(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            8,
            8,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            false,
        );
    }

    /// GP0(0x75): Textured Rectangle (8×8, Opaque, Modulated)
    pub(crate) fn parse_textured_rect_8x8_opaque_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            8,
            8,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            true,
        );
    }

    /// GP0(0x76): Textured Rectangle (8×8, Semi-Transparent, Raw Texture)
    pub(crate) fn parse_textured_rect_8x8_semi_transparent(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            8,
            8,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            false,
        );
    }

    /// GP0(0x77): Textured Rectangle (8×8, Semi-Transparent, Modulated)
    pub(crate) fn parse_textured_rect_8x8_semi_transparent_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            8,
            8,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            true,
        );
    }

    /// GP0(0x7C): Textured Rectangle (16×16, Opaque, Raw Texture)
    pub(crate) fn parse_textured_rect_16x16_opaque(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            16,
            16,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            false,
        );
    }

    /// GP0(0x7D): Textured Rectangle (16×16, Opaque, Modulated)
    pub(crate) fn parse_textured_rect_16x16_opaque_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            16,
            16,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            false,
            true,
        );
    }

    /// GP0(0x7E): Textured Rectangle (16×16, Semi-Transparent, Raw Texture)
    pub(crate) fn parse_textured_rect_16x16_semi_transparent(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            16,
            16,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            false,
        );
    }

    /// GP0(0x7F): Textured Rectangle (16×16, Semi-Transparent, Modulated)
    pub(crate) fn parse_textured_rect_16x16_semi_transparent_modulated(&mut self) {
        if self.command_fifo.len() < 3 {
            return;
        }

        let cmd = self.command_fifo.pop_front().unwrap();
        let vertex = self.command_fifo.pop_front().unwrap();
        let texcoord_clut = self.command_fifo.pop_front().unwrap();

        let color = Color::from_u32(cmd);
        let pos = Vertex::from_u32(vertex);
        let texcoord = TexCoord::from_u32(texcoord_clut);
        let clut_x = ((texcoord_clut >> 16) & 0x3F) * 16;
        let clut_y = (texcoord_clut >> 22) & 0x1FF;

        let texture_info = TextureInfo {
            clut_x: clut_x as u16,
            clut_y: clut_y as u16,
            page_x: self.draw_mode.texture_page_x_base,
            page_y: self.draw_mode.texture_page_y_base,
            depth: self.draw_mode.texture_depth.into(),
        };

        self.render_textured_rect(
            pos.x,
            pos.y,
            16,
            16,
            texcoord.u,
            texcoord.v,
            &texture_info,
            &color,
            true,
            true,
        );
    }

    // =========================================================================
    // Rendering Functions
    // =========================================================================

    /// Render a monochrome (solid color) rectangle
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Rectangle width in pixels
    /// * `height` - Rectangle height in pixels
    /// * `color` - Fill color
    /// * `semi_transparent` - Enable semi-transparency blending
    fn render_monochrome_rect(
        &mut self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        color: &Color,
        semi_transparent: bool,
    ) {
        self.rasterizer.draw_rectangle(
            &mut self.vram,
            &self.draw_mode,
            &self.draw_area,
            self.draw_offset,
            x,
            y,
            width,
            height,
            color,
            semi_transparent,
        );
    }

    /// Render a textured rectangle
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Rectangle width in pixels
    /// * `height` - Rectangle height in pixels
    /// * `tex_u` - Texture U coordinate (top-left)
    /// * `tex_v` - Texture V coordinate (top-left)
    /// * `texture_info` - Texture page and CLUT information
    /// * `color` - Modulation color (if modulated is true)
    /// * `semi_transparent` - Enable semi-transparency blending
    /// * `modulated` - Enable color modulation (multiply texture by color)
    #[allow(clippy::too_many_arguments)]
    fn render_textured_rect(
        &mut self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        tex_u: u8,
        tex_v: u8,
        texture_info: &TextureInfo,
        color: &Color,
        semi_transparent: bool,
        modulated: bool,
    ) {
        self.rasterizer.draw_textured_rectangle(
            &mut self.vram,
            &self.draw_mode,
            &self.draw_area,
            self.draw_offset,
            x,
            y,
            width,
            height,
            tex_u,
            tex_v,
            texture_info,
            color,
            semi_transparent,
            modulated,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Monochrome Rectangle Tests

    #[test]
    fn test_monochrome_rect_variable_size() {
        let mut gpu = GPU::new();

        // GP0(0x60): Variable Size Rectangle Opaque
        gpu.write_gp0(0x60FF0000); // Command + Red
        gpu.write_gp0(0x00320032); // Vertex: (50, 50)
        gpu.write_gp0(0x00640064); // Size: Width=100, Height=100

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_monochrome_rect_1x1() {
        let mut gpu = GPU::new();

        // GP0(0x68): 1×1 Rectangle Opaque
        gpu.write_gp0(0x680000FF); // Command + Blue
        gpu.write_gp0(0x00640064); // Vertex: (100, 100)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_monochrome_rect_8x8() {
        let mut gpu = GPU::new();

        // GP0(0x70): 8×8 Rectangle Opaque
        gpu.write_gp0(0x7000FF00); // Command + Green
        gpu.write_gp0(0x00000000); // Vertex: (0, 0)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_monochrome_rect_16x16() {
        let mut gpu = GPU::new();

        // GP0(0x78): 16×16 Rectangle Opaque
        gpu.write_gp0(0x78FFFF00); // Command + Yellow
        gpu.write_gp0(0x00320032); // Vertex: (50, 50)

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_monochrome_rect_semi_transparent_variants() {
        let mut gpu = GPU::new();

        // Test all semi-transparent variants
        // GP0(0x62): Variable size, semi-transparent
        gpu.write_gp0(0x62FFFFFF);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00200020);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x6A): 1×1, semi-transparent
        gpu.write_gp0(0x6AFFFFFF);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x72): 8×8, semi-transparent
        gpu.write_gp0(0x72FFFFFF);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x7A): 16×16, semi-transparent
        gpu.write_gp0(0x7AFFFFFF);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());
    }

    // Textured Rectangle Tests

    #[test]
    fn test_textured_rect_variable_opaque() {
        let mut gpu = GPU::new();

        // GP0(0x64): Textured Rectangle Variable Size Opaque (Raw)
        gpu.write_gp0(0x64808080); // Command + Color
        gpu.write_gp0(0x00000000); // Vertex: (0, 0)
        gpu.write_gp0(0x00000000); // TexCoord + CLUT
        gpu.write_gp0(0x00400040); // Size: 64×64

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_rect_variable_modulated() {
        let mut gpu = GPU::new();

        // GP0(0x65): Textured Rectangle Variable Size Opaque (Modulated)
        gpu.write_gp0(0x65808080); // Command + Color (modulation)
        gpu.write_gp0(0x00320032); // Vertex: (50, 50)
        gpu.write_gp0(0x00200020); // TexCoord: U=32, V=32
        gpu.write_gp0(0x00400040); // Size: 64×64

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_rect_1x1_variants() {
        let mut gpu = GPU::new();

        // GP0(0x6C): 1×1 Opaque Raw
        gpu.write_gp0(0x6C808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x6D): 1×1 Opaque Modulated
        gpu.write_gp0(0x6D808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x6E): 1×1 Semi-Transparent Raw
        gpu.write_gp0(0x6E808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x6F): 1×1 Semi-Transparent Modulated
        gpu.write_gp0(0x6F808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_rect_8x8_variants() {
        let mut gpu = GPU::new();

        // GP0(0x74): 8×8 Opaque Raw
        gpu.write_gp0(0x74808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x75): 8×8 Opaque Modulated
        gpu.write_gp0(0x75808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x76): 8×8 Semi-Transparent Raw
        gpu.write_gp0(0x76808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // GP0(0x77): 8×8 Semi-Transparent Modulated
        gpu.write_gp0(0x77808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_rect_16x16_variants() {
        let mut gpu = GPU::new();

        // Test all 16×16 variants
        let variants = [0x7C, 0x7D, 0x7E, 0x7F];

        for cmd in variants {
            gpu.write_gp0((cmd << 24) | 0x808080);
            gpu.write_gp0(0x00000000);
            gpu.write_gp0(0x00000000);
            assert!(gpu.command_fifo.is_empty());
        }
    }

    #[test]
    fn test_rect_clut_extraction() {
        let mut gpu = GPU::new();

        // Test CLUT coordinate extraction
        // CLUT at X=320 (320/16 = 20), Y=100
        let clut_x = 20u32; // X / 16
        let clut_y = 100u32;
        let texcoord_clut = (clut_y << 22) | (clut_x << 16);

        gpu.write_gp0(0x64808080); // Command
        gpu.write_gp0(0x00000000); // Vertex
        gpu.write_gp0(texcoord_clut); // CLUT + TexCoord
        gpu.write_gp0(0x00400040); // Size

        assert!(gpu.command_fifo.is_empty());
        // Actual CLUT values validated in rendering
    }

    #[test]
    fn test_rect_texture_page_usage() {
        let mut gpu = GPU::new();

        // Rectangle commands use texture page from draw mode (GP0(E1h))
        // Not from command like polygons
        // Set texture page first
        gpu.write_gp0(0xE1000012); // Page X=128, Y=256

        // Then draw textured rectangle
        gpu.write_gp0(0x64808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00400040);

        assert!(gpu.command_fifo.is_empty());
        assert_eq!(gpu.draw_mode.texture_page_x_base, 128);
        assert_eq!(gpu.draw_mode.texture_page_y_base, 256);
    }

    #[test]
    fn test_rect_size_limits() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: maximum dimensions 1023×511
        gpu.write_gp0(0x60FFFFFF);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x01FF03FF); // Width=1023, Height=511

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_rect_coordinate_format() {
        let mut gpu = GPU::new();

        // Test vertex coordinate format: YYYYXXXX
        // X=200 (0xC8), Y=150 (0x96)
        gpu.write_gp0(0x60FFFFFF);
        gpu.write_gp0(0x009600C8); // Y=150, X=200
        gpu.write_gp0(0x00320032); // 50×50

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_rect_texture_uv_format() {
        let mut gpu = GPU::new();

        // UV format: VVUU (8-bit U, 8-bit V)
        let u = 64u32;
        let v = 128u32;
        let texcoord = (v << 8) | u;

        gpu.write_gp0(0x64808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(texcoord);
        gpu.write_gp0(0x00400040);

        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_rect_modulation_vs_raw() {
        let mut gpu = GPU::new();

        // Raw texture (bit 24 = 1): GP0(0x64) - color ignored
        gpu.write_gp0(0x64FFFFFF);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00200020);
        assert!(gpu.command_fifo.is_empty());

        // Modulated (bit 24 = 0): GP0(0x65) - color used for modulation
        // Per PSX-SPX: (texel.rgb * vertexColor.rgb) / 128
        gpu.write_gp0(0x65808080); // 128,128,128 = brightest for modulation
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00200020);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_rect_partial_command_buffering() {
        let mut gpu = GPU::new();

        // Variable size rectangle needs 3 words
        gpu.write_gp0(0x60FFFFFF);
        assert_eq!(gpu.command_fifo.len(), 1);

        gpu.write_gp0(0x00000000);
        assert_eq!(gpu.command_fifo.len(), 2);

        gpu.write_gp0(0x00640064);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_textured_rect_needs_4_words() {
        let mut gpu = GPU::new();

        // Textured variable rectangle needs 4 words
        gpu.write_gp0(0x64808080);
        assert_eq!(gpu.command_fifo.len(), 1);

        gpu.write_gp0(0x00000000);
        assert_eq!(gpu.command_fifo.len(), 2);

        gpu.write_gp0(0x00000000);
        assert_eq!(gpu.command_fifo.len(), 3);

        gpu.write_gp0(0x00400040);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_fixed_size_rect_2_vs_3_words() {
        let mut gpu = GPU::new();

        // Monochrome 8×8 needs only 2 words (no size word)
        gpu.write_gp0(0x70FFFFFF);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());

        // Textured 8×8 needs 3 words (includes UV+CLUT)
        gpu.write_gp0(0x74808080);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00000000);
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_rect_size_encoding() {
        let mut gpu = GPU::new();

        // Size word format: HhhhWwww (Height in upper 16, Width in lower 16)
        // Width=256 (0x100), Height=128 (0x80)
        gpu.write_gp0(0x60FFFFFF);
        gpu.write_gp0(0x00000000);
        gpu.write_gp0(0x00800100); // Height=128, Width=256

        assert!(gpu.command_fifo.is_empty());
    }
}
