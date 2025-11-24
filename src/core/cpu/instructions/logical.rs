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
    // === Logical Instructions ===

    /// LUI: Load Upper Immediate
    ///
    /// Loads a 16-bit immediate value into the upper 16 bits of a register,
    /// setting the lower 16 bits to 0.
    ///
    /// Format: lui rt, imm
    /// Operation: rt = imm << 16
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_lui(&mut self, instruction: u32) -> Result<()> {
        let (_, _, rt, imm) = decode_i_type(instruction);
        let value = (imm as u32) << 16;
        self.set_reg(rt, value);
        Ok(())
    }

    /// AND: Bitwise AND
    ///
    /// Performs bitwise AND operation on two registers.
    ///
    /// Format: and rd, rs, rt
    /// Operation: rd = rs & rt
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
    pub(crate) fn op_and(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = self.reg(rs) & self.reg(rt);
        self.set_reg(rd, result);
        Ok(())
    }

    /// ANDI: AND Immediate (zero-extended)
    ///
    /// Performs bitwise AND operation with a zero-extended immediate value.
    /// Note: Unlike ADDI, the immediate is ZERO-extended, not sign-extended.
    ///
    /// Format: andi rt, rs, imm
    /// Operation: rt = rs & zero_extend(imm)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_andi(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let result = self.reg(rs) & (imm as u32); // Zero extend
        self.set_reg(rt, result);
        Ok(())
    }

    /// OR: Bitwise OR
    ///
    /// Performs bitwise OR operation on two registers.
    ///
    /// Format: or rd, rs, rt
    /// Operation: rd = rs | rt
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
    pub(crate) fn op_or(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = self.reg(rs) | self.reg(rt);
        self.set_reg(rd, result);
        Ok(())
    }

    /// ORI: OR Immediate (zero-extended)
    ///
    /// Performs bitwise OR operation with a zero-extended immediate value.
    ///
    /// Format: ori rt, rs, imm
    /// Operation: rt = rs | zero_extend(imm)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_ori(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let result = self.reg(rs) | (imm as u32);
        self.set_reg(rt, result);
        Ok(())
    }

    /// XOR: Bitwise XOR
    ///
    /// Performs bitwise XOR operation on two registers.
    ///
    /// Format: xor rd, rs, rt
    /// Operation: rd = rs ^ rt
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
    pub(crate) fn op_xor(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = self.reg(rs) ^ self.reg(rt);
        self.set_reg(rd, result);
        Ok(())
    }

    /// XORI: XOR Immediate (zero-extended)
    ///
    /// Performs bitwise XOR operation with a zero-extended immediate value.
    ///
    /// Format: xori rt, rs, imm
    /// Operation: rt = rs ^ zero_extend(imm)
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_xori(&mut self, instruction: u32) -> Result<()> {
        let (_, rs, rt, imm) = decode_i_type(instruction);
        let result = self.reg(rs) ^ (imm as u32);
        self.set_reg(rt, result);
        Ok(())
    }

    /// NOR: Bitwise NOR (NOT OR)
    ///
    /// Performs bitwise NOR operation on two registers.
    /// This is equivalent to NOT(rs OR rt).
    ///
    /// Format: nor rd, rs, rt
    /// Operation: rd = ~(rs | rt)
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
    pub(crate) fn op_nor(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let result = !(self.reg(rs) | self.reg(rt));
        self.set_reg(rd, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // ========== LUI Tests ==========

    #[test]
    fn test_lui_basic() {
        let mut cpu = create_test_cpu();

        // LUI rt=1, imm=0x1234
        let instruction = 0x3C011234;
        cpu.op_lui(instruction).unwrap();

        assert_eq!(cpu.reg(1), 0x12340000, "LUI should load upper 16 bits");
    }

    #[test]
    fn test_lui_lower_bits_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        // LUI rt=1, imm=0xABCD
        let instruction = 0x3C01ABCD;
        cpu.op_lui(instruction).unwrap();

        assert_eq!(
            cpu.reg(1),
            0xABCD0000,
            "LUI should clear lower 16 bits to zero"
        );
    }

    #[test]
    fn test_lui_zero_immediate() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        // LUI rt=1, imm=0x0000
        let instruction = 0x3C010000;
        cpu.op_lui(instruction).unwrap();

        assert_eq!(cpu.reg(1), 0, "LUI with immediate 0 should clear register");
    }

    #[test]
    fn test_lui_to_r0() {
        let mut cpu = create_test_cpu();

        // LUI rt=0, imm=0xFFFF (should be ignored)
        let instruction = 0x3C00FFFF;
        cpu.op_lui(instruction).unwrap();

        assert_eq!(cpu.reg(0), 0, "LUI to r0 should be ignored");
    }

    // ========== AND Tests ==========

    #[test]
    fn test_and_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0b11110000);
        cpu.set_reg(2, 0b11001100);

        cpu.op_and(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0b11000000,
            "AND should perform bitwise AND operation"
        );
    }

    #[test]
    fn test_and_all_bits_set() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 0x12345678);

        cpu.op_and(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0x12345678, "AND with all 1s should return rt");
    }

    #[test]
    fn test_and_with_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0);

        cpu.op_and(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "AND with 0 should return 0");
    }

    #[test]
    fn test_and_with_r0() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        // AND with r0 (always 0)
        cpu.op_and(1, 0, 2).unwrap();

        assert_eq!(cpu.reg(2), 0, "AND with r0 should return 0");
    }

    #[test]
    fn test_and_same_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        cpu.op_and(1, 1, 2).unwrap();

        assert_eq!(cpu.reg(2), 0x12345678, "AND r1, r1 should return r1 value");
    }

    // ========== ANDI Tests ==========

    #[test]
    fn test_andi_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        // ANDI rt=2, rs=1, imm=0x00FF
        let instruction = 0x302200FF;
        cpu.op_andi(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0x000000FF, "ANDI should mask lower bits");
    }

    #[test]
    fn test_andi_zero_extension() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        // ANDI rt=2, rs=1, imm=0x8000 (should be zero-extended to 0x00008000)
        let instruction = 0x30228000;
        cpu.op_andi(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00008000,
            "ANDI should zero-extend immediate, not sign-extend"
        );
    }

    #[test]
    fn test_andi_clear_upper_bits() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        // ANDI rt=2, rs=1, imm=0x0FFF
        let instruction = 0x30220FFF;
        cpu.op_andi(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00000678,
            "ANDI should clear upper bits not in mask"
        );
    }

    // ========== OR Tests ==========

    #[test]
    fn test_or_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0b11110000);
        cpu.set_reg(2, 0b11001100);

        cpu.op_or(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0b11111100,
            "OR should perform bitwise OR operation"
        );
    }

    #[test]
    fn test_or_with_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0);

        cpu.op_or(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0x12345678, "OR with 0 should return original");
    }

    #[test]
    fn test_or_with_r0_is_move() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xDEADBEEF);

        // OR with r0 is commonly used as MOVE instruction
        cpu.op_or(1, 0, 2).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xDEADBEEF,
            "OR with r0 can be used as MOVE instruction"
        );
    }

    #[test]
    fn test_or_all_ones() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0xFFFFFFFF);

        cpu.op_or(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0xFFFFFFFF, "OR with all 1s should return all 1s");
    }

    // ========== ORI Tests ==========

    #[test]
    fn test_ori_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12340000);

        // ORI rt=2, rs=1, imm=0x5678
        let instruction = 0x34225678;
        cpu.op_ori(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0x12345678, "ORI should combine bits with OR");
    }

    #[test]
    fn test_ori_zero_extension() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);

        // ORI rt=2, rs=1, imm=0x8000 (should be zero-extended to 0x00008000)
        let instruction = 0x34228000;
        cpu.op_ori(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00008000,
            "ORI should zero-extend immediate, not sign-extend"
        );
    }

    #[test]
    fn test_ori_with_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xABCDEF00);

        // ORI rt=2, rs=1, imm=0x0000
        let instruction = 0x34220000;
        cpu.op_ori(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0xABCDEF00, "ORI with 0 should not change value");
    }

    // ========== XOR Tests ==========

    #[test]
    fn test_xor_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0b11110000);
        cpu.set_reg(2, 0b11001100);

        cpu.op_xor(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0b00111100,
            "XOR should perform bitwise XOR operation"
        );
    }

    #[test]
    fn test_xor_with_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0);

        cpu.op_xor(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0x12345678, "XOR with 0 should return original");
    }

    #[test]
    fn test_xor_same_value() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        cpu.op_xor(1, 1, 2).unwrap();

        assert_eq!(cpu.reg(2), 0, "XOR with self should return 0");
    }

    #[test]
    fn test_xor_with_all_ones() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0xFFFFFFFF);

        cpu.op_xor(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            !0x12345678,
            "XOR with all 1s should return bitwise NOT"
        );
    }

    // ========== XORI Tests ==========

    #[test]
    fn test_xori_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFF0000);

        // XORI rt=2, rs=1, imm=0xFFFF
        let instruction = 0x3822FFFF;
        cpu.op_xori(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0xFFFFFFFF, "XORI should toggle bits");
    }

    #[test]
    fn test_xori_zero_extension() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);

        // XORI rt=2, rs=1, imm=0x8000 (should be zero-extended to 0x00008000)
        let instruction = 0x38228000;
        cpu.op_xori(instruction).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00008000,
            "XORI should zero-extend immediate, not sign-extend"
        );
    }

    #[test]
    fn test_xori_toggle_bits() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        // XORI rt=2, rs=1, imm=0x00FF
        let instruction = 0x382200FF;
        cpu.op_xori(instruction).unwrap();

        assert_eq!(cpu.reg(2), 0x12345687, "XORI should toggle lower 8 bits");
    }

    // ========== NOR Tests ==========

    #[test]
    fn test_nor_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0b11110000);
        cpu.set_reg(2, 0b11001100);

        cpu.op_nor(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            !0b11111100,
            "NOR should perform bitwise NOR operation"
        );
    }

    #[test]
    fn test_nor_with_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 0);

        cpu.op_nor(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            !0x12345678,
            "NOR with 0 should return bitwise NOT"
        );
    }

    #[test]
    fn test_nor_with_r0_is_not() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        // NOR with r0 is commonly used as NOT instruction
        cpu.op_nor(1, 0, 2).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xEDCBA987,
            "NOR with r0 can be used as NOT instruction"
        );
    }

    #[test]
    fn test_nor_all_ones() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 0xFFFFFFFF);

        cpu.op_nor(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0, "NOR of all 1s should return 0");
    }

    #[test]
    fn test_nor_both_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0);
        cpu.set_reg(2, 0);

        cpu.op_nor(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0xFFFFFFFF, "NOR of both 0 should return all 1s");
    }

    // ========== Edge Cases and Special Patterns ==========

    #[test]
    fn test_all_logical_operations_respect_r0() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 0xFFFFFFFF);

        // Try to write to r0 with various operations
        cpu.op_and(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "AND to r0 should be ignored");

        cpu.op_or(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "OR to r0 should be ignored");

        cpu.op_xor(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "XOR to r0 should be ignored");

        cpu.op_nor(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "NOR to r0 should be ignored");

        let lui_inst = 0x3C00FFFF; // LUI r0, 0xFFFF
        cpu.op_lui(lui_inst).unwrap();
        assert_eq!(cpu.reg(0), 0, "LUI to r0 should be ignored");
    }

    #[test]
    fn test_logical_operation_combinations() {
        let mut cpu = create_test_cpu();
        let value1 = 0xAAAAAAAA; // 10101010...
        let value2 = 0x55555555; // 01010101...

        cpu.set_reg(1, value1);
        cpu.set_reg(2, value2);

        // AND should give 0
        cpu.op_and(1, 2, 3).unwrap();
        assert_eq!(cpu.reg(3), 0, "AND of alternating bits should be 0");

        // OR should give all 1s
        cpu.op_or(1, 2, 4).unwrap();
        assert_eq!(
            cpu.reg(4),
            0xFFFFFFFF,
            "OR of complementary bits should be all 1s"
        );

        // XOR should give all 1s
        cpu.op_xor(1, 2, 5).unwrap();
        assert_eq!(
            cpu.reg(5),
            0xFFFFFFFF,
            "XOR of complementary bits should be all 1s"
        );

        // NOR should give 0
        cpu.op_nor(1, 2, 6).unwrap();
        assert_eq!(cpu.reg(6), 0, "NOR of complementary bits should be 0");
    }

    #[test]
    fn test_immediate_zero_extension_vs_sign_extension() {
        let mut cpu = create_test_cpu();

        // Test ANDI with 0x8000 (should zero-extend)
        // Set r1 to all 1s so AND shows the mask pattern
        cpu.set_reg(1, 0xFFFFFFFF);
        let andi_inst = 0x30228000;
        cpu.op_andi(andi_inst).unwrap();
        assert_eq!(
            cpu.reg(2),
            0x00008000,
            "ANDI should zero-extend 0x8000 to 0x00008000"
        );

        // Test ORI with 0x8000 (should zero-extend)
        cpu.set_reg(1, 0);
        let ori_inst = 0x34238000;
        cpu.op_ori(ori_inst).unwrap();
        assert_eq!(
            cpu.reg(3),
            0x00008000,
            "ORI should zero-extend 0x8000 to 0x00008000"
        );

        // Test XORI with 0x8000 (should zero-extend)
        cpu.set_reg(1, 0);
        let xori_inst = 0x38248000;
        cpu.op_xori(xori_inst).unwrap();
        assert_eq!(
            cpu.reg(4),
            0x00008000,
            "XORI should zero-extend 0x8000 to 0x00008000"
        );
    }

    #[test]
    fn test_masking_patterns() {
        let mut cpu = create_test_cpu();
        let value = 0x12345678;
        cpu.set_reg(1, value);

        // Extract lower byte (0x78)
        let andi_inst = 0x302200FF;
        cpu.op_andi(andi_inst).unwrap();
        assert_eq!(cpu.reg(2), 0x78, "Should extract lower byte");

        // Extract lower 16 bits (0x5678)
        cpu.set_reg(3, value);
        let andi_inst2 = 0x3063FFFF;
        cpu.op_andi(andi_inst2).unwrap();
        assert_eq!(cpu.reg(3), 0x5678, "Should extract lower 16 bits");
    }
}
