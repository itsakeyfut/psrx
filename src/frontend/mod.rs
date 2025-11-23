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

//! Frontend module
//!
//! This module provides the user interface and window management for the PSRX emulator.
//! It uses winit for window management, wgpu for GPU acceleration, and egui for the UI.
//!
//! # Architecture
//!
//! - [`Application`]: Main application struct that handles the event loop and window
//! - [`renderer`]: Rendering subsystem (wgpu context, etc.)
//! - [`frame_timer`]: Frame timing utilities for 60 FPS emulation
//!
//! # Example
//!
//! ```no_run
//! use winit::event_loop::EventLoop;
//! use psrx::frontend::Application;
//!
//! let event_loop = EventLoop::new().unwrap();
//! let mut app = Application::new("SCPH1001.BIN");
//! event_loop.run_app(&mut app).unwrap();
//! ```

pub mod app;
pub mod frame_timer;
pub mod renderer;

pub use app::Application;
pub use frame_timer::FrameTimer;
pub use renderer::RenderContext;
