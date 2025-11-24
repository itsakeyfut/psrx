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

//! Textured primitive rendering implementation
//!
//! Implements texture-mapped triangle and quadrilateral rasterization with support
//! for 4-bit, 8-bit, and 15-bit texture formats.

use super::super::primitives::{Color, TexCoord, TextureInfo, Vertex};
use super::super::GPU;

impl GPU {
    /// Render a textured triangle
    ///
    /// Applies the drawing offset to all vertices and rasterizes the triangle
    /// with texture mapping using the software rasterizer.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 3 vertices defining the triangle
    /// * `texcoords` - Array of 3 texture coordinates corresponding to vertices
    /// * `texture_info` - Texture page and CLUT information
    /// * `color` - Color tint to modulate with texture
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Texture Mapping
    ///
    /// The texture coordinates are interpolated across the triangle using
    /// barycentric coordinates. The texture is sampled from VRAM at the
    /// specified texture page with the given color depth (4-bit, 8-bit, or 15-bit).
    ///
    /// # Color Modulation
    ///
    /// The color parameter acts as a tint/modulation color that is multiplied
    /// with the sampled texture color. For normal brightness, use (128, 128, 128).
    ///
    /// # Notes
    ///
    /// Semi-transparency is currently ignored (will be implemented in issue #36).
    /// The drawing offset is applied to all vertices before rasterization.
    pub(crate) fn render_textured_triangle(
        &mut self,
        vertices: &[Vertex; 3],
        texcoords: &[TexCoord; 3],
        texture_info: &TextureInfo,
        color: &Color,
        semi_transparent: bool,
    ) {
        // Apply drawing offset
        let v0 = (
            vertices[0].x.wrapping_add(self.draw_offset.0),
            vertices[0].y.wrapping_add(self.draw_offset.1),
        );
        let v1 = (
            vertices[1].x.wrapping_add(self.draw_offset.0),
            vertices[1].y.wrapping_add(self.draw_offset.1),
        );
        let v2 = (
            vertices[2].x.wrapping_add(self.draw_offset.0),
            vertices[2].y.wrapping_add(self.draw_offset.1),
        );

        let t0 = (texcoords[0].u, texcoords[0].v);
        let t1 = (texcoords[1].u, texcoords[1].v);
        let t2 = (texcoords[2].u, texcoords[2].v);

        log::trace!(
            "Rendering {}textured triangle: v=({},{}),({},{}),({},{}) t=({},{}),({},{}),({},{}) color=({},{},{})",
            if semi_transparent { "semi-transparent " } else { "" },
            v0.0, v0.1, v1.0, v1.1, v2.0, v2.1,
            t0.0, t0.1, t1.0, t1.1, t2.0, t2.1,
            color.r, color.g, color.b
        );

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Rasterize the textured triangle with texture window
        self.rasterizer.draw_textured_triangle(
            &mut self.vram,
            v0,
            t0,
            v1,
            t1,
            v2,
            t2,
            texture_info,
            &self.texture_window,
            (color.r, color.g, color.b),
        );
    }

    /// Render a textured quadrilateral
    ///
    /// Splits the quad into two triangles and renders them as textured primitives.
    /// The quad is split along the v0-v2 diagonal.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 4 vertices defining the quad (in order: v0, v1, v2, v3)
    /// * `texcoords` - Array of 4 texture coordinates corresponding to vertices
    /// * `texture_info` - Texture page and CLUT information
    /// * `color` - Color tint to modulate with texture
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Quad Splitting
    ///
    /// The quad is split into two triangles:
    /// - Triangle 1: (v0, v1, v2)
    /// - Triangle 2: (v1, v2, v3)
    ///
    /// This matches the PlayStation GPU's quadrilateral rendering behavior.
    ///
    /// # Notes
    ///
    /// Semi-transparency is currently ignored (will be implemented in issue #36).
    pub(crate) fn render_textured_quad(
        &mut self,
        vertices: &[Vertex; 4],
        texcoords: &[TexCoord; 4],
        texture_info: &TextureInfo,
        color: &Color,
        semi_transparent: bool,
    ) {
        // Split quad into two triangles: (v0,v1,v2) and (v1,v2,v3)
        let tri1_verts = [vertices[0], vertices[1], vertices[2]];
        let tri1_texcoords = [texcoords[0], texcoords[1], texcoords[2]];

        let tri2_verts = [vertices[1], vertices[2], vertices[3]];
        let tri2_texcoords = [texcoords[1], texcoords[2], texcoords[3]];

        self.render_textured_triangle(
            &tri1_verts,
            &tri1_texcoords,
            texture_info,
            color,
            semi_transparent,
        );
        self.render_textured_triangle(
            &tri2_verts,
            &tri2_texcoords,
            texture_info,
            color,
            semi_transparent,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gpu::TextureDepth;

    #[test]
    fn test_textured_triangle_basic() {
        let mut gpu = GPU::new();

        // Set up a simple texture in VRAM first
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7FFF); // White texture
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        }; // Normal brightness

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        // Check center pixel has texture data
        let pixel = gpu.read_vram(150, 133);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_triangle_with_drawing_offset() {
        let mut gpu = GPU::new();

        // Set up texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x001F); // Red texture
            }
        }

        gpu.draw_offset = (50, 30);

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        // Center should be at (150+50, 133+30) = (200, 163)
        let pixel = gpu.read_vram(200, 163);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_triangle_color_modulation() {
        let mut gpu = GPU::new();

        // Set up white texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7FFF); // White
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };

        // Per PSX-SPX: (128,128,128) = normal brightness
        // Lower values darken, higher values brighten (with saturation)
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        let pixel = gpu.read_vram(150, 133);
        // Modulation formula: (texel * color) / 128
        // White (31,31,31) * 128 / 128 = (31,31,31)
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_triangle_texture_coordinates() {
        let mut gpu = GPU::new();

        // Set up gradient texture
        for y in 0..256u16 {
            for x in 0..256u16 {
                let color = ((y >> 3) << 10) | ((x >> 3) << 5) | (x >> 3);
                gpu.write_vram(x, y, color);
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];

        // Per PSX-SPX: Texture coordinates are 8-bit (0-255)
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 255, v: 0 },
            TexCoord { u: 128, v: 255 },
        ];

        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        // Should render with full U/V range
        let pixel = gpu.read_vram(150, 133);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_quad_basic() {
        let mut gpu = GPU::new();

        // Set up texture
        for y in 0..32 {
            for x in 0..32 {
                gpu.write_vram(x, y, 0x03E0); // Green texture
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 31, v: 0 },
            TexCoord { u: 31, v: 31 },
            TexCoord { u: 0, v: 31 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_quad(&vertices, &texcoords, &texture_info, &color, false);

        // Center should have green texture
        let pixel = gpu.read_vram(150, 150);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_quad_decomposition() {
        let mut gpu = GPU::new();

        // Set up texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7C00); // Blue texture
            }
        }

        // Per implementation: Quad splits into (v0,v1,v2) and (v1,v2,v3)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 15, v: 15 },
            TexCoord { u: 0, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_quad(&vertices, &texcoords, &texture_info, &color, false);

        // Check both triangle areas
        assert_ne!(gpu.read_vram(150, 120), 0x0000); // Upper triangle
        assert_ne!(gpu.read_vram(120, 180), 0x0000); // Lower triangle
    }

    #[test]
    fn test_textured_triangle_texture_page() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Texture page X base is N×64, Y base is 0 or 256
        // Set up texture at page (1, 0) = (64, 0)
        for y in 0..16 {
            for x in 64..80 {
                gpu.write_vram(x, y, 0x7FFF); // White
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 64, // Page X = 64
            page_y: 0,  // Page Y = 0
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        let pixel = gpu.read_vram(150, 133);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_triangle_4bit_texture() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: 4-bit textures use 16-color CLUT
        // Set up CLUT at (0, 0)
        for i in 0..16u16 {
            let color = (i << 10) | (i << 5) | i; // Grayscale CLUT
            gpu.write_vram(i, 0, color);
        }

        // Set up 4-bit texture data
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(64 + x, y, 0x0000); // 4-bit indices
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 64, // Page at 64
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T4Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);
    }

    #[test]
    fn test_textured_triangle_8bit_texture() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: 8-bit textures use 256-color CLUT
        // Set up CLUT at (0, 1)
        for i in 0..256u16 {
            let r = (i & 0x1F) as u16;
            let g = ((i >> 3) & 0x1F) as u16;
            let b = ((i >> 6) & 0x1F) as u16;
            let color = (b << 10) | (g << 5) | r;
            gpu.write_vram(i % 16, 1 + (i / 16), color);
        }

        // Set up 8-bit texture data
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(128 + x, y, 0x0000); // 8-bit indices
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 128, // Page at 128
            page_y: 0,
            clut_x: 0,
            clut_y: 1,
            depth: TextureDepth::T8Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);
    }

    #[test]
    fn test_textured_triangle_negative_coordinates() {
        let mut gpu = GPU::new();

        // Set up texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7FFF);
            }
        }

        let vertices = [
            Vertex { x: -50, y: -50 },
            Vertex { x: 100, y: -50 },
            Vertex { x: 25, y: 100 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        // Should handle wrapping
        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);
    }

    #[test]
    fn test_textured_triangle_boundary_coordinates() {
        let mut gpu = GPU::new();

        // Set up large texture area
        for y in 0..512 {
            for x in 0..1024 {
                gpu.write_vram(x, y, 0x7FFF);
            }
        }

        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 512, y: 511 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 255, v: 0 },
            TexCoord { u: 128, v: 255 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);
    }

    #[test]
    fn test_textured_quad_with_offset() {
        let mut gpu = GPU::new();

        // Set up texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7FFF);
            }
        }

        gpu.draw_offset = (100, 50);

        let vertices = [
            Vertex { x: 50, y: 50 },
            Vertex { x: 150, y: 50 },
            Vertex { x: 150, y: 150 },
            Vertex { x: 50, y: 150 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 15, v: 15 },
            TexCoord { u: 0, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_textured_quad(&vertices, &texcoords, &texture_info, &color, false);

        // Center should be at (100+100, 100+50) = (200, 150)
        let pixel = gpu.read_vram(200, 150);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_textured_triangle_darken_modulation() {
        let mut gpu = GPU::new();

        // Set up white texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x7FFF); // White
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };

        // Per PSX-SPX: Values < 128 darken the texture
        let color = Color {
            r: 64,
            g: 64,
            b: 64,
        }; // Half brightness

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        let pixel = gpu.read_vram(150, 133);
        // Should be darker than original white texture
        assert!(pixel < 0x7FFF);
    }

    #[test]
    fn test_textured_triangle_brighten_modulation() {
        let mut gpu = GPU::new();

        // Set up gray texture
        for y in 0..16 {
            for x in 0..16 {
                gpu.write_vram(x, y, 0x4210); // Medium gray
            }
        }

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let texcoords = [
            TexCoord { u: 0, v: 0 },
            TexCoord { u: 15, v: 0 },
            TexCoord { u: 7, v: 15 },
        ];
        let texture_info = TextureInfo {
            page_x: 0,
            page_y: 0,
            clut_x: 0,
            clut_y: 0,
            depth: TextureDepth::T15Bit,
        };

        // Per PSX-SPX: Values > 128 brighten the texture
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        }; // Maximum brightness

        gpu.render_textured_triangle(&vertices, &texcoords, &texture_info, &color, false);

        let pixel = gpu.read_vram(150, 133);
        // Should be brighter (clamped to max)
        assert_ne!(pixel, 0x0000);
    }
}
