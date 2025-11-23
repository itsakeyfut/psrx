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

//! PSRX UI entry point
//!
//! This binary provides the graphical user interface for the PSRX emulator.
//! It uses winit for window management, wgpu for GPU acceleration, and egui for the UI.

use psrx::frontend::Application;
use std::env;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting PSRX UI...");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <BIOS_FILE>", args[0]);
        eprintln!("Example: {} SCPH1001.BIN", args[0]);
        std::process::exit(1);
    }

    let bios_path = &args[1];
    log::info!("BIOS path: {}", bios_path);

    // Create event loop
    let event_loop = EventLoop::new()?;

    // Create application
    let mut app = Application::new(bios_path);

    log::info!("Running event loop...");

    // Run application
    event_loop.run_app(&mut app)?;

    Ok(())
}
