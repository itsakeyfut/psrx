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

//! DMA (Direct Memory Access) Controller
//!
//! This module implements the PlayStation's DMA controller, which provides high-speed
//! data transfers between memory and peripherals without CPU intervention.
//!
//! # DMA Channels
//!
//! The PSX has 7 DMA channels, each dedicated to a specific peripheral:
//!
//! | Channel | Device      | Base Address |
//! |---------|-------------|--------------|
//! | 0       | MDEC In     | 0x1F801080   |
//! | 1       | MDEC Out    | 0x1F801090   |
//! | 2       | GPU         | 0x1F8010A0   |
//! | 3       | CD-ROM      | 0x1F8010B0   |
//! | 4       | SPU         | 0x1F8010C0   |
//! | 5       | PIO         | 0x1F8010D0   |
//! | 6       | OTC         | 0x1F8010E0   |
//!
//! # Channel Registers
//!
//! Each channel has three 32-bit registers:
//! - **MADR** (+0x00): Memory address register
//! - **BCR** (+0x04): Block control register
//! - **CHCR** (+0x08): Channel control register
//!
//! # Global Registers
//!
//! - **DPCR** (0x1F8010F0): DMA control register (channel priorities)
//! - **DICR** (0x1F8010F4): DMA interrupt register
//!
//! # Transfer Modes
//!
//! DMA supports three synchronization modes:
//! - **Mode 0** (Immediate): Transfer entire block at once
//! - **Mode 1** (Block): Transfer in blocks with device sync
//! - **Mode 2** (Linked-list): Follow linked list in memory (GPU only)
//!
//! # References
//!
//! - [PSX-SPX: DMA Controller](http://problemkaputt.de/psx-spx.htm#dmacontroller)

use crate::core::cdrom::CDROM;
use crate::core::gpu::GPU;
use crate::core::spu::SPU;

/// DMA Controller with 7 channels
///
/// The DMA controller manages data transfers between memory and peripherals,
/// allowing high-speed transfers without CPU intervention.
///
/// # Examples
///
/// ```
/// use psrx::core::dma::DMA;
///
/// let mut dma = DMA::new();
/// assert_eq!(dma.read_control(), 0x07654321);
/// ```
pub struct DMA {
    /// 7 DMA channels (MDEC In/Out, GPU, CD-ROM, SPU, PIO, OTC)
    channels: [DMAChannel; 7],

    /// DMA Control Register (DPCR) at 0x1F8010F0
    ///
    /// Contains channel priority and enable bits.
    /// Default: 0x07654321 (channel priorities in order)
    control: u32,

    /// DMA Interrupt Register (DICR) at 0x1F8010F4
    ///
    /// Controls interrupt generation and flags for DMA completion.
    interrupt: u32,
}

/// Single DMA channel
///
/// Each channel manages transfers for one specific peripheral device.
#[derive(Clone)]
pub struct DMAChannel {
    /// Memory Address Register (MADR)
    ///
    /// Base address in RAM for the DMA transfer.
    base_address: u32,

    /// Block Control Register (BCR)
    ///
    /// Controls block size and count:
    /// - Bits 0-15: Block size (words)
    /// - Bits 16-31: Block count
    block_control: u32,

    /// Channel Control Register (CHCR)
    ///
    /// Controls transfer direction, sync mode, and activation:
    /// - Bit 0: Direction (0=to RAM, 1=from RAM)
    /// - Bit 1: Address step (0=forward, 1=backward)
    /// - Bit 8: Chopping enable
    /// - Bits 9-10: Sync mode (0=immediate, 1=block, 2=linked-list)
    /// - Bit 24: Start/busy flag
    /// - Bit 28: Manual trigger
    channel_control: u32,

    /// Channel ID (0-6)
    channel_id: u8,
}

impl DMAChannel {
    /// Direction: Device to RAM
    const TRANSFER_TO_RAM: u32 = 0;

    /// Direction: RAM to Device
    const TRANSFER_FROM_RAM: u32 = 1;

    /// Create a new DMA channel
    ///
    /// # Arguments
    ///
    /// * `channel_id` - Channel number (0-6)
    fn new(channel_id: u8) -> Self {
        Self {
            base_address: 0,
            block_control: 0,
            channel_control: 0,
            channel_id,
        }
    }

    /// Check if channel is active (bit 24 of CHCR)
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        (self.channel_control & 0x0100_0000) != 0
    }

    /// Get transfer direction (bit 0 of CHCR)
    ///
    /// Returns 0 for device→RAM, 1 for RAM→device
    #[inline(always)]
    pub fn direction(&self) -> u32 {
        self.channel_control & 1
    }

    /// Get synchronization mode (bits 9-10 of CHCR)
    ///
    /// - 0: Immediate (transfer all at once)
    /// - 1: Block (sync with device)
    /// - 2: Linked-list (GPU only)
    #[inline(always)]
    pub fn sync_mode(&self) -> u32 {
        (self.channel_control >> 9) & 3
    }

    /// Check if manual trigger is enabled (bit 28 of CHCR)
    #[inline(always)]
    pub fn trigger(&self) -> bool {
        (self.channel_control & 0x1000_0000) != 0
    }

    /// Deactivate the channel (clear bit 24 of CHCR)
    fn deactivate(&mut self) {
        log::trace!("DMA channel {} deactivated", self.channel_id);
        self.channel_control &= !0x0100_0000;
    }
}

impl DMA {
    /// Channel 0: MDEC In (compression input)
    #[allow(dead_code)]
    const CH_MDEC_IN: usize = 0;

    /// Channel 1: MDEC Out (decompression output)
    #[allow(dead_code)]
    const CH_MDEC_OUT: usize = 1;

    /// Channel 2: GPU (graphics)
    pub const CH_GPU: usize = 2;

    /// Channel 3: CD-ROM (disc drive)
    pub const CH_CDROM: usize = 3;

    /// Channel 4: SPU (sound)
    pub const CH_SPU: usize = 4;

    /// Channel 5: PIO (expansion port)
    #[allow(dead_code)]
    const CH_PIO: usize = 5;

    /// Channel 6: OTC (ordering table clear)
    pub const CH_OTC: usize = 6;

    /// Create a new DMA controller
    ///
    /// All channels start inactive with default priority ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use psrx::core::dma::DMA;
    ///
    /// let dma = DMA::new();
    /// ```
    pub fn new() -> Self {
        Self {
            channels: [
                DMAChannel::new(0),
                DMAChannel::new(1),
                DMAChannel::new(2),
                DMAChannel::new(3),
                DMAChannel::new(4),
                DMAChannel::new(5),
                DMAChannel::new(6),
            ],
            control: 0x0765_4321, // Default channel priority
            interrupt: 0,
        }
    }

    /// Process DMA transfers for all active channels
    ///
    /// Should be called periodically (e.g., once per scanline) to handle
    /// active DMA transfers. Respects DPCR enable bits and priority ordering.
    ///
    /// # Arguments
    ///
    /// * `ram` - Main system RAM
    /// * `gpu` - GPU reference for GPU transfers
    /// * `cdrom` - CD-ROM reference for CD-ROM transfers
    /// * `spu` - SPU reference for SPU transfers
    ///
    /// # Returns
    ///
    /// `true` if any transfer generated an interrupt
    pub fn tick(
        &mut self,
        ram: &mut [u8],
        gpu: &mut GPU,
        cdrom: &mut CDROM,
        spu: &mut SPU,
    ) -> bool {
        let mut irq = false;

        // Build list of active channels with their priorities
        let mut active_channels: Vec<(usize, u32)> = Vec::new();
        for ch_id in 0..7 {
            // Check DPCR enable bit before allowing channel to execute
            if self.is_channel_enabled(ch_id)
                && self.channels[ch_id].is_active()
                && self.channels[ch_id].trigger()
            {
                let priority = self.channel_priority(ch_id);
                active_channels.push((ch_id, priority));
            }
        }

        // Sort by priority (higher priority value = higher priority)
        active_channels.sort_by(|a, b| b.1.cmp(&a.1));

        // Execute transfers in priority order
        for (ch_id, _) in active_channels {
            irq |= self.execute_transfer(ch_id, ram, gpu, cdrom, spu);
        }

        irq
    }

    /// Execute a DMA transfer for the specified channel
    ///
    /// # Arguments
    ///
    /// * `ch_id` - Channel ID (0-6)
    /// * `ram` - Main system RAM
    /// * `gpu` - GPU reference
    /// * `cdrom` - CD-ROM reference
    /// * `spu` - SPU reference
    ///
    /// # Returns
    ///
    /// `true` if transfer completed and generated an interrupt
    fn execute_transfer(
        &mut self,
        ch_id: usize,
        ram: &mut [u8],
        gpu: &mut GPU,
        cdrom: &mut CDROM,
        spu: &mut SPU,
    ) -> bool {
        log::debug!(
            "DMA{} transfer: addr=0x{:08X} bcr=0x{:08X} chcr=0x{:08X}",
            ch_id,
            self.channels[ch_id].base_address,
            self.channels[ch_id].block_control,
            self.channels[ch_id].channel_control
        );

        let completed = match ch_id {
            Self::CH_GPU => self.transfer_gpu(ram, gpu),
            Self::CH_CDROM => self.transfer_cdrom(ram, cdrom),
            Self::CH_SPU => self.transfer_spu(ram, spu),
            Self::CH_OTC => self.transfer_otc(ram),
            _ => {
                log::warn!("DMA{} not implemented", ch_id);
                self.channels[ch_id].deactivate();
                false
            }
        };

        // Set interrupt flag for this channel if transfer completed
        if completed {
            self.interrupt |= 1 << (24 + ch_id);
            log::trace!("DMA{} interrupt flag set in DICR", ch_id);

            // Update master flag and determine if IRQ should be raised
            self.update_master_flag();

            // Only signal IRQ if master flag is set
            (self.interrupt & (1 << 31)) != 0
        } else {
            false
        }
    }

    /// Execute GPU DMA transfer (channel 2)
    ///
    /// Supports linked-list mode for command buffer transfers.
    fn transfer_gpu(&mut self, ram: &mut [u8], gpu: &mut GPU) -> bool {
        // Extract channel data first to avoid borrow issues
        let sync_mode = self.channels[Self::CH_GPU].sync_mode();
        let direction = self.channels[Self::CH_GPU].direction();
        let base_address = self.channels[Self::CH_GPU].base_address;
        let block_control = self.channels[Self::CH_GPU].block_control;

        match sync_mode {
            2 => {
                // Linked-list mode (GPU command lists)
                let mut addr = base_address & 0x001F_FFFC;

                loop {
                    // Read linked-list header
                    let header = self.read_ram_u32(ram, addr);
                    let count = (header >> 24) as usize;

                    // Send all words in this node to GPU
                    for i in 0..count {
                        let word = self.read_ram_u32(ram, addr + 4 + (i * 4) as u32);
                        gpu.write_gp0(word);
                    }

                    // Check for end of list marker (bit 23)
                    if (header & 0x0080_0000) != 0 {
                        break;
                    }

                    // Follow link to next node
                    addr = header & 0x001F_FFFC;
                }

                self.channels[Self::CH_GPU].deactivate();
                log::debug!("GPU DMA linked-list transfer complete");
                true
            }
            0 | 1 => {
                // Block mode for VRAM transfers
                let block_size = (block_control & 0xFFFF) as usize;
                let block_count = ((block_control >> 16) & 0xFFFF) as usize;
                let mut addr = base_address & 0x001F_FFFC;

                let total_words = if sync_mode == 0 {
                    block_size
                } else {
                    block_size * block_count
                };

                if direction == DMAChannel::TRANSFER_FROM_RAM {
                    // RAM → GPU
                    for _ in 0..total_words {
                        let word = self.read_ram_u32(ram, addr);
                        gpu.write_gp0(word);
                        addr = (addr + 4) & 0x001F_FFFC;
                    }
                } else if direction == DMAChannel::TRANSFER_TO_RAM {
                    // GPU → RAM (VRAM reads)
                    for _ in 0..total_words {
                        let word = gpu.read_gpuread();
                        self.write_ram_u32(ram, addr, word);
                        addr = (addr + 4) & 0x001F_FFFC;
                    }
                }

                self.channels[Self::CH_GPU].deactivate();
                log::debug!("GPU DMA block transfer complete ({} words)", total_words);
                true
            }
            _ => {
                log::warn!("GPU DMA sync mode {} not supported", sync_mode);
                self.channels[Self::CH_GPU].deactivate();
                false
            }
        }
    }

    /// Execute CD-ROM DMA transfer (channel 3)
    ///
    /// Transfers sector data from CD-ROM to RAM.
    fn transfer_cdrom(&mut self, ram: &mut [u8], cdrom: &mut CDROM) -> bool {
        // Extract channel data first to avoid borrow issues
        let block_control = self.channels[Self::CH_CDROM].block_control;
        let base_address = self.channels[Self::CH_CDROM].base_address;

        // CD-ROM only supports device→RAM transfers
        let block_size = (block_control & 0xFFFF) as usize;
        let block_count = ((block_control >> 16) & 0xFFFF) as usize;

        let mut addr = base_address & 0x001F_FFFC;
        let total_words = block_size * block_count;

        // Transfer data from CD-ROM buffer to RAM (word by word)
        for _ in 0..total_words {
            // Read 4 bytes (1 word) from CD-ROM
            let byte0 = cdrom.get_data_byte();
            let byte1 = cdrom.get_data_byte();
            let byte2 = cdrom.get_data_byte();
            let byte3 = cdrom.get_data_byte();

            let word = u32::from_le_bytes([byte0, byte1, byte2, byte3]);
            self.write_ram_u32(ram, addr, word);

            addr = (addr + 4) & 0x001F_FFFC;
        }

        self.channels[Self::CH_CDROM].deactivate();
        log::debug!(
            "CD-ROM DMA transfer complete ({} words = {} bytes)",
            total_words,
            total_words * 4
        );
        true
    }

    /// Execute SPU DMA transfer (channel 4)
    ///
    /// Transfers data between RAM and SPU RAM.
    /// Supports sync mode 0 (manual/immediate) and sync mode 1 (block).
    fn transfer_spu(&mut self, ram: &mut [u8], spu: &mut SPU) -> bool {
        // Extract channel data first to avoid borrow issues
        let sync_mode = self.channels[Self::CH_SPU].sync_mode();
        let direction = self.channels[Self::CH_SPU].direction();
        let base_address = self.channels[Self::CH_SPU].base_address;
        let block_control = self.channels[Self::CH_SPU].block_control;

        match sync_mode {
            // Sync mode 0: Manual/Immediate
            0 => {
                let words = if block_control & 0xFFFF > 0 {
                    block_control & 0xFFFF
                } else {
                    0x10000
                };

                let mut addr = base_address & 0x001F_FFFC;

                for _ in 0..words {
                    if direction == DMAChannel::TRANSFER_FROM_RAM {
                        // RAM → SPU
                        let value = self.read_ram_u32(ram, addr);
                        spu.dma_write(value);
                    } else {
                        // SPU → RAM
                        let value = spu.dma_read();
                        self.write_ram_u32(ram, addr, value);
                    }

                    addr = (addr + 4) & 0x001F_FFFC;
                }

                spu.flush_dma_fifo();
                self.channels[Self::CH_SPU].deactivate();
                log::debug!("SPU DMA sync mode 0 transfer complete ({} words)", words);
                true
            }

            // Sync mode 1: Block
            1 => {
                let block_size = (block_control & 0xFFFF) as usize;
                let block_count = ((block_control >> 16) & 0xFFFF) as usize;

                let mut addr = base_address & 0x001F_FFFC;

                for _ in 0..block_count {
                    for _ in 0..block_size {
                        if direction == DMAChannel::TRANSFER_FROM_RAM {
                            // RAM → SPU
                            let value = self.read_ram_u32(ram, addr);
                            spu.dma_write(value);
                        } else {
                            // SPU → RAM
                            let value = spu.dma_read();
                            self.write_ram_u32(ram, addr, value);
                        }

                        addr = (addr + 4) & 0x001F_FFFC;
                    }
                }

                spu.flush_dma_fifo();
                self.channels[Self::CH_SPU].deactivate();
                log::debug!(
                    "SPU DMA block transfer complete ({} blocks × {} words = {} words)",
                    block_count,
                    block_size,
                    block_count * block_size
                );
                true
            }

            _ => {
                log::warn!("SPU DMA sync mode {} not supported", sync_mode);
                self.channels[Self::CH_SPU].deactivate();
                false
            }
        }
    }

    /// Execute OTC (Ordering Table Clear) transfer (channel 6)
    ///
    /// Creates a reverse-linked list in RAM for GPU command ordering.
    /// Used to set up GPU command lists for rendering.
    fn transfer_otc(&mut self, ram: &mut [u8]) -> bool {
        // Extract channel data first to avoid borrow issues
        let block_control = self.channels[Self::CH_OTC].block_control;
        let base_address = self.channels[Self::CH_OTC].base_address;

        let count = block_control & 0xFFFF;
        let mut addr = base_address & 0x001F_FFFC;

        // Write reverse-linked list
        for i in 0..count {
            if i == count - 1 {
                // Last entry: end marker
                self.write_ram_u32(ram, addr, 0x00FF_FFFF);
            } else {
                // Link to previous address (reverse order)
                self.write_ram_u32(ram, addr, (addr.wrapping_sub(4)) & 0x001F_FFFC);
            }

            addr = addr.wrapping_sub(4) & 0x001F_FFFC;
        }

        self.channels[Self::CH_OTC].deactivate();
        log::debug!("OTC DMA transfer complete ({} entries)", count);
        true
    }

    /// Read 32-bit word from RAM
    #[inline(always)]
    fn read_ram_u32(&self, ram: &[u8], addr: u32) -> u32 {
        let addr = (addr & 0x001F_FFFC) as usize;
        if addr + 4 > ram.len() {
            log::error!("DMA read out of bounds: 0x{:08X}", addr);
            return 0;
        }
        u32::from_le_bytes([ram[addr], ram[addr + 1], ram[addr + 2], ram[addr + 3]])
    }

    /// Write 32-bit word to RAM
    #[inline(always)]
    fn write_ram_u32(&self, ram: &mut [u8], addr: u32, value: u32) {
        let addr = (addr & 0x001F_FFFC) as usize;
        if addr + 4 > ram.len() {
            log::error!("DMA write out of bounds: 0x{:08X}", addr);
            return;
        }
        let bytes = value.to_le_bytes();
        ram[addr..addr + 4].copy_from_slice(&bytes);
    }

    // DPCR and DICR helper methods

    /// Check if a channel is enabled in DPCR
    ///
    /// Each channel's enable bit is bit 3 of its 4-bit nibble in DPCR.
    /// Channel N's nibble is at bits (N*4) to (N*4+3).
    #[inline(always)]
    fn is_channel_enabled(&self, channel: usize) -> bool {
        let shift = channel * 4;
        (self.control & (0x8 << shift)) != 0
    }

    /// Get the priority of a channel from DPCR
    ///
    /// Each channel's priority is bits 0-2 of its 4-bit nibble in DPCR.
    /// Higher values indicate higher priority.
    #[inline(always)]
    fn channel_priority(&self, channel: usize) -> u32 {
        let shift = channel * 4;
        (self.control >> shift) & 0x7
    }

    /// Compute and update DICR master flag (bit 31)
    ///
    /// The master flag determines whether an IRQ should be raised.
    /// It is set when:
    /// - Force flag (bit 15) is set, OR
    /// - Master enable (bit 23) is set AND any channel has both:
    ///   - Channel interrupt enable (bit 16+N) set
    ///   - Channel interrupt flag (bit 24+N) set
    ///
    /// This method should be called after setting/clearing channel flags.
    fn update_master_flag(&mut self) {
        let force = (self.interrupt & (1 << 15)) != 0;
        let master_enable = (self.interrupt & (1 << 23)) != 0;

        let mut any_triggered = false;
        for ch_id in 0..7 {
            let channel_enable = (self.interrupt & (1 << (16 + ch_id))) != 0;
            let channel_flag = (self.interrupt & (1 << (24 + ch_id))) != 0;
            if channel_enable && channel_flag {
                any_triggered = true;
                break;
            }
        }

        let master_flag = force || (master_enable && any_triggered);

        // Set or clear bit 31
        if master_flag {
            self.interrupt |= 1 << 31;
            log::trace!("DICR master flag set (bit 31)");
        } else {
            self.interrupt &= !(1 << 31);
        }
    }

    // Register access methods

    /// Read channel MADR register
    pub fn read_madr(&self, channel: usize) -> u32 {
        self.channels[channel].base_address
    }

    /// Write channel MADR register
    pub fn write_madr(&mut self, channel: usize, value: u32) {
        self.channels[channel].base_address = value & 0x00FF_FFFF;
        log::trace!("DMA{} MADR = 0x{:08X}", channel, value);
    }

    /// Read channel BCR register
    pub fn read_bcr(&self, channel: usize) -> u32 {
        self.channels[channel].block_control
    }

    /// Write channel BCR register
    pub fn write_bcr(&mut self, channel: usize, value: u32) {
        self.channels[channel].block_control = value;
        log::trace!("DMA{} BCR = 0x{:08X}", channel, value);
    }

    /// Read channel CHCR register
    pub fn read_chcr(&self, channel: usize) -> u32 {
        self.channels[channel].channel_control
    }

    /// Write channel CHCR register
    pub fn write_chcr(&mut self, channel: usize, value: u32) {
        self.channels[channel].channel_control = value;
        log::trace!("DMA{} CHCR = 0x{:08X}", channel, value);

        // Log transfer initiation
        if (value & 0x0100_0000) != 0 {
            log::debug!(
                "DMA{} started: addr=0x{:08X} bcr=0x{:08X} mode={}",
                channel,
                self.channels[channel].base_address,
                self.channels[channel].block_control,
                self.channels[channel].sync_mode()
            );
        }
    }

    /// Read DMA Control Register (DPCR)
    pub fn read_control(&self) -> u32 {
        self.control
    }

    /// Write DMA Control Register (DPCR)
    pub fn write_control(&mut self, value: u32) {
        self.control = value;
        log::trace!("DPCR = 0x{:08X}", value);
    }

    /// Read DMA Interrupt Register (DICR)
    pub fn read_interrupt(&self) -> u32 {
        self.interrupt
    }

    /// Write DMA Interrupt Register (DICR)
    pub fn write_interrupt(&mut self, value: u32) {
        // Update writable bits (6-23), preserving reserved bits 0-5
        // Bits 0-5 are always 0, bits 6-23 are writable, bits 24-31 are handled separately
        self.interrupt = (self.interrupt & 0xFF00_0000) | (value & 0x00FF_FFC0);

        // Handle write-1-to-clear for bits 24-30 (interrupt flags)
        let clear_mask = (value >> 24) & 0x7F;
        self.interrupt &= !(clear_mask << 24);

        // Recompute master flag after clearing flags
        self.update_master_flag();

        log::trace!("DICR = 0x{:08X}", self.interrupt);
    }
}

impl Default for DMA {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test DMA controller
    fn create_test_dma() -> DMA {
        DMA::new()
    }

    // ========== Initialization Tests ==========

    #[test]
    fn test_dma_initialization() {
        let dma = create_test_dma();

        // Verify default DPCR value (0x07654321 per PSX-SPX spec)
        assert_eq!(
            dma.read_control(),
            0x07654321,
            "DPCR should initialize to default priority order"
        );

        // Verify DICR starts at 0
        assert_eq!(dma.read_interrupt(), 0, "DICR should initialize to 0");

        // Verify all channels are inactive
        for ch in 0..7 {
            assert!(
                !dma.channels[ch].is_active(),
                "Channel {} should be inactive after init",
                ch
            );
            assert_eq!(
                dma.read_madr(ch),
                0,
                "Channel {} MADR should be 0",
                ch
            );
            assert_eq!(dma.read_bcr(ch), 0, "Channel {} BCR should be 0", ch);
            assert_eq!(dma.read_chcr(ch), 0, "Channel {} CHCR should be 0", ch);
        }
    }

    // ========== MADR Register Tests ==========

    #[test]
    fn test_madr_read_write() {
        let mut dma = create_test_dma();

        // Write and read MADR for channel 2 (GPU)
        dma.write_madr(DMA::CH_GPU, 0x12345678);
        assert_eq!(
            dma.read_madr(DMA::CH_GPU),
            0x00345678,
            "MADR should mask to 24 bits (per PSX-SPX spec)"
        );

        // Test different channels
        dma.write_madr(DMA::CH_CDROM, 0x80001000);
        assert_eq!(
            dma.read_madr(DMA::CH_CDROM),
            0x00001000,
            "MADR should mask upper bits"
        );
    }

    #[test]
    fn test_madr_address_masking() {
        let mut dma = create_test_dma();

        // Test that addresses are masked to 24 bits (bits 24-31 cleared)
        dma.write_madr(DMA::CH_GPU, 0xFFFFFFFF);
        assert_eq!(
            dma.read_madr(DMA::CH_GPU),
            0x00FFFFFF,
            "MADR should mask to 24 bits"
        );
    }

    // ========== BCR Register Tests ==========

    #[test]
    fn test_bcr_read_write() {
        let mut dma = create_test_dma();

        // Write and read BCR
        dma.write_bcr(DMA::CH_GPU, 0x00100020);
        assert_eq!(
            dma.read_bcr(DMA::CH_GPU),
            0x00100020,
            "BCR should store full 32 bits"
        );

        // Test block size and count extraction (per PSX-SPX spec)
        let bcr = dma.read_bcr(DMA::CH_GPU);
        let block_size = bcr & 0xFFFF;
        let block_count = (bcr >> 16) & 0xFFFF;

        assert_eq!(block_size, 0x0020, "Block size should be lower 16 bits");
        assert_eq!(
            block_count, 0x0010,
            "Block count should be upper 16 bits"
        );
    }

    // ========== CHCR Register Tests ==========

    #[test]
    fn test_chcr_direction_bit() {
        let mut dma = create_test_dma();

        // Bit 0: Direction (0=device→RAM, 1=RAM→device)
        dma.write_chcr(DMA::CH_GPU, 0x00000001);
        assert_eq!(
            dma.channels[DMA::CH_GPU].direction(),
            1,
            "Direction bit should be set to RAM→device"
        );

        dma.write_chcr(DMA::CH_GPU, 0x00000000);
        assert_eq!(
            dma.channels[DMA::CH_GPU].direction(),
            0,
            "Direction bit should be set to device→RAM"
        );
    }

    #[test]
    fn test_chcr_sync_mode_bits() {
        let mut dma = create_test_dma();

        // Bits 9-10: Sync mode (per PSX-SPX spec)
        dma.write_chcr(DMA::CH_GPU, 0x00000000); // Mode 0: Immediate
        assert_eq!(dma.channels[DMA::CH_GPU].sync_mode(), 0, "Sync mode 0");

        dma.write_chcr(DMA::CH_GPU, 0x00000200); // Mode 1: Block
        assert_eq!(dma.channels[DMA::CH_GPU].sync_mode(), 1, "Sync mode 1");

        dma.write_chcr(DMA::CH_GPU, 0x00000400); // Mode 2: Linked-list
        assert_eq!(dma.channels[DMA::CH_GPU].sync_mode(), 2, "Sync mode 2");
    }

    #[test]
    fn test_chcr_active_bit() {
        let mut dma = create_test_dma();

        // Bit 24: Start/busy flag
        dma.write_chcr(DMA::CH_GPU, 0x01000000);
        assert!(
            dma.channels[DMA::CH_GPU].is_active(),
            "Channel should be active when bit 24 is set"
        );

        dma.write_chcr(DMA::CH_GPU, 0x00000000);
        assert!(
            !dma.channels[DMA::CH_GPU].is_active(),
            "Channel should be inactive when bit 24 is clear"
        );
    }

    #[test]
    fn test_chcr_trigger_bit() {
        let mut dma = create_test_dma();

        // Bit 28: Manual trigger
        dma.write_chcr(DMA::CH_GPU, 0x10000000);
        assert!(
            dma.channels[DMA::CH_GPU].trigger(),
            "Trigger should be set when bit 28 is set"
        );

        dma.write_chcr(DMA::CH_GPU, 0x00000000);
        assert!(
            !dma.channels[DMA::CH_GPU].trigger(),
            "Trigger should be clear when bit 28 is clear"
        );
    }

    #[test]
    fn test_channel_deactivation() {
        let mut dma = create_test_dma();

        // Activate channel
        dma.write_chcr(DMA::CH_GPU, 0x01000000);
        assert!(dma.channels[DMA::CH_GPU].is_active());

        // Deactivate channel
        dma.channels[DMA::CH_GPU].deactivate();
        assert!(
            !dma.channels[DMA::CH_GPU].is_active(),
            "Channel should be inactive after deactivate()"
        );
    }

    // ========== DPCR (Control Register) Tests ==========

    #[test]
    fn test_dpcr_channel_enable_bits() {
        let mut dma = create_test_dma();

        // Bit 3, 7, 11, 15, 19, 23, 27: Channel enable (per PSX-SPX spec)
        dma.write_control(0xFFFFFFFF);
        for ch in 0..7 {
            assert!(
                dma.is_channel_enabled(ch),
                "Channel {} should be enabled",
                ch
            );
        }

        // Disable all channels
        dma.write_control(0x00000000);
        for ch in 0..7 {
            assert!(
                !dma.is_channel_enabled(ch),
                "Channel {} should be disabled",
                ch
            );
        }
    }

    #[test]
    fn test_dpcr_channel_priority() {
        let mut dma = create_test_dma();

        // Default priority: 0x07654321
        assert_eq!(
            dma.channel_priority(0),
            1,
            "Channel 0 default priority should be 1"
        );
        assert_eq!(
            dma.channel_priority(1),
            2,
            "Channel 1 default priority should be 2"
        );
        assert_eq!(
            dma.channel_priority(6),
            7,
            "Channel 6 default priority should be 7"
        );

        // Set custom priorities
        dma.write_control(0x76543210);
        assert_eq!(dma.channel_priority(0), 0);
        assert_eq!(dma.channel_priority(1), 1);
        assert_eq!(dma.channel_priority(6), 6);
    }

    #[test]
    fn test_dpcr_individual_channel_enable() {
        let mut dma = create_test_dma();

        // Enable only channel 2 (GPU) - bit 11
        dma.write_control(0x00000808);
        assert!(
            dma.is_channel_enabled(DMA::CH_GPU),
            "Channel 2 should be enabled"
        );
        assert!(
            !dma.is_channel_enabled(DMA::CH_CDROM),
            "Channel 3 should be disabled"
        );
    }

    // ========== DICR (Interrupt Register) Tests ==========

    #[test]
    fn test_dicr_channel_interrupt_enable() {
        let mut dma = create_test_dma();

        // Bits 16-22: Channel interrupt enable masks
        dma.write_interrupt(0x007F0000);
        assert_eq!(
            dma.read_interrupt() & 0x007F0000,
            0x007F0000,
            "All channel interrupt enables should be set"
        );
    }

    #[test]
    fn test_dicr_channel_interrupt_flags() {
        let mut dma = create_test_dma();

        // Bits 24-30: Channel interrupt flags
        // Set flag for channel 2
        dma.interrupt |= 1 << (24 + DMA::CH_GPU);
        assert!(
            (dma.read_interrupt() & (1 << (24 + DMA::CH_GPU))) != 0,
            "Channel 2 interrupt flag should be set"
        );
    }

    #[test]
    fn test_dicr_write_1_to_clear_flags() {
        let mut dma = create_test_dma();

        // Set interrupt flag for channel 2
        dma.interrupt |= 1 << (24 + DMA::CH_GPU);

        // Write 1 to clear (per PSX-SPX spec)
        dma.write_interrupt(1 << (24 + DMA::CH_GPU));

        assert_eq!(
            dma.read_interrupt() & (1 << (24 + DMA::CH_GPU)),
            0,
            "Channel 2 flag should be cleared by writing 1"
        );
    }

    #[test]
    fn test_dicr_master_flag_with_force_bit() {
        let mut dma = create_test_dma();

        // Bit 15: Force IRQ (per PSX-SPX spec)
        dma.write_interrupt(1 << 15);
        dma.update_master_flag();

        assert!(
            (dma.read_interrupt() & (1 << 31)) != 0,
            "Master flag (bit 31) should be set when force bit is set"
        );
    }

    #[test]
    fn test_dicr_master_flag_with_channel_interrupt() {
        let mut dma = create_test_dma();

        // Enable master interrupt (bit 23)
        // Enable channel 2 interrupt (bit 16+2 = 18)
        // Set channel 2 flag (bit 24+2 = 26)
        dma.write_interrupt((1 << 23) | (1 << 18));
        dma.interrupt |= 1 << 26;
        dma.update_master_flag();

        assert!(
            (dma.read_interrupt() & (1 << 31)) != 0,
            "Master flag should be set when master enable + channel enable + channel flag are all set"
        );
    }

    #[test]
    fn test_dicr_master_flag_without_enable() {
        let mut dma = create_test_dma();

        // Set channel flag but not enable bits
        dma.interrupt |= 1 << 26;
        dma.update_master_flag();

        assert_eq!(
            dma.read_interrupt() & (1 << 31),
            0,
            "Master flag should not be set without enable bits"
        );
    }

    #[test]
    fn test_dicr_reserved_bits_read_as_zero() {
        let mut dma = create_test_dma();

        // Bits 0-5 are always 0 (per PSX-SPX spec)
        dma.write_interrupt(0xFFFFFFFF);
        assert_eq!(
            dma.read_interrupt() & 0x0000003F,
            0,
            "Bits 0-5 should always read as 0"
        );
    }

    // ========== Block Control Tests ==========

    #[test]
    fn test_block_control_sync_mode_0_calculation() {
        let mut dma = create_test_dma();

        // Sync mode 0: word count in lower 16 bits (or 0x10000 if 0)
        dma.write_bcr(DMA::CH_OTC, 0x0000);
        let bcr = dma.read_bcr(DMA::CH_OTC);
        let words = if bcr & 0xFFFF == 0 {
            0x10000
        } else {
            bcr & 0xFFFF
        };
        assert_eq!(
            words, 0x10000,
            "Sync mode 0: BCR=0 should mean 0x10000 words"
        );

        dma.write_bcr(DMA::CH_OTC, 0x0100);
        let words = dma.read_bcr(DMA::CH_OTC) & 0xFFFF;
        assert_eq!(words, 0x0100, "Sync mode 0: BCR should be word count");
    }

    #[test]
    fn test_block_control_sync_mode_1_calculation() {
        let mut dma = create_test_dma();

        // Sync mode 1: block_size * block_count
        dma.write_bcr(DMA::CH_GPU, 0x00100020);
        let bcr = dma.read_bcr(DMA::CH_GPU);
        let block_size = (bcr & 0xFFFF) as usize;
        let block_count = ((bcr >> 16) & 0xFFFF) as usize;
        let total = block_size * block_count;

        assert_eq!(block_size, 0x20, "Block size should be 32 words");
        assert_eq!(block_count, 0x10, "Block count should be 16 blocks");
        assert_eq!(total, 0x200, "Total should be 512 words");
    }

    // ========== Channel Priority Tests ==========

    #[test]
    fn test_channel_priority_ordering() {
        let mut dma = create_test_dma();

        // Default: 0x07654321 means priority 7,6,5,4,3,2,1 for channels 6,5,4,3,2,1,0
        let priorities: Vec<u32> = (0..7).map(|ch| dma.channel_priority(ch)).collect();

        assert_eq!(
            priorities,
            vec![1, 2, 3, 4, 5, 6, 7],
            "Default priorities should be 1-7"
        );

        // Verify channel 6 has highest priority
        assert!(
            dma.channel_priority(6) > dma.channel_priority(0),
            "Channel 6 should have higher priority than channel 0"
        );
    }

    // ========== Transfer Mode Tests ==========

    #[test]
    fn test_transfer_direction_ram_to_device() {
        let mut dma = create_test_dma();

        // Direction bit 0: 1 = RAM→device
        dma.write_chcr(DMA::CH_GPU, 0x01000001);
        assert_eq!(
            dma.channels[DMA::CH_GPU].direction(),
            DMAChannel::TRANSFER_FROM_RAM,
            "Direction should be RAM→device"
        );
    }

    #[test]
    fn test_transfer_direction_device_to_ram() {
        let mut dma = create_test_dma();

        // Direction bit 0: 0 = device→RAM
        dma.write_chcr(DMA::CH_CDROM, 0x01000000);
        assert_eq!(
            dma.channels[DMA::CH_CDROM].direction(),
            DMAChannel::TRANSFER_TO_RAM,
            "Direction should be device→RAM"
        );
    }

    #[test]
    fn test_all_sync_modes() {
        let mut dma = create_test_dma();

        // Mode 0: Immediate/Manual
        dma.write_chcr(DMA::CH_GPU, 0x00000000);
        assert_eq!(
            dma.channels[DMA::CH_GPU].sync_mode(),
            0,
            "Sync mode should be 0 (immediate)"
        );

        // Mode 1: Block/Request
        dma.write_chcr(DMA::CH_GPU, 0x00000200);
        assert_eq!(
            dma.channels[DMA::CH_GPU].sync_mode(),
            1,
            "Sync mode should be 1 (block)"
        );

        // Mode 2: Linked-list
        dma.write_chcr(DMA::CH_GPU, 0x00000400);
        assert_eq!(
            dma.channels[DMA::CH_GPU].sync_mode(),
            2,
            "Sync mode should be 2 (linked-list)"
        );
    }

    // ========== Edge Cases and Error Conditions ==========

    #[test]
    fn test_inactive_channel_does_not_transfer() {
        let dma = create_test_dma();

        // Channel should be inactive by default
        for ch in 0..7 {
            assert!(
                !dma.channels[ch].is_active(),
                "Channel {} should be inactive",
                ch
            );
        }
    }

    #[test]
    fn test_disabled_channel_cannot_transfer() {
        let mut dma = create_test_dma();

        // Disable all channels via DPCR
        dma.write_control(0x00000000);

        // Try to activate channel 2 via CHCR
        dma.write_chcr(DMA::CH_GPU, 0x11000401);

        // Channel should be active in CHCR but disabled in DPCR
        assert!(
            dma.channels[DMA::CH_GPU].is_active(),
            "Channel should be active in CHCR"
        );
        assert!(
            !dma.is_channel_enabled(DMA::CH_GPU),
            "Channel should be disabled in DPCR"
        );
    }

    #[test]
    fn test_multiple_channels_can_be_active() {
        let mut dma = create_test_dma();

        // Activate channels 2, 3, 4
        dma.write_chcr(DMA::CH_GPU, 0x11000400);
        dma.write_chcr(DMA::CH_CDROM, 0x11000000);
        dma.write_chcr(DMA::CH_SPU, 0x11000200);

        assert!(
            dma.channels[DMA::CH_GPU].is_active(),
            "Channel 2 should be active"
        );
        assert!(
            dma.channels[DMA::CH_CDROM].is_active(),
            "Channel 3 should be active"
        );
        assert!(
            dma.channels[DMA::CH_SPU].is_active(),
            "Channel 4 should be active"
        );
    }

    #[test]
    fn test_address_masking_to_ram_bounds() {
        let mut dma = create_test_dma();

        // Address should be masked to 24 bits and aligned to word boundary
        dma.write_madr(DMA::CH_GPU, 0xFF123456);
        assert_eq!(
            dma.read_madr(DMA::CH_GPU),
            0x00123456,
            "Address should be masked to 24 bits"
        );

        // In actual transfer, address is further masked to 0x001FFFFC (per code)
        let addr = dma.read_madr(DMA::CH_GPU) & 0x001FFFFC;
        assert_eq!(
            addr, 0x00123454,
            "Transfer address should align to word and mask to 2MB"
        );
    }

    #[test]
    fn test_interrupt_flag_preservation() {
        let mut dma = create_test_dma();

        // Set multiple channel flags
        dma.interrupt = 0x07000000; // Channels 0, 1, 2 flags set

        // Writing DICR should preserve flags unless explicitly cleared
        dma.write_interrupt(0x00FF0000); // Write enable bits only

        assert_eq!(
            dma.read_interrupt() & 0x07000000,
            0x07000000,
            "Interrupt flags should be preserved unless write-1-to-clear"
        );
    }

    #[test]
    fn test_all_channels_have_unique_ids() {
        let dma = create_test_dma();

        for ch in 0..7 {
            assert_eq!(
                dma.channels[ch].channel_id, ch as u8,
                "Channel {} ID should be {}",
                ch, ch
            );
        }
    }

    #[test]
    fn test_complete_transfer_setup() {
        let mut dma = create_test_dma();

        // Complete setup for GPU DMA transfer (linked-list mode)
        dma.write_madr(DMA::CH_GPU, 0x80010000); // RAM address
        dma.write_bcr(DMA::CH_GPU, 0x00000000); // Not used in linked-list
        dma.write_chcr(DMA::CH_GPU, 0x11000401); // Active, trigger, linked-list, RAM→GPU
        dma.write_control(0x08888888); // Enable channel 2
        dma.write_interrupt((1 << 23) | (1 << 18)); // Master enable + channel 2 enable

        // Verify setup
        assert_eq!(dma.read_madr(DMA::CH_GPU), 0x00010000);
        assert!(dma.channels[DMA::CH_GPU].is_active());
        assert_eq!(dma.channels[DMA::CH_GPU].sync_mode(), 2);
        assert_eq!(dma.channels[DMA::CH_GPU].direction(), 1);
        assert!(dma.is_channel_enabled(DMA::CH_GPU));
    }
}
