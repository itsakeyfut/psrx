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

//! Memory region identification and address translation
//!
//! This module handles the PlayStation 1's memory segmentation and address translation.
//! The PSX uses MIPS memory segments with different caching behaviors.

use super::Bus;

/// Memory region identification
///
/// Used to identify which memory region an address belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// Main RAM (2MB)
    RAM,
    /// Scratchpad (1KB)
    Scratchpad,
    /// I/O ports
    IO,
    /// BIOS ROM
    BIOS,
    /// Cache Control registers
    CacheControl,
    /// Expansion regions (1, 2, 3) - typically unused in retail PSX
    Expansion,
    /// Unmapped region
    Unmapped,
}

impl Bus {
    /// Translate virtual address to physical address
    ///
    /// The PlayStation 1 uses MIPS memory segments:
    /// - KUSEG (0x00000000-0x7FFFFFFF): User space, cached
    /// - KSEG0 (0x80000000-0x9FFFFFFF): Kernel space, cached (mirrors physical memory)
    /// - KSEG1 (0xA0000000-0xBFFFFFFF): Kernel space, uncached (mirrors physical memory)
    ///
    /// # Arguments
    ///
    /// * `vaddr` - Virtual address to translate
    ///
    /// # Returns
    ///
    /// Physical address (with upper 3 bits masked off)
    pub(super) fn translate_address(&self, vaddr: u32) -> u32 {
        // Mask upper 3 bits to get physical address
        // This handles KUSEG, KSEG0, and KSEG1 all at once
        vaddr & 0x1FFF_FFFF
    }

    /// Identify memory region for an address
    ///
    /// Determines which memory region (RAM, Scratchpad, I/O, BIOS, or Unmapped)
    /// a given virtual address belongs to.
    ///
    /// # Arguments
    ///
    /// * `vaddr` - Virtual address
    ///
    /// # Returns
    ///
    /// The memory region that contains this address
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::memory::{Bus, MemoryRegion};
    ///
    /// let bus = Bus::new();
    ///
    /// assert_eq!(bus.identify_region(0x00000000), MemoryRegion::RAM);
    /// assert_eq!(bus.identify_region(0x1F800000), MemoryRegion::Scratchpad);
    /// assert_eq!(bus.identify_region(0x1F801000), MemoryRegion::IO);
    /// assert_eq!(bus.identify_region(0xBFC00000), MemoryRegion::BIOS);
    /// assert_eq!(bus.identify_region(0x1FFFFFFF), MemoryRegion::Unmapped);
    /// ```
    pub fn identify_region(&self, vaddr: u32) -> MemoryRegion {
        let paddr = self.translate_address(vaddr);

        if (Self::RAM_START..=Self::RAM_END).contains(&paddr) {
            MemoryRegion::RAM
        } else if (Self::EXP1_LOW_START..=Self::EXP1_LOW_END).contains(&paddr)
            || (Self::EXP2_START..=Self::EXP2_END).contains(&paddr)
            || (Self::EXP3_START..=Self::EXP3_END).contains(&paddr)
        {
            MemoryRegion::Expansion
        } else if (Self::SCRATCHPAD_START..=Self::SCRATCHPAD_END).contains(&paddr) {
            MemoryRegion::Scratchpad
        } else if (Self::IO_START..=Self::IO_END).contains(&paddr) {
            MemoryRegion::IO
        } else if (Self::BIOS_START..=Self::BIOS_END).contains(&paddr) {
            MemoryRegion::BIOS
        } else if paddr == Self::CACHE_CONTROL {
            MemoryRegion::CacheControl
        } else {
            MemoryRegion::Unmapped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_address_kuseg() {
        let bus = Bus::new();

        // KUSEG (0x00000000-0x7FFFFFFF): user space, cached
        // Should map to physical addresses (0x00000000-0x1FFFFFFF)
        assert_eq!(bus.translate_address(0x00000000), 0x00000000);
        assert_eq!(bus.translate_address(0x00000100), 0x00000100);
        assert_eq!(bus.translate_address(0x001FFFFF), 0x001FFFFF);
        assert_eq!(bus.translate_address(0x1FC00000), 0x1FC00000);
        assert_eq!(bus.translate_address(0x7FFFFFFF), 0x1FFFFFFF);
    }

    #[test]
    fn test_translate_address_kseg0() {
        let bus = Bus::new();

        // KSEG0 (0x80000000-0x9FFFFFFF): kernel space, cached
        // Should mirror physical memory (mask upper 3 bits)
        assert_eq!(bus.translate_address(0x80000000), 0x00000000);
        assert_eq!(bus.translate_address(0x80000100), 0x00000100);
        assert_eq!(bus.translate_address(0x801FFFFF), 0x001FFFFF);
        assert_eq!(bus.translate_address(0x9FC00000), 0x1FC00000);
        assert_eq!(bus.translate_address(0x9FFFFFFF), 0x1FFFFFFF);
    }

    #[test]
    fn test_translate_address_kseg1() {
        let bus = Bus::new();

        // KSEG1 (0xA0000000-0xBFFFFFFF): kernel space, uncached
        // Should mirror physical memory (mask upper 3 bits)
        assert_eq!(bus.translate_address(0xA0000000), 0x00000000);
        assert_eq!(bus.translate_address(0xA0000100), 0x00000100);
        assert_eq!(bus.translate_address(0xA01FFFFF), 0x001FFFFF);
        assert_eq!(bus.translate_address(0xBFC00000), 0x1FC00000);
        assert_eq!(bus.translate_address(0xBFFFFFFF), 0x1FFFFFFF);
    }

    #[test]
    fn test_translate_address_kseg2() {
        let bus = Bus::new();

        // KSEG2 (0xC0000000-0xFFFFFFFF): kernel-only region
        // Contains CPU control registers, including cache control
        assert_eq!(bus.translate_address(0xFFFE0130), 0x1FFE0130);
        assert_eq!(bus.translate_address(0xC0000000), 0x00000000);
        assert_eq!(bus.translate_address(0xFFFFFFFF), 0x1FFFFFFF);
    }

    #[test]
    fn test_translate_address_mirrors_same_physical() {
        let bus = Bus::new();

        // All three segments should map to same physical address
        let phys_addr = 0x00010000;
        assert_eq!(bus.translate_address(phys_addr), phys_addr);
        assert_eq!(bus.translate_address(0x80000000 | phys_addr), phys_addr);
        assert_eq!(bus.translate_address(0xA0000000 | phys_addr), phys_addr);
    }

    #[test]
    fn test_identify_region_ram() {
        let bus = Bus::new();

        // RAM: 0x00000000-0x001FFFFF (2MB)
        assert_eq!(bus.identify_region(0x00000000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x00100000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x001FFFFF), MemoryRegion::RAM);

        // Test with KSEG0
        assert_eq!(bus.identify_region(0x80000000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x80100000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x801FFFFF), MemoryRegion::RAM);

        // Test with KSEG1
        assert_eq!(bus.identify_region(0xA0000000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0xA0100000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0xA01FFFFF), MemoryRegion::RAM);
    }

    #[test]
    fn test_identify_region_ram_boundaries() {
        let bus = Bus::new();

        // Test exact boundaries
        assert_eq!(bus.identify_region(0x00000000), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x001FFFFF), MemoryRegion::RAM);

        // Just past RAM end should be Expansion
        assert_eq!(bus.identify_region(0x00200000), MemoryRegion::Expansion);
    }

    #[test]
    fn test_identify_region_scratchpad() {
        let bus = Bus::new();

        // Scratchpad: 0x1F800000-0x1F800FFF (4KB addressable, 1KB physical)
        assert_eq!(bus.identify_region(0x1F800000), MemoryRegion::Scratchpad);
        assert_eq!(bus.identify_region(0x1F8003FF), MemoryRegion::Scratchpad);
        assert_eq!(bus.identify_region(0x1F800400), MemoryRegion::Scratchpad);
        assert_eq!(bus.identify_region(0x1F800FFF), MemoryRegion::Scratchpad);

        // Test with KSEG0
        assert_eq!(bus.identify_region(0x9F800000), MemoryRegion::Scratchpad);

        // Test with KSEG1
        assert_eq!(bus.identify_region(0xBF800000), MemoryRegion::Scratchpad);
    }

    #[test]
    fn test_identify_region_io() {
        let bus = Bus::new();

        // I/O Ports: 0x1F801000-0x1F9FFFFF
        assert_eq!(bus.identify_region(0x1F801000), MemoryRegion::IO);
        assert_eq!(bus.identify_region(0x1F801070), MemoryRegion::IO); // I_STAT
        assert_eq!(bus.identify_region(0x1F801810), MemoryRegion::IO); // GPU GP0
        assert_eq!(bus.identify_region(0x1F801814), MemoryRegion::IO); // GPU GP1
        assert_eq!(bus.identify_region(0x1F802000), MemoryRegion::IO);
        assert_eq!(bus.identify_region(0x1F9FFFFF), MemoryRegion::IO);

        // Test with KSEG0
        assert_eq!(bus.identify_region(0x9F801000), MemoryRegion::IO);

        // Test with KSEG1
        assert_eq!(bus.identify_region(0xBF801000), MemoryRegion::IO);
    }

    #[test]
    fn test_identify_region_bios() {
        let bus = Bus::new();

        // BIOS: 0x1FC00000-0x1FC7FFFF (512KB)
        assert_eq!(bus.identify_region(0x1FC00000), MemoryRegion::BIOS);
        assert_eq!(bus.identify_region(0x1FC40000), MemoryRegion::BIOS);
        assert_eq!(bus.identify_region(0x1FC7FFFF), MemoryRegion::BIOS);

        // Test with KSEG0
        assert_eq!(bus.identify_region(0x9FC00000), MemoryRegion::BIOS);

        // Test with KSEG1 (typical BIOS execution address)
        assert_eq!(bus.identify_region(0xBFC00000), MemoryRegion::BIOS);
        assert_eq!(bus.identify_region(0xBFC7FFFF), MemoryRegion::BIOS);
    }

    #[test]
    fn test_identify_region_cache_control() {
        let bus = Bus::new();

        // Cache Control: 0xFFFE0130
        assert_eq!(bus.identify_region(0xFFFE0130), MemoryRegion::CacheControl);
    }

    #[test]
    fn test_identify_region_expansion() {
        let bus = Bus::new();

        // Expansion Region 1 (lower): 0x00200000-0x1EFFFFFF
        assert_eq!(bus.identify_region(0x00200000), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1EFFFFFF), MemoryRegion::Expansion);

        // Expansion Region 2: 0x1F000000-0x1F7FFFFF
        assert_eq!(bus.identify_region(0x1F000000), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1F000100), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1F7FFFFF), MemoryRegion::Expansion);

        // Expansion Region 3: 0x1FA00000-0x1FBFFFFF
        assert_eq!(bus.identify_region(0x1FA00000), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1FBFFFFF), MemoryRegion::Expansion);

        // Test with KSEG0
        assert_eq!(bus.identify_region(0x80200000), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x9F000000), MemoryRegion::Expansion);

        // Test with KSEG1
        assert_eq!(bus.identify_region(0xA0200000), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0xBF000000), MemoryRegion::Expansion);
    }

    #[test]
    fn test_identify_region_unmapped() {
        let bus = Bus::new();

        // Between BIOS and Cache Control (physical address space)
        // BIOS ends at 0x1FC7FFFF, Cache Control is at 0x1FFE0130
        // The region 0x1FC80000-0x1FFE012F is unmapped (except Cache Control itself)
        assert_eq!(bus.identify_region(0x1FC80000), MemoryRegion::Unmapped);
        assert_eq!(bus.identify_region(0x1FD00000), MemoryRegion::Unmapped);
        assert_eq!(bus.identify_region(0x1FE00000), MemoryRegion::Unmapped);

        // Test with KSEG0/KSEG1 mirrors
        assert_eq!(bus.identify_region(0x9FC80000), MemoryRegion::Unmapped);
        assert_eq!(bus.identify_region(0xBFD00000), MemoryRegion::Unmapped);
    }

    #[test]
    fn test_region_consistency_across_segments() {
        let bus = Bus::new();

        // Test that same physical address is identified as same region across segments
        let test_addresses = vec![
            (0x00000100, MemoryRegion::RAM),
            (0x1F800100, MemoryRegion::Scratchpad),
            (0x1F801810, MemoryRegion::IO),
            (0x1FC00100, MemoryRegion::BIOS),
        ];

        for (phys_addr, expected_region) in test_addresses {
            // KUSEG
            assert_eq!(bus.identify_region(phys_addr), expected_region);

            // KSEG0
            assert_eq!(bus.identify_region(0x80000000 | phys_addr), expected_region);

            // KSEG1
            assert_eq!(bus.identify_region(0xA0000000 | phys_addr), expected_region);
        }
    }

    #[test]
    fn test_edge_case_addresses() {
        let bus = Bus::new();

        // Test address 0x00000000 (start of RAM)
        assert_eq!(bus.translate_address(0x00000000), 0x00000000);
        assert_eq!(bus.identify_region(0x00000000), MemoryRegion::RAM);

        // Test address 0xFFFFFFFF (end of address space)
        assert_eq!(bus.translate_address(0xFFFFFFFF), 0x1FFFFFFF);
        assert_eq!(bus.identify_region(0xFFFFFFFF), MemoryRegion::Unmapped);

        // Test boundary between regions
        assert_eq!(bus.identify_region(0x001FFFFF), MemoryRegion::RAM);
        assert_eq!(bus.identify_region(0x00200000), MemoryRegion::Expansion);

        assert_eq!(bus.identify_region(0x1F7FFFFF), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1F800000), MemoryRegion::Scratchpad);

        assert_eq!(bus.identify_region(0x1F800FFF), MemoryRegion::Scratchpad);
        assert_eq!(bus.identify_region(0x1F801000), MemoryRegion::IO);

        assert_eq!(bus.identify_region(0x1F9FFFFF), MemoryRegion::IO);
        assert_eq!(bus.identify_region(0x1FA00000), MemoryRegion::Expansion);

        assert_eq!(bus.identify_region(0x1FBFFFFF), MemoryRegion::Expansion);
        assert_eq!(bus.identify_region(0x1FC00000), MemoryRegion::BIOS);

        assert_eq!(bus.identify_region(0x1FC7FFFF), MemoryRegion::BIOS);
        assert_eq!(bus.identify_region(0x1FC80000), MemoryRegion::Unmapped);
    }

    #[test]
    fn test_bios_boot_address() {
        let bus = Bus::new();

        // PSX boots from 0xBFC00000 (KSEG1, uncached BIOS)
        assert_eq!(bus.translate_address(0xBFC00000), 0x1FC00000);
        assert_eq!(bus.identify_region(0xBFC00000), MemoryRegion::BIOS);
    }

    #[test]
    fn test_typical_program_addresses() {
        let bus = Bus::new();

        // Typical PSX program load addresses
        // Programs often load to 0x80010000 (KSEG0, cached)
        assert_eq!(bus.translate_address(0x80010000), 0x00010000);
        assert_eq!(bus.identify_region(0x80010000), MemoryRegion::RAM);

        // Stack pointer often starts around 0x801FFF00
        assert_eq!(bus.translate_address(0x801FFF00), 0x001FFF00);
        assert_eq!(bus.identify_region(0x801FFF00), MemoryRegion::RAM);
    }

    #[test]
    fn test_io_port_specific_addresses() {
        let bus = Bus::new();

        // Test specific I/O port addresses used by peripherals
        let io_addresses = vec![
            0x1F801040, // JOY_DATA
            0x1F801044, // JOY_STAT
            0x1F801070, // I_STAT
            0x1F801074, // I_MASK
            0x1F801100, // TIMER0_COUNTER
            0x1F801810, // GPU_GP0
            0x1F801814, // GPU_GP1
            0x1F801800, // CDROM_INDEX
            0x1F8010F0, // DMA_DPCR
            0x1F801C00, // SPU registers start
        ];

        for addr in io_addresses {
            assert_eq!(
                bus.identify_region(addr),
                MemoryRegion::IO,
                "Address 0x{:08X} should be I/O region",
                addr
            );

            // Also test with KSEG1 (typical for I/O access)
            assert_eq!(
                bus.identify_region(0xA0000000 | addr),
                MemoryRegion::IO,
                "Address 0x{:08X} (KSEG1) should be I/O region",
                addr
            );
        }
    }

    #[test]
    fn test_scratchpad_mirroring() {
        let bus = Bus::new();

        // Scratchpad is 1KB (0x400 bytes) but addressable as 4KB (0x1000 bytes)
        // Addresses 0x400-0xFFF should still be identified as Scratchpad
        for offset in [0x000, 0x100, 0x3FF, 0x400, 0x7FF, 0xFFF] {
            let addr = 0x1F800000 + offset;
            assert_eq!(
                bus.identify_region(addr),
                MemoryRegion::Scratchpad,
                "Address 0x{:08X} should be Scratchpad",
                addr
            );
        }
    }
}
