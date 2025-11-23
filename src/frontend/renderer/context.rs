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

//! wgpu rendering context
//!
//! This module provides the wgpu rendering context for the PSRX frontend.
//! It manages the GPU device, queue, surface, and surface configuration.

use std::sync::Arc;
use winit::window::Window;

/// wgpu rendering context
///
/// Manages the GPU device, queue, surface, and surface configuration.
/// This is initialized asynchronously and provides the foundation for
/// all rendering operations in the PSRX frontend.
pub struct RenderContext {
    /// wgpu device for creating GPU resources
    pub device: wgpu::Device,
    /// Command queue for submitting GPU commands
    pub queue: wgpu::Queue,
    /// Surface for rendering to the window
    pub surface: wgpu::Surface<'static>,
    /// Surface configuration (format, size, present mode, etc.)
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl RenderContext {
    /// Create a new rendering context
    ///
    /// # Arguments
    ///
    /// * `window` - The window to render to
    ///
    /// # Returns
    ///
    /// A new `RenderContext` initialized with the given window
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No suitable GPU adapter is found
    /// - Device creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use winit::window::Window;
    /// use psrx::frontend::renderer::RenderContext;
    ///
    /// async fn create_context(window: Arc<Window>) {
    ///     let context = RenderContext::new(&window).await.unwrap();
    /// }
    /// ```
    pub async fn new(window: &Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("Failed to create surface: {}", e))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| format!("Failed to find suitable GPU adapter: {}", e))?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("PSRX Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;

        // Get surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);

        // Use Bgra8UnormSrgb if available, otherwise use the first available format
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Configure surface
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // V-sync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        log::info!(
            "Initialized wgpu context: {}x{}, format: {:?}",
            size.width,
            size.height,
            surface_format
        );

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
        })
    }

    /// Resize the surface
    ///
    /// Called when the window is resized to update the surface configuration.
    ///
    /// # Arguments
    ///
    /// * `new_width` - New width in pixels
    /// * `new_height` - New height in pixels
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);
            log::debug!("Resized surface to {}x{}", new_width, new_height);
        }
    }
}
