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

//! Instruction cache management for memory bus
//!
//! This module handles the ICache prefill and invalidation queues used to maintain
//! cache coherency between the memory bus and the CPU's instruction cache.
//!
//! # ICache Prefill
//!
//! When the BIOS copies code from ROM to RAM (e.g., 0xBFC10000 -> 0xA0000500),
//! we track these writes and queue them for prefilling the CPU's instruction cache.
//! This ensures instructions are cached before RAM is zeroed by BIOS initialization.
//!
//! # ICache Invalidation
//!
//! When memory is written that may contain already-cached instructions
//! (e.g., self-modifying code, runtime patching), we queue the addresses
//! for cache invalidation to maintain coherency.

use super::Bus;

impl Bus {
    /// Drain the icache prefill queue
    ///
    /// Returns all queued (address, instruction) pairs and clears the queue.
    /// This should be called periodically by the System to apply prefills to
    /// the CPU's instruction cache.
    pub fn drain_icache_prefill_queue(&mut self) -> Vec<(u32, u32)> {
        self.icache_prefill_queue.drain(..).collect()
    }

    /// Drain the icache invalidation queue
    ///
    /// Returns all queued addresses for invalidation and clears the queue.
    /// This should be called periodically by the System to invalidate stale
    /// cache entries when memory is modified.
    pub fn drain_icache_invalidate_queue(&mut self) -> Vec<u32> {
        self.icache_invalidate_queue.drain(..).collect()
    }

    /// Drain the icache range invalidation queue
    ///
    /// Returns all queued (start, end) address ranges for invalidation and clears the queue.
    /// This should be called periodically by the System to invalidate ranges of stale
    /// cache entries (e.g., when loading executables).
    pub fn drain_icache_invalidate_range_queue(&mut self) -> Vec<(u32, u32)> {
        self.icache_invalidate_range_queue.drain(..).collect()
    }

    /// Queue an instruction for ICache prefill
    ///
    /// Called when BIOS copies code to RAM. Queues both cached and uncached
    /// address aliases for prefilling.
    ///
    /// # Arguments
    ///
    /// * `paddr` - Physical address where instruction was written
    /// * `instruction` - The instruction word that was written
    pub(super) fn queue_icache_prefill(&mut self, paddr: u32, instruction: u32) {
        let offset = paddr as usize;

        // Only prefill for code in the low memory region
        if (Self::ICACHE_PREFILL_START..=Self::ICACHE_PREFILL_END).contains(&offset) {
            // Queue for cached addresses (KSEG0: 0x80000000-0x9FFFFFFF)
            let cached_addr = 0x80000000 | paddr;
            self.icache_prefill_queue.push((cached_addr, instruction));

            // And for uncached addresses (KUSEG: 0x00000000-0x7FFFFFFF)
            self.icache_prefill_queue.push((paddr, instruction));
        }
    }

    /// Queue an address for ICache invalidation
    ///
    /// Called when RAM is written. Queues both cached and uncached
    /// address aliases for invalidation to maintain cache coherency.
    ///
    /// # Arguments
    ///
    /// * `paddr` - Physical address that was written
    pub(super) fn queue_icache_invalidation(&mut self, paddr: u32) {
        // Queue for icache invalidation (all RAM writes)
        // This maintains cache coherency for self-modifying code,
        // runtime patching, and DMA writes to instruction memory
        let cached_addr = 0x80000000 | paddr;
        self.icache_invalidate_queue.push(cached_addr);
        self.icache_invalidate_queue.push(paddr); // Also uncached alias
    }

    /// Queue an address range for ICache invalidation
    ///
    /// Called when bulk data is written to RAM (e.g., executable loading).
    /// Queues both cached and uncached address aliases for the entire range.
    ///
    /// # Arguments
    ///
    /// * `start_paddr` - Physical start address of the written range
    /// * `end_paddr` - Physical end address (exclusive) of the written range
    pub(super) fn queue_icache_range_invalidation(&mut self, start_paddr: u32, end_paddr: u32) {
        // Queue cached addresses (KSEG0: 0x80000000-0x9FFFFFFF)
        let cached_start = 0x80000000 | start_paddr;
        let cached_end = 0x80000000 | end_paddr;
        self.icache_invalidate_range_queue
            .push((cached_start, cached_end));

        // Also queue uncached aliases (KUSEG: 0x00000000-0x7FFFFFFF)
        self.icache_invalidate_range_queue
            .push((start_paddr, end_paddr));
    }

    /// Get mutable reference to RAM
    ///
    /// Provides direct mutable access to the main system RAM (2MB).
    /// This is primarily used by DMA transfers for efficient bulk data operations.
    ///
    /// # Returns
    ///
    /// Mutable slice to the 2MB RAM buffer
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::memory::Bus;
    ///
    /// let mut bus = Bus::new();
    /// let ram = bus.ram_mut();
    /// ram[0] = 0x42;
    /// ```
    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icache_prefill_queue_basic() {
        let mut bus = Bus::new();

        // Initially empty
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);

        // Queue an instruction in the prefill range
        bus.queue_icache_prefill(0x00000500, 0x12345678);

        // Should have queued both cached and uncached aliases
        let queue = bus.drain_icache_prefill_queue();
        assert_eq!(queue.len(), 2);

        // Check that both aliases are present
        assert!(queue.contains(&(0x80000500, 0x12345678))); // KSEG0 cached
        assert!(queue.contains(&(0x00000500, 0x12345678))); // KUSEG
    }

    #[test]
    fn test_icache_prefill_queue_within_range() {
        let mut bus = Bus::new();

        // Queue instructions at start of range
        bus.queue_icache_prefill(0x00000000, 0x11111111);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);

        // Queue instructions in middle of range
        bus.queue_icache_prefill(0x00008000, 0x22222222);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);

        // Queue instructions at end of range
        bus.queue_icache_prefill(0x0000FFFC, 0x33333333);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);
    }

    #[test]
    fn test_icache_prefill_queue_outside_range() {
        let mut bus = Bus::new();

        // Queue instruction outside prefill range (> 0x10000)
        // Note: ICACHE_PREFILL_END is 0x10000, and the range is ..= (inclusive)
        // so 0x10000 is still within range, but 0x10001 is outside
        bus.queue_icache_prefill(0x00010001, 0x12345678);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);

        // Queue instruction well outside range
        bus.queue_icache_prefill(0x00100000, 0xABCDEF00);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
    }

    #[test]
    fn test_icache_prefill_queue_boundary() {
        let mut bus = Bus::new();

        // Test boundary at 0x10000 (inclusive, because range is ..=)
        bus.queue_icache_prefill(0x0000FFFC, 0x11111111); // Just before end
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);

        bus.queue_icache_prefill(0x00010000, 0x22222222); // At boundary (inclusive, still inside)
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);

        bus.queue_icache_prefill(0x00010001, 0x33333333); // Past boundary (outside)
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
    }

    #[test]
    fn test_icache_prefill_queue_multiple_instructions() {
        let mut bus = Bus::new();

        // Queue multiple instructions
        bus.queue_icache_prefill(0x00000100, 0x11111111);
        bus.queue_icache_prefill(0x00000200, 0x22222222);
        bus.queue_icache_prefill(0x00000300, 0x33333333);

        // Should have 6 entries (2 aliases per instruction)
        let queue = bus.drain_icache_prefill_queue();
        assert_eq!(queue.len(), 6);

        // After draining, queue should be empty
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
    }

    #[test]
    fn test_icache_prefill_queue_drain_clears() {
        let mut bus = Bus::new();

        bus.queue_icache_prefill(0x00000100, 0x12345678);
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);

        // Second drain should be empty
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
    }

    #[test]
    fn test_icache_invalidation_queue_basic() {
        let mut bus = Bus::new();

        // Initially empty
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 0);

        // Queue an address for invalidation
        bus.queue_icache_invalidation(0x00000500);

        // Should have queued both cached and uncached aliases
        let queue = bus.drain_icache_invalidate_queue();
        assert_eq!(queue.len(), 2);

        // Check that both aliases are present
        assert!(queue.contains(&0x80000500)); // KSEG0 cached
        assert!(queue.contains(&0x00000500)); // KUSEG
    }

    #[test]
    fn test_icache_invalidation_queue_multiple_addresses() {
        let mut bus = Bus::new();

        // Queue multiple addresses
        bus.queue_icache_invalidation(0x00000100);
        bus.queue_icache_invalidation(0x00000200);
        bus.queue_icache_invalidation(0x00000300);

        // Should have 6 entries (2 aliases per address)
        let queue = bus.drain_icache_invalidate_queue();
        assert_eq!(queue.len(), 6);

        // Verify some addresses
        assert!(queue.contains(&0x80000100));
        assert!(queue.contains(&0x00000100));
        assert!(queue.contains(&0x80000200));
        assert!(queue.contains(&0x00000200));
    }

    #[test]
    fn test_icache_invalidation_queue_drain_clears() {
        let mut bus = Bus::new();

        bus.queue_icache_invalidation(0x00000100);
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 2);

        // Second drain should be empty
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 0);
    }

    #[test]
    fn test_icache_invalidation_queue_all_ram_addresses() {
        let mut bus = Bus::new();

        // Test at various RAM addresses
        bus.queue_icache_invalidation(0x00000000); // Start of RAM
        bus.queue_icache_invalidation(0x00100000); // Middle of RAM
        bus.queue_icache_invalidation(0x001FFFFC); // Near end of RAM

        let queue = bus.drain_icache_invalidate_queue();
        assert_eq!(queue.len(), 6);
    }

    #[test]
    fn test_icache_range_invalidation_basic() {
        let mut bus = Bus::new();

        // Initially empty
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 0);

        // Queue a range for invalidation
        bus.queue_icache_range_invalidation(0x00000500, 0x00000600);

        // Should have queued both cached and uncached aliases
        let queue = bus.drain_icache_invalidate_range_queue();
        assert_eq!(queue.len(), 2);

        // Check that both range aliases are present
        assert!(queue.contains(&(0x80000500, 0x80000600))); // KSEG0 cached
        assert!(queue.contains(&(0x00000500, 0x00000600))); // KUSEG
    }

    #[test]
    fn test_icache_range_invalidation_multiple_ranges() {
        let mut bus = Bus::new();

        // Queue multiple ranges
        bus.queue_icache_range_invalidation(0x00000100, 0x00000200);
        bus.queue_icache_range_invalidation(0x00010000, 0x00020000);
        bus.queue_icache_range_invalidation(0x00100000, 0x00110000);

        // Should have 6 entries (2 aliases per range)
        let queue = bus.drain_icache_invalidate_range_queue();
        assert_eq!(queue.len(), 6);
    }

    #[test]
    fn test_icache_range_invalidation_drain_clears() {
        let mut bus = Bus::new();

        bus.queue_icache_range_invalidation(0x00000100, 0x00000200);
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 2);

        // Second drain should be empty
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 0);
    }

    #[test]
    fn test_icache_range_invalidation_large_range() {
        let mut bus = Bus::new();

        // Invalidate a large range (e.g., executable load)
        bus.queue_icache_range_invalidation(0x00010000, 0x00080000);

        let queue = bus.drain_icache_invalidate_range_queue();
        assert_eq!(queue.len(), 2);

        // Verify ranges
        assert!(queue.contains(&(0x80010000, 0x80080000)));
        assert!(queue.contains(&(0x00010000, 0x00080000)));
    }

    #[test]
    fn test_icache_range_invalidation_single_instruction() {
        let mut bus = Bus::new();

        // Invalidate a single instruction (4 bytes)
        bus.queue_icache_range_invalidation(0x00000100, 0x00000104);

        let queue = bus.drain_icache_invalidate_range_queue();
        assert_eq!(queue.len(), 2);

        assert!(queue.contains(&(0x80000100, 0x80000104)));
        assert!(queue.contains(&(0x00000100, 0x00000104)));
    }

    #[test]
    fn test_icache_range_invalidation_zero_length() {
        let mut bus = Bus::new();

        // Invalidate zero-length range (start == end)
        bus.queue_icache_range_invalidation(0x00000100, 0x00000100);

        let queue = bus.drain_icache_invalidate_range_queue();
        assert_eq!(queue.len(), 2);

        // Even zero-length ranges should be queued
        assert!(queue.contains(&(0x80000100, 0x80000100)));
        assert!(queue.contains(&(0x00000100, 0x00000100)));
    }

    #[test]
    fn test_icache_all_queues_independent() {
        let mut bus = Bus::new();

        // Queue to all three queues
        bus.queue_icache_prefill(0x00000100, 0x11111111);
        bus.queue_icache_invalidation(0x00000200);
        bus.queue_icache_range_invalidation(0x00000300, 0x00000400);

        // Each queue should have its own entries
        assert_eq!(bus.drain_icache_prefill_queue().len(), 2);
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 2);
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 2);

        // All queues should now be empty
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 0);
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 0);
    }

    #[test]
    fn test_icache_queues_after_reset() {
        let mut bus = Bus::new();

        // Queue some entries
        bus.queue_icache_prefill(0x00000100, 0x11111111);
        bus.queue_icache_invalidation(0x00000200);
        bus.queue_icache_range_invalidation(0x00000300, 0x00000400);

        // Reset should clear all queues
        bus.reset();

        // All queues should be empty after reset
        assert_eq!(bus.drain_icache_prefill_queue().len(), 0);
        assert_eq!(bus.drain_icache_invalidate_queue().len(), 0);
        assert_eq!(bus.drain_icache_invalidate_range_queue().len(), 0);
    }

    #[test]
    fn test_ram_mut_basic() {
        let mut bus = Bus::new();

        // Get mutable reference to RAM
        let ram = bus.ram_mut();

        // Write some data
        ram[0] = 0x42;
        ram[1] = 0x43;
        ram[100] = 0xFF;

        // Verify data was written
        assert_eq!(ram[0], 0x42);
        assert_eq!(ram[1], 0x43);
        assert_eq!(ram[100], 0xFF);
    }

    #[test]
    fn test_ram_mut_size() {
        let mut bus = Bus::new();

        // RAM should be 2MB
        let ram = bus.ram_mut();
        assert_eq!(ram.len(), 2 * 1024 * 1024);
    }

    #[test]
    fn test_ram_mut_full_range() {
        let mut bus = Bus::new();

        let ram = bus.ram_mut();

        // Write to start and end of RAM
        ram[0] = 0x11;
        ram[ram.len() - 1] = 0x22;

        assert_eq!(ram[0], 0x11);
        assert_eq!(ram[ram.len() - 1], 0x22);
    }

    #[test]
    fn test_ram_mut_bulk_write() {
        let mut bus = Bus::new();

        let ram = bus.ram_mut();

        // Write a pattern
        for i in 0..100 {
            ram[i] = (i % 256) as u8;
        }

        // Verify pattern
        for i in 0..100 {
            assert_eq!(ram[i], (i % 256) as u8);
        }
    }

    #[test]
    fn test_icache_prefill_with_instruction_values() {
        let mut bus = Bus::new();

        // Queue various MIPS instructions
        bus.queue_icache_prefill(0x00000000, 0x00000000); // NOP
        bus.queue_icache_prefill(0x00000004, 0x3C080001); // LUI $t0, 0x0001
        bus.queue_icache_prefill(0x00000008, 0x8D090000); // LW $t1, 0($t0)
        bus.queue_icache_prefill(0x0000000C, 0xAD0A0004); // SW $t2, 4($t0)

        let queue = bus.drain_icache_prefill_queue();

        // 4 instructions × 2 aliases = 8 entries
        assert_eq!(queue.len(), 8);

        // Verify specific entries
        assert!(queue.contains(&(0x80000000, 0x00000000)));
        assert!(queue.contains(&(0x00000000, 0x00000000)));
        assert!(queue.contains(&(0x80000004, 0x3C080001)));
        assert!(queue.contains(&(0x00000004, 0x3C080001)));
    }

    #[test]
    fn test_icache_edge_case_addresses() {
        let mut bus = Bus::new();

        // Test at 4-byte boundaries (instruction alignment)
        for addr in [0x0000, 0x0004, 0x0008, 0x000C, 0x0010] {
            bus.queue_icache_prefill(addr, 0xDEADBEEF);
        }

        let queue = bus.drain_icache_prefill_queue();
        assert_eq!(queue.len(), 10); // 5 instructions × 2 aliases
    }

    #[test]
    fn test_icache_unaligned_addresses() {
        let mut bus = Bus::new();

        // PSX requires 4-byte aligned instruction fetches, but the cache
        // system should handle any address for generality
        bus.queue_icache_prefill(0x00000001, 0x12345678); // Unaligned
        bus.queue_icache_prefill(0x00000003, 0xABCDEF00); // Unaligned

        let queue = bus.drain_icache_prefill_queue();
        assert_eq!(queue.len(), 4); // 2 instructions × 2 aliases
    }
}
