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

use super::super::decode::decode_i_type;
use super::super::CPU;
use crate::core::error::Result;

impl CPU {
    // === Branch Instructions ===

    /// Handle BCONDZ instructions (opcode 0x01)
    ///
    /// BCONDZ instructions include BLTZ, BGEZ, BLTZAL, and BGEZAL.
    /// The rt field determines which specific branch instruction it is.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn execute_bcondz(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let offset = ((imm as i16) as i32) << 2;

        // rt field determines the specific instruction
        // Bit 0: BGEZ (1) vs BLTZ (0)
        // Bit 4: link (1) vs no link (0)
        let is_bgez = (rt & 0x01) != 0;
        let is_link = (rt & 0x10) != 0;

        let test = (self.reg(rs) as i32) >= 0;
        let should_branch = if is_bgez { test } else { !test };

        if is_link {
            // Save return address (BLTZAL or BGEZAL)
            self.set_reg(31, self.next_pc);
        }

        if should_branch {
            self.branch(offset);
        }

        Ok(())
    }

    /// BEQ: Branch on Equal
    ///
    /// Conditional branch if two registers are equal.
    /// The branch target is PC + 4 + (offset << 2).
    ///
    /// Format: beq rs, rt, offset
    /// Operation: if (rs == rt) PC = PC + 4 + (sign_extend(offset) << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_beq(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let offset = ((imm as i16) as i32) << 2;

        if self.reg(rs) == self.reg(rt) {
            self.branch(offset);
        }
        Ok(())
    }

    /// BNE: Branch on Not Equal
    ///
    /// Conditional branch if two registers are not equal.
    ///
    /// Format: bne rs, rt, offset
    /// Operation: if (rs != rt) PC = PC + 4 + (sign_extend(offset) << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_bne(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let offset = ((imm as i16) as i32) << 2;

        if self.reg(rs) != self.reg(rt) {
            self.branch(offset);
        }
        Ok(())
    }

    /// BLEZ: Branch on Less Than or Equal to Zero
    ///
    /// Conditional branch if register is less than or equal to zero (signed).
    ///
    /// Format: blez rs, offset
    /// Operation: if (rs <= 0) PC = PC + 4 + (sign_extend(offset) << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_blez(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, _, imm) = decode_i_type(instruction);
        let offset = ((imm as i16) as i32) << 2;

        if (self.reg(rs) as i32) <= 0 {
            self.branch(offset);
        }
        Ok(())
    }

    /// BGTZ: Branch on Greater Than Zero
    ///
    /// Conditional branch if register is greater than zero (signed).
    ///
    /// Format: bgtz rs, offset
    /// Operation: if (rs > 0) PC = PC + 4 + (sign_extend(offset) << 2)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_bgtz(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, _, imm) = decode_i_type(instruction);
        let offset = ((imm as i16) as i32) << 2;

        if (self.reg(rs) as i32) > 0 {
            self.branch(offset);
        }
        Ok(())
    }

    /// Execute a branch (sets next_pc)
    ///
    /// This helper method is used by branch instructions to update the PC.
    /// The offset is relative to the address of the delay slot.
    ///
    /// # Arguments
    ///
    /// * `offset` - Branch offset in bytes (should be pre-shifted)
    ///
    /// # Note
    ///
    /// The branch target is computed as (B + 4) + offset per MIPS semantics,
    /// where B is the branch instruction address. At the time this function
    /// is called (during execute_instruction), self.pc contains the delay slot
    /// address (B + 4), so we use self.pc as the base for the calculation.
    pub(crate) fn branch(&mut self, offset: i32) {
        // self.pc points to the delay-slot address (B + 4) during execution.
        // Target = (B + 4) + offset
        let base = self.pc;
        self.next_pc = base.wrapping_add(offset as u32);
        self.in_branch_delay = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // Helper to create a branch instruction
    fn make_i_type(op: u8, rs: u8, rt: u8, imm: i16) -> u32 {
        ((op as u32) << 26) | ((rs as u32) << 21) | ((rt as u32) << 16) | ((imm as u16) as u32)
    }

    // ========== BEQ Tests ==========

    #[test]
    fn test_beq_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        // BEQ r1, r2, offset=4 (4 instructions forward = 16 bytes)
        let instruction = make_i_type(0x04, 1, 2, 4);
        cpu.op_beq(instruction).unwrap();

        // Branch target = PC + (4 << 2) = 0x80000000 + 16 = 0x80000010
        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BEQ: should branch when registers are equal"
        );
        assert!(cpu.in_branch_delay, "BEQ: should set branch delay flag");
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 200);

        let instruction = make_i_type(0x04, 1, 2, 4);
        cpu.op_beq(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BEQ: should not branch when registers are not equal"
        );
        assert!(
            !cpu.in_branch_delay,
            "BEQ: should not set branch delay flag when not taken"
        );
    }

    #[test]
    fn test_beq_with_r0() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 0);

        // BEQ r0, r1 (both are 0)
        let instruction = make_i_type(0x04, 0, 1, 8);
        cpu.op_beq(instruction).unwrap();

        // 0x80000000 + (8 << 2) = 0x80000000 + 32 = 0x80000020
        assert_eq!(
            cpu.next_pc, 0x80000020,
            "BEQ: should branch when comparing r0 with zero"
        );
    }

    #[test]
    fn test_beq_negative_offset() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000100;
        cpu.next_pc = 0x80000104;
        cpu.set_reg(1, 42);
        cpu.set_reg(2, 42);

        // BEQ with offset=-4 (backward branch)
        let instruction = make_i_type(0x04, 1, 2, -4);
        cpu.op_beq(instruction).unwrap();

        // Branch target = 0x80000100 + (-4 << 2) = 0x80000100 - 16 = 0x800000F0
        assert_eq!(
            cpu.next_pc, 0x800000F0,
            "BEQ: should handle negative offsets (backward branches)"
        );
    }

    // ========== BNE Tests ==========

    #[test]
    fn test_bne_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 200);

        let instruction = make_i_type(0x05, 1, 2, 4);
        cpu.op_bne(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BNE: should branch when registers are not equal"
        );
        assert!(cpu.in_branch_delay, "BNE: should set branch delay flag");
    }

    #[test]
    fn test_bne_not_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        let instruction = make_i_type(0x05, 1, 2, 4);
        cpu.op_bne(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BNE: should not branch when registers are equal"
        );
        assert!(
            !cpu.in_branch_delay,
            "BNE: should not set branch delay flag"
        );
    }

    // ========== BLEZ Tests ==========

    #[test]
    fn test_blez_taken_negative() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, (-10i32) as u32);

        let instruction = make_i_type(0x06, 1, 0, 4);
        cpu.op_blez(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BLEZ: should branch when register is negative"
        );
    }

    #[test]
    fn test_blez_taken_zero() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 0);

        let instruction = make_i_type(0x06, 1, 0, 4);
        cpu.op_blez(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BLEZ: should branch when register is zero"
        );
    }

    #[test]
    fn test_blez_not_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 10);

        let instruction = make_i_type(0x06, 1, 0, 4);
        cpu.op_blez(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BLEZ: should not branch when register is positive"
        );
    }

    // ========== BGTZ Tests ==========

    #[test]
    fn test_bgtz_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 10);

        let instruction = make_i_type(0x07, 1, 0, 4);
        cpu.op_bgtz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BGTZ: should branch when register is positive"
        );
    }

    #[test]
    fn test_bgtz_not_taken_zero() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 0);

        let instruction = make_i_type(0x07, 1, 0, 4);
        cpu.op_bgtz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BGTZ: should not branch when register is zero"
        );
    }

    #[test]
    fn test_bgtz_not_taken_negative() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, (-10i32) as u32);

        let instruction = make_i_type(0x07, 1, 0, 4);
        cpu.op_bgtz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BGTZ: should not branch when register is negative"
        );
    }

    // ========== BCONDZ Tests (BLTZ, BGEZ, BLTZAL, BGEZAL) ==========

    #[test]
    fn test_bltz_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, (-10i32) as u32);

        // BLTZ: rt=0 (bit 0=0 for BLTZ, bit 4=0 for no link)
        let instruction = make_i_type(0x01, 1, 0, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BLTZ: should branch when register is negative"
        );
    }

    #[test]
    fn test_bltz_not_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 10);

        let instruction = make_i_type(0x01, 1, 0, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BLTZ: should not branch when register is positive"
        );
    }

    #[test]
    fn test_bgez_taken() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 10);

        // BGEZ: rt=1 (bit 0=1 for BGEZ, bit 4=0 for no link)
        let instruction = make_i_type(0x01, 1, 1, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BGEZ: should branch when register is positive"
        );
    }

    #[test]
    fn test_bgez_taken_zero() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 0);

        let instruction = make_i_type(0x01, 1, 1, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BGEZ: should branch when register is zero"
        );
    }

    #[test]
    fn test_bltzal_saves_return_address() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, (-10i32) as u32);

        // BLTZAL: rt=0x10 (bit 0=0 for BLTZ, bit 4=1 for link)
        let instruction = make_i_type(0x01, 1, 0x10, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BLTZAL: should branch when negative"
        );
        assert_eq!(
            cpu.reg(31),
            0x80000004,
            "BLTZAL: should save return address to r31"
        );
    }

    #[test]
    fn test_bgezal_saves_return_address() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 10);

        // BGEZAL: rt=0x11 (bit 0=1 for BGEZ, bit 4=1 for link)
        let instruction = make_i_type(0x01, 1, 0x11, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000010,
            "BGEZAL: should branch when positive"
        );
        assert_eq!(
            cpu.reg(31),
            0x80000004,
            "BGEZAL: should save return address to r31"
        );
    }

    #[test]
    fn test_bgezal_not_taken_but_saves() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, (-10i32) as u32);

        // BGEZAL with negative value (branch not taken, but link still happens)
        let instruction = make_i_type(0x01, 1, 0x11, 4);
        cpu.execute_bcondz(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000004,
            "BGEZAL: should not branch when negative"
        );
        assert_eq!(
            cpu.reg(31),
            0x80000004,
            "BGEZAL: should save return address even when not taken"
        );
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_branch_max_positive_offset() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        // Max positive offset: 0x7FFF (15 bits signed)
        let instruction = make_i_type(0x04, 1, 2, 0x7FFF);
        cpu.op_beq(instruction).unwrap();

        // offset = 0x7FFF << 2 = 0x1FFFC
        // 0x80000000 + 0x1FFFC = 0x8001FFFC
        assert_eq!(cpu.next_pc, 0x8001FFFC, "BEQ: max positive offset");
    }

    #[test]
    fn test_branch_max_negative_offset() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80020000;
        cpu.next_pc = 0x80020004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        // Max negative offset: -0x8000 (15 bits signed)
        let instruction = make_i_type(0x04, 1, 2, -0x8000);
        cpu.op_beq(instruction).unwrap();

        // offset = -0x8000 << 2 = -0x20000
        // 0x80020000 + (-0x20000) = 0x80000000
        assert_eq!(cpu.next_pc, 0x80000000, "BEQ: max negative offset");
    }

    #[test]
    fn test_branch_zero_offset() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        // Zero offset (infinite loop if in delay slot)
        let instruction = make_i_type(0x04, 1, 2, 0);
        cpu.op_beq(instruction).unwrap();

        assert_eq!(
            cpu.next_pc, 0x80000000,
            "BEQ: zero offset should loop to PC"
        );
    }
}
