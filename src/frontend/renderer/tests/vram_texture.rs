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

//! Unit tests for VRAM texture RGB conversion

use crate::frontend::renderer::vram_texture::convert_rgb15_to_rgba8;

#[test]
fn test_rgb15_to_rgba8_white() {
    let vram = vec![0x7FFF]; // White (all bits set)
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba, [248, 248, 248, 255]);
}

#[test]
fn test_rgb15_to_rgba8_black() {
    let vram = vec![0x0000]; // Black (all bits clear)
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba, [0, 0, 0, 255]);
}

#[test]
fn test_rgb15_to_rgba8_red() {
    let vram = vec![0x001F]; // Red (bits 0-4 set)
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba, [248, 0, 0, 255]);
}

#[test]
fn test_rgb15_to_rgba8_green() {
    let vram = vec![0x03E0]; // Green (bits 5-9 set)
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba, [0, 248, 0, 255]);
}

#[test]
fn test_rgb15_to_rgba8_blue() {
    let vram = vec![0x7C00]; // Blue (bits 10-14 set)
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba, [0, 0, 248, 255]);
}

#[test]
fn test_rgb15_to_rgba8_multiple_pixels() {
    let vram = vec![0x7FFF, 0x0000, 0x001F]; // White, Black, Red
    let rgba = convert_rgb15_to_rgba8(&vram);
    assert_eq!(rgba[0..4], [248, 248, 248, 255]); // White
    assert_eq!(rgba[4..8], [0, 0, 0, 255]); // Black
    assert_eq!(rgba[8..12], [248, 0, 0, 255]); // Red
}
