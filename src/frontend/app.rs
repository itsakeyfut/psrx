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

//! PSRX application
//!
//! This module provides the main application struct that manages the window,
//! event loop, rendering context, and UI.

use crate::frontend::renderer::RenderContext;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

/// PSRX application
///
/// Manages the window, rendering context, and UI for the PSRX emulator.
/// This struct orchestrates the event loop and handles user input.
pub struct Application {
    /// The application window
    window: Option<Arc<Window>>,
    /// wgpu rendering context
    render_context: Option<RenderContext>,
    /// egui context for UI
    egui_ctx: egui::Context,
    /// egui-winit state for event handling
    egui_state: Option<egui_winit::State>,
    /// egui-wgpu renderer
    egui_renderer: Option<egui_wgpu::Renderer>,
}

impl Application {
    /// Create a new PSRX application
    ///
    /// # Returns
    ///
    /// A new `Application` instance ready to be run with an event loop
    ///
    /// # Example
    ///
    /// ```no_run
    /// use winit::event_loop::EventLoop;
    /// use psrx::frontend::Application;
    ///
    /// let event_loop = EventLoop::new().unwrap();
    /// let mut app = Application::new();
    /// event_loop.run_app(&mut app).unwrap();
    /// ```
    pub fn new() -> Self {
        let egui_ctx = egui::Context::default();

        Self {
            window: None,
            render_context: None,
            egui_ctx,
            egui_state: None,
            egui_renderer: None,
        }
    }

    /// Render a frame
    ///
    /// This method handles:
    /// 1. Getting the next surface texture
    /// 2. Running the egui UI code
    /// 3. Rendering egui to the surface
    /// 4. Presenting the frame
    fn render(&mut self) -> Result<(), String> {
        let window = self.window.as_ref().ok_or("Window not initialized")?;
        let render_context = self
            .render_context
            .as_mut()
            .ok_or("Render context not initialized")?;
        let egui_state = self
            .egui_state
            .as_mut()
            .ok_or("egui state not initialized")?;
        let egui_renderer = self
            .egui_renderer
            .as_mut()
            .ok_or("egui renderer not initialized")?;

        // Get the next frame, handling common surface errors gracefully
        let output = match render_context.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                // Reconfigure the surface to the current size and skip this frame
                render_context.resize(
                    render_context.surface_config.width,
                    render_context.surface_config.height,
                );
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => {
                // Non-fatal; skip this frame
                log::warn!("Surface timeout while acquiring frame");
                return Ok(());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // Treat as fatal and propagate up
                return Err("Surface out of memory while acquiring frame".to_string());
            }
            Err(e) => {
                // Catch-all for any other surface error (e.g. `Other`)
                log::error!("Unexpected surface error: {:?}", e);
                return Err(format!("Failed to get surface texture: {:?}", e));
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin egui frame
        let raw_input = egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Create a simple UI panel
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("PSRX - PlayStation Emulator");
                ui.separator();
                ui.label("Frontend foundation initialized successfully!");

                ui.separator();
                ui.label("Status: ✓ Complete");

                // Add some debug info
                ui.separator();
                ui.collapsing("Debug Info", |ui| {
                    ui.label(format!(
                        "Surface: {}x{}",
                        render_context.surface_config.width, render_context.surface_config.height
                    ));
                    ui.label(format!(
                        "Format: {:?}",
                        render_context.surface_config.format
                    ));
                    ui.label(format!(
                        "Present Mode: {:?}",
                        render_context.surface_config.present_mode
                    ));
                });
            });
        });

        // Handle platform output (e.g., cursor changes)
        egui_state.handle_platform_output(window, full_output.platform_output);

        // Prepare egui rendering
        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        // Upload egui textures
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(
                &render_context.device,
                &render_context.queue,
                *id,
                image_delta,
            );
        }

        // Update egui vertex/index buffers
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                render_context.surface_config.width,
                render_context.surface_config.height,
            ],
            pixels_per_point: window.scale_factor() as f32,
        };

        // Record rendering commands
        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        egui_renderer.update_buffers(
            &render_context.device,
            &render_context.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            // Forget lifetime to make render pass 'static as required by egui-wgpu 0.33
            let mut render_pass = render_pass.forget_lifetime();

            // Render egui
            egui_renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        // Submit commands
        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // Free egui textures
        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Present frame
        output.present();

        Ok(())
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window if it doesn't exist
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("PSRX - PlayStation Emulator")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
                .with_resizable(true);

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            // Initialize rendering context
            let render_context =
                pollster::block_on(RenderContext::new(&window)).expect("Failed to create renderer");

            // Initialize egui-winit state
            let egui_state = egui_winit::State::new(
                self.egui_ctx.clone(),
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                None,
            );

            // Initialize egui-wgpu renderer
            let egui_renderer = egui_wgpu::Renderer::new(
                &render_context.device,
                render_context.surface_config.format,
                egui_wgpu::RendererOptions::default(),
            );

            self.window = Some(window);
            self.render_context = Some(render_context);
            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);

            log::info!("Application initialized successfully");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle the event first
        if let Some(egui_state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let response = egui_state.on_window_event(window, &event);
                if response.consumed {
                    return; // egui consumed the event
                }
            }
        }

        // Handle window events
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    log::error!("Render error: {}", e);
                    event_loop.exit();
                }

                // Request another frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw on each event loop iteration
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
