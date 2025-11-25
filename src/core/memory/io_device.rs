// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 itsakeyfut

//! I/O Device Trait
//!
//! This module defines a trait-based abstraction for memory-mapped I/O devices.
//! By implementing the `IODevice` trait, peripherals can be registered with the
//! memory bus without requiring the Bus to have explicit knowledge of each device type.
//!
//! # Design Goals
//!
//! - **Decoupling**: Bus doesn't need to know about specific peripheral types
//! - **Extensibility**: New peripherals can be added without modifying Bus
//! - **Testability**: Devices can be tested in isolation with mock implementations
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │              Memory Bus                     │
//! ├─────────────────────────────────────────────┤
//! │  Devices: Vec<Box<dyn IODevice>>            │
//! │                                             │
//! │  read_io_port(addr) {                       │
//! │    for device in devices {                  │
//! │      if device.contains(addr) {             │
//! │        return device.read_register(offset)  │
//! │      }                                      │
//! │    }                                        │
//! │  }                                          │
//! └─────────────────────────────────────────────┘
//!           ▲                   ▲
//!           │                   │
//!    ┌──────┴──────┐    ┌──────┴──────┐
//!    │   GPU       │    │  Timers     │
//!    │ (IODevice)  │    │ (IODevice)  │
//!    └─────────────┘    └─────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use psrx::core::memory::IODevice;
//! use psrx::core::error::Result;
//!
//! struct MyPeripheral {
//!     base_addr: u32,
//!     registers: [u32; 4],
//! }
//!
//! impl IODevice for MyPeripheral {
//!     fn address_range(&self) -> (u32, u32) {
//!         (self.base_addr, self.base_addr + 0x0F)
//!     }
//!
//!     fn read_register(&self, offset: u32) -> Result<u32> {
//!         let index = (offset / 4) as usize;
//!         Ok(self.registers.get(index).copied().unwrap_or(0))
//!     }
//!
//!     fn write_register(&mut self, offset: u32, value: u32) -> Result<()> {
//!         let index = (offset / 4) as usize;
//!         if index < self.registers.len() {
//!             self.registers[index] = value;
//!         }
//!         Ok(())
//!     }
//! }
//! ```

use crate::core::error::Result;

/// Trait for memory-mapped I/O devices
///
/// This trait provides a uniform interface for all memory-mapped peripherals
/// in the PlayStation hardware. Each device declares its address range and
/// implements read/write operations for its registers.
///
/// # Register Access
///
/// The trait provides methods for 8-bit, 16-bit, and 32-bit register access.
/// Devices must implement the 32-bit methods; default implementations are provided
/// for 8-bit and 16-bit access that delegate to the 32-bit methods.
///
/// # Address Translation
///
/// The Bus translates physical addresses to device-relative offsets before
/// calling trait methods. For example:
///
/// - Device address range: `0x1F801810 - 0x1F801817`
/// - Physical address: `0x1F801814`
/// - Offset passed to device: `0x04`
///
/// # Thread Safety
///
/// IODevice implementations do not need to be `Send` or `Sync` as the Bus
/// is not shared across threads. However, they should handle interior mutability
/// properly when accessed through `&self` methods (e.g., using `RefCell`).
pub trait IODevice {
    /// Get the address range this device responds to
    ///
    /// Returns a tuple of (start_address, end_address) inclusive.
    /// The Bus will route any memory access within this range to this device.
    ///
    /// # Returns
    ///
    /// `(start, end)` - Start and end physical addresses (inclusive)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use psrx::core::memory::IODevice;
    /// # struct GPU;
    /// # impl IODevice for GPU {
    /// #     fn address_range(&self) -> (u32, u32) {
    /// // GPU registers: 0x1F801810 - 0x1F801817
    /// (0x1F801810, 0x1F801817)
    /// #     }
    /// #     fn read_register(&self, offset: u32) -> psrx::core::error::Result<u32> { Ok(0) }
    /// #     fn write_register(&mut self, offset: u32, value: u32) -> psrx::core::error::Result<()> { Ok(()) }
    /// # }
    /// ```
    fn address_range(&self) -> (u32, u32);

    /// Check if this device contains the given address
    ///
    /// This is a helper method that checks if an address falls within
    /// this device's address range.
    ///
    /// # Arguments
    ///
    /// * `addr` - Physical address to check
    ///
    /// # Returns
    ///
    /// `true` if the address is within this device's range
    fn contains(&self, addr: u32) -> bool {
        let (start, end) = self.address_range();
        addr >= start && addr <= end
    }

    /// Read a 32-bit value from a device register
    ///
    /// The offset is relative to the device's base address. For example,
    /// if the device base is `0x1F801810` and the access is to `0x1F801814`,
    /// the offset will be `0x04`.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address (must be 4-byte aligned)
    ///
    /// # Returns
    ///
    /// The 32-bit register value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The offset is out of range for this device
    /// - The offset is not properly aligned
    /// - The register is write-only
    fn read_register(&self, offset: u32) -> Result<u32>;

    /// Write a 32-bit value to a device register
    ///
    /// The offset is relative to the device's base address.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address (must be 4-byte aligned)
    /// * `value` - 32-bit value to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The offset is out of range for this device
    /// - The offset is not properly aligned
    /// - The register is read-only
    fn write_register(&mut self, offset: u32, value: u32) -> Result<()>;

    /// Read a 16-bit value from a device register
    ///
    /// Default implementation reads the 32-bit value and masks to 16 bits.
    /// Devices can override this if they need special 16-bit handling.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address (must be 2-byte aligned)
    ///
    /// # Returns
    ///
    /// The 16-bit register value
    fn read_register16(&self, offset: u32) -> Result<u16> {
        // Default: read 32-bit and mask
        let value = self.read_register(offset & !0x03)?;
        let shift = (offset & 0x02) * 8;
        Ok(((value >> shift) & 0xFFFF) as u16)
    }

    /// Write a 16-bit value to a device register
    ///
    /// Default implementation performs read-modify-write on the aligned 32-bit word,
    /// updating only the targeted 16-bit half. Devices can override this if they
    /// need special 16-bit handling.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address (must be 2-byte aligned)
    /// * `value` - 16-bit value to write
    fn write_register16(&mut self, offset: u32, value: u16) -> Result<()> {
        // Read-modify-write to update only the target 16-bit field
        let aligned = offset & !0x03;
        let shift = (offset & 0x02) * 8;
        let mask = !(0xFFFFu32 << shift);
        let current = self.read_register(aligned)?;
        let new_value = (current & mask) | ((value as u32) << shift);
        self.write_register(aligned, new_value)
    }

    /// Read an 8-bit value from a device register
    ///
    /// Default implementation reads the 32-bit value and masks to 8 bits.
    /// Devices can override this if they need special 8-bit handling.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address
    ///
    /// # Returns
    ///
    /// The 8-bit register value
    fn read_register8(&self, offset: u32) -> Result<u8> {
        // Default: read 32-bit and mask
        let value = self.read_register(offset & !0x03)?;
        let shift = (offset & 0x03) * 8;
        Ok(((value >> shift) & 0xFF) as u8)
    }

    /// Write an 8-bit value to a device register
    ///
    /// Default implementation performs read-modify-write on the aligned 32-bit word,
    /// updating only the targeted 8-bit byte. Devices can override this if they
    /// need special 8-bit handling.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset from device base address
    /// * `value` - 8-bit value to write
    fn write_register8(&mut self, offset: u32, value: u8) -> Result<()> {
        // Read-modify-write to update only the target 8-bit field
        let aligned = offset & !0x03;
        let shift = (offset & 0x03) * 8;
        let mask = !(0xFFu32 << shift);
        let current = self.read_register(aligned)?;
        let new_value = (current & mask) | ((value as u32) << shift);
        self.write_register(aligned, new_value)
    }

    /// Optional: Device name for debugging
    ///
    /// Returns a human-readable name for this device.
    /// Useful for logging and debugging.
    fn name(&self) -> &str {
        "Unknown Device"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::EmulatorError;

    /// Mock device for testing
    struct MockDevice {
        base: u32,
        size: u32,
        registers: Vec<u32>,
    }

    impl MockDevice {
        fn new(base: u32, register_count: usize) -> Self {
            Self {
                base,
                size: (register_count * 4) as u32,
                registers: vec![0; register_count],
            }
        }
    }

    impl IODevice for MockDevice {
        fn address_range(&self) -> (u32, u32) {
            (self.base, self.base + self.size - 1)
        }

        fn read_register(&self, offset: u32) -> Result<u32> {
            let index = (offset / 4) as usize;
            self.registers
                .get(index)
                .copied()
                .ok_or(EmulatorError::InvalidMemoryAccess {
                    address: self.base + offset,
                })
        }

        fn write_register(&mut self, offset: u32, value: u32) -> Result<()> {
            let index = (offset / 4) as usize;
            if index < self.registers.len() {
                self.registers[index] = value;
                Ok(())
            } else {
                Err(EmulatorError::InvalidMemoryAccess {
                    address: self.base + offset,
                })
            }
        }

        fn name(&self) -> &str {
            "MockDevice"
        }
    }

    #[test]
    fn test_address_range() {
        let device = MockDevice::new(0x1F801000, 4);
        assert_eq!(device.address_range(), (0x1F801000, 0x1F80100F));
    }

    #[test]
    fn test_contains() {
        let device = MockDevice::new(0x1F801000, 4);

        assert!(device.contains(0x1F801000));
        assert!(device.contains(0x1F801008));
        assert!(device.contains(0x1F80100F));

        assert!(!device.contains(0x1F800FFF));
        assert!(!device.contains(0x1F801010));
    }

    #[test]
    fn test_read_write_32bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write and read back
        device.write_register(0x00, 0x12345678).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0x12345678);

        device.write_register(0x04, 0xABCDEF00).unwrap();
        assert_eq!(device.read_register(0x04).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_read_write_16bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write 16-bit value
        device.write_register16(0x00, 0x1234).unwrap();

        // Read back as 16-bit
        assert_eq!(device.read_register16(0x00).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_write_8bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write 8-bit value
        device.write_register8(0x00, 0xAB).unwrap();

        // Read back as 8-bit
        assert_eq!(device.read_register8(0x00).unwrap(), 0xAB);
    }

    #[test]
    fn test_out_of_range() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Out of range access should fail
        assert!(device.read_register(0x10).is_err());
        assert!(device.write_register(0x10, 0).is_err());
    }

    #[test]
    fn test_device_name() {
        let device = MockDevice::new(0x1F801000, 4);
        assert_eq!(device.name(), "MockDevice");
    }

    #[test]
    fn test_read_write_16bit_lower_half() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write to lower 16 bits (offset 0x00, bits 0-15)
        device.write_register16(0x00, 0xABCD).unwrap();

        // Read back as 16-bit
        assert_eq!(device.read_register16(0x00).unwrap(), 0xABCD);

        // Read back as 32-bit (upper 16 bits should be 0)
        assert_eq!(device.read_register(0x00).unwrap(), 0x0000ABCD);
    }

    #[test]
    fn test_read_write_16bit_upper_half() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write to upper 16 bits (offset 0x02, bits 16-31)
        device.write_register16(0x02, 0x1234).unwrap();

        // Read back as 16-bit
        assert_eq!(device.read_register16(0x02).unwrap(), 0x1234);

        // Read back as 32-bit (lower 16 bits should be 0)
        assert_eq!(device.read_register(0x00).unwrap(), 0x12340000);
    }

    #[test]
    fn test_read_write_16bit_both_halves() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write to both halves
        device.write_register16(0x00, 0xABCD).unwrap();
        device.write_register16(0x02, 0x1234).unwrap();

        // Read back as 32-bit
        assert_eq!(device.read_register(0x00).unwrap(), 0x1234ABCD);

        // Read back each half
        assert_eq!(device.read_register16(0x00).unwrap(), 0xABCD);
        assert_eq!(device.read_register16(0x02).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_write_16bit_preserves_other_half() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write 32-bit value
        device.write_register(0x00, 0xDEADBEEF).unwrap();

        // Write only lower 16 bits
        device.write_register16(0x00, 0x1234).unwrap();

        // Upper 16 bits should be preserved
        assert_eq!(device.read_register(0x00).unwrap(), 0xDEAD1234);

        // Write only upper 16 bits
        device.write_register16(0x02, 0x5678).unwrap();

        // Lower 16 bits should be preserved
        assert_eq!(device.read_register(0x00).unwrap(), 0x56781234);
    }

    #[test]
    fn test_read_write_8bit_all_bytes() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write to all 4 bytes
        device.write_register8(0x00, 0x12).unwrap();
        device.write_register8(0x01, 0x34).unwrap();
        device.write_register8(0x02, 0x56).unwrap();
        device.write_register8(0x03, 0x78).unwrap();

        // Read back as 32-bit (little-endian)
        assert_eq!(device.read_register(0x00).unwrap(), 0x78563412);

        // Read back each byte
        assert_eq!(device.read_register8(0x00).unwrap(), 0x12);
        assert_eq!(device.read_register8(0x01).unwrap(), 0x34);
        assert_eq!(device.read_register8(0x02).unwrap(), 0x56);
        assert_eq!(device.read_register8(0x03).unwrap(), 0x78);
    }

    #[test]
    fn test_read_write_8bit_preserves_other_bytes() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write 32-bit value
        device.write_register(0x00, 0xDEADBEEF).unwrap();

        // Write only byte 0
        device.write_register8(0x00, 0x12).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xDEADBE12);

        // Write only byte 1
        device.write_register8(0x01, 0x34).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xDEAD3412);

        // Write only byte 2
        device.write_register8(0x02, 0x56).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xDE563412);

        // Write only byte 3
        device.write_register8(0x03, 0x78).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0x78563412);
    }

    #[test]
    fn test_mixed_size_accesses() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write 32-bit
        device.write_register(0x00, 0x12345678).unwrap();

        // Read as 16-bit
        assert_eq!(device.read_register16(0x00).unwrap(), 0x5678);
        assert_eq!(device.read_register16(0x02).unwrap(), 0x1234);

        // Read as 8-bit
        assert_eq!(device.read_register8(0x00).unwrap(), 0x78);
        assert_eq!(device.read_register8(0x01).unwrap(), 0x56);
        assert_eq!(device.read_register8(0x02).unwrap(), 0x34);
        assert_eq!(device.read_register8(0x03).unwrap(), 0x12);
    }

    #[test]
    fn test_alignment_handling_16bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write to aligned addresses
        device.write_register16(0x00, 0x1234).unwrap();
        device.write_register16(0x02, 0x5678).unwrap();

        // Both writes should work correctly
        assert_eq!(device.read_register(0x00).unwrap(), 0x56781234);
    }

    #[test]
    fn test_alignment_handling_8bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // 8-bit accesses should work at any offset
        device.write_register8(0x00, 0x11).unwrap();
        device.write_register8(0x01, 0x22).unwrap();
        device.write_register8(0x02, 0x33).unwrap();
        device.write_register8(0x03, 0x44).unwrap();

        assert_eq!(device.read_register(0x00).unwrap(), 0x44332211);
    }

    #[test]
    fn test_multiple_registers() {
        let mut device = MockDevice::new(0x1F801000, 8);

        // Write to multiple registers
        device.write_register(0x00, 0x11111111).unwrap();
        device.write_register(0x04, 0x22222222).unwrap();
        device.write_register(0x08, 0x33333333).unwrap();
        device.write_register(0x0C, 0x44444444).unwrap();

        // Read back
        assert_eq!(device.read_register(0x00).unwrap(), 0x11111111);
        assert_eq!(device.read_register(0x04).unwrap(), 0x22222222);
        assert_eq!(device.read_register(0x08).unwrap(), 0x33333333);
        assert_eq!(device.read_register(0x0C).unwrap(), 0x44444444);
    }

    #[test]
    fn test_address_range_boundaries() {
        let device = MockDevice::new(0x1F801000, 4);

        // Test exact start
        assert!(device.contains(0x1F801000));

        // Test exact end
        assert!(device.contains(0x1F80100F));

        // Test one before start
        assert!(!device.contains(0x1F800FFF));

        // Test one after end
        assert!(!device.contains(0x1F801010));
    }

    #[test]
    fn test_contains_with_large_range() {
        let device = MockDevice::new(0x1F801000, 256); // 1KB of registers

        // Test start and end
        assert!(device.contains(0x1F801000));
        assert!(device.contains(0x1F8013FF));

        // Test outside
        assert!(!device.contains(0x1F800FFF));
        assert!(!device.contains(0x1F801400));

        // Test middle
        assert!(device.contains(0x1F801200));
    }

    #[test]
    fn test_read_register16_unaligned_calculation() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write a known 32-bit value
        device.write_register(0x00, 0xAABBCCDD).unwrap();

        // Read 16-bit at offset 0x00 (bits 0-15)
        assert_eq!(device.read_register16(0x00).unwrap(), 0xCCDD);

        // Read 16-bit at offset 0x02 (bits 16-31)
        assert_eq!(device.read_register16(0x02).unwrap(), 0xAABB);
    }

    #[test]
    fn test_read_register8_all_offsets() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write a known 32-bit value (little-endian: 0xDD, 0xCC, 0xBB, 0xAA)
        device.write_register(0x00, 0xAABBCCDD).unwrap();

        // Read each byte
        assert_eq!(device.read_register8(0x00).unwrap(), 0xDD);
        assert_eq!(device.read_register8(0x01).unwrap(), 0xCC);
        assert_eq!(device.read_register8(0x02).unwrap(), 0xBB);
        assert_eq!(device.read_register8(0x03).unwrap(), 0xAA);
    }

    #[test]
    fn test_write_then_partial_read() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write full 32-bit value
        device.write_register(0x00, 0x12345678).unwrap();

        // Read parts
        assert_eq!(device.read_register16(0x00).unwrap(), 0x5678);
        assert_eq!(device.read_register16(0x02).unwrap(), 0x1234);
        assert_eq!(device.read_register8(0x00).unwrap(), 0x78);
        assert_eq!(device.read_register8(0x01).unwrap(), 0x56);
        assert_eq!(device.read_register8(0x02).unwrap(), 0x34);
        assert_eq!(device.read_register8(0x03).unwrap(), 0x12);
    }

    #[test]
    fn test_sequential_8bit_writes_build_32bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write bytes sequentially
        device.write_register8(0x00, 0xEF).unwrap();
        device.write_register8(0x01, 0xBE).unwrap();
        device.write_register8(0x02, 0xAD).unwrap();
        device.write_register8(0x03, 0xDE).unwrap();

        // Should form 0xDEADBEEF in little-endian
        assert_eq!(device.read_register(0x00).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_sequential_16bit_writes_build_32bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write half-words sequentially
        device.write_register16(0x00, 0xBEEF).unwrap();
        device.write_register16(0x02, 0xDEAD).unwrap();

        // Should form 0xDEADBEEF in little-endian
        assert_eq!(device.read_register(0x00).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_zero_value_handling() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write zero
        device.write_register(0x00, 0x00000000).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0x00000000);

        // Write non-zero then zero
        device.write_register(0x00, 0xFFFFFFFF).unwrap();
        device.write_register(0x00, 0x00000000).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0x00000000);

        // Write zero to parts
        device.write_register(0x00, 0xFFFFFFFF).unwrap();
        device.write_register16(0x00, 0x0000).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xFFFF0000);

        device.write_register8(0x02, 0x00).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xFF000000);
    }

    #[test]
    fn test_max_value_handling() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Write max values
        device.write_register(0x00, 0xFFFFFFFF).unwrap();
        assert_eq!(device.read_register(0x00).unwrap(), 0xFFFFFFFF);
        assert_eq!(device.read_register16(0x00).unwrap(), 0xFFFF);
        assert_eq!(device.read_register16(0x02).unwrap(), 0xFFFF);
        assert_eq!(device.read_register8(0x00).unwrap(), 0xFF);
        assert_eq!(device.read_register8(0x01).unwrap(), 0xFF);
        assert_eq!(device.read_register8(0x02).unwrap(), 0xFF);
        assert_eq!(device.read_register8(0x03).unwrap(), 0xFF);
    }

    #[test]
    fn test_register_independence() {
        let mut device = MockDevice::new(0x1F801000, 8);

        // Write to different registers
        device.write_register(0x00, 0x11111111).unwrap();
        device.write_register(0x04, 0x22222222).unwrap();
        device.write_register(0x08, 0x33333333).unwrap();

        // Modify one register
        device.write_register(0x04, 0xFFFFFFFF).unwrap();

        // Others should be unchanged
        assert_eq!(device.read_register(0x00).unwrap(), 0x11111111);
        assert_eq!(device.read_register(0x04).unwrap(), 0xFFFFFFFF);
        assert_eq!(device.read_register(0x08).unwrap(), 0x33333333);
    }

    #[test]
    fn test_out_of_range_8bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Out of range 8-bit access should fail
        assert!(device.read_register8(0x10).is_err());
        assert!(device.write_register8(0x10, 0xFF).is_err());
    }

    #[test]
    fn test_out_of_range_16bit() {
        let mut device = MockDevice::new(0x1F801000, 4);

        // Out of range 16-bit access should fail
        assert!(device.read_register16(0x10).is_err());
        assert!(device.write_register16(0x10, 0xFFFF).is_err());
    }
}
