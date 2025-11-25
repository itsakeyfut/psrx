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

//! Instruction cache for MIPS R3000A CPU
//!
//! This module implements a direct-mapped instruction cache that mimics
//! the behavior of the PSX's real hardware I-cache.
//!
//! # Hardware Specifications
//!
//! The PSX CPU (MIPS R3000A) has a 4KB instruction cache with the following characteristics:
//! - **Size**: 4KB (1024 cache lines × 4 bytes per line)
//! - **Organization**: Direct-mapped
//! - **Line size**: 4 bytes (1 instruction per line)
//! - **Indexing**: Lower 12 bits (bits [11:2]) select the cache line
//! - **Tag**: Upper 20 bits (bits [31:12]) identify the cached address
//!
//! # Design Rationale
//!
//! A direct-mapped cache was chosen for:
//! 1. **Hardware accuracy**: Matches the real PSX I-cache behavior
//! 2. **Performance**: O(1) lookup with no search required
//! 3. **Spatial locality**: Leverages sequential instruction execution patterns
//! 4. **Predictability**: Deterministic eviction policy (no LRU overhead)
//!
//! # Example
//!
//! ```
//! use psrx::core::cpu::icache::InstructionCache;
//!
//! let mut cache = InstructionCache::new();
//!
//! // Store instruction
//! cache.store(0x80010000, 0x3C080000); // lui r8, 0x0000
//!
//! // Fetch instruction (cache hit)
//! assert_eq!(cache.fetch(0x80010000), Some(0x3C080000));
//!
//! // Invalidate entry
//! cache.invalidate(0x80010000);
//! assert_eq!(cache.fetch(0x80010000), None);
//! ```

/// A single cache line in the instruction cache
///
/// Each cache line stores:
/// - **tag**: Upper 20 bits of the address (bits [31:12])
/// - **data**: The 32-bit instruction word
/// - **valid**: Whether this cache line contains valid data
#[derive(Debug, Clone, Copy)]
struct CacheLine {
    /// Address tag (upper 20 bits)
    tag: u32,
    /// Cached instruction word
    data: u32,
    /// Valid bit
    valid: bool,
}

impl CacheLine {
    /// Create a new invalid cache line
    #[inline(always)]
    const fn new() -> Self {
        Self {
            tag: 0,
            data: 0,
            valid: false,
        }
    }
}

/// Direct-mapped instruction cache for MIPS R3000A
///
/// Implements a 4KB instruction cache with 1024 cache lines,
/// matching the PSX hardware specifications.
///
/// # Cache Organization
///
/// ```text
/// Address format (32 bits):
/// [31:12] Tag (20 bits) - Identifies which address is cached
/// [11:2]  Index (10 bits) - Selects cache line (0-1023)
/// [1:0]   Byte offset (always 00 for word-aligned instructions)
/// ```
///
/// # Performance Characteristics
///
/// - **Lookup**: O(1) - Direct indexing, no search required
/// - **Store**: O(1) - Direct replacement
/// - **Invalidate**: O(1) - Single entry
/// - **Invalidate range**: O(n) - Linear scan of affected lines
/// - **Clear**: O(1) - Bulk reset
///
/// # Memory Usage
///
/// - 1024 cache lines × 12 bytes per line = 12KB total
/// - Each line contains: tag (4 bytes) + data (4 bytes) + valid (1 byte) + padding (3 bytes)
pub struct InstructionCache {
    /// Cache lines (1024 entries for 4KB cache)
    lines: Vec<CacheLine>,
}

impl InstructionCache {
    /// Number of cache lines (4KB / 4 bytes per instruction)
    const LINE_COUNT: usize = 1024;

    /// Bit mask for extracting the cache line index (bits [11:2])
    const INDEX_MASK: u32 = 0x3FF; // 10 bits for 1024 lines

    /// Bit shift for extracting the cache line index from address
    const INDEX_SHIFT: u32 = 2;

    /// Bit shift for extracting the tag from address
    const TAG_SHIFT: u32 = 12;

    /// Create a new instruction cache
    ///
    /// Allocates 1024 cache lines, all initially invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let cache = InstructionCache::new();
    /// assert_eq!(cache.len(), 0); // No valid entries
    /// ```
    pub fn new() -> Self {
        Self {
            lines: vec![CacheLine::new(); Self::LINE_COUNT],
        }
    }

    /// Extract cache line index from address
    ///
    /// Takes bits [11:2] of the address to select one of 1024 cache lines.
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address (should be word-aligned)
    ///
    /// # Returns
    ///
    /// Cache line index (0-1023)
    #[inline(always)]
    fn index(&self, addr: u32) -> usize {
        ((addr >> Self::INDEX_SHIFT) & Self::INDEX_MASK) as usize
    }

    /// Extract tag from address
    ///
    /// Takes bits [31:12] of the address for tag comparison.
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address
    ///
    /// # Returns
    ///
    /// Tag value (upper 20 bits)
    #[inline(always)]
    fn tag(&self, addr: u32) -> u32 {
        addr >> Self::TAG_SHIFT
    }

    /// Fetch instruction from cache
    ///
    /// Performs a cache lookup using direct-mapped addressing.
    /// Returns the cached instruction if:
    /// - The cache line is valid
    /// - The tag matches
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address to fetch
    ///
    /// # Returns
    ///
    /// - `Some(instruction)` if cache hit
    /// - `None` if cache miss
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    ///
    /// // Cache miss
    /// assert_eq!(cache.fetch(0x80000000), None);
    ///
    /// // Store and fetch
    /// cache.store(0x80000000, 0x00000000); // nop
    /// assert_eq!(cache.fetch(0x80000000), Some(0x00000000));
    /// ```
    #[inline(always)]
    pub fn fetch(&self, addr: u32) -> Option<u32> {
        let index = self.index(addr);
        let tag = self.tag(addr);

        let line = &self.lines[index];
        if line.valid && line.tag == tag {
            Some(line.data)
        } else {
            None
        }
    }

    /// Store instruction in cache
    ///
    /// Stores an instruction in the cache using direct-mapped addressing.
    /// If another instruction already occupies this cache line, it is evicted.
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address
    /// * `instruction` - 32-bit instruction word
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// cache.store(0x80000000, 0x3C080000); // lui r8, 0x0000
    ///
    /// // Storing to same index with different tag evicts previous entry
    /// cache.store(0x80001000, 0x24080001); // addiu r8, r0, 1
    /// assert_eq!(cache.fetch(0x80000000), None); // Evicted
    /// assert_eq!(cache.fetch(0x80001000), Some(0x24080001)); // New entry
    /// ```
    #[inline(always)]
    pub fn store(&mut self, addr: u32, instruction: u32) {
        let index = self.index(addr);
        let tag = self.tag(addr);

        self.lines[index] = CacheLine {
            tag,
            data: instruction,
            valid: true,
        };
    }

    /// Invalidate cached instruction at given address
    ///
    /// Marks the cache line as invalid. This is essential for cache coherency
    /// when memory is modified after caching (self-modifying code, DMA, etc.).
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address to invalidate
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// cache.store(0x80000000, 0x00000000);
    ///
    /// cache.invalidate(0x80000000);
    /// assert_eq!(cache.fetch(0x80000000), None);
    /// ```
    #[inline(always)]
    pub fn invalidate(&mut self, addr: u32) {
        let index = self.index(addr);
        let tag = self.tag(addr);

        let line = &mut self.lines[index];
        if line.valid && line.tag == tag {
            line.valid = false;
        }
    }

    /// Invalidate cached instructions in given address range
    ///
    /// More efficient than individual invalidations when a large memory
    /// region is modified (e.g., DMA transfer, memset operations).
    ///
    /// # Arguments
    ///
    /// * `start` - Start address (inclusive)
    /// * `end` - End address (inclusive)
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// cache.store(0x80000000, 0x00000000);
    /// cache.store(0x80000004, 0x00000000);
    /// cache.store(0x80000008, 0x00000000);
    ///
    /// // Invalidate first two instructions
    /// cache.invalidate_range(0x80000000, 0x80000004);
    ///
    /// assert_eq!(cache.fetch(0x80000000), None);
    /// assert_eq!(cache.fetch(0x80000004), None);
    /// assert_eq!(cache.fetch(0x80000008), Some(0x00000000)); // Still valid
    /// ```
    pub fn invalidate_range(&mut self, start: u32, end: u32) {
        if start > end {
            return;
        }

        // Align both bounds to 4-byte word addresses
        let mut addr = start & !0x3;
        let end_aligned = end & !0x3;

        loop {
            if addr > end_aligned {
                break;
            }

            let index = self.index(addr);
            let tag = self.tag(addr);

            let line = &mut self.lines[index];
            if line.valid && line.tag == tag {
                line.valid = false;
            }

            if addr == end_aligned {
                break;
            }
            addr = addr.wrapping_add(4);
        }
    }

    /// Prefill cache with instruction at given address
    ///
    /// This is an alias for `store()`, used when memory writes occur to known
    /// code regions, allowing us to cache instructions before execution
    /// (mimicking how real hardware caches instructions during BIOS copy operations).
    ///
    /// # Arguments
    ///
    /// * `addr` - Instruction address
    /// * `instruction` - 32-bit instruction word
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// // Prefill cache when BIOS copies code to RAM
    /// cache.prefill(0x80000500, 0x3C080000); // lui r8, 0x0000
    /// assert_eq!(cache.fetch(0x80000500), Some(0x3C080000));
    /// ```
    #[inline(always)]
    pub fn prefill(&mut self, addr: u32, instruction: u32) {
        self.store(addr, instruction);
    }

    /// Clear all cached instructions
    ///
    /// Invalidates all cache lines. This is faster than individual invalidation
    /// when resetting the entire cache.
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// cache.store(0x80000000, 0x00000000);
    /// cache.store(0x80000004, 0x00000000);
    ///
    /// cache.clear();
    ///
    /// assert_eq!(cache.fetch(0x80000000), None);
    /// assert_eq!(cache.fetch(0x80000004), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.valid = false;
        }
    }

    /// Check if cache is empty
    ///
    /// Returns true if no valid cache entries exist.
    ///
    /// # Returns
    ///
    /// `true` if cache is empty, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// assert!(cache.is_empty());
    ///
    /// cache.store(0x80000000, 0x00000000);
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|line| !line.valid)
    }

    /// Get number of valid cached entries
    ///
    /// Counts how many cache lines contain valid data.
    ///
    /// # Returns
    ///
    /// Number of valid cache entries (0-1024)
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.store(0x80000000, 0x00000000);
    /// cache.store(0x80000004, 0x00000000);
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.lines.iter().filter(|line| line.valid).count()
    }

    /// Get cache hit rate statistics
    ///
    /// Returns the percentage of cache lines that are valid.
    /// This can be used to monitor cache effectiveness.
    ///
    /// # Returns
    ///
    /// Cache occupancy as a percentage (0.0-100.0)
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cpu::icache::InstructionCache;
    ///
    /// let mut cache = InstructionCache::new();
    /// cache.store(0x80000000, 0x00000000);
    ///
    /// let occupancy = cache.occupancy();
    /// assert!(occupancy > 0.0 && occupancy <= 100.0);
    /// ```
    pub fn occupancy(&self) -> f64 {
        (self.len() as f64 / Self::LINE_COUNT as f64) * 100.0
    }
}

impl Default for InstructionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = InstructionCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_store_fetch() {
        let mut cache = InstructionCache::new();

        // Store instruction
        cache.store(0x80000000, 0x3C080000);

        // Fetch should return the stored instruction
        assert_eq!(cache.fetch(0x80000000), Some(0x3C080000));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = InstructionCache::new();

        // Fetch from empty cache should return None
        assert_eq!(cache.fetch(0x80000000), None);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = InstructionCache::new();

        cache.store(0x80000000, 0x3C080000);
        assert_eq!(cache.fetch(0x80000000), Some(0x3C080000));

        cache.invalidate(0x80000000);
        assert_eq!(cache.fetch(0x80000000), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_invalidate_range() {
        let mut cache = InstructionCache::new();

        // Store multiple instructions
        cache.store(0x80000000, 0x00000000);
        cache.store(0x80000004, 0x00000000);
        cache.store(0x80000008, 0x00000000);

        // Invalidate first two
        cache.invalidate_range(0x80000000, 0x80000004);

        assert_eq!(cache.fetch(0x80000000), None);
        assert_eq!(cache.fetch(0x80000004), None);
        assert_eq!(cache.fetch(0x80000008), Some(0x00000000));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = InstructionCache::new();

        cache.store(0x80000000, 0x00000000);
        cache.store(0x80000004, 0x00000000);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_direct_mapped_eviction() {
        let mut cache = InstructionCache::new();

        // Store instruction at 0x80000000
        cache.store(0x80000000, 0x11111111);
        assert_eq!(cache.fetch(0x80000000), Some(0x11111111));

        // Store at address that maps to same cache line (different tag)
        // Address 0x80001000 has same index [11:2] but different tag [31:12]
        cache.store(0x80001000, 0x22222222);

        // First instruction should be evicted
        assert_eq!(cache.fetch(0x80000000), None);
        // Second instruction should be present
        assert_eq!(cache.fetch(0x80001000), Some(0x22222222));
    }

    #[test]
    fn test_sequential_access() {
        let mut cache = InstructionCache::new();

        // Store sequential instructions
        for i in 0..10 {
            let addr = 0x80000000 + i * 4;
            cache.store(addr, i);
        }

        // All should be fetchable
        for i in 0..10 {
            let addr = 0x80000000 + i * 4;
            assert_eq!(cache.fetch(addr), Some(i));
        }

        assert_eq!(cache.len(), 10);
    }

    #[test]
    fn test_prefill() {
        let mut cache = InstructionCache::new();

        cache.prefill(0x80000500, 0x3C080000);
        assert_eq!(cache.fetch(0x80000500), Some(0x3C080000));
    }

    #[test]
    fn test_occupancy() {
        let mut cache = InstructionCache::new();

        assert_eq!(cache.occupancy(), 0.0);

        // Fill half the cache
        for i in 0..512 {
            cache.store(0x80000000 + i * 4, 0x00000000);
        }

        // Should be approximately 50%
        let occ = cache.occupancy();
        assert!((49.0..=51.0).contains(&occ));
    }

    #[test]
    fn test_index_extraction() {
        let cache = InstructionCache::new();

        // Test index extraction
        assert_eq!(cache.index(0x80000000), 0);
        assert_eq!(cache.index(0x80000004), 1);
        assert_eq!(cache.index(0x80000008), 2);

        // Test wrapping (address with same lower 12 bits)
        assert_eq!(cache.index(0x80000000), cache.index(0x80001000));
    }

    #[test]
    fn test_tag_extraction() {
        let cache = InstructionCache::new();

        // Test tag extraction
        assert_eq!(cache.tag(0x80000000), 0x80000);
        assert_eq!(cache.tag(0x80001000), 0x80001);
        assert_eq!(cache.tag(0xBFC00000), 0xBFC00);
    }

    #[test]
    fn test_cache_aliasing() {
        let mut cache = InstructionCache::new();

        // Two addresses with same index but different tags
        let addr1 = 0x80000100; // tag=0x80000, index=64
        let addr2 = 0x80001100; // tag=0x80001, index=64

        cache.store(addr1, 0xAAAAAAAA);
        assert_eq!(cache.fetch(addr1), Some(0xAAAAAAAA));

        // Store to same index with different tag
        cache.store(addr2, 0xBBBBBBBB);

        // First should be evicted
        assert_eq!(cache.fetch(addr1), None);
        assert_eq!(cache.fetch(addr2), Some(0xBBBBBBBB));
    }

    #[test]
    fn test_large_range_invalidation() {
        let mut cache = InstructionCache::new();

        // Fill cache with instructions
        for i in 0..100 {
            cache.store(0x80000000 + i * 4, i);
        }

        assert_eq!(cache.len(), 100);

        // Invalidate large range
        cache.invalidate_range(0x80000000, 0x80000100);

        // Count remaining valid entries
        let remaining = cache.len();
        assert!(remaining < 100);
    }

    // ========== Additional Edge Cases and Boundary Tests ==========

    #[test]
    fn test_cache_min_address() {
        let mut cache = InstructionCache::new();

        // Test with address 0x00000000
        cache.store(0x00000000, 0xDEADBEEF);
        assert_eq!(cache.fetch(0x00000000), Some(0xDEADBEEF));

        cache.invalidate(0x00000000);
        assert_eq!(cache.fetch(0x00000000), None);
    }

    #[test]
    fn test_cache_max_address() {
        let mut cache = InstructionCache::new();

        // Test with maximum 32-bit address
        cache.store(0xFFFFFFFC, 0xCAFEBABE);
        assert_eq!(cache.fetch(0xFFFFFFFC), Some(0xCAFEBABE));

        cache.invalidate(0xFFFFFFFC);
        assert_eq!(cache.fetch(0xFFFFFFFC), None);
    }

    #[test]
    fn test_cache_misaligned_addresses() {
        let mut cache = InstructionCache::new();

        // Cache should handle non-word-aligned addresses
        // (though in real hardware, these would cause exceptions)
        // All these addresses map to:
        // - index = (addr >> 2) & 0x3FF = 0x20000000 & 0x3FF = 0
        // - tag = addr >> 12 = 0x80000 (same for all three)
        // Since they have the same index AND same tag, they map to the same cache line
        cache.store(0x80000001, 0x11111111);
        cache.store(0x80000002, 0x22222222);
        cache.store(0x80000003, 0x33333333);

        // Since all three have the same tag, the last stored value overwrites previous ones
        // All three addresses will fetch the same cached data (last stored)
        assert_eq!(cache.fetch(0x80000001), Some(0x33333333)); // Same tag, gets last stored
        assert_eq!(cache.fetch(0x80000002), Some(0x33333333)); // Same tag, gets last stored
        assert_eq!(cache.fetch(0x80000003), Some(0x33333333)); // Last stored
    }

    #[test]
    fn test_cache_line_collision_pattern() {
        let mut cache = InstructionCache::new();

        // Create addresses that map to the same cache line
        // Cache line = (addr >> 2) & 0x3FF
        let base_addr = 0x80000000;
        let collision_addr = base_addr + (0x400 << 2); // +1024 cache lines (wraps around)

        cache.store(base_addr, 0xAAAAAAAA);
        assert_eq!(cache.fetch(base_addr), Some(0xAAAAAAAA));

        // Store to colliding address should evict previous entry
        cache.store(collision_addr, 0xBBBBBBBB);
        assert_eq!(cache.fetch(base_addr), None);
        assert_eq!(cache.fetch(collision_addr), Some(0xBBBBBBBB));
    }

    #[test]
    fn test_cache_fill_and_evict_pattern() {
        let mut cache = InstructionCache::new();

        // Fill one cache line multiple times with different tags
        let index_addr = 0x80000100; // Fixed index
        for i in 0..10 {
            let addr = index_addr + (i * 0x1000); // Different tags, same index
            cache.store(addr, i);

            // Only the last stored value should be accessible
            assert_eq!(cache.fetch(addr), Some(i));

            // Previous values should be evicted
            if i > 0 {
                let prev_addr = index_addr + ((i - 1) * 0x1000);
                assert_eq!(cache.fetch(prev_addr), None);
            }
        }
    }

    #[test]
    fn test_invalidate_range_boundaries() {
        let mut cache = InstructionCache::new();

        // Store instructions at boundaries
        cache.store(0x80000000, 0x00000001);
        cache.store(0x80000004, 0x00000002);
        cache.store(0x80000008, 0x00000003);
        cache.store(0x8000000C, 0x00000004);

        // Invalidate middle range
        cache.invalidate_range(0x80000004, 0x80000008);

        // First and last should remain valid
        assert_eq!(cache.fetch(0x80000000), Some(0x00000001));
        assert_eq!(cache.fetch(0x8000000C), Some(0x00000004));

        // Middle should be invalidated
        assert_eq!(cache.fetch(0x80000004), None);
        assert_eq!(cache.fetch(0x80000008), None);
    }

    #[test]
    fn test_invalidate_range_reversed() {
        let mut cache = InstructionCache::new();

        // Store some instructions
        for i in 0..10 {
            cache.store(0x80000000 + i * 4, i);
        }

        let len_before = cache.len();

        // Invalid range (start > end) should do nothing
        cache.invalidate_range(0x80000020, 0x80000010);

        assert_eq!(cache.len(), len_before);
    }

    #[test]
    fn test_invalidate_range_single_address() {
        let mut cache = InstructionCache::new();

        cache.store(0x80000100, 0xABCDEF00);
        cache.store(0x80000104, 0x12345678);

        // Invalidate single address (start == end)
        cache.invalidate_range(0x80000100, 0x80000100);

        assert_eq!(cache.fetch(0x80000100), None);
        assert_eq!(cache.fetch(0x80000104), Some(0x12345678));
    }

    #[test]
    fn test_invalidate_range_misaligned() {
        let mut cache = InstructionCache::new();

        // Store word-aligned instructions
        cache.store(0x80000000, 0x11111111);
        cache.store(0x80000004, 0x22222222);
        cache.store(0x80000008, 0x33333333);

        // Invalidate with misaligned boundaries
        // Should still invalidate the affected word-aligned addresses
        cache.invalidate_range(0x80000001, 0x80000007);

        // All should be invalidated due to alignment
        assert_eq!(cache.fetch(0x80000000), None);
        assert_eq!(cache.fetch(0x80000004), None);
        assert_eq!(cache.fetch(0x80000008), Some(0x33333333));
    }

    #[test]
    fn test_cache_refill_after_invalidation() {
        let mut cache = InstructionCache::new();

        let addr = 0x80000200;
        let first_data = 0xDEADBEEF;
        let second_data = 0xCAFEBABE;

        // Store, invalidate, store again
        cache.store(addr, first_data);
        assert_eq!(cache.fetch(addr), Some(first_data));

        cache.invalidate(addr);
        assert_eq!(cache.fetch(addr), None);

        cache.store(addr, second_data);
        assert_eq!(cache.fetch(addr), Some(second_data));
    }

    #[test]
    fn test_cache_multiple_regions() {
        let mut cache = InstructionCache::new();

        // Test caching instructions from different memory regions
        // Note: All these addresses map to index 0, so they collide
        let regions = [
            0x00000000, // RAM (user space) - index 0, tag 0x00000
            0x80000000, // RAM (kernel cached) - index 0, tag 0x80000
            0xA0000000, // RAM (kernel uncached) - index 0, tag 0xA0000
            0xBFC00000, // BIOS ROM - index 0, tag 0xBFC00
        ];

        for (i, &addr) in regions.iter().enumerate() {
            cache.store(addr, i as u32);
        }

        // Only the last entry should be in cache (others evicted by collision)
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.fetch(0xBFC00000), Some(3)); // Last stored
    }

    #[test]
    fn test_cache_wraparound_index() {
        let mut cache = InstructionCache::new();

        // Test addresses that wrap around in the 10-bit index
        // Max index is 1023, so index 0 and 1024 should collide
        let addr1 = 0x00000000; // Index 0
        let addr2 = 0x00001000; // Index 1024 % 1024 = 0 (same index, different tag)

        cache.store(addr1, 0xAAAAAAAA);
        assert_eq!(cache.fetch(addr1), Some(0xAAAAAAAA));

        // Should evict first entry
        cache.store(addr2, 0xBBBBBBBB);
        assert_eq!(cache.fetch(addr1), None);
        assert_eq!(cache.fetch(addr2), Some(0xBBBBBBBB));
    }

    #[test]
    fn test_cache_occupancy_calculation() {
        let mut cache = InstructionCache::new();

        // Empty cache
        assert_eq!(cache.occupancy(), 0.0);

        // Fill 25% of cache (256 entries)
        for i in 0..256 {
            cache.store(0x80000000 + i * 4, i);
        }
        let occ = cache.occupancy();
        assert!((24.0..=26.0).contains(&occ), "Occupancy should be ~25%");

        // Fill 100% of cache (1024 entries)
        for i in 0..1024 {
            cache.store(0x80000000 + i * 4, i);
        }
        let occ = cache.occupancy();
        assert!((99.0..=100.0).contains(&occ), "Occupancy should be ~100%");
    }

    #[test]
    fn test_cache_clear_vs_individual_invalidate() {
        let mut cache = InstructionCache::new();

        // Fill cache
        for i in 0..50 {
            cache.store(0x80000000 + i * 4, i);
        }

        assert_eq!(cache.len(), 50);

        // Clear all at once
        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        // Verify all entries are invalid
        for i in 0..50 {
            assert_eq!(cache.fetch(0x80000000 + i * 4), None);
        }
    }

    #[test]
    fn test_cache_tag_comparison() {
        let mut cache = InstructionCache::new();

        // Addresses with same index but different tags
        let addr1 = 0x00000100; // Tag = 0x00000, Index = 64
        let addr2 = 0x10000100; // Tag = 0x10000, Index = 64
        let addr3 = 0x20000100; // Tag = 0x20000, Index = 64

        cache.store(addr1, 0x11111111);
        assert_eq!(cache.fetch(addr1), Some(0x11111111));

        cache.store(addr2, 0x22222222);
        assert_eq!(cache.fetch(addr1), None); // Evicted
        assert_eq!(cache.fetch(addr2), Some(0x22222222));

        cache.store(addr3, 0x33333333);
        assert_eq!(cache.fetch(addr2), None); // Evicted
        assert_eq!(cache.fetch(addr3), Some(0x33333333));
    }

    #[test]
    fn test_cache_default_trait() {
        let cache: InstructionCache = Default::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_invalidate_nonexistent() {
        let mut cache = InstructionCache::new();

        // Invalidating non-existent entry should not panic
        cache.invalidate(0x12345678);

        // Cache should still be empty
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stress_sequential_access() {
        let mut cache = InstructionCache::new();

        // Simulate sequential code execution (common pattern)
        let start_addr = 0xBFC00000;
        let num_instructions = 2048; // More than cache size

        for i in 0..num_instructions {
            let addr = start_addr + i * 4;
            cache.store(addr, i);
        }

        // Cache should be full (1024 entries)
        assert_eq!(cache.len(), 1024);

        // Only the last 1024 instructions should be cached
        // due to collision and eviction
        let occ = cache.occupancy();
        assert_eq!(occ, 100.0);
    }

    #[test]
    fn test_cache_invalidate_range_entire_cache() {
        let mut cache = InstructionCache::new();

        // Fill cache
        for i in 0..100 {
            cache.store(0x80000000 + i * 4, i);
        }

        assert!(!cache.is_empty());

        // Invalidate entire range
        cache.invalidate_range(0x00000000, 0xFFFFFFFF);

        // All entries in range should be invalidated
        for i in 0..100 {
            assert_eq!(cache.fetch(0x80000000 + i * 4), None);
        }
    }

    #[test]
    fn test_cache_prefill_vs_store() {
        let mut cache = InstructionCache::new();

        let addr = 0x80000500;
        let data = 0x12345678;

        // prefill is an alias for store
        cache.prefill(addr, data);
        assert_eq!(cache.fetch(addr), Some(data));

        // Store should work the same way
        cache.store(addr, data);
        assert_eq!(cache.fetch(addr), Some(data));
    }

    #[test]
    fn test_cache_bios_execution_pattern() {
        let mut cache = InstructionCache::new();

        // Simulate BIOS execution pattern
        let bios_start = 0xBFC00000;

        // BIOS typically executes sequentially from start
        for i in 0..512 {
            let addr = bios_start + i * 4;
            cache.store(addr, i);

            // Verify immediate fetch works
            assert_eq!(cache.fetch(addr), Some(i));
        }

        // All should still be cached
        assert_eq!(cache.len(), 512);
    }

    #[test]
    fn test_cache_ram_mirrors() {
        let mut cache = InstructionCache::new();

        // PSX has RAM mirrors at different addresses
        // 0x00000000 (KUSEG), 0x80000000 (KSEG0 cached), 0xA0000000 (KSEG1 uncached)
        // All three addresses map to the same cache index (1024)
        // index = (addr >> 2) & 0x3FF = (0x1000 >> 2) & 0x3FF = 0x400 & 0x3FF = 0x0
        // So they all map to index 0 with different tags
        let kuseg_addr = 0x00001000;
        let kseg0_addr = 0x80001000;
        let kseg1_addr = 0xA0001000;

        cache.store(kuseg_addr, 0x11111111);
        cache.store(kseg0_addr, 0x22222222);
        cache.store(kseg1_addr, 0x33333333);

        // Since they map to the same index with different tags,
        // only the last one should remain (direct-mapped cache behavior)
        assert_eq!(cache.fetch(kuseg_addr), None); // Evicted
        assert_eq!(cache.fetch(kseg0_addr), None); // Evicted
        assert_eq!(cache.fetch(kseg1_addr), Some(0x33333333)); // Last stored
    }
}
