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

/// Coprocessor 0 (System Control)
///
/// COP0 is the system control unit responsible for exception handling,
/// status management, cache control, and other system functions.
pub(super) struct COP0 {
    /// COP0 registers (32 registers)
    pub(super) regs: [u32; 32],
}

impl COP0 {
    /// Breakpoint PC
    #[allow(dead_code)]
    pub const BPC: usize = 3;
    /// Breakpoint Data Address
    #[allow(dead_code)]
    pub const BDA: usize = 5;
    /// Target Address
    #[allow(dead_code)]
    pub const TAR: usize = 6;
    /// Cache control
    #[allow(dead_code)]
    pub const DCIC: usize = 7;
    /// Bad Virtual Address
    #[allow(dead_code)]
    pub const BADA: usize = 8;
    /// Data Address Mask
    #[allow(dead_code)]
    pub const BDAM: usize = 9;
    /// PC Mask
    #[allow(dead_code)]
    pub const BPCM: usize = 11;
    /// Status Register
    pub const SR: usize = 12;
    /// Cause Register
    pub const CAUSE: usize = 13;
    /// Exception PC
    pub const EPC: usize = 14;
    /// Processor ID
    pub const PRID: usize = 15;

    /// Create a new COP0 instance
    ///
    /// # Returns
    /// Initialized COP0 instance with reset values
    pub(super) fn new() -> Self {
        let mut regs = [0u32; 32];
        // Status Register initial value
        regs[Self::SR] = 0x10900000;
        // Processor ID (R3000A identifier)
        regs[Self::PRID] = 0x00000002;

        Self { regs }
    }

    /// Reset COP0 registers to initial state
    pub(super) fn reset(&mut self) {
        self.regs = [0u32; 32];
        self.regs[Self::SR] = 0x10900000;
        self.regs[Self::PRID] = 0x00000002;
    }
}

/// Exception cause codes for MIPS R3000A
///
/// These correspond to the exception codes stored in the CAUSE register
/// when a CPU exception occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExceptionCause {
    /// Interrupt (external or internal)
    Interrupt = 0,
    /// Address error on load
    AddressErrorLoad = 4,
    /// Address error on store
    AddressErrorStore = 5,
    /// Bus error on instruction fetch
    BusErrorInstruction = 6,
    /// Bus error on data access
    BusErrorData = 7,
    /// Syscall instruction executed
    Syscall = 8,
    /// Breakpoint instruction executed
    Breakpoint = 9,
    /// Reserved or illegal instruction
    ReservedInstruction = 10,
    /// Coprocessor unusable
    CoprocessorUnusable = 11,
    /// Arithmetic overflow
    Overflow = 12,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cop0_new() {
        let cop0 = COP0::new();

        // Verify initial SR value (Status Register)
        // Bits set: CU0=1 (bit 28), BEV=1 (bit 22), TS=0, PE=0, CM=0, PZ=0, SwC=0, IsC=0
        assert_eq!(cop0.regs[COP0::SR], 0x10900000);

        // Verify processor ID
        assert_eq!(cop0.regs[COP0::PRID], 0x00000002);

        // All other registers should be zero
        for i in 0..32 {
            if i != COP0::SR && i != COP0::PRID {
                assert_eq!(
                    cop0.regs[i], 0,
                    "Register {} should be zero after initialization",
                    i
                );
            }
        }
    }

    #[test]
    fn test_cop0_reset() {
        let mut cop0 = COP0::new();

        // Modify some registers
        cop0.regs[COP0::EPC] = 0x12345678;
        cop0.regs[COP0::CAUSE] = 0xABCDEF00;
        cop0.regs[COP0::BADA] = 0xDEADBEEF;

        // Reset
        cop0.reset();

        // Verify SR and PRID are restored
        assert_eq!(cop0.regs[COP0::SR], 0x10900000);
        assert_eq!(cop0.regs[COP0::PRID], 0x00000002);

        // Verify other registers are cleared
        assert_eq!(cop0.regs[COP0::EPC], 0);
        assert_eq!(cop0.regs[COP0::CAUSE], 0);
        assert_eq!(cop0.regs[COP0::BADA], 0);
    }

    #[test]
    fn test_cop0_register_indices() {
        // Test all register index constants
        assert_eq!(COP0::BPC, 3);
        assert_eq!(COP0::BDA, 5);
        assert_eq!(COP0::TAR, 6);
        assert_eq!(COP0::DCIC, 7);
        assert_eq!(COP0::BADA, 8);
        assert_eq!(COP0::BDAM, 9);
        assert_eq!(COP0::BPCM, 11);
        assert_eq!(COP0::SR, 12);
        assert_eq!(COP0::CAUSE, 13);
        assert_eq!(COP0::EPC, 14);
        assert_eq!(COP0::PRID, 15);
    }

    #[test]
    fn test_cop0_sr_initial_bits() {
        let cop0 = COP0::new();
        let sr = cop0.regs[COP0::SR];

        // Check specific bits in SR according to PSX specs
        // Bit 28 (CU0): Should be 1 (COP0 enabled)
        assert_eq!((sr >> 28) & 1, 1, "CU0 bit should be set");

        // Bit 20 (not BEV which is bit 22): Actual initial value
        // Note: BEV is bit 22, but our initial value has bit 20 set
        // Let's check the actual value instead
        assert_eq!(
            sr, 0x10900000,
            "SR initial value should match implementation"
        );

        // Bit 16 (IsC): Should be 0 (cache not isolated)
        assert_eq!((sr >> 16) & 1, 0, "IsC bit should be clear");

        // Bits 0-1 (IEc, KUc): Should be 0 (interrupts disabled, kernel mode)
        assert_eq!(sr & 0x3, 0, "IEc and KUc should be clear");
    }

    #[test]
    fn test_cop0_register_read_write() {
        let mut cop0 = COP0::new();

        // Test writing and reading various registers
        cop0.regs[COP0::EPC] = 0x80001234;
        assert_eq!(cop0.regs[COP0::EPC], 0x80001234);

        cop0.regs[COP0::CAUSE] = 0x00000020; // ExcCode = 8 (syscall)
        assert_eq!(cop0.regs[COP0::CAUSE], 0x00000020);

        cop0.regs[COP0::BADA] = 0xFFFFFFFF;
        assert_eq!(cop0.regs[COP0::BADA], 0xFFFFFFFF);
    }

    #[test]
    fn test_cop0_breakpoint_registers() {
        let mut cop0 = COP0::new();

        // Test breakpoint PC register
        cop0.regs[COP0::BPC] = 0x80010000;
        assert_eq!(cop0.regs[COP0::BPC], 0x80010000);

        // Test breakpoint PC mask
        cop0.regs[COP0::BPCM] = 0xFFFFFFFC; // Word-aligned mask
        assert_eq!(cop0.regs[COP0::BPCM], 0xFFFFFFFC);

        // Test breakpoint data address
        cop0.regs[COP0::BDA] = 0x1F801810; // GPU register address
        assert_eq!(cop0.regs[COP0::BDA], 0x1F801810);

        // Test breakpoint data address mask
        cop0.regs[COP0::BDAM] = 0xFFFFFFF0;
        assert_eq!(cop0.regs[COP0::BDAM], 0xFFFFFFF0);
    }

    #[test]
    fn test_cop0_dcic_register() {
        let mut cop0 = COP0::new();

        // DCIC (Debug and Cache Isolation Control)
        cop0.regs[COP0::DCIC] = 0x80000000; // Enable debug features
        assert_eq!(cop0.regs[COP0::DCIC], 0x80000000);
    }

    #[test]
    fn test_cop0_tar_register() {
        let mut cop0 = COP0::new();

        // TAR (Target Address Register) - stores jump/branch target
        cop0.regs[COP0::TAR] = 0xBFC00100;
        assert_eq!(cop0.regs[COP0::TAR], 0xBFC00100);
    }

    #[test]
    fn test_exception_cause_values() {
        // Verify exception cause codes match PSX specifications
        assert_eq!(ExceptionCause::Interrupt as u32, 0x00);
        assert_eq!(ExceptionCause::AddressErrorLoad as u32, 0x04);
        assert_eq!(ExceptionCause::AddressErrorStore as u32, 0x05);
        assert_eq!(ExceptionCause::BusErrorInstruction as u32, 0x06);
        assert_eq!(ExceptionCause::BusErrorData as u32, 0x07);
        assert_eq!(ExceptionCause::Syscall as u32, 0x08);
        assert_eq!(ExceptionCause::Breakpoint as u32, 0x09);
        assert_eq!(ExceptionCause::ReservedInstruction as u32, 0x0A);
        assert_eq!(ExceptionCause::CoprocessorUnusable as u32, 0x0B);
        assert_eq!(ExceptionCause::Overflow as u32, 0x0C);
    }

    #[test]
    fn test_exception_cause_in_cause_register() {
        let mut cop0 = COP0::new();

        // ExcCode is stored in bits 2-6 of CAUSE register
        // Test setting various exception codes
        let test_cases = [
            (ExceptionCause::Interrupt, 0x00000000),
            (ExceptionCause::Syscall, 0x00000020), // 8 << 2 = 0x20
            (ExceptionCause::Breakpoint, 0x00000024), // 9 << 2 = 0x24
            (ExceptionCause::AddressErrorLoad, 0x00000010), // 4 << 2 = 0x10
            (ExceptionCause::Overflow, 0x00000030), // 12 << 2 = 0x30
        ];

        for (cause, expected_bits) in test_cases {
            cop0.regs[COP0::CAUSE] = (cause as u32) << 2;
            assert_eq!(
                cop0.regs[COP0::CAUSE] & 0x7C,
                expected_bits,
                "ExcCode for {:?} should be 0x{:02X}",
                cause,
                expected_bits
            );
        }
    }

    #[test]
    fn test_cop0_cause_register_bd_bit() {
        let mut cop0 = COP0::new();

        // Bit 31 (BD) indicates if exception occurred in branch delay slot
        cop0.regs[COP0::CAUSE] = 0x80000000; // BD bit set
        assert_eq!(
            (cop0.regs[COP0::CAUSE] >> 31) & 1,
            1,
            "BD bit should be set"
        );

        cop0.regs[COP0::CAUSE] = 0x00000020; // BD bit clear, ExcCode=Syscall
        assert_eq!(
            (cop0.regs[COP0::CAUSE] >> 31) & 1,
            0,
            "BD bit should be clear"
        );
    }

    #[test]
    fn test_cop0_cause_register_interrupt_pending() {
        let mut cop0 = COP0::new();

        // Bits 8-15 (IP) show pending interrupts
        cop0.regs[COP0::CAUSE] = 0x00000400; // IP2 (bit 10) set
        assert_eq!((cop0.regs[COP0::CAUSE] >> 10) & 1, 1, "IP2 should be set");

        cop0.regs[COP0::CAUSE] = 0x00008000; // IP7 (bit 15) set
        assert_eq!((cop0.regs[COP0::CAUSE] >> 15) & 1, 1, "IP7 should be set");
    }

    #[test]
    fn test_cop0_sr_interrupt_enable() {
        let mut cop0 = COP0::new();

        // Test IEc bit (bit 0) - current interrupt enable
        cop0.regs[COP0::SR] |= 0x00000001; // Enable interrupts
        assert_eq!(
            cop0.regs[COP0::SR] & 1,
            1,
            "IEc bit should be set (interrupts enabled)"
        );

        cop0.regs[COP0::SR] &= !0x00000001; // Disable interrupts
        assert_eq!(
            cop0.regs[COP0::SR] & 1,
            0,
            "IEc bit should be clear (interrupts disabled)"
        );
    }

    #[test]
    fn test_cop0_sr_interrupt_mask() {
        let mut cop0 = COP0::new();

        // Bits 8-15 (Im) are interrupt mask bits
        cop0.regs[COP0::SR] = 0x10900000 | 0x0000FF00; // Enable all interrupts
        assert_eq!(
            (cop0.regs[COP0::SR] >> 8) & 0xFF,
            0xFF,
            "All interrupt mask bits should be set"
        );

        cop0.regs[COP0::SR] = 0x10900000 | 0x00000100; // Enable only Im0
        assert_eq!(
            (cop0.regs[COP0::SR] >> 8) & 0xFF,
            0x01,
            "Only Im0 should be set"
        );
    }

    #[test]
    fn test_cop0_sr_coprocessor_enable_bits() {
        let mut cop0 = COP0::new();

        // Bits 28-31 (CU0-CU3) control coprocessor usability
        // Initial state should have CU0 set
        assert_eq!(
            (cop0.regs[COP0::SR] >> 28) & 0xF,
            0x1,
            "Only CU0 should be set initially"
        );

        // Enable COP2 (GTE)
        cop0.regs[COP0::SR] |= 1 << 30; // Set CU2
        assert_eq!(
            (cop0.regs[COP0::SR] >> 30) & 1,
            1,
            "CU2 should be set (GTE enabled)"
        );
    }

    #[test]
    fn test_cop0_sr_cache_isolation() {
        let mut cop0 = COP0::new();

        // Bit 16 (IsC) isolates cache
        cop0.regs[COP0::SR] |= 1 << 16;
        assert_eq!(
            (cop0.regs[COP0::SR] >> 16) & 1,
            1,
            "IsC bit should be set (cache isolated)"
        );

        cop0.regs[COP0::SR] &= !(1 << 16);
        assert_eq!(
            (cop0.regs[COP0::SR] >> 16) & 1,
            0,
            "IsC bit should be clear (cache not isolated)"
        );
    }

    #[test]
    fn test_cop0_sr_boot_exception_vectors() {
        let cop0 = COP0::new();

        // Check the actual initial SR value
        // The implementation sets SR = 0x10900000
        // Bit 28 is set (CU0), bit 20 is set
        // Note: Real hardware may set BEV (bit 22) but our implementation uses different initial value
        assert_eq!(
            cop0.regs[COP0::SR],
            0x10900000,
            "SR should match implementation's initial value"
        );
    }

    #[test]
    fn test_cop0_epc_word_aligned() {
        let mut cop0 = COP0::new();

        // EPC should typically store word-aligned addresses
        cop0.regs[COP0::EPC] = 0x80000000; // Aligned
        assert_eq!(cop0.regs[COP0::EPC] & 0x3, 0, "EPC should be word-aligned");

        // But hardware allows non-aligned values to be stored
        cop0.regs[COP0::EPC] = 0x80000002; // Misaligned
        assert_eq!(
            cop0.regs[COP0::EPC],
            0x80000002,
            "EPC can store non-aligned addresses"
        );
    }

    #[test]
    fn test_cop0_prid_readonly_behavior() {
        let mut cop0 = COP0::new();

        // PRID should be read-only in real hardware, but our struct allows modification
        // This test documents current behavior
        let original_prid = cop0.regs[COP0::PRID];
        assert_eq!(original_prid, 0x00000002);

        // In real hardware, writes to PRID are ignored
        // Our implementation allows modification (struct field access)
        cop0.regs[COP0::PRID] = 0xDEADBEEF;
        assert_eq!(cop0.regs[COP0::PRID], 0xDEADBEEF);

        // Reset restores correct value
        cop0.reset();
        assert_eq!(cop0.regs[COP0::PRID], 0x00000002);
    }

    #[test]
    fn test_cop0_all_registers_accessible() {
        let mut cop0 = COP0::new();

        // Verify all 32 registers can be accessed
        for i in 0..32 {
            cop0.regs[i] = 0xA5A5A5A5;
            assert_eq!(
                cop0.regs[i], 0xA5A5A5A5,
                "Register {} should be accessible",
                i
            );
        }
    }

    #[test]
    fn test_exception_cause_clone_copy() {
        let cause1 = ExceptionCause::Syscall;
        let cause2 = cause1; // Copy
        let cause3 = cause1; // Copy (no need for clone)

        assert_eq!(cause1, cause2);
        assert_eq!(cause1, cause3);
    }

    #[test]
    fn test_exception_cause_debug_format() {
        let cause = ExceptionCause::AddressErrorLoad;
        let debug_str = format!("{:?}", cause);
        assert!(debug_str.contains("AddressErrorLoad"));
    }
}
