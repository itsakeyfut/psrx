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
use super::super::{ExceptionCause, CPU};
use crate::core::error::Result;

impl CPU {
    // === Arithmetic Instructions ===

    /// ADD: Add (with overflow exception)
    ///
    /// Adds two registers with signed overflow detection.
    /// If overflow occurs, triggers an Overflow exception.
    ///
    /// Format: add rd, rs, rt
    /// Operation: rd = rs + rt
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success (exception is triggered internally on overflow)
    pub(crate) fn op_add(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let a = self.reg(rs) as i32;
        let b = self.reg(rt) as i32;

        match a.checked_add(b) {
            Some(result) => {
                self.set_reg(rd, result as u32);
                Ok(())
            }
            None => {
                self.exception(ExceptionCause::Overflow);
                Ok(())
            }
        }
    }

    /// ADDU: Add Unsigned (no overflow exception)
    ///
    /// Adds two registers without overflow detection.
    /// Overflow wraps around (modulo 2^32).
    ///
    /// Format: addu rd, rs, rt
    /// Operation: rd = rs + rt
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_addu(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = self.reg(rs).wrapping_add(self.reg(rt));
        self.set_reg(rd, result);
        Ok(())
    }

    /// ADDI: Add Immediate (with overflow exception)
    ///
    /// Adds a sign-extended immediate value to a register with overflow detection.
    ///
    /// Format: addi rt, rs, imm
    /// Operation: rt = rs + sign_extend(imm)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_addi(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let imm = (imm as i16) as i32; // Sign extend
        let a = self.reg(rs) as i32;

        match a.checked_add(imm) {
            Some(result) => {
                self.set_reg(rt, result as u32);
                Ok(())
            }
            None => {
                self.exception(ExceptionCause::Overflow);
                Ok(())
            }
        }
    }

    /// ADDIU: Add Immediate Unsigned (no overflow exception)
    ///
    /// Adds a sign-extended immediate value to a register without overflow detection.
    /// Despite the name "unsigned", the immediate is sign-extended.
    ///
    /// Format: addiu rt, rs, imm
    /// Operation: rt = rs + sign_extend(imm)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_addiu(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let imm = (imm as i16) as u32; // Sign extend
        let result = self.reg(rs).wrapping_add(imm);
        self.set_reg(rt, result);
        Ok(())
    }

    /// SUB: Subtract (with overflow exception)
    ///
    /// Subtracts two registers with signed overflow detection.
    ///
    /// Format: sub rd, rs, rt
    /// Operation: rd = rs - rt
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register (minuend)
    /// * `rt` - Second source register (subtrahend)
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sub(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let a = self.reg(rs) as i32;
        let b = self.reg(rt) as i32;

        match a.checked_sub(b) {
            Some(result) => {
                self.set_reg(rd, result as u32);
                Ok(())
            }
            None => {
                self.exception(ExceptionCause::Overflow);
                Ok(())
            }
        }
    }

    /// SUBU: Subtract Unsigned (no overflow exception)
    ///
    /// Subtracts two registers without overflow detection.
    ///
    /// Format: subu rd, rs, rt
    /// Operation: rd = rs - rt
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register (minuend)
    /// * `rt` - Second source register (subtrahend)
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_subu(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = self.reg(rs).wrapping_sub(self.reg(rt));
        self.set_reg(rd, result);
        Ok(())
    }

    /// SLT: Set on Less Than (signed)
    ///
    /// Compares two registers as signed integers.
    /// Sets rd to 1 if rs < rt, otherwise 0.
    ///
    /// Format: slt rd, rs, rt
    /// Operation: rd = (rs < rt) ? 1 : 0
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_slt(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let a = self.reg(rs) as i32;
        let b = self.reg(rt) as i32;
        let result = if a < b { 1 } else { 0 };
        self.set_reg(rd, result);
        Ok(())
    }

    /// SLTU: Set on Less Than Unsigned
    ///
    /// Compares two registers as unsigned integers.
    /// Sets rd to 1 if rs < rt, otherwise 0.
    ///
    /// Format: sltu rd, rs, rt
    /// Operation: rd = (rs < rt) ? 1 : 0
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sltu(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let a = self.reg(rs);
        let b = self.reg(rt);
        let result = if a < b { 1 } else { 0 };
        self.set_reg(rd, result);
        Ok(())
    }

    /// SLTI: Set on Less Than Immediate (signed)
    ///
    /// Compares a register with a sign-extended immediate as signed integers.
    /// Sets rt to 1 if rs < imm, otherwise 0.
    ///
    /// Format: slti rt, rs, imm
    /// Operation: rt = (rs < sign_extend(imm)) ? 1 : 0
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_slti(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let imm = (imm as i16) as i32;
        let a = self.reg(rs) as i32;
        let result = if a < imm { 1 } else { 0 };
        self.set_reg(rt, result);
        Ok(())
    }

    /// SLTIU: Set on Less Than Immediate Unsigned
    ///
    /// Compares a register with a sign-extended immediate as unsigned integers.
    /// Despite the name, the immediate is sign-extended before comparison.
    /// Sets rt to 1 if rs < imm, otherwise 0.
    ///
    /// Format: sltiu rt, rs, imm
    /// Operation: rt = (rs < sign_extend(imm)) ? 1 : 0
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sltiu(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let imm = (imm as i16) as u32; // Sign extend then treat as unsigned
        let a = self.reg(rs);
        let result = if a < imm { 1 } else { 0 };
        self.set_reg(rt, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a CPU instance for testing
    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // ========== ADD Tests ==========

    #[test]
    fn test_add_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 10);
        cpu.set_reg(2, 20);

        cpu.op_add(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 30, "ADD: 10 + 20 should equal 30");
        assert_eq!(cpu.reg(1), 10, "Source register r1 should not change");
        assert_eq!(cpu.reg(2), 20, "Source register r2 should not change");
    }

    #[test]
    fn test_add_negative_numbers() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-10i32) as u32);
        cpu.set_reg(2, (-20i32) as u32);

        cpu.op_add(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3) as i32,
            -30,
            "ADD: (-10) + (-20) should equal -30"
        );
    }

    #[test]
    fn test_add_positive_overflow_triggers_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x7FFFFFFF); // Max positive i32
        cpu.set_reg(2, 1);

        cpu.op_add(1, 2, 3).unwrap();

        // Overflow exception should be triggered, rd should not be updated
        // Based on PSX-SPX: overflow causes exception, PC and EPC are set
        assert!(
            cpu.cop0.regs[13] & 0x7C != 0,
            "Overflow exception should set CAUSE register"
        );
    }

    #[test]
    fn test_add_negative_overflow_triggers_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // Min negative i32
        cpu.set_reg(2, (-1i32) as u32);

        cpu.op_add(1, 2, 3).unwrap();

        assert!(
            cpu.cop0.regs[13] & 0x7C != 0,
            "Negative overflow should trigger exception"
        );
    }

    #[test]
    fn test_add_zero_register_destination() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 42);
        cpu.set_reg(2, 100);

        // Write to r0 (should be ignored per PSX-SPX spec)
        cpu.op_add(1, 2, 0).unwrap();

        assert_eq!(cpu.reg(0), 0, "Register r0 must always be zero");
    }

    #[test]
    fn test_add_from_zero_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(2, 100);

        // Add from r0 (always 0)
        cpu.op_add(0, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 100, "ADD r0 + 100 should equal 100");
    }

    // ========== ADDU Tests ==========

    #[test]
    fn test_addu_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 15);
        cpu.set_reg(2, 25);

        cpu.op_addu(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 40, "ADDU: 15 + 25 should equal 40");
    }

    #[test]
    fn test_addu_overflow_no_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 1);

        cpu.op_addu(1, 2, 3).unwrap();

        // No exception, wraps around (per PSX-SPX spec)
        assert_eq!(
            cpu.reg(3),
            0,
            "ADDU overflow should wrap to 0 without exception"
        );
        assert_eq!(
            cpu.cop0.regs[13] & 0x7C,
            0,
            "ADDU should not trigger exception"
        );
    }

    #[test]
    fn test_addu_large_unsigned_values() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000);
        cpu.set_reg(2, 0x80000000);

        cpu.op_addu(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "ADDU: 0x80000000 + 0x80000000 wraps to 0");
    }

    // ========== ADDI Tests ==========

    #[test]
    fn test_addi_positive_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);

        // ADDI rt=2, rs=1, imm=50 (0x0032)
        let instruction = 0x20220032; // opcode=0x08, rs=1, rt=2, imm=50
        cpu.op_addi(instruction).unwrap();

        assert_eq!(cpu.reg(2), 150, "ADDI: 100 + 50 should equal 150");
    }

    #[test]
    fn test_addi_negative_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);

        // ADDI rt=2, rs=1, imm=-50 (0xFFCE sign-extended)
        let instruction = 0x2022FFCE;
        cpu.op_addi(instruction).unwrap();

        assert_eq!(cpu.reg(2), 50, "ADDI: 100 + (-50) should equal 50");
    }

    #[test]
    fn test_addi_sign_extension() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);

        // ADDI with immediate 0x8000 (should sign-extend to 0xFFFF8000)
        let instruction = 0x20228000;
        cpu.op_addi(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xFFFF8000,
            "ADDI should sign-extend immediate value"
        );
    }

    #[test]
    fn test_addi_overflow_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x7FFFFFFF);

        // ADDI with immediate 1
        let instruction = 0x20220001;
        cpu.op_addi(instruction).unwrap();

        assert!(
            cpu.cop0.regs[13] & 0x7C != 0,
            "ADDI overflow should trigger exception"
        );
    }

    // ========== ADDIU Tests ==========

    #[test]
    fn test_addiu_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 200);

        // ADDIU rt=2, rs=1, imm=100
        let instruction = 0x24220064;
        cpu.op_addiu(instruction).unwrap();

        assert_eq!(cpu.reg(2), 300, "ADDIU: 200 + 100 should equal 300");
    }

    #[test]
    fn test_addiu_overflow_no_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        // ADDIU with immediate 1
        let instruction = 0x24220001;
        cpu.op_addiu(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0,
            "ADDIU overflow should wrap without exception"
        );
        assert_eq!(
            cpu.cop0.regs[13] & 0x7C,
            0,
            "ADDIU should not trigger exception"
        );
    }

    #[test]
    fn test_addiu_sign_extension() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);

        // ADDIU with immediate 0xFFFF (sign-extended to 0xFFFFFFFF = -1)
        let instruction = 0x2422FFFF;
        cpu.op_addiu(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0xFFFFFFFF, "ADDIU should sign-extend immediate");
    }

    // ========== SUB Tests ==========

    #[test]
    fn test_sub_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 30);

        cpu.op_sub(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 70, "SUB: 100 - 30 should equal 70");
    }

    #[test]
    fn test_sub_negative_result() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 30);
        cpu.set_reg(2, 100);

        cpu.op_sub(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3) as i32, -70, "SUB: 30 - 100 should equal -70");
    }

    #[test]
    fn test_sub_overflow_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // Min i32
        cpu.set_reg(2, 1); // Subtracting positive from min causes overflow

        cpu.op_sub(1, 2, 3).unwrap();

        assert!(
            cpu.cop0.regs[13] & 0x7C != 0,
            "SUB overflow should trigger exception"
        );
    }

    // ========== SUBU Tests ==========

    #[test]
    fn test_subu_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 500);
        cpu.set_reg(2, 200);

        cpu.op_subu(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 300, "SUBU: 500 - 200 should equal 300");
    }

    #[test]
    fn test_subu_underflow_no_exception() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);
        cpu.set_reg(2, 1);

        cpu.op_subu(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0xFFFFFFFF,
            "SUBU underflow should wrap to max u32"
        );
        assert_eq!(
            cpu.cop0.regs[13] & 0x7C,
            0,
            "SUBU should not trigger exception"
        );
    }

    // ========== SLT Tests ==========

    #[test]
    fn test_slt_less_than_true() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-10i32) as u32);
        cpu.set_reg(2, 5);

        cpu.op_slt(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 1, "SLT: -10 < 5 should be true (1)");
    }

    #[test]
    fn test_slt_less_than_false() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 10);
        cpu.set_reg(2, 5);

        cpu.op_slt(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "SLT: 10 < 5 should be false (0)");
    }

    #[test]
    fn test_slt_equal() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 42);
        cpu.set_reg(2, 42);

        cpu.op_slt(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "SLT: 42 < 42 should be false (0)");
    }

    #[test]
    fn test_slt_negative_comparison() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-100i32) as u32);
        cpu.set_reg(2, (-50i32) as u32);

        cpu.op_slt(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 1, "SLT: -100 < -50 should be true (1)");
    }

    // ========== SLTU Tests ==========

    #[test]
    fn test_sltu_less_than_true() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 10);
        cpu.set_reg(2, 20);

        cpu.op_sltu(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 1, "SLTU: 10 < 20 should be true (1)");
    }

    #[test]
    fn test_sltu_unsigned_comparison() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-1i32) as u32); // 0xFFFFFFFF (max unsigned)
        cpu.set_reg(2, 1);

        cpu.op_sltu(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0,
            "SLTU: 0xFFFFFFFF < 1 should be false (0) for unsigned"
        );
    }

    #[test]
    fn test_sltu_equal() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 100);

        cpu.op_sltu(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "SLTU: 100 < 100 should be false (0)");
    }

    // ========== SLTI Tests ==========

    #[test]
    fn test_slti_less_than_positive_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 10);

        // SLTI rt=2, rs=1, imm=20
        let instruction = 0x28220014;
        cpu.op_slti(instruction).unwrap();

        assert_eq!(cpu.reg(2), 1, "SLTI: 10 < 20 should be true (1)");
    }

    #[test]
    fn test_slti_less_than_negative_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-50i32) as u32);

        // SLTI rt=2, rs=1, imm=-10 (0xFFF6)
        let instruction = 0x2822FFF6;
        cpu.op_slti(instruction).unwrap();

        assert_eq!(cpu.reg(2), 1, "SLTI: -50 < -10 should be true (1)");
    }

    #[test]
    fn test_slti_greater_than() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);

        // SLTI rt=2, rs=1, imm=50
        let instruction = 0x28220032;
        cpu.op_slti(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0, "SLTI: 100 < 50 should be false (0)");
    }

    // ========== SLTIU Tests ==========

    #[test]
    fn test_sltiu_less_than_true() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 5);

        // SLTIU rt=2, rs=1, imm=10
        let instruction = 0x2C22000A;
        cpu.op_sltiu(instruction).unwrap();

        assert_eq!(cpu.reg(2), 1, "SLTIU: 5 < 10 should be true (1)");
    }

    #[test]
    fn test_sltiu_sign_extended_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 10);

        // SLTIU rt=2, rs=1, imm=0xFFFF (sign-extends to 0xFFFFFFFF)
        let instruction = 0x2C22FFFF;
        cpu.op_sltiu(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            1,
            "SLTIU: 10 < 0xFFFFFFFF should be true (1) for unsigned"
        );
    }

    #[test]
    fn test_sltiu_greater_than() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);

        // SLTIU rt=2, rs=1, imm=50
        let instruction = 0x2C220032;
        cpu.op_sltiu(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0, "SLTIU: 100 < 50 should be false (0)");
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_all_operations_respect_r0_immutability() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 200);

        // Try to write to r0 with various operations
        cpu.op_add(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "ADD to r0 should be ignored");

        cpu.op_addu(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "ADDU to r0 should be ignored");

        cpu.op_sub(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SUB to r0 should be ignored");

        cpu.op_subu(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SUBU to r0 should be ignored");

        cpu.op_slt(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SLT to r0 should be ignored");

        cpu.op_sltu(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SLTU to r0 should be ignored");
    }

    #[test]
    fn test_operations_with_maximum_values() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, u32::MAX);
        cpu.set_reg(2, u32::MAX);

        // ADDU with max values
        cpu.op_addu(1, 2, 3).unwrap();
        assert_eq!(cpu.reg(3), u32::MAX.wrapping_add(u32::MAX));

        // SUBU with max values
        cpu.op_subu(1, 2, 4).unwrap();
        assert_eq!(cpu.reg(4), 0, "max - max should equal 0");
    }
}
