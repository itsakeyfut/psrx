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
    // === Multiply/Divide Instructions ===

    /// MULT: Multiply (signed)
    ///
    /// Multiplies two 32-bit signed integers and stores the 64-bit result
    /// in the HI and LO registers.
    ///
    /// Format: mult rs, rt
    /// Operation: (HI, LO) = rs * rt (signed 64-bit result)
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply 100 * 200 = 20000
    /// // LO = 20000 (0x4E20), HI = 0
    /// cpu.set_reg(1, 100);
    /// cpu.set_reg(2, 200);
    /// cpu.op_mult(1, 2);
    /// ```
    pub(crate) fn op_mult(&mut self, rs: u8, rt: u8) -> Result<()> {
        let a = self.reg(rs) as i32 as i64;
        let b = self.reg(rt) as i32 as i64;
        let result = a * b;

        self.lo = result as u32;
        self.hi = (result >> 32) as u32;
        Ok(())
    }

    /// MULTU: Multiply Unsigned
    ///
    /// Multiplies two 32-bit unsigned integers and stores the 64-bit result
    /// in the HI and LO registers.
    ///
    /// Format: multu rs, rt
    /// Operation: (HI, LO) = rs * rt (unsigned 64-bit result)
    ///
    /// # Arguments
    ///
    /// * `rs` - First source register
    /// * `rt` - Second source register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply 0xFFFFFFFF * 2
    /// // Result = 0x1FFFFFFFE
    /// // LO = 0xFFFFFFFE, HI = 1
    /// cpu.set_reg(1, 0xFFFFFFFF);
    /// cpu.set_reg(2, 2);
    /// cpu.op_multu(1, 2);
    /// ```
    pub(crate) fn op_multu(&mut self, rs: u8, rt: u8) -> Result<()> {
        let a = self.reg(rs) as u64;
        let b = self.reg(rt) as u64;
        let result = a * b;

        self.lo = result as u32;
        self.hi = (result >> 32) as u32;
        Ok(())
    }

    /// DIV: Divide (signed)
    ///
    /// Divides two 32-bit signed integers and stores quotient in LO
    /// and remainder in HI.
    ///
    /// Format: div rs, rt
    /// Operation: LO = rs / rt (quotient), HI = rs % rt (remainder)
    ///
    /// # Arguments
    ///
    /// * `rs` - Dividend register
    /// * `rt` - Divisor register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Special Cases
    ///
    /// * Division by zero: LO = 0xFFFFFFFF or 1 (based on sign), HI = numerator
    /// * Overflow (0x80000000 / -1): LO = 0x80000000, HI = 0
    ///
    /// # Example
    ///
    /// ```ignore
    /// // 100 / 7 = 14 remainder 2
    /// cpu.set_reg(1, 100);
    /// cpu.set_reg(2, 7);
    /// cpu.op_div(1, 2);
    /// // LO = 14, HI = 2
    /// ```
    pub(crate) fn op_div(&mut self, rs: u8, rt: u8) -> Result<()> {
        let numerator = self.reg(rs) as i32;
        let denominator = self.reg(rt) as i32;

        if denominator == 0 {
            // PSX doesn't trap on divide by zero
            // Result is undefined but follows a pattern
            self.lo = if numerator >= 0 { 0xFFFFFFFF } else { 1 };
            self.hi = numerator as u32;
        } else if numerator as u32 == 0x80000000 && denominator == -1 {
            // Overflow case: i32::MIN / -1
            self.lo = 0x80000000;
            self.hi = 0;
        } else {
            self.lo = (numerator / denominator) as u32;
            self.hi = (numerator % denominator) as u32;
        }
        Ok(())
    }

    /// DIVU: Divide Unsigned
    ///
    /// Divides two 32-bit unsigned integers and stores quotient in LO
    /// and remainder in HI.
    ///
    /// Format: divu rs, rt
    /// Operation: LO = rs / rt (quotient), HI = rs % rt (remainder)
    ///
    /// # Arguments
    ///
    /// * `rs` - Dividend register
    /// * `rt` - Divisor register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Special Cases
    ///
    /// * Division by zero: LO = 0xFFFFFFFF, HI = numerator
    ///
    /// # Example
    ///
    /// ```ignore
    /// // 100 / 7 = 14 remainder 2
    /// cpu.set_reg(1, 100);
    /// cpu.set_reg(2, 7);
    /// cpu.op_divu(1, 2);
    /// // LO = 14, HI = 2
    /// ```
    pub(crate) fn op_divu(&mut self, rs: u8, rt: u8) -> Result<()> {
        let numerator = self.reg(rs);
        let denominator = self.reg(rt);

        if denominator == 0 {
            // PSX doesn't trap on divide by zero
            self.lo = 0xFFFFFFFF;
            self.hi = numerator;
        } else {
            self.lo = numerator / denominator;
            self.hi = numerator % denominator;
        }
        Ok(())
    }

    /// MFHI: Move From HI
    ///
    /// Copies the value from the HI register to a general-purpose register.
    ///
    /// Format: mfhi rd
    /// Operation: rd = HI
    ///
    /// # Arguments
    ///
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// cpu.hi = 0x12345678;
    /// cpu.op_mfhi(3);
    /// assert_eq!(cpu.reg(3), 0x12345678);
    /// ```
    pub(crate) fn op_mfhi(&mut self, rd: u8) -> Result<()> {
        self.set_reg(rd, self.hi);
        Ok(())
    }

    /// MFLO: Move From LO
    ///
    /// Copies the value from the LO register to a general-purpose register.
    ///
    /// Format: mflo rd
    /// Operation: rd = LO
    ///
    /// # Arguments
    ///
    /// * `rd` - Destination register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// cpu.lo = 0xABCDEF00;
    /// cpu.op_mflo(4);
    /// assert_eq!(cpu.reg(4), 0xABCDEF00);
    /// ```
    pub(crate) fn op_mflo(&mut self, rd: u8) -> Result<()> {
        self.set_reg(rd, self.lo);
        Ok(())
    }

    /// MTHI: Move To HI
    ///
    /// Copies the value from a general-purpose register to the HI register.
    ///
    /// Format: mthi rs
    /// Operation: HI = rs
    ///
    /// # Arguments
    ///
    /// * `rs` - Source register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// cpu.set_reg(5, 0x12345678);
    /// cpu.op_mthi(5);
    /// assert_eq!(cpu.hi, 0x12345678);
    /// ```
    pub(crate) fn op_mthi(&mut self, rs: u8) -> Result<()> {
        self.hi = self.reg(rs);
        Ok(())
    }

    /// MTLO: Move To LO
    ///
    /// Copies the value from a general-purpose register to the LO register.
    ///
    /// Format: mtlo rs
    /// Operation: LO = rs
    ///
    /// # Arguments
    ///
    /// * `rs` - Source register
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Example
    ///
    /// ```ignore
    /// cpu.set_reg(6, 0xABCDEF00);
    /// cpu.op_mtlo(6);
    /// assert_eq!(cpu.lo, 0xABCDEF00);
    /// ```
    pub(crate) fn op_mtlo(&mut self, rs: u8) -> Result<()> {
        self.lo = self.reg(rs);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // ========== MULT Tests ==========

    #[test]
    fn test_mult_basic_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 200);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(cpu.lo, 20000, "MULT: 100 * 200 should give LO=20000");
        assert_eq!(cpu.hi, 0, "MULT: no overflow, HI should be 0");
    }

    #[test]
    fn test_mult_negative_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-10i32) as u32);
        cpu.set_reg(2, 20);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(cpu.lo, (-200i32) as u32, "MULT: -10 * 20 should give -200");
        assert_eq!(cpu.hi, 0xFFFFFFFF, "MULT: negative result, HI should be -1");
    }

    #[test]
    fn test_mult_both_negative() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-10i32) as u32);
        cpu.set_reg(2, (-20i32) as u32);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(cpu.lo, 200, "MULT: -10 * -20 should give 200");
        assert_eq!(cpu.hi, 0, "MULT: positive result, HI should be 0");
    }

    #[test]
    fn test_mult_overflow_to_hi() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // -2147483648
        cpu.set_reg(2, 2);

        cpu.op_mult(1, 2).unwrap();

        // -2147483648 * 2 = -4294967296 = 0xFFFFFFFF00000000
        assert_eq!(cpu.lo, 0, "MULT: overflow result LO should be 0");
        assert_eq!(cpu.hi, 0xFFFFFFFF, "MULT: overflow result HI should be -1");
    }

    #[test]
    fn test_mult_large_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x7FFFFFFF); // Max positive i32
        cpu.set_reg(2, 2);

        cpu.op_mult(1, 2).unwrap();

        // 2147483647 * 2 = 4294967294 = 0xFFFFFFFE
        assert_eq!(cpu.lo, 0xFFFFFFFE, "MULT: large multiplication LO");
        assert_eq!(cpu.hi, 0, "MULT: large multiplication HI");
    }

    #[test]
    fn test_mult_by_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 12345);
        cpu.set_reg(2, 0);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(cpu.lo, 0, "MULT: multiplication by 0 should give 0");
        assert_eq!(cpu.hi, 0, "MULT: HI should also be 0");
    }

    #[test]
    fn test_mult_by_one() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x12345678);
        cpu.set_reg(2, 1);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(cpu.lo, 0x12345678, "MULT: multiplication by 1 unchanged");
        assert_eq!(cpu.hi, 0, "MULT: HI should be 0");
    }

    #[test]
    fn test_mult_by_negative_one() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, (-1i32) as u32);

        cpu.op_mult(1, 2).unwrap();

        assert_eq!(
            cpu.lo,
            (-100i32) as u32,
            "MULT: multiplication by -1 should negate"
        );
        assert_eq!(cpu.hi, 0xFFFFFFFF, "MULT: HI should be -1");
    }

    // ========== MULTU Tests ==========

    #[test]
    fn test_multu_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 200);

        cpu.op_multu(1, 2).unwrap();

        assert_eq!(cpu.lo, 20000, "MULTU: 100 * 200 should give 20000");
        assert_eq!(cpu.hi, 0, "MULTU: no overflow, HI should be 0");
    }

    #[test]
    fn test_multu_large_values() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF); // Max u32
        cpu.set_reg(2, 2);

        cpu.op_multu(1, 2).unwrap();

        // 0xFFFFFFFF * 2 = 0x1FFFFFFFE
        assert_eq!(cpu.lo, 0xFFFFFFFE, "MULTU: large multiplication LO");
        assert_eq!(cpu.hi, 1, "MULTU: large multiplication HI");
    }

    #[test]
    fn test_multu_max_values() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 0xFFFFFFFF);

        cpu.op_multu(1, 2).unwrap();

        // 0xFFFFFFFF * 0xFFFFFFFF = 0xFFFFFFFE00000001
        assert_eq!(
            cpu.lo, 0x00000001,
            "MULTU: max * max should give LO=0x00000001"
        );
        assert_eq!(
            cpu.hi, 0xFFFFFFFE,
            "MULTU: max * max should give HI=0xFFFFFFFE"
        );
    }

    #[test]
    fn test_multu_by_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF);
        cpu.set_reg(2, 0);

        cpu.op_multu(1, 2).unwrap();

        assert_eq!(cpu.lo, 0, "MULTU: multiplication by 0 should give 0");
        assert_eq!(cpu.hi, 0, "MULTU: HI should also be 0");
    }

    #[test]
    fn test_multu_vs_mult() {
        let mut cpu = create_test_cpu();
        let value = 0x80000000; // -2147483648 as signed, large positive as unsigned

        // MULT (signed)
        cpu.set_reg(1, value);
        cpu.set_reg(2, 2);
        cpu.op_mult(1, 2).unwrap();
        let mult_lo = cpu.lo;
        let mult_hi = cpu.hi;

        // MULTU (unsigned)
        cpu.set_reg(1, value);
        cpu.set_reg(2, 2);
        cpu.op_multu(1, 2).unwrap();
        let multu_lo = cpu.lo;
        let multu_hi = cpu.hi;

        assert_eq!(mult_lo, 0, "MULT: signed result LO");
        assert_eq!(mult_hi, 0xFFFFFFFF, "MULT: signed result HI");
        assert_eq!(multu_lo, 0, "MULTU: unsigned result LO");
        assert_eq!(multu_hi, 1, "MULTU: unsigned result HI");
        assert_ne!(
            mult_hi, multu_hi,
            "MULT and MULTU should differ on negative values"
        );
    }

    // ========== DIV Tests ==========

    #[test]
    fn test_div_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 7);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(cpu.lo, 14, "DIV: 100 / 7 should give quotient 14");
        assert_eq!(cpu.hi, 2, "DIV: 100 % 7 should give remainder 2");
    }

    #[test]
    fn test_div_exact() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 10);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(cpu.lo, 10, "DIV: 100 / 10 should give quotient 10");
        assert_eq!(cpu.hi, 0, "DIV: 100 % 10 should give remainder 0");
    }

    #[test]
    fn test_div_negative_dividend() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-100i32) as u32);
        cpu.set_reg(2, 7);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(
            cpu.lo,
            (-14i32) as u32,
            "DIV: -100 / 7 should give quotient -14"
        );
        assert_eq!(
            cpu.hi,
            (-2i32) as u32,
            "DIV: -100 % 7 should give remainder -2"
        );
    }

    #[test]
    fn test_div_negative_divisor() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, (-7i32) as u32);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(
            cpu.lo,
            (-14i32) as u32,
            "DIV: 100 / -7 should give quotient -14"
        );
        assert_eq!(cpu.hi, 2, "DIV: 100 % -7 should give remainder 2");
    }

    #[test]
    fn test_div_both_negative() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-100i32) as u32);
        cpu.set_reg(2, (-7i32) as u32);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(cpu.lo, 14, "DIV: -100 / -7 should give quotient 14");
        assert_eq!(
            cpu.hi,
            (-2i32) as u32,
            "DIV: -100 % -7 should give remainder -2"
        );
    }

    #[test]
    fn test_div_by_zero_positive() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 0);

        cpu.op_div(1, 2).unwrap();

        // PSX-SPX: division by zero gives LO=0xFFFFFFFF (if positive), HI=numerator
        assert_eq!(
            cpu.lo, 0xFFFFFFFF,
            "DIV: positive / 0 should give LO=0xFFFFFFFF"
        );
        assert_eq!(cpu.hi, 100, "DIV: division by 0 should leave HI=numerator");
    }

    #[test]
    fn test_div_by_zero_negative() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, (-100i32) as u32);
        cpu.set_reg(2, 0);

        cpu.op_div(1, 2).unwrap();

        // PSX-SPX: division by zero gives LO=1 (if negative), HI=numerator
        assert_eq!(cpu.lo, 1, "DIV: negative / 0 should give LO=1");
        assert_eq!(
            cpu.hi,
            (-100i32) as u32,
            "DIV: division by 0 should leave HI=numerator"
        );
    }

    #[test]
    fn test_div_overflow() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x80000000); // i32::MIN
        cpu.set_reg(2, (-1i32) as u32);

        cpu.op_div(1, 2).unwrap();

        // PSX-SPX: i32::MIN / -1 overflow gives LO=i32::MIN, HI=0
        assert_eq!(
            cpu.lo, 0x80000000,
            "DIV: i32::MIN / -1 should give LO=i32::MIN"
        );
        assert_eq!(cpu.hi, 0, "DIV: overflow should give HI=0");
    }

    #[test]
    fn test_div_by_one() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 12345);
        cpu.set_reg(2, 1);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(cpu.lo, 12345, "DIV: division by 1 should give original");
        assert_eq!(cpu.hi, 0, "DIV: remainder should be 0");
    }

    #[test]
    fn test_div_smaller_than_divisor() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 5);
        cpu.set_reg(2, 10);

        cpu.op_div(1, 2).unwrap();

        assert_eq!(cpu.lo, 0, "DIV: 5 / 10 should give quotient 0");
        assert_eq!(cpu.hi, 5, "DIV: 5 % 10 should give remainder 5");
    }

    // ========== DIVU Tests ==========

    #[test]
    fn test_divu_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 7);

        cpu.op_divu(1, 2).unwrap();

        assert_eq!(cpu.lo, 14, "DIVU: 100 / 7 should give quotient 14");
        assert_eq!(cpu.hi, 2, "DIVU: 100 % 7 should give remainder 2");
    }

    #[test]
    fn test_divu_large_values() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF); // Max u32
        cpu.set_reg(2, 2);

        cpu.op_divu(1, 2).unwrap();

        assert_eq!(
            cpu.lo, 0x7FFFFFFF,
            "DIVU: 0xFFFFFFFF / 2 should give 0x7FFFFFFF"
        );
        assert_eq!(cpu.hi, 1, "DIVU: 0xFFFFFFFF % 2 should give remainder 1");
    }

    #[test]
    fn test_divu_by_zero() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 0);

        cpu.op_divu(1, 2).unwrap();

        // PSX-SPX: unsigned division by zero gives LO=0xFFFFFFFF, HI=numerator
        assert_eq!(
            cpu.lo, 0xFFFFFFFF,
            "DIVU: division by 0 should give LO=0xFFFFFFFF"
        );
        assert_eq!(cpu.hi, 100, "DIVU: division by 0 should leave HI=numerator");
    }

    #[test]
    fn test_divu_vs_div() {
        let mut cpu = create_test_cpu();
        let value = 0x80000000; // Large positive as unsigned, negative as signed

        // DIV (signed)
        cpu.set_reg(1, value);
        cpu.set_reg(2, 2);
        cpu.op_div(1, 2).unwrap();
        let div_lo = cpu.lo;

        // DIVU (unsigned)
        cpu.set_reg(1, value);
        cpu.set_reg(2, 2);
        cpu.op_divu(1, 2).unwrap();
        let divu_lo = cpu.lo;

        assert_eq!(div_lo, 0xC0000000, "DIV: signed division result");
        assert_eq!(divu_lo, 0x40000000, "DIVU: unsigned division result");
        assert_ne!(
            div_lo, divu_lo,
            "DIV and DIVU should differ on negative values"
        );
    }

    // ========== MFHI/MFLO Tests ==========

    #[test]
    fn test_mfhi_basic() {
        let mut cpu = create_test_cpu();
        cpu.hi = 0x12345678;

        cpu.op_mfhi(5).unwrap();

        assert_eq!(cpu.reg(5), 0x12345678, "MFHI: should copy HI to register");
    }

    #[test]
    fn test_mflo_basic() {
        let mut cpu = create_test_cpu();
        cpu.lo = 0xABCDEF00;

        cpu.op_mflo(6).unwrap();

        assert_eq!(cpu.reg(6), 0xABCDEF00, "MFLO: should copy LO to register");
    }

    #[test]
    fn test_mfhi_to_r0() {
        let mut cpu = create_test_cpu();
        cpu.hi = 0xFFFFFFFF;

        cpu.op_mfhi(0).unwrap();

        assert_eq!(cpu.reg(0), 0, "MFHI to r0 should be ignored");
    }

    #[test]
    fn test_mflo_to_r0() {
        let mut cpu = create_test_cpu();
        cpu.lo = 0xFFFFFFFF;

        cpu.op_mflo(0).unwrap();

        assert_eq!(cpu.reg(0), 0, "MFLO to r0 should be ignored");
    }

    // ========== MTHI/MTLO Tests ==========

    #[test]
    fn test_mthi_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(7, 0xDEADBEEF);

        cpu.op_mthi(7).unwrap();

        assert_eq!(cpu.hi, 0xDEADBEEF, "MTHI: should copy register to HI");
    }

    #[test]
    fn test_mtlo_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(8, 0xCAFEBABE);

        cpu.op_mtlo(8).unwrap();

        assert_eq!(cpu.lo, 0xCAFEBABE, "MTLO: should copy register to LO");
    }

    #[test]
    fn test_mthi_from_r0() {
        let mut cpu = create_test_cpu();
        cpu.hi = 0xFFFFFFFF;

        cpu.op_mthi(0).unwrap();

        assert_eq!(cpu.hi, 0, "MTHI from r0 should set HI to 0");
    }

    #[test]
    fn test_mtlo_from_r0() {
        let mut cpu = create_test_cpu();
        cpu.lo = 0xFFFFFFFF;

        cpu.op_mtlo(0).unwrap();

        assert_eq!(cpu.lo, 0, "MTLO from r0 should set LO to 0");
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_mult_then_mfhi_mflo() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0xFFFFFFFF); // -1 signed
        cpu.set_reg(2, 2);

        cpu.op_mult(1, 2).unwrap();
        cpu.op_mflo(3).unwrap();
        cpu.op_mfhi(4).unwrap();

        assert_eq!(cpu.reg(3), (-2i32) as u32, "MULT result in LO via MFLO");
        assert_eq!(cpu.reg(4), 0xFFFFFFFF, "MULT result in HI via MFHI");
    }

    #[test]
    fn test_div_then_mfhi_mflo() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 100);
        cpu.set_reg(2, 7);

        cpu.op_div(1, 2).unwrap();
        cpu.op_mflo(3).unwrap();
        cpu.op_mfhi(4).unwrap();

        assert_eq!(cpu.reg(3), 14, "DIV quotient in LO via MFLO");
        assert_eq!(cpu.reg(4), 2, "DIV remainder in HI via MFHI");
    }

    #[test]
    fn test_mthi_mtlo_then_mfhi_mflo() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(1, 0x11111111);
        cpu.set_reg(2, 0x22222222);

        cpu.op_mthi(1).unwrap();
        cpu.op_mtlo(2).unwrap();
        cpu.op_mfhi(3).unwrap();
        cpu.op_mflo(4).unwrap();

        assert_eq!(cpu.reg(3), 0x11111111, "MTHI/MFHI round trip");
        assert_eq!(cpu.reg(4), 0x22222222, "MTLO/MFLO round trip");
    }

    #[test]
    fn test_sequential_operations() {
        let mut cpu = create_test_cpu();

        // First multiplication
        cpu.set_reg(1, 10);
        cpu.set_reg(2, 20);
        cpu.op_mult(1, 2).unwrap();
        assert_eq!(cpu.lo, 200, "First MULT result");

        // Second multiplication overwrites
        cpu.set_reg(3, 5);
        cpu.set_reg(4, 6);
        cpu.op_mult(3, 4).unwrap();
        assert_eq!(cpu.lo, 30, "Second MULT overwrites LO");
        assert_eq!(cpu.hi, 0, "Second MULT overwrites HI");
    }
}
