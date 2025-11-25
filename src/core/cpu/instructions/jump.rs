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

use super::super::decode::decode_j_type;
use super::super::CPU;
use crate::core::error::Result;

impl CPU {
    // === Jump Instructions ===

    /// J: Jump
    ///
    /// Unconditional jump to target address.
    /// The target address is formed by combining the upper 4 bits of PC
    /// with the 26-bit target field shifted left by 2.
    ///
    /// Format: j target
    /// Operation: PC = (PC & 0xF0000000) | (target << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_j(&mut self, instruction: u32) -> Result<()> {
        let (_, target) = decode_j_type(instruction);
        // Upper 4 bits of PC + target << 2
        let pc_high = self.pc & 0xF0000000;
        self.next_pc = pc_high | (target << 2);
        self.in_branch_delay = true;
        Ok(())
    }

    /// JAL: Jump and Link
    ///
    /// Unconditional jump to target address, saving return address in r31.
    /// The return address is the address of the instruction after the delay slot.
    ///
    /// Format: jal target
    /// Operation: r31 = PC + 8; PC = (PC & 0xF0000000) | (target << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_jal(&mut self, instruction: u32) -> Result<()> {
        let (_, target) = decode_j_type(instruction);
        // Save return address to r31 (next_pc already points to delay slot + 4)
        self.set_reg(31, self.next_pc);

        let pc_high = self.pc & 0xF0000000;
        self.next_pc = pc_high | (target << 2);
        self.in_branch_delay = true;
        Ok(())
    }

    /// JR: Jump Register
    ///
    /// Unconditional jump to address in register.
    /// Used for function returns and indirect jumps.
    ///
    /// Format: jr rs
    /// Operation: PC = rs
    ///
    /// # Arguments
    ///
    /// * `rs` - Source register containing target address
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_jr(&mut self, rs: u8) -> Result<()> {
        self.next_pc = self.reg(rs);
        self.in_branch_delay = true;
        Ok(())
    }

    /// JALR: Jump And Link Register
    ///
    /// Unconditional jump to address in register, saving return address.
    /// The return address is saved to register rd (typically r31).
    ///
    /// Format: jalr rs, rd
    /// Operation: rd = PC + 8; PC = rs
    ///
    /// # Arguments
    ///
    /// * `rs` - Source register containing target address
    /// * `rd` - Destination register for return address (default r31 if 0)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_jalr(&mut self, rs: u8, rd: u8) -> Result<()> {
        // Save return address (next_pc already points to delay slot + 4)
        self.set_reg(rd, self.next_pc);
        // Jump to address in rs
        self.next_pc = self.reg(rs);
        self.in_branch_delay = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // ========== J (Jump) Tests ==========

    #[test]
    fn test_j_basic() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // J 0x00001000 (target = 0x1000, shifted = 0x4000)
        // opcode=2, target=0x1000
        let instruction = 0x08001000;
        cpu.op_j(instruction).unwrap();

        // Expected: (PC & 0xF0000000) | (0x1000 << 2) = 0x80000000 | 0x4000 = 0x80004000
        assert_eq!(cpu.next_pc, 0x80004000, "J: should jump to 0x80004000");
        assert!(cpu.in_branch_delay, "J: should set branch delay flag");
    }

    #[test]
    fn test_j_preserves_upper_4_bits() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0xA0123456;
        cpu.next_pc = 0xA012345A;

        // J with target that would change upper bits if not masked
        // 0x0BFFFFFF: opcode=0x02, target=0x3FFFFFF (max 26-bit value)
        let instruction = 0x0BFFFFFF; // target = 0x3FFFFFF, shifted = 0xFFFFFFC
        cpu.op_j(instruction).unwrap();

        // Upper 4 bits (0xA) should be preserved
        assert_eq!(
            cpu.next_pc & 0xF0000000,
            0xA0000000,
            "J: should preserve upper 4 bits of PC"
        );
        assert_eq!(
            cpu.next_pc, 0xAFFFFFFC,
            "J: full address check with preserved upper bits"
        );
    }

    #[test]
    fn test_j_zero_target() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0xBFC00000;
        cpu.next_pc = 0xBFC00004;

        // J 0 (jump to region base)
        let instruction = 0x08000000;
        cpu.op_j(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0xB0000000,
            "J: jump to 0 should go to region base"
        );
    }

    #[test]
    fn test_j_boundary_address() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x00000000;
        cpu.next_pc = 0x00000004;

        // J to max target in 256MB region
        let instruction = 0x0BFFFFFF; // target = 0x3FFFFFF
        cpu.op_j(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x0FFFFFFC,
            "J: should handle max target in region"
        );
    }

    // ========== JAL (Jump and Link) Tests ==========

    #[test]
    fn test_jal_basic() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80001000; // Current PC (actually delay slot address during execution)
        cpu.next_pc = 0x80001004; // Return address

        // JAL 0x00002000 (target = 0x2000, shifted = 0x8000)
        let instruction = 0x0C002000;
        cpu.op_jal(instruction).unwrap();

        assert_eq!(cpu.next_pc, 0x80008000, "JAL: should jump to 0x80008000");
        assert_eq!(
            cpu.reg(31),
            0x80001004,
            "JAL: should save return address to r31"
        );
        assert!(cpu.in_branch_delay, "JAL: should set branch delay flag");
    }

    #[test]
    fn test_jal_preserves_upper_4_bits() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x9F800000;
        cpu.next_pc = 0x9F800004;

        let instruction = 0x0C100000; // JAL 0x100000
        cpu.op_jal(instruction).unwrap();

        assert_eq!(
            cpu.next_pc & 0xF0000000,
            0x90000000,
            "JAL: should preserve upper 4 bits"
        );
    }

    #[test]
    fn test_jal_return_address() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0xBFC00100; // Delay slot address
        cpu.next_pc = 0xBFC00104; // Address after delay slot

        let instruction = 0x0C000500; // JAL some function
        cpu.op_jal(instruction).unwrap();

        // Return address should point to instruction after delay slot
        assert_eq!(
            cpu.reg(31),
            0xBFC00104,
            "JAL: return address should be next_pc"
        );
    }

    #[test]
    fn test_jal_nested_calls() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // First JAL
        let instruction1 = 0x0C000100;
        cpu.op_jal(instruction1).unwrap();
        let first_return = cpu.reg(31);

        // Simulate moving to the jumped location
        cpu.pc = cpu.next_pc;
        cpu.next_pc = cpu.pc + 4;

        // Second JAL (overwrites r31)
        let instruction2 = 0x0C000200;
        cpu.op_jal(instruction2).unwrap();
        let second_return = cpu.reg(31);

        assert_eq!(first_return, 0x80000004, "First JAL return address");
        assert_ne!(
            first_return, second_return,
            "Second JAL should overwrite r31"
        );
    }

    // ========== JR (Jump Register) Tests ==========

    #[test]
    fn test_jr_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(5, 0x80001234);

        cpu.op_jr(5).unwrap();

        assert_eq!(cpu.next_pc, 0x80001234, "JR: should jump to register value");
        assert!(cpu.in_branch_delay, "JR: should set branch delay flag");
    }

    #[test]
    fn test_jr_return_from_function() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(31, 0xBFC00100); // Simulated return address

        cpu.op_jr(31).unwrap();

        assert_eq!(
            cpu.next_pc, 0xBFC00100,
            "JR: should return to saved address"
        );
    }

    #[test]
    fn test_jr_from_r0() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;

        cpu.op_jr(0).unwrap();

        assert_eq!(cpu.next_pc, 0, "JR from r0 should jump to address 0");
    }

    #[test]
    fn test_jr_any_address() {
        let mut cpu = create_test_cpu();

        // JR can jump to any 32-bit address (unlike J which uses 28 bits)
        let target_addresses = [
            0x00000000, 0x80000000, 0xA0000000, 0xBFC00000, 0xFFFFFFFC, 0x12345678,
        ];

        for (i, &addr) in target_addresses.iter().enumerate() {
            cpu.set_reg(10, addr);
            cpu.op_jr(10).unwrap();
            assert_eq!(
                cpu.next_pc, addr,
                "JR test {}: should jump to 0x{:08X}",
                i, addr
            );
        }
    }

    // ========== JALR (Jump and Link Register) Tests ==========

    #[test]
    fn test_jalr_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(5, 0x80002000); // Target address
        cpu.pc = 0x80001000;
        cpu.next_pc = 0x80001004;

        cpu.op_jalr(5, 7).unwrap(); // Jump to r5, save return in r7

        assert_eq!(cpu.next_pc, 0x80002000, "JALR: should jump to r5 value");
        assert_eq!(
            cpu.reg(7),
            0x80001004,
            "JALR: should save return address to r7"
        );
        assert!(cpu.in_branch_delay, "JALR: should set branch delay flag");
    }

    #[test]
    fn test_jalr_to_r31() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(8, 0x90000000);
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        cpu.op_jalr(8, 31).unwrap(); // Save return to r31 (standard)

        assert_eq!(cpu.next_pc, 0x90000000, "JALR: should jump to r8 value");
        assert_eq!(
            cpu.reg(31),
            0x80000004,
            "JALR: should save return address to r31"
        );
    }

    #[test]
    fn test_jalr_same_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(5, 0x80001000);
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // JALR r5, r5 (jump to r5, save return in r5)
        // Current implementation: writes rd first, then reads rs
        // This means if rs==rd, the written value is used for jump
        cpu.op_jalr(5, 5).unwrap();

        // The return address is saved to r5, overwriting the original value
        // Then the jump address is read from r5 (which now has the return address)
        assert_eq!(
            cpu.next_pc, 0x80000004,
            "JALR: current implementation writes rd before reading rs"
        );
        assert_eq!(
            cpu.reg(5),
            0x80000004,
            "JALR: r5 should contain return address"
        );
    }

    #[test]
    fn test_jalr_to_r0() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(5, 0x80001234);
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        cpu.op_jalr(5, 0).unwrap(); // Try to save return to r0 (should be ignored)

        assert_eq!(cpu.next_pc, 0x80001234, "JALR: should still jump correctly");
        assert_eq!(cpu.reg(0), 0, "JALR: write to r0 should be ignored");
    }

    #[test]
    fn test_jalr_from_r0() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        cpu.op_jalr(0, 5).unwrap(); // Jump to r0 (address 0), save return to r5

        assert_eq!(cpu.next_pc, 0, "JALR: should jump to 0 (r0 value)");
        assert_eq!(
            cpu.reg(5),
            0x80000004,
            "JALR: should still save return address"
        );
    }

    #[test]
    fn test_jalr_custom_link_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(10, 0xA0000000);
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // Use r15 as link register (unusual but valid)
        cpu.op_jalr(10, 15).unwrap();

        assert_eq!(cpu.next_pc, 0xA0000000, "JALR: should jump to r10");
        assert_eq!(cpu.reg(15), 0x80000004, "JALR: should save to r15");
        assert_eq!(cpu.reg(31), 0, "JALR: should not modify r31");
    }

    // ========== Edge Cases and Integration Tests ==========

    #[test]
    fn test_all_jump_instructions_set_branch_delay() {
        let mut cpu = create_test_cpu();

        cpu.in_branch_delay = false;
        cpu.op_j(0x08000000).unwrap();
        assert!(cpu.in_branch_delay, "J should set branch delay");

        cpu.in_branch_delay = false;
        cpu.op_jal(0x0C000000).unwrap();
        assert!(cpu.in_branch_delay, "JAL should set branch delay");

        cpu.in_branch_delay = false;
        cpu.set_reg(1, 0x80000000);
        cpu.op_jr(1).unwrap();
        assert!(cpu.in_branch_delay, "JR should set branch delay");

        cpu.in_branch_delay = false;
        cpu.set_reg(2, 0x80000000);
        cpu.op_jalr(2, 31).unwrap();
        assert!(cpu.in_branch_delay, "JALR should set branch delay");
    }

    #[test]
    fn test_jal_jr_function_call_pattern() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000; // Caller
        cpu.next_pc = 0x80000004;

        // JAL to function at 0x80001000
        let jal_inst = 0x0C000400; // target = 0x400 << 2 = 0x1000
        cpu.op_jal(jal_inst).unwrap();

        let function_addr = cpu.next_pc;
        let return_addr = cpu.reg(31);

        assert_eq!(function_addr, 0x80001000, "JAL jumps to function");
        assert_eq!(return_addr, 0x80000004, "JAL saves return address");

        // Simulate function execution, then JR r31 to return
        cpu.pc = function_addr;
        cpu.next_pc = function_addr + 4;
        cpu.op_jr(31).unwrap();

        assert_eq!(cpu.next_pc, return_addr, "JR r31 should return to caller");
    }

    #[test]
    fn test_jump_target_alignment() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // J instruction targets should be 4-byte aligned (target << 2)
        let instruction = 0x08000001; // target = 1, shifted = 4
        cpu.op_j(instruction).unwrap();
        assert_eq!(cpu.next_pc & 0x3, 0, "J target should be 4-byte aligned");

        // JR can jump to any address (even misaligned, though that would cause exception later)
        cpu.set_reg(5, 0x80000001); // Misaligned address
        cpu.op_jr(5).unwrap();
        assert_eq!(
            cpu.next_pc, 0x80000001,
            "JR can set misaligned address (exception happens on execution)"
        );
    }

    #[test]
    fn test_pc_region_boundaries() {
        let mut cpu = create_test_cpu();

        // Test different PC regions for J instruction
        let regions = [0x00000000, 0x80000000, 0xA0000000, 0xF0000000];

        for &region in &regions {
            cpu.pc = region;
            cpu.next_pc = region + 4;

            let instruction = 0x08000100; // target = 0x100, shifted = 0x400
            cpu.op_j(instruction).unwrap();

            assert_eq!(
                cpu.next_pc & 0xF0000000,
                region & 0xF0000000,
                "J should preserve region for PC base 0x{:08X}",
                region
            );
            assert_eq!(
                cpu.next_pc,
                (region & 0xF0000000) | 0x00000400,
                "J target calculation for region 0x{:08X}",
                region
            );
        }
    }
}
