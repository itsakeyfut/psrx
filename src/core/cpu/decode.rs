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

/// Decode R-type instruction
///
/// R-type instructions are used for register-to-register operations.
///
/// Format: | op (6) | rs (5) | rt (5) | rd (5) | shamt (5) | funct (6) |
///
/// # Arguments
///
/// * `instr` - The 32-bit instruction
///
/// # Returns
///
/// Tuple of (rs, rt, rd, shamt, funct)
#[inline(always)]
pub(super) fn decode_r_type(instr: u32) -> (u8, u8, u8, u8, u8) {
    let rs = ((instr >> 21) & 0x1F) as u8;
    let rt = ((instr >> 16) & 0x1F) as u8;
    let rd = ((instr >> 11) & 0x1F) as u8;
    let shamt = ((instr >> 6) & 0x1F) as u8;
    let funct = (instr & 0x3F) as u8;
    (rs, rt, rd, shamt, funct)
}

/// Decode I-type instruction
///
/// I-type instructions are used for immediate operations, loads, stores, and branches.
///
/// Format: | op (6) | rs (5) | rt (5) | immediate (16) |
///
/// # Arguments
///
/// * `instr` - The 32-bit instruction
///
/// # Returns
///
/// Tuple of (op, rs, rt, imm)
#[inline(always)]
pub(super) fn decode_i_type(instr: u32) -> (u8, u8, u8, u16) {
    let op = ((instr >> 26) & 0x3F) as u8;
    let rs = ((instr >> 21) & 0x1F) as u8;
    let rt = ((instr >> 16) & 0x1F) as u8;
    let imm = (instr & 0xFFFF) as u16;
    (op, rs, rt, imm)
}

/// Decode J-type instruction
///
/// J-type instructions are used for jump operations.
///
/// Format: | op (6) | target (26) |
///
/// # Arguments
///
/// * `instr` - The 32-bit instruction
///
/// # Returns
///
/// Tuple of (op, target)
#[inline(always)]
pub(super) fn decode_j_type(instr: u32) -> (u8, u32) {
    let op = ((instr >> 26) & 0x3F) as u8;
    let target = instr & 0x03FFFFFF;
    (op, target)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== R-Type Tests ==========

    #[test]
    fn test_decode_r_type_basic() {
        // Example: ADD r3, r1, r2
        // Format: 000000 00001 00010 00011 00000 100000
        // Encoding: 0x00221820
        let instr = 0x00221820;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 1); // r1
        assert_eq!(rt, 2); // r2
        assert_eq!(rd, 3); // r3
        assert_eq!(shamt, 0);
        assert_eq!(funct, 0x20); // ADD function code
    }

    #[test]
    fn test_decode_r_type_all_zeros() {
        // NOP instruction (all zeros)
        let instr = 0x00000000;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 0);
        assert_eq!(rt, 0);
        assert_eq!(rd, 0);
        assert_eq!(shamt, 0);
        assert_eq!(funct, 0);
    }

    #[test]
    fn test_decode_r_type_all_ones() {
        // All bits set (invalid instruction, but test decode)
        let instr = 0xFFFFFFFF;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 0x1F); // All 5 bits set
        assert_eq!(rt, 0x1F);
        assert_eq!(rd, 0x1F);
        assert_eq!(shamt, 0x1F);
        assert_eq!(funct, 0x3F); // All 6 bits set
    }

    #[test]
    fn test_decode_r_type_shift_instructions() {
        // SLL r4, r5, 8 (shift left logical by 8)
        // Format: 000000 00000 00101 00100 01000 000000
        // Encoding: 0x00052200
        let instr = 0x00052200;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 0);
        assert_eq!(rt, 5); // r5 (source)
        assert_eq!(rd, 4); // r4 (dest)
        assert_eq!(shamt, 8); // Shift amount
        assert_eq!(funct, 0x00); // SLL function code
    }

    #[test]
    fn test_decode_r_type_jr_instruction() {
        // JR r31 (jump register - return)
        // Format: 000000 11111 00000 00000 00000 001000
        // Encoding: 0x03E00008
        let instr = 0x03E00008;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 31); // r31 (return address)
        assert_eq!(rt, 0);
        assert_eq!(rd, 0);
        assert_eq!(shamt, 0);
        assert_eq!(funct, 0x08); // JR function code
    }

    #[test]
    fn test_decode_r_type_mult_instruction() {
        // MULT r5, r6 (multiply)
        // Format: 000000 00101 00110 00000 00000 011000
        // Encoding: 0x00A60018
        let instr = 0x00A60018;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 5);
        assert_eq!(rt, 6);
        assert_eq!(rd, 0); // Not used for MULT
        assert_eq!(shamt, 0);
        assert_eq!(funct, 0x18); // MULT function code
    }

    #[test]
    fn test_decode_r_type_mfhi_instruction() {
        // MFHI r8 (move from HI register)
        // Format: 000000 00000 00000 01000 00000 010000
        // Encoding: 0x00004010
        let instr = 0x00004010;
        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);

        assert_eq!(rs, 0);
        assert_eq!(rt, 0);
        assert_eq!(rd, 8); // Destination register
        assert_eq!(shamt, 0);
        assert_eq!(funct, 0x10); // MFHI function code
    }

    #[test]
    fn test_decode_r_type_syscall_instruction() {
        // SYSCALL
        // Format: 000000 <code> 001100
        // Encoding: 0x0000000C (code = 0)
        let instr = 0x0000000C;
        let (_rs, _rt, _rd, _shamt, funct) = decode_r_type(instr);

        assert_eq!(funct, 0x0C); // SYSCALL function code
    }

    #[test]
    fn test_decode_r_type_break_instruction() {
        // BREAK
        // Format: 000000 <code> 001101
        // Encoding: 0x0000000D (code = 0)
        let instr = 0x0000000D;
        let (_rs, _rt, _rd, _shamt, funct) = decode_r_type(instr);

        assert_eq!(funct, 0x0D); // BREAK function code
    }

    // ========== I-Type Tests ==========

    #[test]
    fn test_decode_i_type_basic() {
        // ADDI r2, r1, 0x42
        // Format: 001000 00001 00010 0000000001000010
        // Encoding: 0x24220042
        let instr = 0x24220042;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x09); // ADDIU opcode (ADDI is same encoding)
        assert_eq!(rs, 1); // r1 (source)
        assert_eq!(rt, 2); // r2 (dest)
        assert_eq!(imm, 0x42); // Immediate value
    }

    #[test]
    fn test_decode_i_type_negative_immediate() {
        // ADDI r3, r2, -1 (0xFFFF)
        // Format: 001000 00010 00011 1111111111111111
        // Encoding: 0x2043FFFF
        let instr = 0x2043FFFF;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x08); // ADDI opcode
        assert_eq!(rs, 2);
        assert_eq!(rt, 3);
        assert_eq!(imm, 0xFFFF); // -1 as unsigned 16-bit
        assert_eq!(imm as i16, -1); // Sign-extended interpretation
    }

    #[test]
    fn test_decode_i_type_load_word() {
        // LW r2, 4(r1) (load word)
        // Format: 100011 00001 00010 0000000000000100
        // Encoding: 0x8C220004
        let instr = 0x8C220004;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x23); // LW opcode
        assert_eq!(rs, 1); // Base register
        assert_eq!(rt, 2); // Destination register
        assert_eq!(imm, 4); // Offset
    }

    #[test]
    fn test_decode_i_type_store_word() {
        // SW r3, 0(r1) (store word)
        // Format: 101011 00001 00011 0000000000000000
        // Encoding: 0xAC230000
        let instr = 0xAC230000;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x2B); // SW opcode
        assert_eq!(rs, 1); // Base register
        assert_eq!(rt, 3); // Source register
        assert_eq!(imm, 0); // Offset
    }

    #[test]
    fn test_decode_i_type_branch_equal() {
        // BEQ r1, r2, offset
        // Format: 000100 00001 00010 0000000000001000
        // Encoding: 0x10220008
        let instr = 0x10220008;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x04); // BEQ opcode
        assert_eq!(rs, 1);
        assert_eq!(rt, 2);
        assert_eq!(imm, 8); // Branch offset (in instructions, not bytes)
    }

    #[test]
    fn test_decode_i_type_lui() {
        // LUI r1, 0x1234
        // Format: 001111 00000 00001 0001001000110100
        // Encoding: 0x3C011234
        let instr = 0x3C011234;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x0F); // LUI opcode
        assert_eq!(rs, 0); // Not used for LUI
        assert_eq!(rt, 1); // Destination register
        assert_eq!(imm, 0x1234); // Upper 16 bits to load
    }

    #[test]
    fn test_decode_i_type_andi() {
        // ANDI r2, r1, 0xFF00
        // Format: 001100 00001 00010 1111111100000000
        // Encoding: 0x3022FF00
        let instr = 0x3022FF00;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x0C); // ANDI opcode
        assert_eq!(rs, 1);
        assert_eq!(rt, 2);
        assert_eq!(imm, 0xFF00);
    }

    #[test]
    fn test_decode_i_type_ori() {
        // ORI r2, r1, 0x00FF
        // Format: 001101 00001 00010 0000000011111111
        // Encoding: 0x342200FF
        let instr = 0x342200FF;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x0D); // ORI opcode
        assert_eq!(rs, 1);
        assert_eq!(rt, 2);
        assert_eq!(imm, 0x00FF);
    }

    #[test]
    fn test_decode_i_type_all_zeros() {
        let instr = 0x00000000;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0);
        assert_eq!(rs, 0);
        assert_eq!(rt, 0);
        assert_eq!(imm, 0);
    }

    #[test]
    fn test_decode_i_type_all_ones() {
        let instr = 0xFFFFFFFF;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x3F); // All opcode bits set
        assert_eq!(rs, 0x1F); // All rs bits set
        assert_eq!(rt, 0x1F); // All rt bits set
        assert_eq!(imm, 0xFFFF); // All immediate bits set
    }

    #[test]
    fn test_decode_i_type_load_byte() {
        // LB r3, -4(r5) (load byte with negative offset)
        // Format: 100000 00101 00011 1111111111111100
        // Encoding: 0x80A3FFFC
        let instr = 0x80A3FFFC;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x20); // LB opcode
        assert_eq!(rs, 5);
        assert_eq!(rt, 3);
        assert_eq!(imm, 0xFFFC);
        assert_eq!(imm as i16, -4); // Sign-extended
    }

    #[test]
    fn test_decode_i_type_slti() {
        // SLTI r2, r1, 100
        // Format: 001010 00001 00010 0000000001100100
        // Encoding: 0x28220064
        let instr = 0x28220064;
        let (op, rs, rt, imm) = decode_i_type(instr);

        assert_eq!(op, 0x0A); // SLTI opcode
        assert_eq!(rs, 1);
        assert_eq!(rt, 2);
        assert_eq!(imm, 100);
    }

    // ========== J-Type Tests ==========

    #[test]
    fn test_decode_j_type_basic() {
        // J 0x00400000 (jump to address)
        // Format: 000010 00000100000000000000000000
        // Target: 0x00100000 (word address)
        // Encoding: 0x08100000
        let instr = 0x08100000;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x02); // J opcode
        assert_eq!(target, 0x00100000); // 26-bit target
    }

    #[test]
    fn test_decode_j_type_jal() {
        // JAL 0x00400000 (jump and link)
        // Format: 000011 00000100000000000000000000
        // Encoding: 0x0C100000
        let instr = 0x0C100000;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x03); // JAL opcode
        assert_eq!(target, 0x00100000);
    }

    #[test]
    fn test_decode_j_type_max_target() {
        // J with maximum target address
        // Format: 000010 11111111111111111111111111
        // Encoding: 0x0BFFFFFF
        let instr = 0x0BFFFFFF;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x02);
        assert_eq!(target, 0x03FFFFFF); // All 26 bits set
    }

    #[test]
    fn test_decode_j_type_zero_target() {
        // J 0 (jump to address 0)
        // Format: 000010 00000000000000000000000000
        // Encoding: 0x08000000
        let instr = 0x08000000;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x02);
        assert_eq!(target, 0);
    }

    #[test]
    fn test_decode_j_type_all_ones() {
        let instr = 0xFFFFFFFF;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x3F); // All opcode bits set
        assert_eq!(target, 0x03FFFFFF); // All target bits set
    }

    #[test]
    fn test_decode_j_type_bios_jump() {
        // Jump to BIOS area (0xBFC00000)
        // Target: 0x2FF00000 (word address)
        // Format: 000010 10111111000000000000000000
        // Encoding: 0x0BF00000
        let instr = 0x0BF00000;
        let (op, target) = decode_j_type(instr);

        assert_eq!(op, 0x02);
        assert_eq!(target, 0x03F00000);
        // Full address would be: (PC & 0xF0000000) | (target << 2)
        // = 0xB0000000 | (0x03F00000 << 2) = 0xBFC00000
    }

    // ========== Edge Cases and Bit Field Extraction Tests ==========

    #[test]
    fn test_decode_bit_field_isolation() {
        // Test that bit fields don't overlap
        let instr = 0xAAAAAAAA; // Alternating bit pattern

        let (op, _, _, _) = decode_i_type(instr);
        assert_eq!(op, (instr >> 26) as u8 & 0x3F);

        let (rs, rt, rd, shamt, funct) = decode_r_type(instr);
        assert_eq!(rs, ((instr >> 21) & 0x1F) as u8);
        assert_eq!(rt, ((instr >> 16) & 0x1F) as u8);
        assert_eq!(rd, ((instr >> 11) & 0x1F) as u8);
        assert_eq!(shamt, ((instr >> 6) & 0x1F) as u8);
        assert_eq!(funct, (instr & 0x3F) as u8);
    }

    #[test]
    fn test_decode_opcode_extraction_consistency() {
        // Same opcode should be extracted for all instruction types
        let opcodes = [0x00, 0x08, 0x23, 0x2B, 0x0F, 0x02, 0x03, 0x3F];

        for &opcode in &opcodes {
            let instr = (opcode as u32) << 26;

            let (op_i, _, _, _) = decode_i_type(instr);
            let (op_j, _) = decode_j_type(instr);

            assert_eq!(op_i, opcode);
            assert_eq!(op_j, opcode);
        }
    }

    #[test]
    fn test_decode_register_field_boundaries() {
        // Test register fields with boundary values (0-31)
        for reg in 0..32 {
            // Test rs field
            let instr_rs = (reg << 21) as u32;
            let (rs, _, _, _, _) = decode_r_type(instr_rs);
            assert_eq!(rs, reg as u8);

            // Test rt field
            let instr_rt = (reg << 16) as u32;
            let (_, rt, _, _, _) = decode_r_type(instr_rt);
            assert_eq!(rt, reg as u8);

            // Test rd field
            let instr_rd = (reg << 11) as u32;
            let (_, _, rd, _, _) = decode_r_type(instr_rd);
            assert_eq!(rd, reg as u8);
        }
    }

    #[test]
    fn test_decode_immediate_sign_extension() {
        // Test various immediate values that require sign extension
        let test_cases = [
            (0x0000, 0i16),      // Zero
            (0x0001, 1i16),      // Positive
            (0x7FFF, 32767i16),  // Max positive
            (0x8000, -32768i16), // Min negative
            (0xFFFF, -1i16),     // -1
            (0xFF00, -256i16),   // -256
            (0x0100, 256i16),    // 256
        ];

        for (imm_bits, expected_signed) in test_cases {
            let instr = imm_bits as u32; // Store in lower 16 bits
            let (_, _, _, imm) = decode_i_type(instr);
            assert_eq!(imm, imm_bits);
            assert_eq!(imm as i16, expected_signed);
        }
    }

    #[test]
    fn test_decode_shift_amount_range() {
        // Test shift amount (shamt) field with all valid values (0-31)
        for shamt_val in 0..32 {
            let instr = (shamt_val << 6) as u32;
            let (_, _, _, shamt, _) = decode_r_type(instr);
            assert_eq!(shamt, shamt_val as u8);
        }
    }
}
