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

use crate::core::system::System;
use crate::frontend::frame_timer::FrameTimer;
use crate::frontend::input::InputHandler;
use crate::frontend::renderer::{DisplayRenderer, RenderContext};
use crate::frontend::ui::{UiAction, UiState};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
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
    /// Display renderer for VRAM
    display_renderer: Option<DisplayRenderer>,
    /// PlayStation system (CPU, GPU, Memory, etc.)
    system: Option<System>,
    /// Frame timer for 60 FPS timing
    frame_timer: FrameTimer,
    /// Emulation paused state
    paused: bool,
    /// BIOS file path
    bios_path: String,
    /// Input handler for keyboard/gamepad
    input_handler: InputHandler,
    /// Show input configuration UI
    show_input_config: bool,
    /// UI state manager
    ui_state: UiState,
    /// Exit requested flag
    exit_requested: bool,
}

impl Application {
    /// Create a new PSRX application
    ///
    /// # Arguments
    ///
    /// * `bios_path` - Path to the BIOS file (e.g., "SCPH1001.BIN")
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
    /// let mut app = Application::new("SCPH1001.BIN");
    /// event_loop.run_app(&mut app).unwrap();
    /// ```
    pub fn new(bios_path: &str) -> Self {
        let egui_ctx = egui::Context::default();
        let input_handler = InputHandler::new();

        Self {
            window: None,
            render_context: None,
            egui_ctx,
            egui_state: None,
            egui_renderer: None,
            display_renderer: None,
            system: None,
            frame_timer: FrameTimer::new(60),
            paused: false,
            bios_path: bios_path.to_string(),
            input_handler,
            show_input_config: false,
            ui_state: UiState::new(),
            exit_requested: false,
        }
    }

    /// Toggle pause/resume emulation
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::frontend::Application;
    ///
    /// let mut app = Application::new("SCPH1001.BIN");
    /// app.toggle_pause();
    /// ```
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        log::info!(
            "Emulation {}",
            if self.paused { "paused" } else { "resumed" }
        );
    }

    /// Step one frame (when paused)
    ///
    /// Executes exactly one frame of emulation when paused.
    /// Does nothing if emulation is not paused.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::frontend::Application;
    ///
    /// let mut app = Application::new("SCPH1001.BIN");
    /// app.toggle_pause(); // Pause first
    /// app.step_frame();   // Step one frame
    /// ```
    pub fn step_frame(&mut self) {
        if self.paused {
            if let Some(ref mut system) = self.system {
                if let Err(e) = system.run_frame() {
                    log::error!("Failed to step frame: {}", e);
                }
                // Request redraw
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    /// Reset the emulation
    ///
    /// Resets the PlayStation system to its initial state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::frontend::Application;
    ///
    /// let mut app = Application::new("SCPH1001.BIN");
    /// app.reset();
    /// ```
    pub fn reset(&mut self) {
        if let Some(ref mut system) = self.system {
            system.reset();
            log::info!("System reset");
        }
    }

    /// Toggle fullscreen mode
    ///
    /// Switches between windowed and fullscreen mode.
    pub fn toggle_fullscreen(&mut self) {
        if let Some(window) = &self.window {
            let is_fullscreen = window.fullscreen().is_some();
            if is_fullscreen {
                window.set_fullscreen(None);
                log::info!("Switched to windowed mode");
            } else {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                log::info!("Switched to fullscreen mode");
            }
        }
    }

    /// Toggle input configuration UI
    pub fn toggle_input_config(&mut self) {
        self.show_input_config = !self.show_input_config;
    }

    /// Handle UI action
    ///
    /// Processes actions triggered from the UI (menu bar, buttons, etc.)
    fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::None => {}
            UiAction::TogglePause => {
                self.toggle_pause();
            }
            UiAction::StepFrame => {
                self.step_frame();
            }
            UiAction::Reset => {
                self.reset();
            }
            UiAction::ToggleFullscreen => {
                self.toggle_fullscreen();
            }
            UiAction::LoadBios => {
                self.open_bios_dialog();
            }
            UiAction::LoadDisc => {
                self.open_disc_dialog();
            }
            UiAction::Exit => {
                // Set exit flag - will be handled in the event loop
                self.exit_requested = true;
                log::info!("Exit requested from UI");
            }
            UiAction::EnableCpuTracing => {
                log::info!("CPU tracing requested (not yet implemented)");
                // TODO: Implement CPU tracing enable
            }
            UiAction::DumpVram => {
                self.dump_vram_to_file();
            }
            UiAction::ToggleInputConfig => {
                self.toggle_input_config();
            }
        }
    }

    /// Open BIOS file dialog
    fn open_bios_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("BIOS", &["bin", "BIN"])
            .set_title("Select BIOS file")
            .pick_file();

        if let Some(path) = path {
            if let Some(path_str) = path.to_str() {
                if let Some(ref mut system) = self.system {
                    match system.load_bios(path_str) {
                        Ok(_) => {
                            system.reset();
                            self.bios_path = path_str.to_string();
                            log::info!("Loaded BIOS: {}", path_str);
                        }
                        Err(e) => {
                            log::error!("Failed to load BIOS: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Open disc image file dialog
    fn open_disc_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Disc Image", &["cue", "CUE", "bin", "BIN", "iso", "ISO"])
            .set_title("Select disc image")
            .pick_file();

        if let Some(path) = path {
            if let Some(path_str) = path.to_str() {
                log::info!("Disc loading requested: {} (not yet implemented)", path_str);
                // TODO: Implement disc loading in future phase
            }
        }
    }

    /// Dump VRAM to file
    fn dump_vram_to_file(&mut self) {
        if let Some(ref _system) = self.system {
            let path = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .set_title("Save VRAM dump")
                .set_file_name("vram_dump.png")
                .save_file();

            if let Some(_path) = path {
                log::info!("VRAM dump requested (not yet implemented)");
                // TODO: Implement VRAM dumping to PNG
                // This would require image encoding library like `image` or `png`
            }
        }
    }

    /// Render a frame
    ///
    /// This method handles:
    /// 1. Getting the next surface texture
    /// 2. Rendering VRAM display from the GPU
    /// 3. Running the egui UI code with performance metrics
    /// 4. Rendering egui to the surface
    /// 5. Presenting the frame
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
        let display_renderer = self
            .display_renderer
            .as_mut()
            .ok_or("Display renderer not initialized")?;
        let system = self.system.as_ref().ok_or("System not initialized")?;

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
        let paused = self.paused;
        let show_input_config = self.show_input_config;
        let input_handler = &self.input_handler;

        // Track UI actions and state changes
        let mut ui_action = UiAction::None;
        let mut should_toggle_input_config = false;

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Render the debug UI (menu bar, status bar, debug panels)
            ui_action = self.ui_state.render(ctx, system, &self.frame_timer, paused);

            // Input configuration window
            if show_input_config {
                egui::Window::new("Input Configuration")
                    .default_width(400.0)
                    .show(ctx, |ui| {
                        ui.heading("Keyboard Mapping");
                        ui.label("PSX controller button mappings:");
                        ui.separator();

                        // Show all button mappings
                        let mappings = input_handler.get_button_mappings();
                        for (button_name, keys) in mappings {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:12}", button_name));
                                ui.label("→");
                                let keys_str = keys
                                    .iter()
                                    .map(|k| format!("{:?}", k))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                ui.label(keys_str);
                            });
                        }

                        ui.separator();

                        // Show conflicts if any
                        let conflicts = input_handler.detect_conflicts();
                        if !conflicts.is_empty() {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!(
                                    "Note: {} buttons have multiple keys mapped",
                                    conflicts.len()
                                ),
                            );
                            ui.label("This is normal and allows flexibility in controls.");
                        }

                        ui.separator();

                        ui.label("Configuration file:");
                        ui.label(input_handler.config_path());

                        ui.separator();

                        if ui.button("Close").clicked() {
                            should_toggle_input_config = true;
                        }

                        ui.separator();
                        ui.label("To customize key bindings, edit the input.toml file.");
                    });
            }
        });

        // Handle deferred actions
        if should_toggle_input_config {
            self.show_input_config = !self.show_input_config;
        }

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

        // Render VRAM display first (clears to black and draws VRAM)
        // Get GPU data from the System
        let gpu = system.gpu();
        let mut gpu_borrow = gpu.borrow_mut();

        // Only update VRAM texture if GPU modified it (dirty flag optimization)
        let vram_dirty = gpu_borrow.is_vram_dirty();
        if vram_dirty {
            let vram = &gpu_borrow.vram;
            display_renderer.update_vram(&render_context.queue, vram);
            gpu_borrow.clear_vram_dirty_flag();
        }

        let display_area = gpu_borrow.display_area();
        drop(gpu_borrow); // Drop borrow before render

        display_renderer.render_display(
            &mut encoder,
            &view,
            &display_area,
            &render_context.device,
            &render_context.queue,
        );

        // Update egui buffers
        egui_renderer.update_buffers(
            &render_context.device,
            &render_context.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Render egui on top of VRAM display
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load existing VRAM display
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

        // Handle UI actions after all borrows are released
        self.handle_ui_action(ui_action);

        Ok(())
    }
}

impl Default for Application {
    fn default() -> Self {
        // Default BIOS path (users should provide this via command line)
        Self::new("SCPH1001.BIN")
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

            // Initialize display renderer
            let display_renderer =
                DisplayRenderer::new(&render_context.device, render_context.surface_config.format);

            // Initialize PlayStation System
            let mut system = System::new();
            if let Err(e) = system.load_bios(&self.bios_path) {
                log::error!("Failed to load BIOS from '{}': {}", self.bios_path, e);
                panic!("Cannot start emulator without valid BIOS");
            }
            system.reset();
            log::info!("System initialized with BIOS: {}", self.bios_path);

            self.window = Some(window);
            self.render_context = Some(render_context);
            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);
            self.display_renderer = Some(display_renderer);
            self.system = Some(system);

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
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let pressed = event.state.is_pressed();

                    // Handle hotkeys first (only on press)
                    if pressed {
                        match key_code {
                            KeyCode::Space => {
                                self.toggle_pause();
                                return; // Don't pass to controller
                            }
                            KeyCode::F10 => {
                                self.step_frame();
                                return;
                            }
                            KeyCode::F5 => {
                                self.reset();
                                return;
                            }
                            KeyCode::F11 => {
                                self.toggle_fullscreen();
                                return;
                            }
                            KeyCode::F12 => {
                                // TODO: Screenshot functionality (Phase 6)
                                log::info!("Screenshot hotkey pressed (not yet implemented)");
                                return;
                            }
                            _ => {}
                        }
                    }

                    // Handle controller input
                    if let Some((button, pressed)) =
                        self.input_handler.handle_keyboard(key_code, pressed)
                    {
                        if let Some(ref system) = self.system {
                            let controller_ports = system.controller_ports();
                            let mut ports = controller_ports.borrow_mut();
                            if let Some(controller) = ports.get_controller_mut(0) {
                                controller.set_button_state(button, pressed);
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    log::error!("Render error: {}", e);
                    event_loop.exit();
                }
            }
            _ => {}
        }

        // Check if exit was requested from UI
        if self.exit_requested {
            log::info!("Exiting application");
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Main emulation loop - runs at 60 FPS
        if !self.paused && self.frame_timer.should_run_frame() {
            // Run emulation frame
            if let Some(ref mut system) = self.system {
                if let Err(e) = system.run_frame() {
                    log::error!("Emulation error: {}", e);
                    self.paused = true;
                }
            }

            // Update frame timer
            self.frame_timer.tick();

            // Request redraw
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }

        // Set control flow based on emulation state to avoid busy-waiting
        if self.paused {
            // When paused, wait for events (keyboard input, etc.)
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        } else {
            // When running, wake up at the next frame time for 60 FPS pacing
            let next_frame = self.frame_timer.next_frame_instant();
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(next_frame));
        }
    }
}
