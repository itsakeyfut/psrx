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

//! Quadrilateral rendering implementation
//!
//! Implements monochrome (flat-shaded) quad rasterization by decomposing into triangles.

use super::super::primitives::{Color, Vertex};
use super::super::GPU;

impl GPU {
    /// Render a monochrome (flat-shaded) quadrilateral
    ///
    /// Quads are rendered as two triangles: (v0, v1, v2) and (v0, v2, v3).
    /// Applies the drawing offset and delegates to triangle rendering.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 4 vertices defining the quad (in order)
    /// * `color` - Flat color for the entire quad
    /// * `semi_transparent` - Whether semi-transparency is enabled
    pub(crate) fn render_monochrome_quad(
        &mut self,
        vertices: &[Vertex; 4],
        color: &Color,
        semi_transparent: bool,
    ) {
        // Quads are rendered as two triangles
        let tri1 = [vertices[0], vertices[1], vertices[2]];
        let tri2 = [vertices[0], vertices[2], vertices[3]];

        self.render_monochrome_triangle(&tri1, color, semi_transparent);
        self.render_monochrome_triangle(&tri2, color, semi_transparent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monochrome_quad_basic_rendering() {
        let mut gpu = GPU::new();

        // Draw a simple blue quad
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 0,
            g: 0,
            b: 255,
        };

        gpu.render_monochrome_quad(&vertices, &color, false);

        // Verify center pixel
        let center_pixel = gpu.read_vram(150, 150);
        // Blue in RGB15 is 0x7C00
        assert_eq!(center_pixel & 0x7C00, 0x7C00);
    }

    #[test]
    fn test_monochrome_quad_with_drawing_offset() {
        let mut gpu = GPU::new();

        // Set drawing offset
        gpu.draw_offset = (100, 50);

        let vertices = [
            Vertex { x: 50, y: 50 },
            Vertex { x: 150, y: 50 },
            Vertex { x: 150, y: 150 },
            Vertex { x: 50, y: 150 },
        ];
        let color = Color {
            r: 255,
            g: 128,
            b: 0,
        };

        gpu.render_monochrome_quad(&vertices, &color, false);

        // Center should be at (100+100, 100+50) = (200, 150)
        let pixel = gpu.read_vram(200, 150);
        // Orange: verify red and some green
        assert_eq!(pixel & 0x001F, 31); // Red component
        assert!((pixel >> 5) & 0x1F > 0); // Green component
    }

    #[test]
    fn test_monochrome_quad_decomposition_into_triangles() {
        let mut gpu = GPU::new();

        // Per implementation: Quad (v0, v1, v2, v3) = Triangle(v0, v1, v2) + Triangle(v0, v2, v3)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 0,
        };

        gpu.render_monochrome_quad(&vertices, &color, false);

        // Check multiple points to verify both triangles are drawn
        // Upper triangle area
        assert_ne!(gpu.read_vram(150, 120), 0x0000);
        // Lower triangle area
        assert_ne!(gpu.read_vram(120, 180), 0x0000);
    }

    #[test]
    fn test_monochrome_quad_semi_transparent() {
        let mut gpu = GPU::new();

        // Draw opaque background quad
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let bg_color = Color {
            r: 248,
            g: 248,
            b: 248,
        };

        gpu.render_monochrome_quad(&vertices, &bg_color, false);

        // Draw semi-transparent quad on top (average mode)
        gpu.draw_mode.semi_transparency = 0;
        let fg_color = Color { r: 0, g: 0, b: 0 };

        gpu.render_monochrome_quad(&vertices, &fg_color, true);

        // Center should be blended
        let pixel = gpu.read_vram(150, 150);
        let expected_gray = (15 << 10) | (15 << 5) | 15;
        assert_eq!(pixel, expected_gray);
    }

    #[test]
    fn test_monochrome_quad_non_rectangular() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Quads don't have to be rectangular (can be trapezoids, etc.)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 250, y: 120 }, // Skewed
            Vertex { x: 230, y: 200 },
            Vertex { x: 80, y: 180 },
        ];
        let color = Color {
            r: 0,
            g: 255,
            b: 255,
        };

        // Should render without issues
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_degenerate_colinear_vertices() {
        let mut gpu = GPU::new();

        // All vertices on same line (degenerate quad)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 300, y: 100 },
            Vertex { x: 400, y: 100 },
        ];
        let color = Color {
            r: 255,
            g: 0,
            b: 0,
        };

        // Should not crash
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_self_intersecting() {
        let mut gpu = GPU::new();

        // Self-intersecting quad (bow-tie shape)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 200 }, // Cross pattern
            Vertex { x: 200, y: 100 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 128,
            g: 255,
            b: 128,
        };

        // Should render as two triangles (implementation-defined behavior)
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_negative_coordinates() {
        let mut gpu = GPU::new();

        // Quad with negative coordinates
        let vertices = [
            Vertex { x: -100, y: -100 },
            Vertex { x: 100, y: -100 },
            Vertex { x: 100, y: 100 },
            Vertex { x: -100, y: 100 },
        ];
        let color = Color {
            r: 192,
            g: 192,
            b: 192,
        };

        // Should handle wrapping
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_boundary_coordinates() {
        let mut gpu = GPU::new();

        // Quad at VRAM boundaries
        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 1023, y: 511 },
            Vertex { x: 0, y: 511 },
        ];
        let color = Color {
            r: 255,
            g: 0,
            b: 255,
        };

        // Should fill entire VRAM
        gpu.render_monochrome_quad(&vertices, &color, false);

        // Check corners
        let corner = gpu.read_vram(0, 0);
        assert_ne!(corner, 0x0000);
    }

    #[test]
    fn test_monochrome_quad_maximum_size() {
        let mut gpu = GPU::new();

        // Maximum size quad per PSX-SPX
        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 1023, y: 511 },
            Vertex { x: 0, y: 511 },
        ];
        let color = Color {
            r: 64,
            g: 128,
            b: 255,
        };

        // Should not crash
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_single_pixel() {
        let mut gpu = GPU::new();

        // Minimal quad (all vertices very close)
        let vertices = [
            Vertex { x: 150, y: 150 },
            Vertex { x: 151, y: 150 },
            Vertex { x: 151, y: 151 },
            Vertex { x: 150, y: 151 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        // Should render at least one pixel
        gpu.render_monochrome_quad(&vertices, &color, false);
        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x7FFF);
    }

    #[test]
    fn test_monochrome_quad_color_conversion() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 200,
            g: 150,
            b: 100,
        };

        gpu.render_monochrome_quad(&vertices, &color, false);

        let pixel = gpu.read_vram(150, 150);

        // Verify 8-bit to 5-bit conversion
        let r = pixel & 0x1F;
        let g = (pixel >> 5) & 0x1F;
        let b = (pixel >> 10) & 0x1F;

        assert_eq!(r, 200 >> 3); // 25
        assert_eq!(g, 150 >> 3); // 18
        assert_eq!(b, 100 >> 3); // 12
    }

    #[test]
    fn test_monochrome_quad_all_black() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color { r: 0, g: 0, b: 0 };

        gpu.render_monochrome_quad(&vertices, &color, false);

        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x0000);
    }

    #[test]
    fn test_monochrome_quad_all_white() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        gpu.render_monochrome_quad(&vertices, &color, false);

        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x7FFF);
    }

    #[test]
    fn test_monochrome_quad_concave() {
        let mut gpu = GPU::new();

        // Concave quad (arrow shape)
        let vertices = [
            Vertex { x: 100, y: 150 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 150 }, // Indent
            Vertex { x: 200, y: 200 },
        ];
        let color = Color {
            r: 255,
            g: 192,
            b: 0,
        };

        // Should render as two triangles
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_with_extreme_offset() {
        let mut gpu = GPU::new();

        // Extreme offset causing wrapping
        gpu.draw_offset = (30000, 20000);

        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 100, y: 0 },
            Vertex { x: 100, y: 100 },
            Vertex { x: 0, y: 100 },
        ];
        let color = Color {
            r: 128,
            g: 0,
            b: 128,
        };

        // Should handle wrapping
        gpu.render_monochrome_quad(&vertices, &color, false);
    }

    #[test]
    fn test_monochrome_quad_clockwise_vs_counterclockwise() {
        let mut gpu = GPU::new();

        // Clockwise winding
        let vertices_cw = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let color = Color {
            r: 255,
            g: 0,
            b: 0,
        };

        gpu.render_monochrome_quad(&vertices_cw, &color, false);

        // Counter-clockwise winding
        let vertices_ccw = [
            Vertex { x: 300, y: 100 },
            Vertex { x: 300, y: 200 },
            Vertex { x: 400, y: 200 },
            Vertex { x: 400, y: 100 },
        ];

        gpu.render_monochrome_quad(&vertices_ccw, &color, false);

        // Both should render
        assert_ne!(gpu.read_vram(150, 150), 0x0000);
        assert_ne!(gpu.read_vram(350, 150), 0x0000);
    }

    #[test]
    fn test_monochrome_quad_multiple_overlapping() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];

        // First quad: cyan
        let color1 = Color {
            r: 0,
            g: 255,
            b: 255,
        };
        gpu.render_monochrome_quad(&vertices, &color1, false);

        // Second quad: yellow (overwrites)
        let color2 = Color {
            r: 255,
            g: 255,
            b: 0,
        };
        gpu.render_monochrome_quad(&vertices, &color2, false);

        // Should show yellow (latest)
        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel & 0x03FF, 0x03FF); // Red + Green
    }

    #[test]
    fn test_monochrome_quad_large_coordinates() {
        let mut gpu = GPU::new();

        // Large coordinates that will wrap
        let vertices = [
            Vertex { x: 20000, y: 10000 },
            Vertex { x: 21000, y: 10000 },
            Vertex { x: 21000, y: 11000 },
            Vertex { x: 20000, y: 11000 },
        ];
        let color = Color {
            r: 100,
            g: 150,
            b: 200,
        };

        // Should wrap and not crash
        gpu.render_monochrome_quad(&vertices, &color, false);
    }
}
