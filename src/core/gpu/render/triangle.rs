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

//! Triangle rendering implementation
//!
//! Implements monochrome (flat-shaded) triangle rasterization with optional semi-transparency.

use super::super::primitives::{BlendMode, Color, Vertex};
use super::super::GPU;

impl GPU {
    /// Render a monochrome (flat-shaded) triangle
    ///
    /// Applies the drawing offset to all vertices and rasterizes the triangle
    /// using the software rasterizer. If semi-transparency is enabled, the triangle
    /// is blended with the existing background using the current blend mode.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 3 vertices defining the triangle
    /// * `color` - Flat color for the entire triangle
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Semi-Transparency
    ///
    /// When semi-transparency is enabled, the GPU's current semi-transparency mode
    /// (from draw_mode.semi_transparency) determines the blending formula:
    /// - Mode 0 (Average): 0.5×Background + 0.5×Foreground
    /// - Mode 1 (Additive): 1.0×Background + 1.0×Foreground
    /// - Mode 2 (Subtractive): 1.0×Background - 1.0×Foreground
    /// - Mode 3 (AddQuarter): 1.0×Background + 0.25×Foreground
    ///
    /// # Notes
    ///
    /// The drawing offset is applied to all vertices before rasterization.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // This is a private method used internally by the GPU
    /// use psrx::core::gpu::{GPU, Vertex, Color};
    ///
    /// let mut gpu = GPU::new();
    ///
    /// // Draw opaque red triangle
    /// let vertices = [
    ///     Vertex { x: 100, y: 100 },
    ///     Vertex { x: 200, y: 100 },
    ///     Vertex { x: 150, y: 200 },
    /// ];
    /// let color = Color { r: 255, g: 0, b: 0 };
    /// gpu.render_monochrome_triangle(&vertices, &color, false);
    ///
    /// // Draw semi-transparent black triangle on top
    /// let color2 = Color { r: 0, g: 0, b: 0 };
    /// gpu.render_monochrome_triangle(&vertices, &color2, true);
    /// ```
    pub(crate) fn render_monochrome_triangle(
        &mut self,
        vertices: &[Vertex; 3],
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

        log::trace!(
            "Rendering {}triangle: ({}, {}), ({}, {}), ({}, {}) color=({},{},{}){}",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            },
            v0.0,
            v0.1,
            v1.0,
            v1.1,
            v2.0,
            v2.1,
            color.r,
            color.g,
            color.b,
            if semi_transparent {
                format!(" mode={}", self.draw_mode.semi_transparency)
            } else {
                String::new()
            }
        );

        // Convert color to 15-bit RGB format
        let color_15bit = color.to_rgb15();

        // Rasterize the triangle with or without blending
        if semi_transparent {
            // Use blending mode from draw_mode
            let blend_mode = BlendMode::from_bits(self.draw_mode.semi_transparency);
            self.rasterizer.draw_triangle_blended(
                &mut self.vram,
                v0,
                v1,
                v2,
                color_15bit,
                blend_mode,
            );
        } else {
            // Opaque rendering
            self.rasterizer
                .draw_triangle(&mut self.vram, v0, v1, v2, color_15bit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monochrome_triangle_basic_rendering() {
        let mut gpu = GPU::new();

        // Draw a simple red triangle
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let color = Color { r: 255, g: 0, b: 0 };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Verify VRAM has been modified (at least center pixel should be colored)
        let center_x = 150u16;
        let center_y = 133u16;
        let pixel = gpu.read_vram(center_x, center_y);

        // Red in RGB15 is 0x001F (5 bits for red)
        assert_eq!(pixel & 0x001F, 0x001F);
    }

    #[test]
    fn test_monochrome_triangle_with_drawing_offset() {
        let mut gpu = GPU::new();

        // Set drawing offset
        gpu.draw_offset = (50, 30);

        // Draw triangle at base position
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let color = Color { r: 0, g: 255, b: 0 };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Verify triangle is rendered at offset position
        // Center should be at (150+50, 133+30) = (200, 163)
        let pixel = gpu.read_vram(200, 163);

        // Green in RGB15 is 0x03E0 (5 bits for green, shifted left by 5)
        assert_eq!(pixel & 0x03E0, 0x03E0);
    }

    #[test]
    fn test_monochrome_triangle_coordinate_wrapping() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Coordinates wrap around VRAM boundaries
        // Drawing offset causes wrapping: large positive + positive = wrap
        gpu.draw_offset = (1000, 500);

        let vertices = [
            Vertex { x: 50, y: 50 },
            Vertex { x: 100, y: 50 },
            Vertex { x: 75, y: 100 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        // Should not crash even with wrapping coordinates
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_negative_coordinates() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Coordinates can be negative (signed 16-bit)
        let vertices = [
            Vertex { x: -50, y: -50 },
            Vertex { x: 50, y: -50 },
            Vertex { x: 0, y: 50 },
        ];
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        // Should handle negative coordinates (they wrap around)
        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Verify center pixel within VRAM bounds is rendered
        // After wrapping, negative coordinates become large positive values
        let pixel = gpu.read_vram(0, 0);
        // Should have some color (gray = 0x4210)
        assert_eq!(pixel & 0x7FFF, 0x4210);
    }

    #[test]
    fn test_monochrome_triangle_maximum_vertex_distance() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Maximum vertex distance is 1023 horizontal, 511 vertical
        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 512, y: 511 },
        ];
        let color = Color {
            r: 255,
            g: 0,
            b: 255,
        };

        // Should handle maximum distances
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_semi_transparent_average() {
        let mut gpu = GPU::new();

        // First draw an opaque background
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let bg_color = Color {
            r: 248,
            g: 248,
            b: 248,
        }; // White

        gpu.render_monochrome_triangle(&vertices, &bg_color, false);

        // Draw semi-transparent black triangle on top
        // Mode 0 (Average): 0.5*Background + 0.5*Foreground
        gpu.draw_mode.semi_transparency = 0;
        let fg_color = Color { r: 0, g: 0, b: 0 }; // Black

        gpu.render_monochrome_triangle(&vertices, &fg_color, true);

        // Check center pixel - should be blended (approximately half intensity)
        let pixel = gpu.read_vram(150, 133);
        // White (31,31,31) + Black (0,0,0) / 2 = Gray (15,15,15) = 0x3DEF
        let expected_gray = (15 << 10) | (15 << 5) | 15;
        assert_eq!(pixel, expected_gray);
    }

    #[test]
    fn test_monochrome_triangle_semi_transparent_additive() {
        let mut gpu = GPU::new();

        // Draw background
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let bg_color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_monochrome_triangle(&vertices, &bg_color, false);

        // Mode 1 (Additive): 1.0*Background + 1.0*Foreground (clamped)
        gpu.draw_mode.semi_transparency = 1;
        let fg_color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_monochrome_triangle(&vertices, &fg_color, true);

        // Should be clamped to max (31,31,31)
        let pixel = gpu.read_vram(150, 133);
        assert_eq!(pixel, 0x7FFF); // Max RGB15
    }

    #[test]
    fn test_monochrome_triangle_semi_transparent_subtractive() {
        let mut gpu = GPU::new();

        // Draw background
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let bg_color = Color {
            r: 248,
            g: 248,
            b: 248,
        };

        gpu.render_monochrome_triangle(&vertices, &bg_color, false);

        // Mode 2 (Subtractive): 1.0*Background - 1.0*Foreground (clamped to 0)
        gpu.draw_mode.semi_transparency = 2;
        let fg_color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_monochrome_triangle(&vertices, &fg_color, true);

        // Should subtract and clamp to positive values
        let pixel = gpu.read_vram(150, 133);
        // 31 - 16 = 15 per channel
        let expected = (15 << 10) | (15 << 5) | 15;
        assert_eq!(pixel, expected);
    }

    #[test]
    fn test_monochrome_triangle_semi_transparent_add_quarter() {
        let mut gpu = GPU::new();

        // Draw background
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let bg_color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_monochrome_triangle(&vertices, &bg_color, false);

        // Mode 3 (AddQuarter): 1.0*Background + 0.25*Foreground
        gpu.draw_mode.semi_transparency = 3;
        let fg_color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        gpu.render_monochrome_triangle(&vertices, &fg_color, true);

        // Should be background + quarter of foreground
        // 16 + (16 / 4) = 16 + 4 = 20 per channel
        let pixel = gpu.read_vram(150, 133);
        let expected = (20 << 10) | (20 << 5) | 20;
        assert_eq!(pixel, expected);
    }

    #[test]
    fn test_monochrome_triangle_degenerate_colinear_vertices() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Degenerate triangles (colinear points) should not crash
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 300, y: 100 }, // All on same horizontal line
        ];
        let color = Color { r: 255, g: 0, b: 0 };

        // Should not crash, may or may not render
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_degenerate_single_point() {
        let mut gpu = GPU::new();

        // All vertices at same point
        let vertices = [
            Vertex { x: 150, y: 150 },
            Vertex { x: 150, y: 150 },
            Vertex { x: 150, y: 150 },
        ];
        let color = Color { r: 0, g: 255, b: 0 };

        // Should not crash
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_vertex_order_independence() {
        let mut gpu = GPU::new();

        // Triangle with vertices in different winding orders should render
        // Per PSX-SPX: Backface culling is not automatic
        let vertices_cw = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let vertices_ccw = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 150, y: 200 },
            Vertex { x: 200, y: 100 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 0,
        };

        // Both should render without error
        gpu.render_monochrome_triangle(&vertices_cw, &color, false);
        gpu.render_monochrome_triangle(&vertices_ccw, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_boundary_coordinates() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Coordinates at VRAM boundaries (1024×512)
        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 0, y: 511 },
        ];
        let color = Color { r: 0, g: 0, b: 255 };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Check corner pixel
        let pixel = gpu.read_vram(0, 0);
        // Blue in RGB15 is 0x7C00
        assert_eq!(pixel & 0x7C00, 0x7C00);
    }

    #[test]
    fn test_monochrome_triangle_color_conversion() {
        let mut gpu = GPU::new();

        // Test 8-bit to 5-bit color conversion
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];

        // 255 → 31, 128 → 16, 0 → 0 in 5-bit
        let color = Color {
            r: 255,
            g: 128,
            b: 64,
        };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        let pixel = gpu.read_vram(150, 133);

        // Extract RGB components
        let r = pixel & 0x1F;
        let g = (pixel >> 5) & 0x1F;
        let b = (pixel >> 10) & 0x1F;

        assert_eq!(r, 31); // 255 >> 3 = 31
        assert_eq!(g, 16); // 128 >> 3 = 16
        assert_eq!(b, 8); // 64 >> 3 = 8
    }

    #[test]
    fn test_monochrome_triangle_zero_area() {
        let mut gpu = GPU::new();

        // Triangle with near-zero area (very thin)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 101, y: 100 },
            Vertex { x: 100, y: 101 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        // Should not crash
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_large_coordinates_wrapping() {
        let mut gpu = GPU::new();

        // Test wrapping with large coordinates beyond i16 range
        let vertices = [
            Vertex { x: 30000, y: 30000 },
            Vertex { x: 31000, y: 30000 },
            Vertex { x: 30500, y: 31000 },
        ];
        let color = Color {
            r: 128,
            g: 64,
            b: 192,
        };

        // Should wrap around and not crash
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_triangle_all_black() {
        let mut gpu = GPU::new();

        // Black triangle
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let color = Color { r: 0, g: 0, b: 0 };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Verify black pixel at center
        let pixel = gpu.read_vram(150, 133);
        assert_eq!(pixel, 0x0000);
    }

    #[test]
    fn test_monochrome_triangle_all_white() {
        let mut gpu = GPU::new();

        // White triangle
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        gpu.render_monochrome_triangle(&vertices, &color, false);

        // Verify white pixel at center (RGB15 max = 0x7FFF)
        let pixel = gpu.read_vram(150, 133);
        assert_eq!(pixel, 0x7FFF);
    }

    #[test]
    fn test_monochrome_triangle_multiple_overlapping() {
        let mut gpu = GPU::new();

        // Draw multiple overlapping triangles
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];

        // First triangle: red
        let color1 = Color { r: 255, g: 0, b: 0 };
        gpu.render_monochrome_triangle(&vertices, &color1, false);

        // Second triangle: green (overwrites)
        let color2 = Color { r: 0, g: 255, b: 0 };
        gpu.render_monochrome_triangle(&vertices, &color2, false);

        // Center should be green (latest draw)
        let pixel = gpu.read_vram(150, 133);
        assert_eq!(pixel & 0x03E0, 0x03E0); // Green bits
    }

    #[test]
    fn test_monochrome_triangle_with_extreme_offset() {
        let mut gpu = GPU::new();

        // Extreme drawing offset that causes wrapping
        gpu.draw_offset = (32000, 16000);

        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 100, y: 0 },
            Vertex { x: 50, y: 100 },
        ];
        let color = Color {
            r: 192,
            g: 192,
            b: 0,
        };

        // Should handle wrapping gracefully
        gpu.render_monochrome_triangle(&vertices, &color, false);
    }
}
