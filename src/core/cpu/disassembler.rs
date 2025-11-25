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

//! MIPS instruction disassembler for debugging
//!
//! Converts binary instruction encodings to human-readable assembly mnemonics.

use super::decode::{decode_i_type, decode_j_type, decode_r_type};

/// Instruction disassembler
///
/// Converts 32-bit MIPS instruction encodings to human-readable assembly format.
///
/// # Example
/// ```
/// use psrx::core::cpu::Disassembler;
///
/// let instruction = 0x00000000; // NOP
/// let disasm = Disassembler::disassemble(instruction, 0xBFC00000);
/// assert_eq!(disasm, "nop");
/// ```
pub struct Disassembler;

impl Disassembler {
    /// Disassemble a single instruction to human-readable format
    ///
    /// # Arguments
    ///
    /// * `instruction` - The 32-bit instruction to disassemble
    /// * `pc` - Program counter (used for jump target calculation)
    ///
    /// # Returns
    ///
    /// String containing the disassembled instruction
    ///
    /// # Example
    /// ```
    /// use psrx::core::cpu::Disassembler;
    ///
    /// let instruction = 0x3C011234; // LUI r1, 0x1234
    /// let disasm = Disassembler::disassemble(instruction, 0xBFC00000);
    /// assert_eq!(disasm, "lui r1, 0x1234");
    /// ```
    pub fn disassemble(instruction: u32, pc: u32) -> String {
        let opcode = instruction >> 26;

        match opcode {
            0x00 => Self::disasm_special(instruction),
            0x01 => Self::disasm_regimm(instruction),
            0x02 => {
                let (_, target) = decode_j_type(instruction);
                let addr = (pc & 0xF000_0000) | (target << 2);
                format!("j 0x{:08X}", addr)
            }
            0x03 => {
                let (_, target) = decode_j_type(instruction);
                let addr = (pc & 0xF000_0000) | (target << 2);
                format!("jal 0x{:08X}", addr)
            }
            0x04 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("beq r{}, r{}, {}", rs, rt, (imm as i16))
            }
            0x05 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("bne r{}, r{}, {}", rs, rt, (imm as i16))
            }
            0x06 => {
                let (_, rs, _, imm) = decode_i_type(instruction);
                format!("blez r{}, {}", rs, (imm as i16))
            }
            0x07 => {
                let (_, rs, _, imm) = decode_i_type(instruction);
                format!("bgtz r{}, {}", rs, (imm as i16))
            }
            0x08 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("addi r{}, r{}, {}", rt, rs, (imm as i16))
            }
            0x09 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("addiu r{}, r{}, {}", rt, rs, (imm as i16))
            }
            0x0A => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("slti r{}, r{}, {}", rt, rs, (imm as i16))
            }
            0x0B => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("sltiu r{}, r{}, {}", rt, rs, (imm as i16))
            }
            0x0C => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("andi r{}, r{}, 0x{:04X}", rt, rs, imm)
            }
            0x0D => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("ori r{}, r{}, 0x{:04X}", rt, rs, imm)
            }
            0x0E => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("xori r{}, r{}, 0x{:04X}", rt, rs, imm)
            }
            0x0F => {
                let (_, _, rt, imm) = decode_i_type(instruction);
                format!("lui r{}, 0x{:04X}", rt, imm)
            }
            0x10 => Self::disasm_cop0(instruction),
            0x20 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lb r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x21 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lh r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x22 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lwl r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x23 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lw r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x24 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lbu r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x25 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lhu r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x26 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("lwr r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x28 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("sb r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x29 => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("sh r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x2A => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("swl r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x2B => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("sw r{}, {}(r{})", rt, (imm as i16), rs)
            }
            0x2E => {
                let (_, rs, rt, imm) = decode_i_type(instruction);
                format!("swr r{}, {}(r{})", rt, (imm as i16), rs)
            }
            _ => format!("??? 0x{:08X}", instruction),
        }
    }

    /// Disassemble SPECIAL (opcode 0x00) instruction
    fn disasm_special(instruction: u32) -> String {
        let (rs, rt, rd, shamt, funct) = decode_r_type(instruction);

        match funct {
            0x00 if instruction == 0 => "nop".to_string(),
            0x00 => format!("sll r{}, r{}, {}", rd, rt, shamt),
            0x02 => format!("srl r{}, r{}, {}", rd, rt, shamt),
            0x03 => format!("sra r{}, r{}, {}", rd, rt, shamt),
            0x04 => format!("sllv r{}, r{}, r{}", rd, rt, rs),
            0x06 => format!("srlv r{}, r{}, r{}", rd, rt, rs),
            0x07 => format!("srav r{}, r{}, r{}", rd, rt, rs),
            0x08 => format!("jr r{}", rs),
            0x09 => {
                if rd == 31 {
                    format!("jalr r{}", rs)
                } else {
                    format!("jalr r{}, r{}", rd, rs)
                }
            }
            0x0C => "syscall".to_string(),
            0x0D => "break".to_string(),
            0x10 => format!("mfhi r{}", rd),
            0x11 => format!("mthi r{}", rs),
            0x12 => format!("mflo r{}", rd),
            0x13 => format!("mtlo r{}", rs),
            0x18 => format!("mult r{}, r{}", rs, rt),
            0x19 => format!("multu r{}, r{}", rs, rt),
            0x1A => format!("div r{}, r{}", rs, rt),
            0x1B => format!("divu r{}, r{}", rs, rt),
            0x20 => format!("add r{}, r{}, r{}", rd, rs, rt),
            0x21 => format!("addu r{}, r{}, r{}", rd, rs, rt),
            0x22 => format!("sub r{}, r{}, r{}", rd, rs, rt),
            0x23 => format!("subu r{}, r{}, r{}", rd, rs, rt),
            0x24 => format!("and r{}, r{}, r{}", rd, rs, rt),
            0x25 => format!("or r{}, r{}, r{}", rd, rs, rt),
            0x26 => format!("xor r{}, r{}, r{}", rd, rs, rt),
            0x27 => format!("nor r{}, r{}, r{}", rd, rs, rt),
            0x2A => format!("slt r{}, r{}, r{}", rd, rs, rt),
            0x2B => format!("sltu r{}, r{}, r{}", rd, rs, rt),
            _ => format!("??? 0x{:08X}", instruction),
        }
    }

    /// Disassemble REGIMM (opcode 0x01) instruction
    fn disasm_regimm(instruction: u32) -> String {
        let (_, rs, rt, imm) = decode_i_type(instruction);

        match rt {
            0x00 => format!("bltz r{}, {}", rs, (imm as i16)),
            0x01 => format!("bgez r{}, {}", rs, (imm as i16)),
            0x10 => format!("bltzal r{}, {}", rs, (imm as i16)),
            0x11 => format!("bgezal r{}, {}", rs, (imm as i16)),
            _ => format!("??? 0x{:08X}", instruction),
        }
    }

    /// Disassemble COP0 (coprocessor 0) instruction
    fn disasm_cop0(instruction: u32) -> String {
        let rs = (instruction >> 21) & 0x1F;
        let rt = (instruction >> 16) & 0x1F;
        let rd = (instruction >> 11) & 0x1F;

        match rs {
            0x00 => format!("mfc0 r{}, cop0r{}", rt, rd),
            0x04 => format!("mtc0 r{}, cop0r{}", rt, rd),
            0x10 => {
                let funct = instruction & 0x3F;
                match funct {
                    0x10 => "rfe".to_string(),
                    _ => format!("??? 0x{:08X}", instruction),
                }
            }
            _ => format!("??? 0x{:08X}", instruction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disasm_nop() {
        let result = Disassembler::disassemble(0x00000000, 0);
        assert_eq!(result, "nop");
    }

    #[test]
    fn test_disasm_lui() {
        let result = Disassembler::disassemble(0x3C011234, 0); // LUI r1, 0x1234
        assert_eq!(result, "lui r1, 0x1234");
    }

    #[test]
    fn test_disasm_addiu() {
        let result = Disassembler::disassemble(0x24220042, 0); // ADDIU r2, r1, 66
        assert_eq!(result, "addiu r2, r1, 66");
    }

    #[test]
    fn test_disasm_or() {
        let result = Disassembler::disassemble(0x00411825, 0); // OR r3, r2, r1
        assert_eq!(result, "or r3, r2, r1");
    }

    #[test]
    fn test_disasm_sw() {
        let result = Disassembler::disassemble(0xAC220000, 0); // SW r2, 0(r1)
        assert_eq!(result, "sw r2, 0(r1)");
    }

    #[test]
    fn test_disasm_lw() {
        let result = Disassembler::disassemble(0x8C220004, 0); // LW r2, 4(r1)
        assert_eq!(result, "lw r2, 4(r1)");
    }

    #[test]
    fn test_disasm_j() {
        let result = Disassembler::disassemble(0x0BF00000, 0xBFC00000); // J 0xBFC00000
        assert_eq!(result, "j 0xBFC00000");
    }

    #[test]
    fn test_disasm_jr() {
        let result = Disassembler::disassemble(0x03E00008, 0); // JR r31
        assert_eq!(result, "jr r31");
    }

    #[test]
    fn test_disasm_unknown() {
        let result = Disassembler::disassemble(0xFFFFFFFF, 0);
        assert!(result.starts_with("???"));
    }

    // ========== Additional R-Type (SPECIAL) Tests ==========

    #[test]
    fn test_disasm_srl() {
        // SRL r1, r2, 4
        // Format: 000000 00000 00010 00001 00100 000010
        // instr = (0 << 26) | (0 << 21) | (2 << 16) | (1 << 11) | (4 << 6) | 2
        // Encoding: 0x00020902
        let result = Disassembler::disassemble(0x00020902, 0);
        assert_eq!(result, "srl r1, r2, 4");
    }

    #[test]
    fn test_disasm_sra() {
        // SRA r3, r2, 8
        let result = Disassembler::disassemble(0x00021A03, 0);
        assert_eq!(result, "sra r3, r2, 8");
    }

    #[test]
    fn test_disasm_sllv() {
        // SLLV r4, r3, r2
        let result = Disassembler::disassemble(0x00432004, 0);
        assert_eq!(result, "sllv r4, r3, r2");
    }

    #[test]
    fn test_disasm_srlv() {
        // SRLV r5, r4, r3
        let result = Disassembler::disassemble(0x00642806, 0);
        assert_eq!(result, "srlv r5, r4, r3");
    }

    #[test]
    fn test_disasm_srav() {
        // SRAV r6, r5, r4
        let result = Disassembler::disassemble(0x00853007, 0);
        assert_eq!(result, "srav r6, r5, r4");
    }

    #[test]
    fn test_disasm_jalr_explicit_rd() {
        // JALR r10, r8
        let result = Disassembler::disassemble(0x01005009, 0);
        assert_eq!(result, "jalr r10, r8");
    }

    #[test]
    fn test_disasm_jalr_implicit_rd() {
        // JALR r31, r10 (commonly written as just "jalr r10")
        let result = Disassembler::disassemble(0x014FF809, 0);
        assert_eq!(result, "jalr r10");
    }

    #[test]
    fn test_disasm_syscall() {
        let result = Disassembler::disassemble(0x0000000C, 0);
        assert_eq!(result, "syscall");
    }

    #[test]
    fn test_disasm_break() {
        let result = Disassembler::disassemble(0x0000000D, 0);
        assert_eq!(result, "break");
    }

    #[test]
    fn test_disasm_mthi() {
        // MTHI r5
        let result = Disassembler::disassemble(0x00A00011, 0);
        assert_eq!(result, "mthi r5");
    }

    #[test]
    fn test_disasm_mflo() {
        // MFLO r6
        let result = Disassembler::disassemble(0x00003012, 0);
        assert_eq!(result, "mflo r6");
    }

    #[test]
    fn test_disasm_mtlo() {
        // MTLO r7
        let result = Disassembler::disassemble(0x00E00013, 0);
        assert_eq!(result, "mtlo r7");
    }

    #[test]
    fn test_disasm_mult() {
        // MULT r8, r9
        let result = Disassembler::disassemble(0x01090018, 0);
        assert_eq!(result, "mult r8, r9");
    }

    #[test]
    fn test_disasm_multu() {
        // MULTU r10, r11
        let result = Disassembler::disassemble(0x014B0019, 0);
        assert_eq!(result, "multu r10, r11");
    }

    #[test]
    fn test_disasm_div() {
        // DIV r12, r13
        let result = Disassembler::disassemble(0x018D001A, 0);
        assert_eq!(result, "div r12, r13");
    }

    #[test]
    fn test_disasm_divu() {
        // DIVU r14, r15
        let result = Disassembler::disassemble(0x01CF001B, 0);
        assert_eq!(result, "divu r14, r15");
    }

    #[test]
    fn test_disasm_add() {
        // ADD r3, r1, r2
        let result = Disassembler::disassemble(0x00221820, 0);
        assert_eq!(result, "add r3, r1, r2");
    }

    #[test]
    fn test_disasm_addu() {
        // ADDU r4, r2, r3
        let result = Disassembler::disassemble(0x00432021, 0);
        assert_eq!(result, "addu r4, r2, r3");
    }

    #[test]
    fn test_disasm_sub() {
        // SUB r5, r3, r4
        let result = Disassembler::disassemble(0x00642822, 0);
        assert_eq!(result, "sub r5, r3, r4");
    }

    #[test]
    fn test_disasm_subu() {
        // SUBU r6, r4, r5
        let result = Disassembler::disassemble(0x00853023, 0);
        assert_eq!(result, "subu r6, r4, r5");
    }

    #[test]
    fn test_disasm_and() {
        // AND r7, r5, r6
        let result = Disassembler::disassemble(0x00A63824, 0);
        assert_eq!(result, "and r7, r5, r6");
    }

    #[test]
    fn test_disasm_xor() {
        // XOR r9, r7, r8
        let result = Disassembler::disassemble(0x00E84826, 0);
        assert_eq!(result, "xor r9, r7, r8");
    }

    #[test]
    fn test_disasm_nor() {
        // NOR r10, r8, r9
        let result = Disassembler::disassemble(0x01095027, 0);
        assert_eq!(result, "nor r10, r8, r9");
    }

    #[test]
    fn test_disasm_slt() {
        // SLT r11, r9, r10
        let result = Disassembler::disassemble(0x012A582A, 0);
        assert_eq!(result, "slt r11, r9, r10");
    }

    #[test]
    fn test_disasm_sltu() {
        // SLTU r12, r10, r11
        let result = Disassembler::disassemble(0x014B602B, 0);
        assert_eq!(result, "sltu r12, r10, r11");
    }

    // ========== Additional I-Type Tests ==========

    #[test]
    fn test_disasm_bne() {
        // BNE r1, r2, 8
        let result = Disassembler::disassemble(0x14220008, 0);
        assert_eq!(result, "bne r1, r2, 8");
    }

    #[test]
    fn test_disasm_blez() {
        // BLEZ r3, 4
        let result = Disassembler::disassemble(0x18600004, 0);
        assert_eq!(result, "blez r3, 4");
    }

    #[test]
    fn test_disasm_bgtz() {
        // BGTZ r4, -2
        let result = Disassembler::disassemble(0x1C80FFFE, 0);
        assert_eq!(result, "bgtz r4, -2");
    }

    #[test]
    fn test_disasm_addi() {
        // ADDI r5, r4, -100
        let result = Disassembler::disassemble(0x2085FF9C, 0);
        assert_eq!(result, "addi r5, r4, -100");
    }

    #[test]
    fn test_disasm_addiu_negative() {
        // ADDIU r6, r5, -1
        let result = Disassembler::disassemble(0x24A6FFFF, 0);
        assert_eq!(result, "addiu r6, r5, -1");
    }

    #[test]
    fn test_disasm_slti() {
        // SLTI r7, r6, 100
        let result = Disassembler::disassemble(0x28C70064, 0);
        assert_eq!(result, "slti r7, r6, 100");
    }

    #[test]
    fn test_disasm_sltiu() {
        // SLTIU r8, r7, 200
        let result = Disassembler::disassemble(0x2CE800C8, 0);
        assert_eq!(result, "sltiu r8, r7, 200");
    }

    #[test]
    fn test_disasm_xori() {
        // XORI r9, r8, 0xABCD
        let result = Disassembler::disassemble(0x3909ABCD, 0);
        assert_eq!(result, "xori r9, r8, 0xABCD");
    }

    #[test]
    fn test_disasm_lb() {
        // LB r10, -128(r9)
        let result = Disassembler::disassemble(0x812AFF80, 0);
        assert_eq!(result, "lb r10, -128(r9)");
    }

    #[test]
    fn test_disasm_lh() {
        // LH r11, 256(r10)
        let result = Disassembler::disassemble(0x854B0100, 0);
        assert_eq!(result, "lh r11, 256(r10)");
    }

    #[test]
    fn test_disasm_lwl() {
        // LWL r12, 3(r11)
        let result = Disassembler::disassemble(0x896C0003, 0);
        assert_eq!(result, "lwl r12, 3(r11)");
    }

    #[test]
    fn test_disasm_lbu() {
        // LBU r13, 0xFF(r12)
        let result = Disassembler::disassemble(0x918D00FF, 0);
        assert_eq!(result, "lbu r13, 255(r12)");
    }

    #[test]
    fn test_disasm_lhu() {
        // LHU r14, 0x1234(r13)
        let result = Disassembler::disassemble(0x95AE1234, 0);
        assert_eq!(result, "lhu r14, 4660(r13)");
    }

    #[test]
    fn test_disasm_lwr() {
        // LWR r15, 2(r14)
        let result = Disassembler::disassemble(0x99CF0002, 0);
        assert_eq!(result, "lwr r15, 2(r14)");
    }

    #[test]
    fn test_disasm_sb() {
        // SB r16, 64(r15)
        let result = Disassembler::disassemble(0xA1F00040, 0);
        assert_eq!(result, "sb r16, 64(r15)");
    }

    #[test]
    fn test_disasm_sh() {
        // SH r17, -8(r16)
        let result = Disassembler::disassemble(0xA611FFF8, 0);
        assert_eq!(result, "sh r17, -8(r16)");
    }

    #[test]
    fn test_disasm_swl() {
        // SWL r18, 1(r17)
        let result = Disassembler::disassemble(0xAA320001, 0);
        assert_eq!(result, "swl r18, 1(r17)");
    }

    #[test]
    fn test_disasm_swr() {
        // SWR r19, 3(r18)
        // Format: 101110 10010 10011 0000000000000011
        // Opcode 0x2E (46) is SWR, rs=18, rt=19, imm=3
        // Encoding: 0xBA530003
        let result = Disassembler::disassemble(0xBA530003, 0);
        assert_eq!(result, "swr r19, 3(r18)");
    }

    // ========== REGIMM Tests ==========

    #[test]
    fn test_disasm_bltz() {
        // BLTZ r5, -10
        let result = Disassembler::disassemble(0x04A0FFF6, 0);
        assert_eq!(result, "bltz r5, -10");
    }

    #[test]
    fn test_disasm_bgez() {
        // BGEZ r6, 20
        let result = Disassembler::disassemble(0x04C10014, 0);
        assert_eq!(result, "bgez r6, 20");
    }

    #[test]
    fn test_disasm_bltzal() {
        // BLTZAL r7, -100
        let result = Disassembler::disassemble(0x04F0FF9C, 0);
        assert_eq!(result, "bltzal r7, -100");
    }

    #[test]
    fn test_disasm_bgezal() {
        // BGEZAL r8, 100
        let result = Disassembler::disassemble(0x05110064, 0);
        assert_eq!(result, "bgezal r8, 100");
    }

    // ========== COP0 Tests ==========

    #[test]
    fn test_disasm_mfc0() {
        // MFC0 r5, cop0r12 (Status Register)
        let result = Disassembler::disassemble(0x40056000, 0);
        assert_eq!(result, "mfc0 r5, cop0r12");
    }

    #[test]
    fn test_disasm_mfc0_cause() {
        // MFC0 r6, cop0r13 (Cause Register)
        let result = Disassembler::disassemble(0x40066800, 0);
        assert_eq!(result, "mfc0 r6, cop0r13");
    }

    #[test]
    fn test_disasm_mtc0() {
        // MTC0 r7, cop0r12 (Status Register)
        let result = Disassembler::disassemble(0x40876000, 0);
        assert_eq!(result, "mtc0 r7, cop0r12");
    }

    #[test]
    fn test_disasm_mtc0_epc() {
        // MTC0 r8, cop0r14 (EPC)
        let result = Disassembler::disassemble(0x40887000, 0);
        assert_eq!(result, "mtc0 r8, cop0r14");
    }

    #[test]
    fn test_disasm_rfe() {
        // RFE (Return From Exception)
        let result = Disassembler::disassemble(0x42000010, 0);
        assert_eq!(result, "rfe");
    }

    // ========== Jump Target Calculation Tests ==========

    #[test]
    fn test_disasm_j_with_region() {
        // J instruction preserves upper 4 bits of PC
        // PC = 0x80001000, Target = 0x00100000
        // Final address = 0x80400000
        let result = Disassembler::disassemble(0x08100000, 0x80001000);
        assert_eq!(result, "j 0x80400000");
    }

    #[test]
    fn test_disasm_jal_bios_region() {
        // JAL from BIOS region
        // PC = 0xBFC00100, Target = 0x00000100
        // Final address = (PC & 0xF0000000) | (target << 2)
        // = 0xB0000000 | (0x00000100 << 2) = 0xB0000400
        let result = Disassembler::disassemble(0x0C000100, 0xBFC00100);
        assert_eq!(result, "jal 0xB0000400");
    }

    #[test]
    fn test_disasm_j_kernel_region() {
        // J from kernel region
        // PC = 0xA0001000, Target = 0x00200000
        // Final address = 0xA0800000
        let result = Disassembler::disassemble(0x08200000, 0xA0001000);
        assert_eq!(result, "j 0xA0800000");
    }

    // ========== Negative Offsets and Edge Cases ==========

    #[test]
    fn test_disasm_beq_negative_offset() {
        // BEQ with negative offset
        let result = Disassembler::disassemble(0x1022FFFF, 0);
        assert_eq!(result, "beq r1, r2, -1");
    }

    #[test]
    fn test_disasm_bne_max_positive_offset() {
        // BNE with max positive offset (32767)
        let result = Disassembler::disassemble(0x14227FFF, 0);
        assert_eq!(result, "bne r1, r2, 32767");
    }

    #[test]
    fn test_disasm_bne_max_negative_offset() {
        // BNE with max negative offset (-32768)
        let result = Disassembler::disassemble(0x14228000, 0);
        assert_eq!(result, "bne r1, r2, -32768");
    }

    #[test]
    fn test_disasm_lw_negative_offset() {
        // LW with large negative offset
        let result = Disassembler::disassemble(0x8C228000, 0);
        assert_eq!(result, "lw r2, -32768(r1)");
    }

    #[test]
    fn test_disasm_sw_max_positive_offset() {
        // SW with max positive offset (32767)
        let result = Disassembler::disassemble(0xAC227FFF, 0);
        assert_eq!(result, "sw r2, 32767(r1)");
    }

    // ========== Special Register Cases ==========

    #[test]
    fn test_disasm_move_pseudo() {
        // MOVE pseudo-instruction (OR rd, rs, r0)
        let result = Disassembler::disassemble(0x00201825, 0);
        assert_eq!(result, "or r3, r1, r0");
    }

    #[test]
    fn test_disasm_zero_register() {
        // Operations involving r0
        let result = Disassembler::disassemble(0x00001020, 0); // ADD r2, r0, r0
        assert_eq!(result, "add r2, r0, r0");
    }

    #[test]
    fn test_disasm_return_address() {
        // JR r31 (common return instruction)
        let result = Disassembler::disassemble(0x03E00008, 0);
        assert_eq!(result, "jr r31");
    }

    // ========== Unknown Instruction Tests ==========

    #[test]
    fn test_disasm_invalid_special_funct() {
        // Invalid SPECIAL function code
        let result = Disassembler::disassemble(0x0000003F, 0);
        assert!(result.starts_with("???"));
    }

    #[test]
    fn test_disasm_invalid_regimm_rt() {
        // Invalid REGIMM rt field
        let result = Disassembler::disassemble(0x041FFFFF, 0);
        assert!(result.starts_with("???"));
    }

    #[test]
    fn test_disasm_invalid_cop0_rs() {
        // Invalid COP0 rs field
        let result = Disassembler::disassemble(0x42FFFFFF, 0);
        assert!(result.starts_with("???"));
    }

    #[test]
    fn test_disasm_invalid_cop0_funct() {
        // Invalid COP0 function in CO space
        let result = Disassembler::disassemble(0x4200001F, 0);
        assert!(result.starts_with("???"));
    }

    #[test]
    fn test_disasm_unimplemented_opcode() {
        // Unimplemented opcode (e.g., 0x30 - not defined)
        let result = Disassembler::disassemble(0xC0000000, 0);
        assert!(result.starts_with("???"));
    }
}
