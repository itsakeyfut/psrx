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

//! Display renderer for PlayStation VRAM
//!
//! This module provides the display rendering pipeline for the PlayStation GPU's VRAM.
//! It handles texture management, shader setup, and rendering of the display area
//! to the output surface.

use super::vram_texture::VramTexture;
use crate::core::gpu::DisplayArea;

/// Display area uniform buffer
///
/// GPU uniform buffer containing display area parameters.
/// This is uploaded to the GPU and used by the display shader
/// to crop and map the VRAM texture to the output.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DisplayAreaUniform {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl From<&DisplayArea> for DisplayAreaUniform {
    fn from(area: &DisplayArea) -> Self {
        Self {
            x: area.x as f32,
            y: area.y as f32,
            width: area.width as f32,
            height: area.height as f32,
        }
    }
}

/// Display renderer
///
/// Manages the rendering pipeline for displaying PlayStation VRAM contents.
/// Uses wgpu shaders to render the VRAM texture with display area cropping,
/// aspect ratio preservation, and filtering options.
///
/// # Examples
///
/// ```no_run
/// use psrx::frontend::renderer::DisplayRenderer;
/// use psrx::core::gpu::DisplayArea;
///
/// # async fn example(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) {
/// let mut renderer = DisplayRenderer::new(device, surface_format);
///
/// // Render VRAM to output texture
/// let vram: Vec<u16> = vec![0; 1024 * 512];
/// let display_area = DisplayArea::default();
///
/// # let output_view = todo!();
/// # let mut encoder = todo!();
/// renderer.render(&mut encoder, &output_view, &vram, &display_area, device, queue);
/// # }
/// ```
pub struct DisplayRenderer {
    /// Render pipeline for display shader
    pipeline: wgpu::RenderPipeline,
    /// Bind group layout for VRAM texture, sampler, and uniform
    bind_group_layout: wgpu::BindGroupLayout,
    /// VRAM texture wrapper
    vram_texture: VramTexture,
    /// Texture sampler (point or linear filtering)
    sampler: wgpu::Sampler,
    /// Display area uniform buffer
    uniform_buffer: wgpu::Buffer,
}

impl DisplayRenderer {
    /// Create a new display renderer
    ///
    /// # Arguments
    ///
    /// * `device` - wgpu device for creating GPU resources
    /// * `surface_format` - Output surface texture format
    ///
    /// # Returns
    ///
    /// A new `DisplayRenderer` ready to render VRAM to the output
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use psrx::frontend::renderer::DisplayRenderer;
    ///
    /// # async fn example(device: &wgpu::Device) {
    /// let renderer = DisplayRenderer::new(device, wgpu::TextureFormat::Bgra8UnormSrgb);
    /// # }
    /// ```
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create VRAM texture
        let vram_texture = VramTexture::new(device);

        // Create sampler (point filtering for pixel-perfect display)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("VRAM Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Point filtering (pixel-perfect)
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Create uniform buffer for display area
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Display Area Uniform Buffer"),
            size: std::mem::size_of::<DisplayAreaUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Display Bind Group Layout"),
            entries: &[
                // VRAM texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Display area uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Load shader
        let shader_source = include_str!("shaders/display.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Display Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Display Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Display Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // No vertex buffer (fullscreen triangle)
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        log::info!("Display renderer initialized");

        Self {
            pipeline,
            bind_group_layout,
            vram_texture,
            sampler,
            uniform_buffer,
        }
    }

    /// Render VRAM to output texture
    ///
    /// Updates the VRAM texture and renders it to the output view with
    /// display area cropping applied.
    ///
    /// # Arguments
    ///
    /// * `encoder` - Command encoder for recording GPU commands
    /// * `output_view` - Output texture view to render to
    /// * `vram` - PSX VRAM data (1024×512 pixels, 16-bit RGB15)
    /// * `display_area` - Display area configuration from GPU
    /// * `device` - wgpu device (unused, for future expansion)
    /// * `queue` - wgpu queue for uploading data
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use psrx::frontend::renderer::DisplayRenderer;
    /// use psrx::core::gpu::DisplayArea;
    ///
    /// # async fn example(
    /// #     device: &wgpu::Device,
    /// #     queue: &wgpu::Queue,
    /// #     surface_format: wgpu::TextureFormat,
    /// #     output_view: &wgpu::TextureView,
    /// # ) {
    /// let mut renderer = DisplayRenderer::new(device, surface_format);
    ///
    /// let vram: Vec<u16> = vec![0; 1024 * 512];
    /// let display_area = DisplayArea::default();
    ///
    /// let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    ///     label: Some("Render Encoder"),
    /// });
    ///
    /// renderer.render(&mut encoder, output_view, &vram, &display_area, device, queue);
    ///
    /// queue.submit(std::iter::once(encoder.finish()));
    /// # }
    /// ```
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        vram: &[u16],
        display_area: &DisplayArea,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // Update VRAM texture
        self.vram_texture.update(queue, vram);

        // Update display area uniform
        let uniform_data = DisplayAreaUniform::from(display_area);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform_data]),
        );

        // Create bind group
        let bind_group = _device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Display Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.vram_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Display Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set pipeline and bind group
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            // Draw fullscreen triangle (3 vertices, no vertex buffer)
            render_pass.draw(0..3, 0..1);
        }
    }
}
