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

//! Gradient (Gouraud-shaded) rendering implementation
//!
//! Implements gradient triangle and quad rasterization with per-vertex colors.

use super::super::primitives::{Color, Vertex};
use super::super::GPU;

impl GPU {
    /// Render a gradient (Gouraud-shaded) triangle
    ///
    /// Applies the drawing offset to all vertices and rasterizes the triangle
    /// with color interpolation using barycentric coordinates.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 3 vertices defining the triangle
    /// * `colors` - Array of 3 colors, one per vertex
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Algorithm
    ///
    /// Colors are interpolated across the triangle interior using barycentric
    /// coordinates, providing smooth Gouraud shading. This creates a gradient
    /// effect commonly used for lighting.
    ///
    /// # Notes
    ///
    /// Semi-transparency is currently ignored (will be implemented in #36).
    /// The drawing offset is applied to all vertices before rasterization.
    pub(crate) fn render_gradient_triangle(
        &mut self,
        vertices: &[Vertex; 3],
        colors: &[Color; 3],
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
            "Rendering {}gradient triangle: ({}, {}), ({}, {}), ({}, {}) colors=({},{},{}), ({},{},{}), ({},{},{})",
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
            colors[0].r,
            colors[0].g,
            colors[0].b,
            colors[1].r,
            colors[1].g,
            colors[1].b,
            colors[2].r,
            colors[2].g,
            colors[2].b
        );

        let c0 = (colors[0].r, colors[0].g, colors[0].b);
        let c1 = (colors[1].r, colors[1].g, colors[1].b);
        let c2 = (colors[2].r, colors[2].g, colors[2].b);

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Rasterize the gradient triangle
        self.rasterizer
            .draw_gradient_triangle(&mut self.vram, v0, c0, v1, c1, v2, c2);
    }

    /// Render a gradient (Gouraud-shaded) quadrilateral
    ///
    /// Renders a quad as two triangles with gradient shading. The quad is
    /// split into triangles (v0, v1, v2) and (v1, v2, v3).
    ///
    /// # Arguments
    ///
    /// * `vertices` - Array of 4 vertices defining the quad
    /// * `colors` - Array of 4 colors, one per vertex
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Notes
    ///
    /// The quad is rendered as two gradient triangles. Colors are interpolated
    /// independently for each triangle, which may create a visible seam if the
    /// quad is not coplanar in 3D space.
    pub(crate) fn render_gradient_quad(
        &mut self,
        vertices: &[Vertex; 4],
        colors: &[Color; 4],
        semi_transparent: bool,
    ) {
        log::trace!(
            "Rendering {}gradient quad as two triangles",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            }
        );

        // Render as two triangles: (v0, v1, v2) and (v1, v2, v3)
        self.render_gradient_triangle(
            &[vertices[0], vertices[1], vertices[2]],
            &[colors[0], colors[1], colors[2]],
            semi_transparent,
        );

        self.render_gradient_triangle(
            &[vertices[1], vertices[2], vertices[3]],
            &[colors[1], colors[2], colors[3]],
            semi_transparent,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_triangle_basic() {
        let mut gpu = GPU::new();

        // Triangle with red, green, blue at vertices
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            }, // Red
            Color {
                r: 0,
                g: 255,
                b: 0,
            }, // Green
            Color {
                r: 0,
                g: 0,
                b: 255,
            }, // Blue
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Center should have interpolated color (mix of all three)
        let pixel = gpu.read_vram(150, 133);
        // Should have components of all three colors
        assert!((pixel & 0x001F) > 0); // Red component
        assert!(((pixel >> 5) & 0x1F) > 0); // Green component
        assert!(((pixel >> 10) & 0x1F) > 0); // Blue component
    }

    #[test]
    fn test_gradient_triangle_black_to_white() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Gouraud shading interpolates colors
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color { r: 0, g: 0, b: 0 }, // Black
            Color { r: 0, g: 0, b: 0 }, // Black
            Color {
                r: 248,
                g: 248,
                b: 248,
            }, // White
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Top edge should be black
        let top_pixel = gpu.read_vram(150, 100);
        assert_eq!(top_pixel, 0x0000);

        // Bottom should be brighter
        let bottom_pixel = gpu.read_vram(150, 180);
        assert!(bottom_pixel > 0);
    }

    #[test]
    fn test_gradient_triangle_with_drawing_offset() {
        let mut gpu = GPU::new();

        gpu.draw_offset = (50, 30);

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Center should be at (150+50, 133+30) = (200, 163)
        let pixel = gpu.read_vram(200, 163);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_gradient_triangle_uniform_color() {
        let mut gpu = GPU::new();

        // All vertices same color (should look like flat-shaded)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color {
                r: 128,
                g: 128,
                b: 128,
            },
            Color {
                r: 128,
                g: 128,
                b: 128,
            },
            Color {
                r: 128,
                g: 128,
                b: 128,
            },
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // All pixels should be same color
        let pixel1 = gpu.read_vram(150, 120);
        let pixel2 = gpu.read_vram(150, 160);
        assert_eq!(pixel1, pixel2);
    }

    #[test]
    fn test_gradient_triangle_color_interpolation() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Colors interpolated using barycentric coordinates
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            }, // Red at v0
            Color {
                r: 255,
                g: 0,
                b: 0,
            }, // Red at v1
            Color {
                r: 0,
                g: 0,
                b: 255,
            }, // Blue at v2
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Top edge should be pure red
        let top_pixel = gpu.read_vram(150, 100);
        assert_eq!(top_pixel & 0x001F, 0x001F); // Red component

        // Bottom vertex should be pure blue
        let bottom_pixel = gpu.read_vram(150, 200);
        assert_eq!((bottom_pixel >> 10) & 0x1F, 31); // Blue component
    }

    #[test]
    fn test_gradient_triangle_negative_coordinates() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: -50, y: -50 },
            Vertex { x: 100, y: -50 },
            Vertex { x: 25, y: 100 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
        ];

        // Should handle wrapping
        gpu.render_gradient_triangle(&vertices, &colors, false);
    }

    #[test]
    fn test_gradient_triangle_boundary_coordinates() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 512, y: 511 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Check corner
        assert_ne!(gpu.read_vram(0, 0), 0x0000);
    }

    #[test]
    fn test_gradient_triangle_degenerate() {
        let mut gpu = GPU::new();

        // Colinear vertices
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 300, y: 100 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
        ];

        // Should not crash
        gpu.render_gradient_triangle(&vertices, &colors, false);
    }

    #[test]
    fn test_gradient_quad_basic() {
        let mut gpu = GPU::new();

        // Quad with different color at each corner
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            }, // Red
            Color {
                r: 0,
                g: 255,
                b: 0,
            }, // Green
            Color {
                r: 0,
                g: 0,
                b: 255,
            }, // Blue
            Color {
                r: 255,
                g: 255,
                b: 0,
            }, // Yellow
        ];

        gpu.render_gradient_quad(&vertices, &colors, false);

        // Center should have mixed colors
        let pixel = gpu.read_vram(150, 150);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_gradient_quad_decomposition() {
        let mut gpu = GPU::new();

        // Per implementation: Quad splits into (v0,v1,v2) and (v1,v2,v3)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];

        gpu.render_gradient_quad(&vertices, &colors, false);

        // Check both triangle areas are drawn
        assert_ne!(gpu.read_vram(150, 120), 0x0000); // Upper triangle
        assert_ne!(gpu.read_vram(120, 180), 0x0000); // Lower triangle
    }

    #[test]
    fn test_gradient_quad_with_offset() {
        let mut gpu = GPU::new();

        gpu.draw_offset = (100, 50);

        let vertices = [
            Vertex { x: 50, y: 50 },
            Vertex { x: 150, y: 50 },
            Vertex { x: 150, y: 150 },
            Vertex { x: 50, y: 150 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];

        gpu.render_gradient_quad(&vertices, &colors, false);

        // Center should be at (100+100, 100+50) = (200, 150)
        let pixel = gpu.read_vram(200, 150);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_gradient_quad_uniform_color() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
            Vertex { x: 100, y: 200 },
        ];
        let colors = [
            Color {
                r: 200,
                g: 100,
                b: 50,
            },
            Color {
                r: 200,
                g: 100,
                b: 50,
            },
            Color {
                r: 200,
                g: 100,
                b: 50,
            },
            Color {
                r: 200,
                g: 100,
                b: 50,
            },
        ];

        gpu.render_gradient_quad(&vertices, &colors, false);

        // All pixels should be same color
        let pixel1 = gpu.read_vram(120, 120);
        let pixel2 = gpu.read_vram(180, 180);
        assert_eq!(pixel1, pixel2);
    }

    #[test]
    fn test_gradient_triangle_high_contrast() {
        let mut gpu = GPU::new();

        // High contrast gradient
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color { r: 0, g: 0, b: 0 }, // Black
            Color { r: 0, g: 0, b: 0 }, // Black
            Color {
                r: 255,
                g: 255,
                b: 255,
            }, // White
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        // Gradient should be smooth from dark to light
        let mid_pixel = gpu.read_vram(150, 150);
        assert!(mid_pixel > 0 && mid_pixel < 0x7FFF);
    }

    #[test]
    fn test_gradient_quad_non_rectangular() {
        let mut gpu = GPU::new();

        // Non-rectangular quad (trapezoid)
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 250, y: 120 },
            Vertex { x: 230, y: 200 },
            Vertex { x: 80, y: 180 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];

        // Should render without issues
        gpu.render_gradient_quad(&vertices, &colors, false);
    }

    #[test]
    fn test_gradient_triangle_single_channel_gradient() {
        let mut gpu = GPU::new();

        // Gradient only in red channel
        let vertices = [
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = [
            Color { r: 0, g: 0, b: 0 },
            Color { r: 0, g: 0, b: 0 },
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
        ];

        gpu.render_gradient_triangle(&vertices, &colors, false);

        let pixel = gpu.read_vram(150, 150);
        // Should have red component, no green or blue
        assert!((pixel & 0x001F) > 0);
        assert_eq!((pixel >> 5) & 0x1F, 0);
        assert_eq!((pixel >> 10) & 0x1F, 0);
    }

    #[test]
    fn test_gradient_quad_boundary_coordinates() {
        let mut gpu = GPU::new();

        let vertices = [
            Vertex { x: 0, y: 0 },
            Vertex { x: 1023, y: 0 },
            Vertex { x: 1023, y: 511 },
            Vertex { x: 0, y: 511 },
        ];
        let colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
            },
            Color {
                r: 0,
                g: 0,
                b: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];

        // Full VRAM gradient
        gpu.render_gradient_quad(&vertices, &colors, false);

        assert_ne!(gpu.read_vram(0, 0), 0x0000);
    }
}
