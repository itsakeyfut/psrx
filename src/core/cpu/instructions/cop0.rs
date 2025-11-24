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

//! Coprocessor 0 (System Control) instructions

use super::super::cop0::COP0;
use super::CPU;
use crate::core::error::Result;

impl CPU {
    /// MFC0: Move From Coprocessor 0
    ///
    /// Moves the contents of a COP0 register to a general-purpose register.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Format
    ///
    /// MFC0 rt, rd
    ///
    /// # Example
    ///
    /// ```text
    /// MFC0 $t0, $12  # Move Status Register to $t0
    /// ```
    pub(crate) fn op_mfc0(&mut self, instruction: u32) -> Result<()> {
        let rt = ((instruction >> 16) & 0x1F) as u8;
        let rd = ((instruction >> 11) & 0x1F) as u8;

        let value = self.cop0.regs[rd as usize];
        self.set_reg_delayed(rt, value);
        Ok(())
    }

    /// MTC0: Move To Coprocessor 0
    ///
    /// Moves the contents of a general-purpose register to a COP0 register.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The full 32-bit instruction
    ///
    /// # Format
    ///
    /// MTC0 rt, rd
    ///
    /// # Example
    ///
    /// ```text
    /// MTC0 $t0, $12  # Move $t0 to Status Register
    /// ```
    pub(crate) fn op_mtc0(&mut self, instruction: u32) -> Result<()> {
        let rt = ((instruction >> 16) & 0x1F) as u8;
        let rd = ((instruction >> 11) & 0x1F) as u8;

        let value = self.reg(rt);
        self.cop0.regs[rd as usize] = value;
        Ok(())
    }

    /// RFE: Return From Exception
    ///
    /// Restores the previous processor mode by shifting the mode bits
    /// in the Status Register (SR) right by 2 bits.
    ///
    /// # Arguments
    ///
    /// * `_instruction` - The full 32-bit instruction (unused)
    ///
    /// # Details
    ///
    /// The Status Register contains mode bits in positions [5:0]:
    /// - Bits [1:0]: Current mode (KUc, IEc)
    /// - Bits [3:2]: Previous mode (KUp, IEp)
    /// - Bits [5:4]: Old mode (KUo, IEo)
    ///
    /// RFE shifts these bits right by 2, restoring the previous mode.
    ///
    /// # Example
    ///
    /// ```text
    /// RFE  # Return from exception handler
    /// ```
    pub(crate) fn op_rfe(&mut self, _instruction: u32) -> Result<()> {
        let sr = self.cop0.regs[COP0::SR];
        // Shift mode bits right by 2 (restore previous mode)
        let mode = sr & 0x3F;
        self.cop0.regs[COP0::SR] = (sr & !0x3F) | (mode >> 2);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // Helper to create instruction
    fn make_cop0_inst(op: u8, rt: u8, rd: u8) -> u32 {
        (0x10 << 26) | ((op as u32) << 21) | ((rt as u32) << 16) | ((rd as u32) << 11)
    }

    // Helper to apply load delay (simulates next instruction cycle)
    fn apply_load_delay(cpu: &mut CPU) {
        if let Some(delay) = cpu.load_delay.take() {
            cpu.regs[delay.reg as usize] = delay.value;
        }
    }

    // ========== MFC0 Tests ==========

    #[test]
    fn test_mfc0_basic() {
        let mut cpu = create_test_cpu();
        cpu.cop0.regs[COP0::SR] = 0x12345678; // Set Status Register

        // MFC0 r5, SR (rd=12)
        let instruction = make_cop0_inst(0x00, 5, COP0::SR as u8);
        cpu.op_mfc0(instruction).unwrap();

        // Need to apply load delay
        apply_load_delay(&mut cpu);

        assert_eq!(
            cpu.reg(5),
            0x12345678,
            "MFC0: should read Status Register value"
        );
    }

    #[test]
    fn test_mfc0_all_cop0_registers() {
        let mut cpu = create_test_cpu();

        // Test reading all COP0 registers
        for rd in 0..32 {
            let test_value = 0x10000000 + (rd as u32 * 0x1111);
            cpu.cop0.regs[rd] = test_value;

            let instruction = make_cop0_inst(0x00, 10, rd as u8);
            cpu.op_mfc0(instruction).unwrap();
            apply_load_delay(&mut cpu);

            assert_eq!(
                cpu.reg(10),
                test_value,
                "MFC0: should read COP0 register {} correctly",
                rd
            );
        }
    }

    #[test]
    fn test_mfc0_to_r0() {
        let mut cpu = create_test_cpu();
        cpu.cop0.regs[COP0::SR] = 0xFFFFFFFF;

        // MFC0 r0, SR (should be ignored)
        let instruction = make_cop0_inst(0x00, 0, COP0::SR as u8);
        cpu.op_mfc0(instruction).unwrap();
        apply_load_delay(&mut cpu);

        assert_eq!(cpu.reg(0), 0, "MFC0: write to r0 should be ignored");
    }

    #[test]
    fn test_mfc0_load_delay_slot() {
        let mut cpu = create_test_cpu();
        cpu.cop0.regs[COP0::CAUSE] = 0xABCD1234;

        // MFC0 r8, CAUSE
        let instruction = make_cop0_inst(0x00, 8, COP0::CAUSE as u8);
        cpu.op_mfc0(instruction).unwrap();

        // Before apply_load_delay, register should not be updated
        assert_eq!(cpu.reg(8), 0, "MFC0: value not available before delay slot");

        // After delay slot
        apply_load_delay(&mut cpu);
        assert_eq!(
            cpu.reg(8),
            0xABCD1234,
            "MFC0: value available after delay slot"
        );
    }

    // ========== MTC0 Tests ==========

    #[test]
    fn test_mtc0_basic() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(7, 0x87654321);

        // MTC0 r7, SR
        let instruction = make_cop0_inst(0x04, 7, COP0::SR as u8);
        cpu.op_mtc0(instruction).unwrap();

        assert_eq!(
            cpu.cop0.regs[COP0::SR],
            0x87654321,
            "MTC0: should write to Status Register"
        );
    }

    #[test]
    fn test_mtc0_all_cop0_registers() {
        let mut cpu = create_test_cpu();

        // Test writing all COP0 registers
        for rd in 0..32 {
            let test_value = 0x20000000 + (rd as u32 * 0x2222);
            cpu.set_reg(15, test_value);

            let instruction = make_cop0_inst(0x04, 15, rd as u8);
            cpu.op_mtc0(instruction).unwrap();

            assert_eq!(
                cpu.cop0.regs[rd], test_value,
                "MTC0: should write COP0 register {} correctly",
                rd
            );
        }
    }

    #[test]
    fn test_mtc0_from_r0() {
        let mut cpu = create_test_cpu();
        cpu.cop0.regs[COP0::SR] = 0xFFFFFFFF;

        // MTC0 r0, SR (write 0)
        let instruction = make_cop0_inst(0x04, 0, COP0::SR as u8);
        cpu.op_mtc0(instruction).unwrap();

        assert_eq!(cpu.cop0.regs[COP0::SR], 0, "MTC0: should write 0 from r0");
    }

    #[test]
    fn test_mtc0_epc_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(10, 0xBFC00100);

        // MTC0 r10, EPC (Exception Program Counter)
        let instruction = make_cop0_inst(0x04, 10, COP0::EPC as u8);
        cpu.op_mtc0(instruction).unwrap();

        assert_eq!(
            cpu.cop0.regs[COP0::EPC],
            0xBFC00100,
            "MTC0: should set EPC register"
        );
    }

    #[test]
    fn test_mtc0_cause_register() {
        let mut cpu = create_test_cpu();
        cpu.set_reg(11, 0x00000024); // CAUSE with some bits set

        // MTC0 r11, CAUSE
        let instruction = make_cop0_inst(0x04, 11, COP0::CAUSE as u8);
        cpu.op_mtc0(instruction).unwrap();

        assert_eq!(
            cpu.cop0.regs[COP0::CAUSE],
            0x00000024,
            "MTC0: should set CAUSE register"
        );
    }

    // ========== RFE Tests ==========

    #[test]
    fn test_rfe_basic() {
        let mut cpu = create_test_cpu();
        // Set up SR with mode stack: KUo=1, IEo=1, KUp=0, IEp=0, KUc=1, IEc=1
        // Binary: ...110011 (bits 5:0)
        cpu.cop0.regs[COP0::SR] = 0x00000033;

        cpu.op_rfe(0).unwrap();

        // After RFE: shift right by 2 bits: ...001100
        let expected = 0x0000000C;
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            expected & 0x3F,
            "RFE: should shift mode bits right by 2"
        );
    }

    #[test]
    fn test_rfe_preserves_upper_bits() {
        let mut cpu = create_test_cpu();
        // SR with upper bits set and mode = 0x2A (101010)
        cpu.cop0.regs[COP0::SR] = 0xFFFF002A;

        cpu.op_rfe(0).unwrap();

        // Mode bits shifted: 101010 >> 2 = 001010 (0x0A)
        // Upper bits should be preserved
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x0A,
            "RFE: mode bits should be shifted"
        );
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & !0x3F,
            0xFFFF0000,
            "RFE: upper bits should be preserved"
        );
    }

    #[test]
    fn test_rfe_all_mode_combinations() {
        let mut cpu = create_test_cpu();

        // Test all possible 6-bit mode combinations (0-63)
        for mode in 0..64 {
            cpu.cop0.regs[COP0::SR] = mode;
            cpu.op_rfe(0).unwrap();

            let expected = mode >> 2;
            assert_eq!(
                cpu.cop0.regs[COP0::SR],
                expected,
                "RFE: mode 0b{:06b} should shift to 0b{:06b}",
                mode,
                expected
            );
        }
    }

    #[test]
    fn test_rfe_nested_exceptions() {
        let mut cpu = create_test_cpu();

        // Simulate nested exception scenario:
        // Original: KUo=0, IEo=0, KUp=1, IEp=1, KUc=0, IEc=0
        // Binary: 001100 (0x0C)
        cpu.cop0.regs[COP0::SR] = 0x0000000C;

        // First RFE (return from inner exception)
        cpu.op_rfe(0).unwrap();

        // After first RFE: 000011 (0x03)
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x03,
            "RFE: first return should restore previous mode"
        );

        // Second RFE (return from outer exception)
        cpu.op_rfe(0).unwrap();

        // After second RFE: 000000 (0x00)
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x00,
            "RFE: second return should restore old mode"
        );
    }

    #[test]
    fn test_rfe_with_interrupts_disabled() {
        let mut cpu = create_test_cpu();
        // SR: interrupts disabled in all modes (all IE bits = 0)
        // KUo=1, IEo=0, KUp=1, IEp=0, KUc=1, IEc=0
        // Binary: 101010 (0x2A)
        cpu.cop0.regs[COP0::SR] = 0x0000002A;

        cpu.op_rfe(0).unwrap();

        // After RFE: 001010 (0x0A)
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x0A,
            "RFE: should work with interrupts disabled"
        );
    }

    #[test]
    fn test_rfe_with_interrupts_enabled() {
        let mut cpu = create_test_cpu();
        // SR: interrupts enabled in all modes (all IE bits = 1)
        // KUo=0, IEo=1, KUp=0, IEp=1, KUc=0, IEc=1
        // Binary: 010101 (0x15)
        cpu.cop0.regs[COP0::SR] = 0x00000015;

        cpu.op_rfe(0).unwrap();

        // After RFE: 000101 (0x05)
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x05,
            "RFE: should work with interrupts enabled"
        );
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_mfc0_mtc0_round_trip() {
        let mut cpu = create_test_cpu();
        let test_value = 0xDEADBEEF;

        // Write to COP0 register
        cpu.set_reg(5, test_value);
        let mtc0_inst = make_cop0_inst(0x04, 5, COP0::BADA as u8);
        cpu.op_mtc0(mtc0_inst).unwrap();

        // Read it back
        let mfc0_inst = make_cop0_inst(0x00, 6, COP0::BADA as u8);
        cpu.op_mfc0(mfc0_inst).unwrap();
        apply_load_delay(&mut cpu);

        assert_eq!(
            cpu.reg(6),
            test_value,
            "MFC0/MTC0: round trip should preserve value"
        );
    }

    #[test]
    fn test_cop0_register_independence() {
        let mut cpu = create_test_cpu();

        // Set different values in multiple COP0 registers
        cpu.set_reg(10, 0x11111111);
        cpu.op_mtc0(make_cop0_inst(0x04, 10, COP0::SR as u8))
            .unwrap();

        cpu.set_reg(11, 0x22222222);
        cpu.op_mtc0(make_cop0_inst(0x04, 11, COP0::CAUSE as u8))
            .unwrap();

        cpu.set_reg(12, 0x33333333);
        cpu.op_mtc0(make_cop0_inst(0x04, 12, COP0::EPC as u8))
            .unwrap();

        // Verify each register has correct value
        assert_eq!(cpu.cop0.regs[COP0::SR], 0x11111111);
        assert_eq!(cpu.cop0.regs[COP0::CAUSE], 0x22222222);
        assert_eq!(cpu.cop0.regs[COP0::EPC], 0x33333333);
    }

    #[test]
    fn test_exception_handler_pattern() {
        let mut cpu = create_test_cpu();

        // Simulate what happens during exception:
        // Original SR bits [5:0]: KUo=0, IEo=0, KUp=0, IEp=1, KUc=0, IEc=1 = 000101 (0x05)
        // After exception, mode shifts left by 2:
        // New SR bits [5:0]: KUo=0, IEo=1, KUp=0, IEp=1, KUc=0, IEc=0 = 010100 (0x14)
        cpu.cop0.regs[COP0::SR] = 0x00000014;

        // At end of exception handler: RFE shifts right by 2
        cpu.op_rfe(0).unwrap();

        // After RFE: should restore to 000101 (0x05)
        assert_eq!(
            cpu.cop0.regs[COP0::SR] & 0x3F,
            0x05,
            "Exception handler: RFE should restore interrupted mode"
        );
    }
}
