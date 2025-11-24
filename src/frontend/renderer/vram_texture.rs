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

//! VRAM texture management
//!
//! This module provides texture management for the PlayStation GPU's VRAM.
//! It handles conversion from the PSX 15-bit RGB format to the 32-bit RGBA8
//! format used by wgpu for rendering.

/// VRAM texture wrapper
///
/// Manages a wgpu texture that holds the PlayStation GPU's VRAM contents.
/// The texture is 1024×512 pixels in RGBA8 format, converted from the
/// original 15-bit RGB (5-5-5) format used by the PSX hardware.
///
/// # Examples
///
/// ```no_run
/// use psrx::frontend::renderer::VramTexture;
///
/// # async fn example(device: &wgpu::Device, queue: &wgpu::Queue) {
/// let mut vram_texture = VramTexture::new(device);
///
/// // Update with PSX VRAM data (16-bit RGB15)
/// let vram: Vec<u16> = vec![0; 1024 * 512];
/// vram_texture.update(queue, &vram);
/// # }
/// ```
pub struct VramTexture {
    /// wgpu texture handle
    texture: wgpu::Texture,
    /// Texture view for binding to shaders
    pub view: wgpu::TextureView,
    /// Dirty flag indicating if texture needs update
    dirty: bool,
}

impl VramTexture {
    /// VRAM width in pixels
    pub const WIDTH: u32 = 1024;
    /// VRAM height in pixels
    pub const HEIGHT: u32 = 512;

    /// Create a new VRAM texture
    ///
    /// # Arguments
    ///
    /// * `device` - wgpu device for creating GPU resources
    ///
    /// # Returns
    ///
    /// A new `VramTexture` initialized with black pixels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use psrx::frontend::renderer::VramTexture;
    ///
    /// # async fn example(device: &wgpu::Device) {
    /// let vram_texture = VramTexture::new(device);
    /// # }
    /// ```
    pub fn new(device: &wgpu::Device) -> Self {
        let size = wgpu::Extent3d {
            width: Self::WIDTH,
            height: Self::HEIGHT,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VRAM Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            dirty: false,
        }
    }

    /// Update VRAM texture with new data
    ///
    /// Converts the PSX 15-bit RGB format to RGBA8 and uploads to the GPU.
    ///
    /// # Arguments
    ///
    /// * `queue` - wgpu queue for submitting upload commands
    /// * `vram` - PSX VRAM data in 16-bit RGB15 format (1024×512 pixels)
    ///
    /// # Panics
    ///
    /// Panics if the VRAM buffer size is not exactly 1024×512 pixels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use psrx::frontend::renderer::VramTexture;
    ///
    /// # async fn example(device: &wgpu::Device, queue: &wgpu::Queue) {
    /// let mut vram_texture = VramTexture::new(device);
    /// let vram: Vec<u16> = vec![0; 1024 * 512];
    /// vram_texture.update(queue, &vram);
    /// # }
    /// ```
    pub fn update(&mut self, queue: &wgpu::Queue, vram: &[u16]) {
        assert_eq!(
            vram.len(),
            (Self::WIDTH * Self::HEIGHT) as usize,
            "VRAM buffer size must be 1024×512 pixels"
        );

        // Convert RGB15 to RGBA8
        let rgba_data = convert_rgb15_to_rgba8(vram);

        // Upload to GPU
        // wgpu 27 API: write_texture(ImageCopyTexture, &[u8], TexelCopyBufferLayout, Extent3d)
        queue.write_texture(
            self.texture.as_image_copy(),
            &rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(Self::WIDTH * 4), // 4 bytes per RGBA pixel
                rows_per_image: Some(Self::HEIGHT),
            },
            wgpu::Extent3d {
                width: Self::WIDTH,
                height: Self::HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        self.dirty = false;
    }

    /// Mark texture as dirty (needs update)
    ///
    /// This can be used to track when VRAM has been modified and needs
    /// to be re-uploaded to the GPU.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use psrx::frontend::renderer::VramTexture;
    ///
    /// # async fn example(device: &wgpu::Device) {
    /// let mut vram_texture = VramTexture::new(device);
    /// vram_texture.mark_dirty();
    /// # }
    /// ```
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if texture is dirty (needs update)
    ///
    /// # Returns
    ///
    /// `true` if the texture has been marked dirty and needs update
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Convert PSX RGB15 format to RGBA8
///
/// Converts PlayStation VRAM pixel data from 15-bit RGB (5-5-5) format
/// to 32-bit RGBA8 format. Each 5-bit color channel is expanded to 8 bits
/// by left-shifting by 3 bits.
///
/// # Arguments
///
/// * `vram` - VRAM data in 16-bit RGB15 format
///
/// # Returns
///
/// RGBA8 pixel data (4 bytes per pixel)
///
/// # Format Details
///
/// PSX RGB15 format (16-bit):
/// - Bits 0-4: Red (5 bits)
/// - Bits 5-9: Green (5 bits)
/// - Bits 10-14: Blue (5 bits)
/// - Bit 15: Mask bit (ignored)
///
/// RGBA8 format (32-bit):
/// - Byte 0: Red (8 bits)
/// - Byte 1: Green (8 bits)
/// - Byte 2: Blue (8 bits)
/// - Byte 3: Alpha (always 255)
///
/// # Examples
///
/// ```
/// use psrx::frontend::renderer::convert_rgb15_to_rgba8;
///
/// let vram = vec![0x7FFF, 0x0000, 0x001F]; // White, Black, Red
/// let rgba = convert_rgb15_to_rgba8(&vram);
/// assert_eq!(rgba[0..4], [248, 248, 248, 255]); // White
/// assert_eq!(rgba[4..8], [0, 0, 0, 255]);       // Black
/// assert_eq!(rgba[8..12], [248, 0, 0, 255]);    // Red
/// ```
pub fn convert_rgb15_to_rgba8(vram: &[u16]) -> Vec<u8> {
    vram.iter()
        .flat_map(|&color| {
            // Extract 5-bit color channels
            let r = ((color & 0x1F) << 3) as u8;
            let g = (((color >> 5) & 0x1F) << 3) as u8;
            let b = (((color >> 10) & 0x1F) << 3) as u8;

            // Return RGBA8 pixel (alpha always 255)
            [r, g, b, 255u8]
        })
        .collect()
}
