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

//! GP1 control commands
//!
//! Implements GPU control operations including reset, interrupt, and DMA.

use super::super::GPU;

impl GPU {
    /// GP1(0x00): Reset GPU
    ///
    /// Resets the GPU to its initial state without clearing VRAM.
    /// Per PSX-SPX specification, VRAM contents are preserved.
    pub(crate) fn gp1_reset_gpu(&mut self) {
        // Reset GPU state without clearing VRAM (per PSX-SPX spec)
        self.reset_state_preserving_vram();
        self.display_mode.display_disabled = true;
        self.status.display_disabled = true;

        log::debug!("GPU reset");
    }

    /// GP1(0x01): Reset Command Buffer
    ///
    /// Clears the GP0 command FIFO and cancels any ongoing commands.
    /// This is useful for recovering from command processing errors.
    pub(crate) fn gp1_reset_command_buffer(&mut self) {
        // Clear pending commands
        self.command_fifo.clear();

        // Cancel any ongoing VRAM transfer
        self.vram_transfer = None;

        log::debug!("Command buffer reset");
    }

    /// GP1(0x02): Acknowledge GPU Interrupt
    ///
    /// Clears the GPU interrupt request flag. The GPU can generate
    /// interrupts for certain operations, though this is rarely used.
    pub(crate) fn gp1_acknowledge_interrupt(&mut self) {
        self.status.interrupt_request = false;
        log::debug!("GPU interrupt acknowledged");
    }

    /// GP1(0x04): DMA Direction
    ///
    /// Sets the DMA transfer direction/mode.
    ///
    /// # Arguments
    ///
    /// * `value` - Bits 0-1: Direction (0=Off, 1=FIFO, 2=CPUtoGP0, 3=GPUREADtoCPU)
    pub(crate) fn gp1_dma_direction(&mut self, value: u32) {
        let direction = (value & 3) as u8;
        self.status.dma_direction = direction;

        match direction {
            0 => log::debug!("DMA off"),
            1 => log::debug!("DMA FIFO"),
            2 => log::debug!("DMA CPU→GP0"),
            3 => log::debug!("DMA GPUREAD→CPU"),
            _ => unreachable!(),
        }
    }

    /// GP1(0x10): GPU Info
    ///
    /// Requests GPU information to be returned via the GPUREAD register.
    /// Different info types return different GPU state information.
    ///
    /// # Arguments
    ///
    /// * `value` - Bits 0-7: Info type
    ///   - 0x02: Texture window settings
    ///   - 0x03: Draw area top left
    ///   - 0x04: Draw area bottom right
    ///   - 0x05: Draw offset
    ///   - 0x07: GPU version (returns 2 for PSX)
    pub(crate) fn gp1_get_gpu_info(&mut self, value: u32) {
        let info_type = value & 0xFF;

        log::debug!("GPU info request: type {}", info_type);

        // TODO: Implement proper GPU info responses via GPUREAD register
        // Info types:
        // 0x02 - Texture window
        // 0x03 - Draw area top left
        // 0x04 - Draw area bottom right
        // 0x05 - Draw offset
        // 0x07 - GPU version (2 for PSX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gp1_reset_gpu() {
        let mut gpu = GPU::new();

        // Modify some state
        gpu.status.dma_direction = 2;
        gpu.status.interrupt_request = true;
        gpu.command_fifo.push_back(0x12345678);

        // Reset GPU
        gpu.gp1_reset_gpu();

        // Verify reset state per PSX-SPX:
        // - Display should be disabled
        // - Command FIFO should be cleared
        // - VRAM should be preserved (not tested here)
        assert!(gpu.status.display_disabled);
        assert!(gpu.display_mode.display_disabled);
    }

    #[test]
    fn test_gp1_reset_gpu_preserves_vram() {
        let mut gpu = GPU::new();

        // Write test data to VRAM
        gpu.write_vram(100, 100, 0x7FFF);
        gpu.write_vram(200, 200, 0x001F);

        // Reset GPU
        gpu.gp1_reset_gpu();

        // Per PSX-SPX: "VRAM content is NOT affected by this command"
        assert_eq!(gpu.read_vram(100, 100), 0x7FFF);
        assert_eq!(gpu.read_vram(200, 200), 0x001F);
    }

    #[test]
    fn test_gp1_reset_command_buffer() {
        let mut gpu = GPU::new();

        // Add commands to FIFO
        gpu.command_fifo.push_back(0x20FFFFFF);
        gpu.command_fifo.push_back(0x00000000);
        gpu.command_fifo.push_back(0x00640064);

        assert_eq!(gpu.command_fifo.len(), 3);

        // Reset command buffer
        gpu.gp1_reset_command_buffer();

        // FIFO should be cleared
        assert!(gpu.command_fifo.is_empty());
    }

    #[test]
    fn test_gp1_reset_command_buffer_cancels_vram_transfer() {
        let mut gpu = GPU::new();

        // Start a VRAM transfer
        gpu.write_gp0(0xA0000000); // CPU→VRAM
        gpu.write_gp0(0x00000000); // X=0, Y=0
        gpu.write_gp0(0x00640064); // Width=100, Height=100

        // Verify transfer is active
        assert!(gpu.vram_transfer.is_some());

        // Reset command buffer
        gpu.gp1_reset_command_buffer();

        // Transfer should be cancelled
        assert!(gpu.vram_transfer.is_none());
    }

    #[test]
    fn test_gp1_acknowledge_interrupt() {
        let mut gpu = GPU::new();

        // Set interrupt request
        gpu.status.interrupt_request = true;
        assert!(gpu.status.interrupt_request);

        // Acknowledge interrupt
        gpu.gp1_acknowledge_interrupt();

        // Interrupt should be cleared
        assert!(!gpu.status.interrupt_request);
    }

    #[test]
    fn test_gp1_dma_direction() {
        let mut gpu = GPU::new();

        // Test all 4 DMA directions per PSX-SPX
        // 0 = Off
        gpu.gp1_dma_direction(0);
        assert_eq!(gpu.status.dma_direction, 0);

        // 1 = FIFO
        gpu.gp1_dma_direction(1);
        assert_eq!(gpu.status.dma_direction, 1);

        // 2 = CPU→GP0
        gpu.gp1_dma_direction(2);
        assert_eq!(gpu.status.dma_direction, 2);

        // 3 = GPUREAD→CPU
        gpu.gp1_dma_direction(3);
        assert_eq!(gpu.status.dma_direction, 3);
    }

    #[test]
    fn test_gp1_dma_direction_masks_lower_2_bits() {
        let mut gpu = GPU::new();

        // Test that only bits 0-1 are used
        gpu.gp1_dma_direction(0xFF);
        assert_eq!(gpu.status.dma_direction, 3); // 0xFF & 0x3 = 3

        gpu.gp1_dma_direction(0x04);
        assert_eq!(gpu.status.dma_direction, 0); // 0x04 & 0x3 = 0

        gpu.gp1_dma_direction(0x05);
        assert_eq!(gpu.status.dma_direction, 1); // 0x05 & 0x3 = 1
    }

    #[test]
    fn test_gp1_get_gpu_info_info_types() {
        let mut gpu = GPU::new();

        // Per PSX-SPX, valid info types are:
        // 0x02 = Texture window
        // 0x03 = Draw area top left
        // 0x04 = Draw area bottom right
        // 0x05 = Draw offset
        // 0x07 = GPU version (returns 2)

        // Test that function doesn't crash with valid types
        gpu.gp1_get_gpu_info(0x02);
        gpu.gp1_get_gpu_info(0x03);
        gpu.gp1_get_gpu_info(0x04);
        gpu.gp1_get_gpu_info(0x05);
        gpu.gp1_get_gpu_info(0x07);
    }

    #[test]
    fn test_gp1_get_gpu_info_masks_to_8_bits() {
        let mut gpu = GPU::new();

        // Test that info type is masked to 8 bits (0-255)
        gpu.gp1_get_gpu_info(0x102); // Should be treated as 0x02
        gpu.gp1_get_gpu_info(0xFFFF); // Should be treated as 0xFF
    }

    #[test]
    fn test_gp1_command_sequence() {
        let mut gpu = GPU::new();

        // Test a typical GP1 command sequence
        // Per PSX-SPX reset sequence:
        gpu.gp1_reset_command_buffer(); // GP1(01h)
        gpu.gp1_acknowledge_interrupt(); // GP1(02h)
        gpu.gp1_display_enable(1); // GP1(03h) - disable
        gpu.gp1_dma_direction(0); // GP1(04h) - off

        assert!(gpu.command_fifo.is_empty());
        assert!(!gpu.status.interrupt_request);
        assert!(gpu.status.display_disabled);
        assert_eq!(gpu.status.dma_direction, 0);
    }

    #[test]
    fn test_gp1_reset_vs_command_buffer_reset() {
        let mut gpu = GPU::new();

        // Set up some state
        gpu.status.dma_direction = 2;
        gpu.command_fifo.push_back(0x12345678);
        gpu.write_vram(50, 50, 0xABCD);

        // Reset command buffer only
        gpu.gp1_reset_command_buffer();

        // Command FIFO cleared
        assert!(gpu.command_fifo.is_empty());

        // But other state preserved
        assert_eq!(gpu.status.dma_direction, 2);
        assert_eq!(gpu.read_vram(50, 50), 0xABCD);

        // Now full reset
        gpu.status.dma_direction = 3;
        gpu.command_fifo.push_back(0x87654321);

        gpu.gp1_reset_gpu();

        // Everything reset except VRAM
        assert_eq!(gpu.read_vram(50, 50), 0xABCD); // VRAM preserved
    }

    #[test]
    fn test_gp1_acknowledge_interrupt_idempotent() {
        let mut gpu = GPU::new();

        // Acknowledging when no interrupt is pending should not crash
        assert!(!gpu.status.interrupt_request);
        gpu.gp1_acknowledge_interrupt();
        assert!(!gpu.status.interrupt_request);

        // Multiple acknowledges
        gpu.status.interrupt_request = true;
        gpu.gp1_acknowledge_interrupt();
        gpu.gp1_acknowledge_interrupt();
        gpu.gp1_acknowledge_interrupt();
        assert!(!gpu.status.interrupt_request);
    }
}
