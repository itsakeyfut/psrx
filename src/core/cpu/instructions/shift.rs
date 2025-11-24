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

use super::super::CPU;
use crate::core::error::Result;

impl CPU {
    // === Shift Instructions ===

    /// SLL: Shift Left Logical
    ///
    /// Shifts the value in rt left by shamt bits, storing the result in rd.
    /// Note: SLL with all fields = 0 is NOP.
    ///
    /// Format: sll rd, rt, shamt
    /// Operation: rd = rt << shamt
    ///
    /// # Arguments
    ///
    /// * `rt` - Source register
    /// * `rd` - Destination register
    /// * `shamt` - Shift amount (0-31)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sll(&mut self, rt: u8, rd: u8, shamt: u8) -> Result<()> {
        let value = self.reg(rt) << shamt;
        self.set_reg(rd, value);
        Ok(())
    }

    /// SRL: Shift Right Logical (zero-fill)
    ///
    /// Shifts the value in rt right by shamt bits, filling with zeros.
    ///
    /// Format: srl rd, rt, shamt
    /// Operation: rd = rt >> shamt (zero-fill)
    ///
    /// # Arguments
    ///
    /// * `rt` - Source register
    /// * `rd` - Destination register
    /// * `shamt` - Shift amount (0-31)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_srl(&mut self, rt: u8, rd: u8, shamt: u8) -> Result<()> {
        let result = self.reg(rt) >> shamt;
        self.set_reg(rd, result);
        Ok(())
    }

    /// SRA: Shift Right Arithmetic (sign-extend)
    ///
    /// Shifts the value in rt right by shamt bits, preserving the sign bit.
    ///
    /// Format: sra rd, rt, shamt
    /// Operation: rd = rt >> shamt (sign-extend)
    ///
    /// # Arguments
    ///
    /// * `rt` - Source register
    /// * `rd` - Destination register
    /// * `shamt` - Shift amount (0-31)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sra(&mut self, rt: u8, rd: u8, shamt: u8) -> Result<()> {
        let result = ((self.reg(rt) as i32) >> shamt) as u32;
        self.set_reg(rd, result);
        Ok(())
    }

    /// SLLV: Shift Left Logical Variable
    ///
    /// Shifts the value in rt left by the amount specified in the lower 5 bits of rs.
    ///
    /// Format: sllv rd, rt, rs
    /// Operation: rd = rt << (rs & 0x1F)
    ///
    /// # Arguments
    ///
    /// * `rs` - Register containing shift amount (lower 5 bits used)
    /// * `rt` - Source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_sllv(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let shamt = self.reg(rs) & 0x1F; // Only lower 5 bits
        let result = self.reg(rt) << shamt;
        self.set_reg(rd, result);
        Ok(())
    }

    /// SRLV: Shift Right Logical Variable
    ///
    /// Shifts the value in rt right by the amount specified in the lower 5 bits of rs,
    /// filling with zeros.
    ///
    /// Format: srlv rd, rt, rs
    /// Operation: rd = rt >> (rs & 0x1F)
    ///
    /// # Arguments
    ///
    /// * `rs` - Register containing shift amount (lower 5 bits used)
    /// * `rt` - Source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_srlv(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let shamt = self.reg(rs) & 0x1F;
        let result = self.reg(rt) >> shamt;
        self.set_reg(rd, result);
        Ok(())
    }

    /// SRAV: Shift Right Arithmetic Variable
    ///
    /// Shifts the value in rt right by the amount specified in the lower 5 bits of rs,
    /// preserving the sign bit.
    ///
    /// Format: srav rd, rt, rs
    /// Operation: rd = rt >> (rs & 0x1F) (sign-extend)
    ///
    /// # Arguments
    ///
    /// * `rs` - Register containing shift amount (lower 5 bits used)
    /// * `rt` - Source register
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    pub(crate) fn op_srav(&mut self, rs: u8, rt: u8, rd: u8) -> Result<()> {
        let shamt = self.reg(rs) & 0x1F;
        let result = ((self.reg(rt) as i32) >> shamt) as u32;
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

    // ========== SLL Tests ==========

    #[test]
    fn test_sll_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x00000001);

        cpu.op_sll(1, 2, 4).unwrap();

        assert_eq!(cpu.reg(2), 0x00000010, "SLL: 1 << 4 should equal 16");
    }

    #[test]
    fn test_sll_zero_shift() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);

        cpu.op_sll(1, 2, 0).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x12345678,
            "SLL: shift by 0 should not change value"
        );
    }

    #[test]
    fn test_sll_nop() {
        let mut cpu = create_test_cpu();
        // SLL $0, $0, 0 is the encoding for NOP
        cpu.op_sll(0, 0, 0).unwrap();

        assert_eq!(cpu.reg(0), 0, "NOP should not modify any registers");
    }

    #[test]
    fn test_sll_max_shift() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        cpu.op_sll(1, 2, 31).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x80000000,
            "SLL: max shift should shift to MSB"
        );
    }

    #[test]
    fn test_sll_bits_shifted_out() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000001);

        cpu.op_sll(1, 2, 1).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00000002,
            "SLL: bits shifted out should be lost"
        );
    }

    #[test]
    fn test_sll_to_r0() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        cpu.op_sll(1, 0, 5).unwrap();

        assert_eq!(cpu.reg(0), 0, "SLL to r0 should be ignored");
    }

    #[test]
    fn test_sll_from_r0() {
        let mut cpu = create_test_cpu();

        cpu.op_sll(0, 2, 10).unwrap();

        assert_eq!(cpu.reg(2), 0, "SLL from r0 should give 0");
    }

    // ========== SRL Tests ==========

    #[test]
    fn test_srl_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000);

        cpu.op_srl(1, 2, 4).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x08000000,
            "SRL: 0x80000000 >> 4 should be 0x08000000"
        );
    }

    #[test]
    fn test_srl_zero_shift() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xABCDEF00);

        cpu.op_srl(1, 2, 0).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xABCDEF00,
            "SRL: shift by 0 should not change value"
        );
    }

    #[test]
    fn test_srl_zero_fill() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        cpu.op_srl(1, 2, 1).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x7FFFFFFF,
            "SRL: should fill with zeros, not sign bit"
        );
    }

    #[test]
    fn test_srl_max_shift() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        cpu.op_srl(1, 2, 31).unwrap();

        assert_eq!(cpu.reg(2), 1, "SRL: max shift should leave only LSB");
    }

    #[test]
    fn test_srl_negative_value() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-1i32) as u32); // 0xFFFFFFFF

        cpu.op_srl(1, 2, 8).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x00FFFFFF,
            "SRL: negative values should be zero-filled"
        );
    }

    // ========== SRA Tests ==========

    #[test]
    fn test_sra_positive_value() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x40000000);

        cpu.op_sra(1, 2, 4).unwrap();

        assert_eq!(
            cpu.reg(2),
            0x04000000,
            "SRA: positive value >> 4 should be 0x04000000"
        );
    }

    #[test]
    fn test_sra_negative_value() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // -2147483648

        cpu.op_sra(1, 2, 4).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xF8000000,
            "SRA: negative value should sign-extend"
        );
    }

    #[test]
    fn test_sra_negative_one() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF); // -1

        cpu.op_sra(1, 2, 16).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xFFFFFFFF,
            "SRA: -1 should remain -1 after arithmetic shift"
        );
    }

    #[test]
    fn test_sra_max_shift_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x7FFFFFFF); // Max positive

        cpu.op_sra(1, 2, 31).unwrap();

        assert_eq!(cpu.reg(2), 0, "SRA: max shift of positive should be 0");
    }

    #[test]
    fn test_sra_max_shift_negative() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // Min negative

        cpu.op_sra(1, 2, 31).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xFFFFFFFF,
            "SRA: max shift of negative should be -1"
        );
    }

    #[test]
    fn test_sra_zero_shift() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xABCDEF00);

        cpu.op_sra(1, 2, 0).unwrap();

        assert_eq!(
            cpu.reg(2),
            0xABCDEF00,
            "SRA: shift by 0 should not change value"
        );
    }

    // ========== SLLV Tests ==========

    #[test]
    fn test_sllv_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 4); // shift amount
        cpu.set_reg(2, 0x00000001);

        cpu.op_sllv(1, 2, 3).unwrap();

        assert_eq!(cpu.reg(3), 0x00000010, "SLLV: 1 << 4 should equal 16");
    }

    #[test]
    fn test_sllv_only_lower_5_bits() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 36); // 36 & 0x1F = 4
        cpu.set_reg(2, 0x00000001);

        cpu.op_sllv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x00000010,
            "SLLV: should only use lower 5 bits (36 & 0x1F = 4)"
        );
    }

    #[test]
    fn test_sllv_max_shift_amount() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 31);
        cpu.set_reg(2, 0x00000001);

        cpu.op_sllv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x80000000,
            "SLLV: shift by 31 should move to MSB"
        );
    }

    #[test]
    fn test_sllv_large_shift_value() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF); // All bits set, & 0x1F = 31
        cpu.set_reg(2, 0x00000002);

        cpu.op_sllv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x00000000,
            "SLLV: shift by 31 should shift out all bits except MSB"
        );
    }

    // ========== SRLV Tests ==========

    #[test]
    fn test_srlv_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 4); // shift amount
        cpu.set_reg(2, 0x80000000);

        cpu.op_srlv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x08000000,
            "SRLV: 0x80000000 >> 4 should be 0x08000000"
        );
    }

    #[test]
    fn test_srlv_only_lower_5_bits() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 68); // 68 & 0x1F = 4
        cpu.set_reg(2, 0x80000000);

        cpu.op_srlv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x08000000,
            "SRLV: should only use lower 5 bits (68 & 0x1F = 4)"
        );
    }

    #[test]
    fn test_srlv_zero_fill() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 1);
        cpu.set_reg(2, 0xFFFFFFFF);

        cpu.op_srlv(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x7FFFFFFF,
            "SRLV: should fill with zeros, not sign bit"
        );
    }

    // ========== SRAV Tests ==========

    #[test]
    fn test_srav_basic_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 4); // shift amount
        cpu.set_reg(2, 0x40000000);

        cpu.op_srav(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0x04000000,
            "SRAV: positive value >> 4 should be 0x04000000"
        );
    }

    #[test]
    fn test_srav_basic_negative() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 4); // shift amount
        cpu.set_reg(2, 0x80000000); // -2147483648

        cpu.op_srav(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0xF8000000,
            "SRAV: negative value should sign-extend"
        );
    }

    #[test]
    fn test_srav_only_lower_5_bits() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100); // 100 & 0x1F = 4
        cpu.set_reg(2, 0x80000000);

        cpu.op_srav(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0xF8000000,
            "SRAV: should only use lower 5 bits (100 & 0x1F = 4)"
        );
    }

    #[test]
    fn test_srav_negative_one() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 31);
        cpu.set_reg(2, 0xFFFFFFFF); // -1

        cpu.op_srav(1, 2, 3).unwrap();

        assert_eq!(
            cpu.reg(3),
            0xFFFFFFFF,
            "SRAV: -1 should remain -1 after max shift"
        );
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_all_shift_operations_respect_r0() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);

        cpu.op_sll(1, 0, 5).unwrap();
        assert_eq!(cpu.reg(0), 0, "SLL to r0 should be ignored");

        cpu.op_srl(1, 0, 5).unwrap();
        assert_eq!(cpu.reg(0), 0, "SRL to r0 should be ignored");

        cpu.op_sra(1, 0, 5).unwrap();
        assert_eq!(cpu.reg(0), 0, "SRA to r0 should be ignored");

        cpu.set_reg(2, 5);
        cpu.op_sllv(2, 1, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SLLV to r0 should be ignored");

        cpu.op_srlv(2, 1, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SRLV to r0 should be ignored");

        cpu.op_srav(2, 1, 0).unwrap();
        assert_eq!(cpu.reg(0), 0, "SRAV to r0 should be ignored");
    }

    #[test]
    fn test_shift_boundary_values() {
        let mut cpu = create_test_cpu();

        // Test shift amounts at boundaries
        cpu.set_reg(1, 1);
        cpu.op_sll(1, 2, 0).unwrap();
        assert_eq!(cpu.reg(2), 1, "Shift by 0 should not change value");

        cpu.set_reg(1, 1);
        cpu.op_sll(1, 2, 1).unwrap();
        assert_eq!(cpu.reg(2), 2, "Shift by 1 should double value");

        cpu.set_reg(1, 1);
        cpu.op_sll(1, 2, 31).unwrap();
        assert_eq!(cpu.reg(2), 0x80000000, "Shift by 31 should reach MSB");
    }

    #[test]
    fn test_sra_vs_srl_comparison() {
        let mut cpu = create_test_cpu();
        let negative_value = 0x80000000;

        // SRL with negative value (zero-fill)
        cpu.set_reg(1, negative_value);
        cpu.op_srl(1, 2, 1).unwrap();
        let srl_result = cpu.reg(2);

        // SRA with negative value (sign-extend)
        cpu.set_reg(1, negative_value);
        cpu.op_sra(1, 3, 1).unwrap();
        let sra_result = cpu.reg(3);

        assert_eq!(srl_result, 0x40000000, "SRL should zero-fill");
        assert_eq!(sra_result, 0xC0000000, "SRA should sign-extend");
        assert_ne!(
            srl_result, sra_result,
            "SRL and SRA should differ on negative values"
        );
    }

    #[test]
    fn test_variable_shift_with_zero_amount() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0); // shift amount = 0
        cpu.set_reg(2, 0x12345678);

        cpu.op_sllv(1, 2, 3).unwrap();
        assert_eq!(
            cpu.reg(3),
            0x12345678,
            "SLLV with 0 shift should not change value"
        );

        cpu.op_srlv(1, 2, 4).unwrap();
        assert_eq!(
            cpu.reg(4),
            0x12345678,
            "SRLV with 0 shift should not change value"
        );

        cpu.op_srav(1, 2, 5).unwrap();
        assert_eq!(
            cpu.reg(5),
            0x12345678,
            "SRAV with 0 shift should not change value"
        );
    }
}
