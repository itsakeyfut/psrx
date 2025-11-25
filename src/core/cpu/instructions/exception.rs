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

//! Exception-triggering instructions

use super::super::ExceptionCause;
use super::CPU;
use crate::core::error::Result;

impl CPU {
    /// SYSCALL: System Call
    ///
    /// Triggers a system call exception, transferring control to the
    /// exception handler. This is typically used by user programs to
    /// request operating system services.
    ///
    /// # Arguments
    ///
    /// * `_instruction` - The full 32-bit instruction (unused)
    ///
    /// # Exception
    ///
    /// Always triggers ExceptionCause::Syscall
    ///
    /// # Example
    ///
    /// ```text
    /// SYSCALL  # Trigger system call exception
    /// ```
    pub(crate) fn op_syscall(&mut self, _instruction: u32) -> Result<()> {
        self.exception(ExceptionCause::Syscall);
        Ok(())
    }

    /// BREAK: Breakpoint
    ///
    /// Triggers a breakpoint exception, transferring control to the
    /// exception handler. This is typically used by debuggers to set
    /// breakpoints in code.
    ///
    /// # Arguments
    ///
    /// * `_instruction` - The full 32-bit instruction (unused)
    ///
    /// # Exception
    ///
    /// Always triggers ExceptionCause::Breakpoint
    ///
    /// # Example
    ///
    /// ```text
    /// BREAK  # Trigger breakpoint exception
    /// ```
    pub(crate) fn op_break(&mut self, _instruction: u32) -> Result<()> {
        self.exception(ExceptionCause::Breakpoint);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cpu::cop0::COP0;

    fn create_test_cpu() -> CPU {
        CPU::new()
    }

    // ========== SYSCALL Tests ==========

    #[test]
    fn test_syscall_triggers_exception() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80001000;
        cpu.next_pc = 0x80001004;

        // Initial CAUSE should be 0
        assert_eq!(cpu.cop0.regs[COP0::CAUSE], 0, "CAUSE should start at 0");

        cpu.op_syscall(0).unwrap();

        // CAUSE register should have Syscall exception code
        let cause = cpu.cop0.regs[COP0::CAUSE];
        let exc_code = (cause >> 2) & 0x1F;
        assert_eq!(
            exc_code,
            ExceptionCause::Syscall as u32,
            "SYSCALL should set exception code to 8"
        );
    }

    #[test]
    fn test_syscall_saves_epc() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0xBFC00100;
        cpu.next_pc = 0xBFC00104;

        cpu.op_syscall(0).unwrap();

        // EPC should point to the SYSCALL instruction
        // Implementation saves (pc - 4) as EPC
        assert_eq!(
            cpu.cop0.regs[COP0::EPC],
            0xBFC000FC,
            "SYSCALL should save PC-4 to EPC"
        );
    }

    #[test]
    fn test_syscall_disables_interrupts() {
        let mut cpu = create_test_cpu();
        // Set SR with interrupts enabled (IEc = 1)
        cpu.cop0.regs[COP0::SR] = 0x00000001;

        cpu.op_syscall(0).unwrap();

        // IEc should be cleared (interrupts disabled)
        let sr = cpu.cop0.regs[COP0::SR];
        assert_eq!(sr & 0x01, 0, "SYSCALL should disable interrupts (IEc=0)");
    }

    #[test]
    fn test_syscall_shifts_mode_stack() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // Initial SR: KUc=1, IEc=1 (user mode, interrupts enabled)
        cpu.cop0.regs[COP0::SR] = 0x00000003;

        cpu.op_syscall(0).unwrap();

        // After exception: mode stack should shift left
        // KUc,IEc -> KUp,IEp; new KUc,IEc = 0 (kernel mode, IE disabled)
        let sr = cpu.cop0.regs[COP0::SR];
        let prev_mode = (sr >> 2) & 0x03;
        let curr_mode = sr & 0x03;

        assert_eq!(prev_mode, 0x03, "Previous mode should be saved");
        assert_eq!(curr_mode, 0x00, "Current mode should be kernel with IE=0");
    }

    #[test]
    fn test_syscall_jumps_to_exception_vector() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000100;
        cpu.next_pc = 0x80000104;

        // BEV=0: exception vector at 0x80000080
        cpu.cop0.regs[COP0::SR] = 0x00000000;

        cpu.op_syscall(0).unwrap();

        // PC should be set to exception vector
        assert_eq!(
            cpu.pc, 0x80000080,
            "SYSCALL should jump to exception vector"
        );
    }

    #[test]
    fn test_syscall_with_bev_set() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000100;
        cpu.next_pc = 0x80000104;

        // BEV=1: exception vector at 0xBFC00180
        cpu.cop0.regs[COP0::SR] = 0x00400000; // BEV bit

        cpu.op_syscall(0).unwrap();

        // PC should be set to BFC00180 (BIOS exception vector)
        assert_eq!(
            cpu.pc, 0xBFC00180,
            "SYSCALL with BEV=1 should jump to BIOS exception vector"
        );
    }

    // ========== BREAK Tests ==========

    #[test]
    fn test_break_triggers_exception() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80002000;
        cpu.next_pc = 0x80002004;

        cpu.op_break(0).unwrap();

        // CAUSE register should have Breakpoint exception code
        let cause = cpu.cop0.regs[COP0::CAUSE];
        let exc_code = (cause >> 2) & 0x1F;
        assert_eq!(
            exc_code,
            ExceptionCause::Breakpoint as u32,
            "BREAK should set exception code to 9"
        );
    }

    #[test]
    fn test_break_saves_epc() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0xA0000200;
        cpu.next_pc = 0xA0000204;

        cpu.op_break(0).unwrap();

        // EPC should point to the BREAK instruction
        // Implementation saves (pc - 4) as EPC
        assert_eq!(
            cpu.cop0.regs[COP0::EPC],
            0xA00001FC,
            "BREAK should save PC-4 to EPC"
        );
    }

    #[test]
    fn test_break_disables_interrupts() {
        let mut cpu = create_test_cpu();
        // Set SR with interrupts enabled
        cpu.cop0.regs[COP0::SR] = 0x00000001;

        cpu.op_break(0).unwrap();

        // IEc should be cleared
        let sr = cpu.cop0.regs[COP0::SR];
        assert_eq!(sr & 0x01, 0, "BREAK should disable interrupts (IEc=0)");
    }

    #[test]
    fn test_break_shifts_mode_stack() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000000;
        cpu.next_pc = 0x80000004;

        // Initial SR: user mode, interrupts enabled
        cpu.cop0.regs[COP0::SR] = 0x00000003;

        cpu.op_break(0).unwrap();

        // Mode stack should shift left
        let sr = cpu.cop0.regs[COP0::SR];
        let prev_mode = (sr >> 2) & 0x03;
        let curr_mode = sr & 0x03;

        assert_eq!(prev_mode, 0x03, "Previous mode should be saved");
        assert_eq!(curr_mode, 0x00, "Current mode should be kernel with IE=0");
    }

    #[test]
    fn test_break_jumps_to_exception_vector() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80000500;
        cpu.next_pc = 0x80000504;

        // BEV=0
        cpu.cop0.regs[COP0::SR] = 0x00000000;

        cpu.op_break(0).unwrap();

        assert_eq!(cpu.pc, 0x80000080, "BREAK should jump to exception vector");
    }

    // ========== Comparison Tests ==========

    #[test]
    fn test_syscall_vs_break_exception_codes() {
        let mut cpu1 = create_test_cpu();
        let mut cpu2 = create_test_cpu();

        cpu1.op_syscall(0).unwrap();
        cpu2.op_break(0).unwrap();

        let syscall_code = (cpu1.cop0.regs[COP0::CAUSE] >> 2) & 0x1F;
        let break_code = (cpu2.cop0.regs[COP0::CAUSE] >> 2) & 0x1F;

        assert_ne!(
            syscall_code, break_code,
            "SYSCALL and BREAK should have different exception codes"
        );
        assert_eq!(syscall_code, 8, "SYSCALL code should be 8");
        assert_eq!(break_code, 9, "BREAK code should be 9");
    }

    #[test]
    fn test_multiple_exceptions_update_cause() {
        let mut cpu = create_test_cpu();

        // First exception: SYSCALL
        cpu.op_syscall(0).unwrap();
        let first_cause = cpu.cop0.regs[COP0::CAUSE];
        let first_code = (first_cause >> 2) & 0x1F;

        // Second exception: BREAK
        cpu.op_break(0).unwrap();
        let second_cause = cpu.cop0.regs[COP0::CAUSE];
        let second_code = (second_cause >> 2) & 0x1F;

        assert_eq!(first_code, 8, "First exception: SYSCALL");
        assert_eq!(second_code, 9, "Second exception: BREAK");
        assert_ne!(
            first_cause, second_cause,
            "CAUSE should update for each exception"
        );
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_exception_in_delay_slot() {
        let mut cpu = create_test_cpu();
        cpu.pc = 0x80001000;
        cpu.next_pc = 0x80001004;
        cpu.in_branch_delay = true; // In delay slot

        cpu.op_syscall(0).unwrap();

        // In delay slot, BD bit in CAUSE should be set
        let cause = cpu.cop0.regs[COP0::CAUSE];
        let bd_bit = (cause >> 31) & 0x01;

        assert_eq!(
            bd_bit, 1,
            "BD bit should be set when exception in delay slot"
        );
    }

    #[test]
    fn test_nested_exceptions() {
        let mut cpu = create_test_cpu();

        // First exception
        cpu.cop0.regs[COP0::SR] = 0x00000001; // IE enabled
        cpu.op_syscall(0).unwrap();

        // SR after first exception: mode shifted, IE disabled
        let sr_after_first = cpu.cop0.regs[COP0::SR];

        // Second exception (nested)
        cpu.op_break(0).unwrap();

        // SR after second exception: mode shifted again
        let sr_after_second = cpu.cop0.regs[COP0::SR];

        // Mode stack should continue to shift
        let first_old = (sr_after_first >> 4) & 0x03;
        let second_old = (sr_after_second >> 4) & 0x03;

        assert_ne!(
            first_old, second_old,
            "Mode stack should continue shifting on nested exceptions"
        );
    }

    #[test]
    fn test_exception_from_different_pc_values() {
        let test_pcs = [0x80000000, 0xBFC00000, 0xA0000100, 0x00001000, 0xFFFFFFFC];

        for &test_pc in &test_pcs {
            let mut cpu = create_test_cpu();
            cpu.pc = test_pc;
            cpu.next_pc = test_pc.wrapping_add(4);

            cpu.op_syscall(0).unwrap();

            // EPC = pc - 4
            let expected_epc = test_pc.wrapping_sub(4);
            assert_eq!(
                cpu.cop0.regs[COP0::EPC],
                expected_epc,
                "EPC should be set correctly for PC=0x{:08X}",
                test_pc
            );
        }
    }
}
