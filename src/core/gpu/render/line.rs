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

//! Line rendering implementation
//!
//! Implements line and polyline rasterization using Bresenham's algorithm.

use super::super::primitives::{Color, Vertex};
use super::super::GPU;

impl GPU {
    /// Render a monochrome line
    ///
    /// Applies the drawing offset to both vertices and rasterizes the line
    /// using Bresenham's algorithm.
    ///
    /// # Arguments
    ///
    /// * `v0` - Start vertex
    /// * `v1` - End vertex
    /// * `color` - Line color
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Notes
    ///
    /// Semi-transparency is currently ignored (will be implemented in #36).
    /// The drawing offset is applied to both endpoints before rasterization.
    pub(crate) fn render_line(
        &mut self,
        v0: Vertex,
        v1: Vertex,
        color: Color,
        semi_transparent: bool,
    ) {
        // Apply drawing offset
        let x0 = v0.x.wrapping_add(self.draw_offset.0);
        let y0 = v0.y.wrapping_add(self.draw_offset.1);
        let x1 = v1.x.wrapping_add(self.draw_offset.0);
        let y1 = v1.y.wrapping_add(self.draw_offset.1);

        log::trace!(
            "Rendering {}line: ({}, {}) -> ({}, {}) color=({},{},{})",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            },
            x0,
            y0,
            x1,
            y1,
            color.r,
            color.g,
            color.b
        );

        // Convert color to 15-bit RGB format
        let color_15bit = color.to_rgb15();

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Rasterize the line
        self.rasterizer
            .draw_line(&mut self.vram, x0, y0, x1, y1, color_15bit);
    }

    /// Render a polyline (connected line segments)
    ///
    /// Applies the drawing offset to all vertices and draws connected line
    /// segments between consecutive vertices.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Slice of vertices defining the polyline
    /// * `color` - Line color
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Notes
    ///
    /// Requires at least 2 vertices. If fewer than 2 vertices are provided,
    /// no drawing occurs.
    pub(crate) fn render_polyline(
        &mut self,
        vertices: &[Vertex],
        color: Color,
        semi_transparent: bool,
    ) {
        if vertices.len() < 2 {
            return;
        }

        log::trace!(
            "Rendering {}polyline with {} vertices, color=({},{},{})",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            },
            vertices.len(),
            color.r,
            color.g,
            color.b
        );

        // Convert color to 15-bit RGB format
        let color_15bit = color.to_rgb15();

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Apply drawing offset to all vertices
        let points: Vec<(i16, i16)> = vertices
            .iter()
            .map(|v| {
                (
                    v.x.wrapping_add(self.draw_offset.0),
                    v.y.wrapping_add(self.draw_offset.1),
                )
            })
            .collect();

        // Rasterize the polyline
        self.rasterizer
            .draw_polyline(&mut self.vram, &points, color_15bit);
    }

    /// Render a shaded line with Gouraud shading
    ///
    /// Applies the drawing offset to both vertices and rasterizes the line
    /// with color interpolation between the two endpoints.
    ///
    /// # Arguments
    ///
    /// * `v0` - Start vertex
    /// * `c0` - Start vertex color
    /// * `v1` - End vertex
    /// * `c1` - End vertex color
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Notes
    ///
    /// Semi-transparency is currently ignored (will be implemented in #36).
    /// The drawing offset is applied to both endpoints before rasterization.
    pub(crate) fn render_shaded_line(
        &mut self,
        v0: Vertex,
        c0: Color,
        v1: Vertex,
        c1: Color,
        semi_transparent: bool,
    ) {
        // Apply drawing offset
        let x0 = v0.x.wrapping_add(self.draw_offset.0);
        let y0 = v0.y.wrapping_add(self.draw_offset.1);
        let x1 = v1.x.wrapping_add(self.draw_offset.0);
        let y1 = v1.y.wrapping_add(self.draw_offset.1);

        log::trace!(
            "Rendering {}shaded line: ({}, {}) color=({},{},{}) -> ({}, {}) color=({},{},{})",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            },
            x0,
            y0,
            c0.r,
            c0.g,
            c0.b,
            x1,
            y1,
            c1.r,
            c1.g,
            c1.b
        );

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Rasterize the line with color interpolation
        self.rasterizer.draw_gradient_line(
            &mut self.vram,
            x0,
            y0,
            (c0.r, c0.g, c0.b),
            x1,
            y1,
            (c1.r, c1.g, c1.b),
        );
    }

    /// Render a shaded polyline (connected line segments with per-vertex colors)
    ///
    /// Applies the drawing offset to all vertices and draws connected line
    /// segments with color interpolation between consecutive vertices.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Slice of vertices defining the polyline
    /// * `colors` - Slice of colors for each vertex
    /// * `semi_transparent` - Whether semi-transparency is enabled
    ///
    /// # Notes
    ///
    /// Requires at least 2 vertices and 2 colors. If fewer than 2 are provided,
    /// no drawing occurs. The number of colors should match the number of vertices.
    pub(crate) fn render_shaded_polyline(
        &mut self,
        vertices: &[Vertex],
        colors: &[Color],
        semi_transparent: bool,
    ) {
        if vertices.len() < 2 || colors.len() < 2 {
            return;
        }

        log::trace!(
            "Rendering {}shaded polyline with {} vertices",
            if semi_transparent {
                "semi-transparent "
            } else {
                ""
            },
            vertices.len()
        );

        // For now, ignore semi_transparent (will be implemented in #36)
        let _ = semi_transparent;

        // Apply drawing offset to all vertices
        let points: Vec<(i16, i16)> = vertices
            .iter()
            .map(|v| {
                (
                    v.x.wrapping_add(self.draw_offset.0),
                    v.y.wrapping_add(self.draw_offset.1),
                )
            })
            .collect();

        // Convert colors to tuples
        let color_tuples: Vec<(u8, u8, u8)> = colors.iter().map(|c| (c.r, c.g, c.b)).collect();

        // Rasterize the shaded polyline
        self.rasterizer
            .draw_gradient_polyline(&mut self.vram, &points, &color_tuples);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_line_horizontal() {
        let mut gpu = GPU::new();

        // Horizontal line
        let v0 = Vertex { x: 100, y: 150 };
        let v1 = Vertex { x: 200, y: 150 };
        let color = Color {
            r: 255,
            g: 0,
            b: 0,
        };

        gpu.render_line(v0, v1, color, false);

        // Check midpoint
        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel & 0x001F, 0x001F); // Red
    }

    #[test]
    fn test_render_line_vertical() {
        let mut gpu = GPU::new();

        // Vertical line
        let v0 = Vertex { x: 150, y: 100 };
        let v1 = Vertex { x: 150, y: 200 };
        let color = Color {
            r: 0,
            g: 255,
            b: 0,
        };

        gpu.render_line(v0, v1, color, false);

        // Check midpoint
        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel & 0x03E0, 0x03E0); // Green
    }

    #[test]
    fn test_render_line_diagonal() {
        let mut gpu = GPU::new();

        // Diagonal line (45 degrees)
        let v0 = Vertex { x: 100, y: 100 };
        let v1 = Vertex { x: 200, y: 200 };
        let color = Color {
            r: 0,
            g: 0,
            b: 255,
        };

        gpu.render_line(v0, v1, color, false);

        // Check point along diagonal
        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel & 0x7C00, 0x7C00); // Blue
    }

    #[test]
    fn test_render_line_single_pixel() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Zero-length line should draw a single pixel
        let v0 = Vertex { x: 150, y: 150 };
        let v1 = Vertex { x: 150, y: 150 };
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        gpu.render_line(v0, v1, color, false);

        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x7FFF); // White
    }

    #[test]
    fn test_render_line_with_drawing_offset() {
        let mut gpu = GPU::new();

        // Set drawing offset
        gpu.draw_offset = (50, 30);

        let v0 = Vertex { x: 100, y: 100 };
        let v1 = Vertex { x: 200, y: 100 };
        let color = Color {
            r: 255,
            g: 128,
            b: 0,
        };

        gpu.render_line(v0, v1, color, false);

        // Line should be at (150+50, 100+30) = (200, 130)
        let pixel = gpu.read_vram(200, 130);
        assert_eq!(pixel & 0x001F, 31); // Red component
    }

    #[test]
    fn test_render_line_negative_coordinates() {
        let mut gpu = GPU::new();

        // Line with negative coordinates (will wrap)
        let v0 = Vertex { x: -50, y: -50 };
        let v1 = Vertex { x: 50, y: 50 };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        // Should handle wrapping gracefully
        gpu.render_line(v0, v1, color, false);
    }

    #[test]
    fn test_render_line_boundary_coordinates() {
        let mut gpu = GPU::new();

        // Line from corner to corner
        let v0 = Vertex { x: 0, y: 0 };
        let v1 = Vertex { x: 1023, y: 511 };
        let color = Color {
            r: 255,
            g: 0,
            b: 255,
        };

        gpu.render_line(v0, v1, color, false);

        // Check start and end points
        assert_ne!(gpu.read_vram(0, 0), 0x0000);
    }

    #[test]
    fn test_render_line_maximum_distance() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Maximum distance is 1023 horizontal, 511 vertical
        let v0 = Vertex { x: 0, y: 0 };
        let v1 = Vertex { x: 1023, y: 0 };
        let color = Color {
            r: 255,
            g: 255,
            b: 0,
        };

        // Should handle maximum distance
        gpu.render_line(v0, v1, color, false);
    }

    #[test]
    fn test_render_line_color_conversion() {
        let mut gpu = GPU::new();

        let v0 = Vertex { x: 100, y: 150 };
        let v1 = Vertex { x: 200, y: 150 };
        let color = Color {
            r: 200,
            g: 150,
            b: 100,
        };

        gpu.render_line(v0, v1, color, false);

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
    fn test_render_polyline_basic() {
        let mut gpu = GPU::new();

        // Triangle polyline
        let vertices = vec![
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
            Vertex { x: 100, y: 100 }, // Close the triangle
        ];
        let color = Color {
            r: 255,
            g: 0,
            b: 0,
        };

        gpu.render_polyline(&vertices, color, false);

        // Check a point along one of the edges
        assert_ne!(gpu.read_vram(150, 100), 0x0000);
    }

    #[test]
    fn test_render_polyline_single_vertex() {
        let mut gpu = GPU::new();

        // Per implementation: should not draw anything
        let vertices = vec![Vertex { x: 150, y: 150 }];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        // Should not crash
        gpu.render_polyline(&vertices, color, false);
    }

    #[test]
    fn test_render_polyline_empty() {
        let mut gpu = GPU::new();

        // Empty vertex list
        let vertices: Vec<Vertex> = vec![];
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        // Should not crash
        gpu.render_polyline(&vertices, color, false);
    }

    #[test]
    fn test_render_polyline_two_vertices() {
        let mut gpu = GPU::new();

        // Minimum valid polyline (single line segment)
        let vertices = vec![Vertex { x: 100, y: 100 }, Vertex { x: 200, y: 200 }];
        let color = Color {
            r: 0,
            g: 255,
            b: 255,
        };

        gpu.render_polyline(&vertices, color, false);

        // Check midpoint
        assert_ne!(gpu.read_vram(150, 150), 0x0000);
    }

    #[test]
    fn test_render_polyline_with_offset() {
        let mut gpu = GPU::new();

        gpu.draw_offset = (100, 50);

        let vertices = vec![
            Vertex { x: 50, y: 50 },
            Vertex { x: 100, y: 50 },
            Vertex { x: 100, y: 100 },
        ];
        let color = Color {
            r: 255,
            g: 255,
            b: 0,
        };

        gpu.render_polyline(&vertices, color, false);

        // Should be offset
        let pixel = gpu.read_vram(200, 100);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_render_shaded_line_basic() {
        let mut gpu = GPU::new();

        // Line from red to blue
        let v0 = Vertex { x: 100, y: 150 };
        let c0 = Color {
            r: 255,
            g: 0,
            b: 0,
        };
        let v1 = Vertex { x: 200, y: 150 };
        let c1 = Color {
            r: 0,
            g: 0,
            b: 255,
        };

        gpu.render_shaded_line(v0, c0, v1, c1, false);

        // Midpoint should have interpolated color
        let pixel = gpu.read_vram(150, 150);
        // Should have both red and blue components
        assert!((pixel & 0x001F) > 0); // Some red
        assert!((pixel >> 10) > 0); // Some blue
    }

    #[test]
    fn test_render_shaded_line_gradient() {
        let mut gpu = GPU::new();

        // Per PSX-SPX: Shaded lines use Gouraud interpolation
        let v0 = Vertex { x: 100, y: 100 };
        let c0 = Color {
            r: 0,
            g: 0,
            b: 0,
        }; // Black
        let v1 = Vertex { x: 200, y: 100 };
        let c1 = Color {
            r: 248,
            g: 248,
            b: 248,
        }; // White

        gpu.render_shaded_line(v0, c0, v1, c1, false);

        // Start should be darker, end should be lighter
        let start_pixel = gpu.read_vram(100, 100);
        let end_pixel = gpu.read_vram(200, 100);

        assert!(end_pixel > start_pixel); // Brightness increases
    }

    #[test]
    fn test_render_shaded_line_with_offset() {
        let mut gpu = GPU::new();

        gpu.draw_offset = (50, 30);

        let v0 = Vertex { x: 100, y: 100 };
        let c0 = Color {
            r: 255,
            g: 0,
            b: 0,
        };
        let v1 = Vertex { x: 200, y: 100 };
        let c1 = Color {
            r: 0,
            g: 255,
            b: 0,
        };

        gpu.render_shaded_line(v0, c0, v1, c1, false);

        // Should be offset to (150+50, 100+30) = (200, 130)
        let pixel = gpu.read_vram(200, 130);
        assert_ne!(pixel, 0x0000);
    }

    #[test]
    fn test_render_shaded_polyline_basic() {
        let mut gpu = GPU::new();

        // Rainbow polyline
        let vertices = vec![
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 200, y: 200 },
        ];
        let colors = vec![
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

        gpu.render_shaded_polyline(&vertices, &colors, false);

        // Check that something was drawn
        assert_ne!(gpu.read_vram(150, 100), 0x0000);
    }

    #[test]
    fn test_render_shaded_polyline_insufficient_vertices() {
        let mut gpu = GPU::new();

        // Only 1 vertex (should not draw)
        let vertices = vec![Vertex { x: 150, y: 150 }];
        let colors = vec![Color {
            r: 255,
            g: 255,
            b: 255,
        }];

        // Should not crash
        gpu.render_shaded_polyline(&vertices, &colors, false);
    }

    #[test]
    fn test_render_shaded_polyline_insufficient_colors() {
        let mut gpu = GPU::new();

        // 3 vertices but only 1 color (edge case)
        let vertices = vec![
            Vertex { x: 100, y: 100 },
            Vertex { x: 200, y: 100 },
            Vertex { x: 150, y: 200 },
        ];
        let colors = vec![Color {
            r: 255,
            g: 0,
            b: 0,
        }];

        // Should not crash (implementation checks colors.len() < 2)
        gpu.render_shaded_polyline(&vertices, &colors, false);
    }

    #[test]
    fn test_render_line_all_angles() {
        let mut gpu = GPU::new();

        let center = Vertex { x: 512, y: 256 };

        // Test lines at various angles
        for angle in [0, 45, 90, 135, 180, 225, 270, 315] {
            let rad = (angle as f32).to_radians();
            let dx = (100.0 * rad.cos()) as i16;
            let dy = (100.0 * rad.sin()) as i16;

            let endpoint = Vertex {
                x: center.x + dx,
                y: center.y + dy,
            };

            let color = Color {
                r: 255,
                g: 255,
                b: 255,
            };

            gpu.render_line(center, endpoint, color, false);
        }

        // Center should definitely have lines through it
        assert_ne!(gpu.read_vram(512, 256), 0x0000);
    }

    #[test]
    fn test_render_line_steep_vs_shallow() {
        let mut gpu = GPU::new();

        // Shallow line (more horizontal)
        let v0 = Vertex { x: 100, y: 150 };
        let v1 = Vertex { x: 200, y: 160 };
        let color = Color {
            r: 255,
            g: 0,
            b: 0,
        };

        gpu.render_line(v0, v1, color, false);

        // Steep line (more vertical)
        let v2 = Vertex { x: 300, y: 100 };
        let v3 = Vertex { x: 310, y: 200 };

        gpu.render_line(v2, v3, color, false);

        // Both should render
        assert_ne!(gpu.read_vram(150, 155), 0x0000);
        assert_ne!(gpu.read_vram(305, 150), 0x0000);
    }

    #[test]
    fn test_render_line_coordinate_wrapping() {
        let mut gpu = GPU::new();

        // Large coordinates that will wrap
        let v0 = Vertex { x: 30000, y: 20000 };
        let v1 = Vertex { x: 31000, y: 21000 };
        let color = Color {
            r: 128,
            g: 128,
            b: 128,
        };

        // Should wrap around and not crash
        gpu.render_line(v0, v1, color, false);
    }

    #[test]
    fn test_render_polyline_many_segments() {
        let mut gpu = GPU::new();

        // Polyline with many vertices (zigzag pattern)
        let mut vertices = vec![];
        for i in 0..10 {
            vertices.push(Vertex {
                x: 100 + i * 20,
                y: 100 + (i % 2) * 50,
            });
        }

        let color = Color {
            r: 255,
            g: 128,
            b: 64,
        };

        gpu.render_polyline(&vertices, color, false);

        // Check that some segments were drawn
        assert_ne!(gpu.read_vram(110, 125), 0x0000);
    }

    #[test]
    fn test_render_line_black() {
        let mut gpu = GPU::new();

        let v0 = Vertex { x: 100, y: 150 };
        let v1 = Vertex { x: 200, y: 150 };
        let color = Color { r: 0, g: 0, b: 0 };

        gpu.render_line(v0, v1, color, false);

        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x0000);
    }

    #[test]
    fn test_render_line_white() {
        let mut gpu = GPU::new();

        let v0 = Vertex { x: 100, y: 150 };
        let v1 = Vertex { x: 200, y: 150 };
        let color = Color {
            r: 255,
            g: 255,
            b: 255,
        };

        gpu.render_line(v0, v1, color, false);

        let pixel = gpu.read_vram(150, 150);
        assert_eq!(pixel, 0x7FFF);
    }
}
