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

//! CPU execution tracer for debugging
//!
//! Logs CPU execution state to a file for analysis and debugging.

use super::{Disassembler, CPU};
use crate::core::error::Result;
use crate::core::memory::Bus;
use std::fs::File;
use std::io::Write;

/// CPU execution tracer
///
/// Records CPU state and instruction execution to a file for debugging purposes.
/// Each line in the trace file shows:
/// - Program counter
/// - Raw instruction encoding
/// - Disassembled instruction
/// - Values of first few registers
///
/// # Example
/// ```no_run
/// use psrx::core::cpu::{CPU, CpuTracer};
/// use psrx::core::memory::Bus;
///
/// let mut cpu = CPU::new();
/// let mut bus = Bus::new();
/// let mut tracer = CpuTracer::new("trace.log").unwrap();
///
/// // Execute and trace
/// tracer.trace(&cpu, &bus).unwrap();
/// cpu.step(&mut bus).unwrap();
/// ```
pub struct CpuTracer {
    /// Enable/disable tracing
    enabled: bool,
    /// Output file handle
    output: File,
}

impl CpuTracer {
    /// Create a new CPU tracer
    ///
    /// Opens a file for writing trace output. If the file exists, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output trace file
    ///
    /// # Returns
    ///
    /// - `Ok(CpuTracer)` if the file was opened successfully
    /// - `Err(EmulatorError)` if file creation fails
    ///
    /// # Example
    /// ```no_run
    /// use psrx::core::cpu::CpuTracer;
    ///
    /// let tracer = CpuTracer::new("trace.log").unwrap();
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        let output = File::create(path)?;
        Ok(Self {
            enabled: true,
            output,
        })
    }

    /// Enable or disable tracing
    ///
    /// When disabled, trace() calls will return immediately without writing.
    ///
    /// # Arguments
    ///
    /// * `enabled` - true to enable tracing, false to disable
    ///
    /// # Example
    /// ```no_run
    /// use psrx::core::cpu::CpuTracer;
    ///
    /// let mut tracer = CpuTracer::new("trace.log").unwrap();
    /// tracer.set_enabled(false); // Disable tracing
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if tracing is enabled
    ///
    /// # Returns
    ///
    /// true if tracing is enabled, false otherwise
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Trace current CPU state
    ///
    /// Writes a single line to the trace file containing:
    /// - PC (program counter)
    /// - Raw instruction encoding
    /// - Disassembled instruction
    /// - Selected register values (r1, r2, r3)
    ///
    /// If tracing is disabled, this function returns immediately.
    ///
    /// # Arguments
    ///
    /// * `cpu` - CPU instance to trace
    /// * `bus` - Memory bus for fetching instructions
    ///
    /// # Returns
    ///
    /// - `Ok(())` if trace was written successfully
    /// - `Err(EmulatorError)` if writing fails or memory access fails
    ///
    /// # Example
    /// ```no_run
    /// use psrx::core::cpu::{CPU, CpuTracer};
    /// use psrx::core::memory::Bus;
    ///
    /// let cpu = CPU::new();
    /// let bus = Bus::new();
    /// let mut tracer = CpuTracer::new("trace.log").unwrap();
    ///
    /// tracer.trace(&cpu, &bus).unwrap();
    /// ```
    pub fn trace(&mut self, cpu: &CPU, bus: &Bus) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Log first trace call
        static FIRST_TRACE_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FIRST_TRACE_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            log::info!("CpuTracer::trace() called for the first time");
        }

        let pc = cpu.pc();
        let instruction = match bus.read32(pc) {
            Ok(inst) => inst,
            Err(e) => {
                log::warn!("Failed to read instruction at PC=0x{:08X}: {}", pc, e);
                return Err(e);
            }
        };
        let disasm = Disassembler::disassemble(instruction, pc);

        if let Err(e) = writeln!(
            self.output,
            "PC=0x{:08X} [0x{:08X}] {:30} | r1={:08X} r2={:08X} r3={:08X}",
            pc,
            instruction,
            disasm,
            cpu.reg(1),
            cpu.reg(2),
            cpu.reg(3)
        ) {
            log::warn!("Failed to write trace line: {}", e);
            return Err(e.into());
        }

        Ok(())
    }

    /// Trace with custom register selection
    ///
    /// Like `trace()`, but allows specifying which registers to display.
    ///
    /// # Arguments
    ///
    /// * `cpu` - CPU instance to trace
    /// * `bus` - Memory bus for fetching instructions
    /// * `regs` - Slice of register numbers to display (up to 8 registers)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if trace was written successfully
    /// - `Err(EmulatorError)` if writing fails or memory access fails
    ///
    /// # Example
    /// ```no_run
    /// use psrx::core::cpu::{CPU, CpuTracer};
    /// use psrx::core::memory::Bus;
    ///
    /// let cpu = CPU::new();
    /// let bus = Bus::new();
    /// let mut tracer = CpuTracer::new("trace.log").unwrap();
    ///
    /// // Trace with registers 4, 5, 6
    /// tracer.trace_with_regs(&cpu, &bus, &[4, 5, 6]).unwrap();
    /// ```
    pub fn trace_with_regs(&mut self, cpu: &CPU, bus: &Bus, regs: &[u8]) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let pc = cpu.pc();
        let instruction = bus.read32(pc)?;
        let disasm = Disassembler::disassemble(instruction, pc);

        write!(
            self.output,
            "PC=0x{:08X} [0x{:08X}] {:30} |",
            pc, instruction, disasm
        )?;

        for &reg in regs.iter().take(8) {
            if reg >= 32 {
                return Err(crate::core::error::EmulatorError::InvalidRegister { index: reg });
            }
            write!(self.output, " r{}={:08X}", reg, cpu.reg(reg))?;
        }

        writeln!(self.output)?;

        Ok(())
    }

    /// Flush the output buffer
    ///
    /// Forces any buffered trace data to be written to disk.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if flush succeeded
    /// - `Err(EmulatorError)` if flushing fails
    pub fn flush(&mut self) -> Result<()> {
        self.output.flush()?;
        Ok(())
    }
}

impl Drop for CpuTracer {
    fn drop(&mut self) {
        // Flush any remaining data when tracer is dropped
        let _ = self.output.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_tracer_creation() {
        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace.log");
        let tracer = CpuTracer::new(trace_path.to_str().unwrap());
        assert!(tracer.is_ok());
    }

    #[test]
    fn test_tracer_enable_disable() {
        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_enable.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();
        assert!(tracer.is_enabled());

        tracer.set_enabled(false);
        assert!(!tracer.is_enabled());

        tracer.set_enabled(true);
        assert!(tracer.is_enabled());
    }

    #[test]
    fn test_tracer_basic_trace() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        // Write a NOP instruction
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_basic.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();
        let result = tracer.trace(&cpu, &bus);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify trace file contains expected content
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("PC=0xBFC00000"));
        assert!(contents.contains("nop"));
    }

    #[test]
    fn test_tracer_disabled() {
        let cpu = CPU::new();
        let bus = Bus::new();

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_disabled.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();
        tracer.set_enabled(false);

        // This should succeed but not write anything
        let result = tracer.trace(&cpu, &bus);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tracer_with_custom_regs() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        // Set some register values
        cpu.set_reg(4, 0x12345678);
        cpu.set_reg(5, 0xABCDEF00);

        // Write a NOP instruction
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_custom.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();
        let result = tracer.trace_with_regs(&cpu, &bus, &[4, 5]);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify trace file contains custom register values
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("r4=12345678"));
        assert!(contents.contains("r5=ABCDEF00"));
    }

    // ========== Additional Tracer Tests ==========

    #[test]
    fn test_tracer_multiple_traces() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        // Write several instructions
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]); // NOP
        bus.write_bios_for_test(4, &[0x34, 0x22, 0x00, 0x42]); // ORI r2, r1, 0x4200

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_multiple.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace multiple times
        tracer.trace(&cpu, &bus).unwrap();
        tracer.trace(&cpu, &bus).unwrap();
        tracer.trace(&cpu, &bus).unwrap();

        tracer.flush().unwrap();

        // Verify multiple lines were written
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let line_count = contents.lines().count();
        assert_eq!(line_count, 3);
    }

    #[test]
    fn test_tracer_all_register_values() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        // Set various register values
        for i in 1..32 {
            cpu.set_reg(i, 0x1000 + i as u32);
        }

        // Write a NOP instruction
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_all_regs.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace with many registers
        let regs = [1, 2, 3, 4, 5, 6, 7, 8];
        let result = tracer.trace_with_regs(&cpu, &bus, &regs);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify all requested registers appear in trace
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        for &reg in &regs {
            let expected = format!("r{}={:08X}", reg, 0x1000 + reg as u32);
            assert!(
                contents.contains(&expected),
                "Trace should contain {}",
                expected
            );
        }
    }

    #[test]
    fn test_tracer_invalid_register() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        // Write a NOP instruction
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_invalid_reg.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Try to trace with invalid register number
        let result = tracer.trace_with_regs(&cpu, &bus, &[32]); // r32 doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_tracer_enable_disable_toggle() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_toggle.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace while enabled
        assert!(tracer.is_enabled());
        tracer.trace(&cpu, &bus).unwrap();

        // Disable and trace (should not write)
        tracer.set_enabled(false);
        assert!(!tracer.is_enabled());
        tracer.trace(&cpu, &bus).unwrap();

        // Re-enable and trace
        tracer.set_enabled(true);
        assert!(tracer.is_enabled());
        tracer.trace(&cpu, &bus).unwrap();

        tracer.flush().unwrap();

        // Should only have 2 lines (first and third trace)
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let line_count = contents.lines().count();
        assert_eq!(line_count, 2);
    }

    #[test]
    fn test_tracer_zero_register() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_r0.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace with r0 (should always be 0)
        let result = tracer.trace_with_regs(&cpu, &bus, &[0]);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify r0 is always zero
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("r0=00000000"));
    }

    #[test]
    fn test_tracer_high_registers() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        // Set high register values (r28-r31)
        cpu.set_reg(28, 0x10000000); // gp
        cpu.set_reg(29, 0x801FFF00); // sp
        cpu.set_reg(30, 0x801FFF08); // fp
        cpu.set_reg(31, 0xBFC00100); // ra

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_high_regs.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        let result = tracer.trace_with_regs(&cpu, &bus, &[28, 29, 30, 31]);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify high registers are traced correctly
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("r28=10000000"));
        assert!(contents.contains("r29=801FFF00"));
        assert!(contents.contains("r30=801FFF08"));
        assert!(contents.contains("r31=BFC00100"));
    }

    #[test]
    fn test_tracer_different_instructions() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        // Write different instruction types
        bus.write_bios_for_test(0, &[0x34, 0x01, 0x12, 0x3C]); // LUI r1, 0x1234
        bus.write_bios_for_test(4, &[0x42, 0x00, 0x22, 0x24]); // ADDIU r2, r1, 66

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_diff_instr.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace first instruction
        tracer.trace(&cpu, &bus).unwrap();

        tracer.flush().unwrap();

        // Verify instruction was disassembled correctly
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("lui"));
    }

    #[test]
    fn test_tracer_register_limit() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_reg_limit.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Try to trace more than 8 registers (should only trace first 8)
        let regs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = tracer.trace_with_regs(&cpu, &bus, &regs);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Verify only first 8 registers are traced
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("r8="));
        assert!(!contents.contains("r9=")); // r9 should not be traced
    }

    #[test]
    fn test_tracer_empty_register_list() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_empty_regs.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace with empty register list
        let result = tracer.trace_with_regs(&cpu, &bus, &[]);
        assert!(result.is_ok());

        tracer.flush().unwrap();

        // Should still have trace line but no register values
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("PC="));
        assert!(contents.contains("nop"));
    }

    #[test]
    fn test_tracer_flush_behavior() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_flush.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace and flush multiple times
        tracer.trace(&cpu, &bus).unwrap();
        tracer.flush().unwrap();

        tracer.trace(&cpu, &bus).unwrap();
        tracer.flush().unwrap();

        // Verify both traces were written
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents.lines().count(), 2);
    }

    #[test]
    fn test_tracer_different_pc_values() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        // Write instructions at different locations
        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]); // NOP at 0xBFC00000
        bus.write_bios_for_test(0x100, &[0x08, 0x00, 0xE0, 0x03]); // JR r31 at 0xBFC00100

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_diff_pc.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Trace first location
        tracer.trace(&cpu, &bus).unwrap();

        // Change PC and trace again
        cpu.set_pc(0xBFC00100);
        tracer.trace(&cpu, &bus).unwrap();

        tracer.flush().unwrap();

        // Verify different PC values are in trace
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("PC=0xBFC00000"));
        assert!(contents.contains("PC=0xBFC00100"));
    }

    #[test]
    fn test_tracer_instruction_encoding() {
        let cpu = CPU::new();
        let mut bus = Bus::new();

        // Write specific instruction with known encoding
        bus.write_bios_for_test(0, &[0x34, 0x12, 0x01, 0x3C]); // LUI r1, 0x1234

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_encoding.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        tracer.trace(&cpu, &bus).unwrap();
        tracer.flush().unwrap();

        // Verify raw instruction encoding appears in trace
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("0x3C011234"));
    }

    #[test]
    fn test_tracer_default_registers() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        // Set the default traced registers (r1, r2, r3)
        cpu.set_reg(1, 0xAABBCCDD);
        cpu.set_reg(2, 0x11223344);
        cpu.set_reg(3, 0x55667788);

        bus.write_bios_for_test(0, &[0x00, 0x00, 0x00, 0x00]);

        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace_defaults.log");
        let mut tracer = CpuTracer::new(trace_path.to_str().unwrap()).unwrap();

        // Use default trace (should show r1, r2, r3)
        tracer.trace(&cpu, &bus).unwrap();
        tracer.flush().unwrap();

        // Verify default registers appear
        let mut file = File::open(&trace_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("r1=AABBCCDD"));
        assert!(contents.contains("r2=11223344"));
        assert!(contents.contains("r3=55667788"));
    }
}
