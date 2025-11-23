// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 itsakeyfut
//
// Display shader for PlayStation VRAM rendering
//
// This shader renders the PlayStation GPU's VRAM to the display window,
// handling display area cropping and aspect ratio preservation.

// Display area uniform
//
// Defines the region of VRAM (in pixels) that should be displayed.
// This corresponds to the GPU's display area settings.
struct DisplayArea {
    x: f32,      // X coordinate in VRAM (0-1023)
    y: f32,      // Y coordinate in VRAM (0-511)
    width: f32,  // Width in pixels
    height: f32, // Height in pixels
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Bind group 0: VRAM texture and display area
@group(0) @binding(0) var vram_texture: texture_2d<f32>;
@group(0) @binding(1) var vram_sampler: sampler;
@group(0) @binding(2) var<uniform> display_area: DisplayArea;

// Vertex shader
//
// Generates a fullscreen triangle using only the vertex index.
// This technique avoids the need for a vertex buffer.
//
// The triangle covers the entire clip space from (-1, -1) to (1, 1):
//   v0 = (-1, -1) - bottom-left
//   v1 = ( 3, -1) - bottom-right (off-screen)
//   v2 = (-1,  3) - top-left (off-screen)
//
// Texture coordinates range from (0, 0) to (1, 1) over the visible area.
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle positions (clip space)
    let pos = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),  // Bottom-left
        vec2( 3.0, -1.0),  // Bottom-right (off-screen)
        vec2(-1.0,  3.0),  // Top-left (off-screen)
    );

    // Texture coordinates (0 to 1)
    let tex = array<vec2<f32>, 3>(
        vec2(0.0, 0.0),
        vec2(2.0, 0.0),
        vec2(0.0, 2.0),
    );

    out.position = vec4<f32>(pos[vi], 0.0, 1.0);
    out.tex_coords = tex[vi];

    return out;
}

// Fragment shader
//
// Samples the VRAM texture within the display area and outputs the color.
// Handles display area cropping by mapping texture coordinates to the
// specified VRAM region.
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Map texture coordinates (0-1) to VRAM coordinates (pixels)
    let vram_x = display_area.x + input.tex_coords.x * display_area.width;
    let vram_y = display_area.y + input.tex_coords.y * display_area.height;

    // Convert to normalized texture coordinates (0-1)
    let uv = vec2<f32>(vram_x / 1024.0, vram_y / 512.0);

    // Sample VRAM texture
    return textureSample(vram_texture, vram_sampler, uv);
}
