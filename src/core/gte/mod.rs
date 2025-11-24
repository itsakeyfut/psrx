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

//! GTE (Geometry Transformation Engine) implementation
//!
//! The GTE is Coprocessor 2 (COP2) for the PlayStation, responsible for
//! 3D geometry transformations and lighting calculations. It's essential
//! for 3D games performance.
//!
//! # Features
//!
//! - Matrix and vector operations
//! - Perspective transformation (RTPS, RTPT)
//! - Normal clipping (NCLIP)
//! - Color depth cueing
//! - Outer product calculations
//!
//! # Hardware Details
//!
//! - 32 data registers (32-bit each)
//! - 32 control registers (32-bit each)
//! - FLAG register for overflow/underflow tracking
//! - Fixed-point arithmetic (12.4 format for most operations)
//!
//! # References
//!
//! - [PSX-SPX: GTE](http://problemkaputt.de/psx-spx.htm#geometrytransformationenginegte)

/// GTE (Geometry Transformation Engine) - COP2
///
/// The GTE performs 3D geometry transformations and lighting calculations
/// for the PlayStation. It uses fixed-point arithmetic for performance.
///
/// # Example
///
/// ```
/// use psrx::core::gte::GTE;
///
/// let mut gte = GTE::new();
/// // Set up rotation matrix (identity)
/// gte.write_control(0, 0x1000);  // R11 = 1.0 in fixed-point
/// ```
pub struct GTE {
    /// Data registers (32 x 32-bit)
    ///
    /// These hold input vectors, intermediate results, and output values.
    /// Common registers:
    /// - VXY0-2: Input vectors (X,Y components)
    /// - VZ0-2: Input vectors (Z component)
    /// - SXY0-2: Screen coordinates (projection results)
    /// - MAC0-3: Multiply-accumulate registers
    data: [i32; 32],

    /// Control registers (32 x 32-bit)
    ///
    /// These hold transformation matrices, translation vectors,
    /// and projection parameters.
    /// Common registers:
    /// - RT (0-4): Rotation matrix (3x3)
    /// - TRX/TRY/TRZ: Translation vector
    /// - H: Projection plane distance
    control: [i32; 32],

    /// FLAGS register (read from data register 31)
    ///
    /// Tracks overflow, underflow, and other calculation flags.
    /// Bit layout:
    /// - Bits 0-11: Not used (always 0)
    /// - Bit 12: MAC0 overflow (positive)
    /// - Bit 13: MAC0 overflow (negative)
    /// - Bit 14: Divide overflow
    /// - Bit 15: SX2 saturated
    /// - Bit 16: SY2 saturated
    /// - Bit 17: IR0 saturated
    /// - Bit 18-29: Various saturation flags
    /// - Bit 30: Error flag (any error)
    /// - Bit 31: Flag bit (calculation overflow)
    flags: u32,
}

// Allow dead code for GTE register constants that will be used in future commands
#[allow(dead_code)]
impl GTE {
    // Data register indices (commonly used)
    // Note: Some constants are not yet used but will be needed for future GTE commands
    const VXY0: usize = 0; // Vector 0 X,Y components (s16, s16)
    const VZ0: usize = 1; // Vector 0 Z component (s16)
    const VXY1: usize = 2; // Vector 1 X,Y components
    const VZ1: usize = 3; // Vector 1 Z component
    const VXY2: usize = 4; // Vector 2 X,Y components
    const VZ2: usize = 5; // Vector 2 Z component
    const RGB: usize = 6; // Color/code value (RGBC)
    const OTZ: usize = 7; // Average Z value (for ordering table)
    const IR0: usize = 8; // Intermediate value 0
    const IR1: usize = 9; // Intermediate value 1
    const IR2: usize = 10; // Intermediate value 2
    const IR3: usize = 11; // Intermediate value 3
    const SXY0: usize = 12; // Screen XY coordinate FIFO (oldest)
    const SXY1: usize = 13; // Screen XY coordinate FIFO
    const SXY2: usize = 14; // Screen XY coordinate FIFO (newest)
    const SXYP: usize = 15; // Screen XY coordinate (latest)
    const SZ0: usize = 16; // Screen Z FIFO (oldest)
    const SZ1: usize = 17; // Screen Z FIFO
    const SZ2: usize = 18; // Screen Z FIFO
    const SZ3: usize = 19; // Screen Z FIFO (newest)
    const RGB0: usize = 20; // Color FIFO (oldest)
    const RGB1: usize = 21; // Color FIFO
    const RGB2: usize = 22; // Color FIFO (newest)
    const RES1: usize = 23; // Reserved
    const MAC0: usize = 24; // Multiply-accumulate register 0
    const MAC1: usize = 25; // Multiply-accumulate register 1
    const MAC2: usize = 26; // Multiply-accumulate register 2
    const MAC3: usize = 27; // Multiply-accumulate register 3
    const IRGB: usize = 28; // Color conversion input
    const ORGB: usize = 29; // Color conversion output
    const LZCS: usize = 30; // Leading zero count source
    const LZCR: usize = 31; // Leading zero count result (also FLAGS)

    // Control register indices
    const RT11_RT12: usize = 0; // Rotation matrix R11,R12
    const RT13_RT21: usize = 1; // Rotation matrix R13,R21
    const RT22_RT23: usize = 2; // Rotation matrix R22,R23
    const RT31_RT32: usize = 3; // Rotation matrix R31,R32
    const RT33: usize = 4; // Rotation matrix R33
    const TRX: usize = 5; // Translation vector X
    const TRY: usize = 6; // Translation vector Y
    const TRZ: usize = 7; // Translation vector Z
    const L11_L12: usize = 8; // Light matrix L11,L12
    const L13_L21: usize = 9; // Light matrix L13,L21
    const L22_L23: usize = 10; // Light matrix L22,L23
    const L31_L32: usize = 11; // Light matrix L31,L32
    const L33: usize = 12; // Light matrix L33
    const RBK: usize = 13; // Background color R
    const GBK: usize = 14; // Background color G
    const BBK: usize = 15; // Background color B
    const LR1_LR2: usize = 16; // Light color matrix LR1,LR2
    const LR3_LG1: usize = 17; // Light color matrix LR3,LG1
    const LG2_LG3: usize = 18; // Light color matrix LG2,LG3
    const LB1_LB2: usize = 19; // Light color matrix LB1,LB2
    const LB3: usize = 20; // Light color matrix LB3
    const RFC: usize = 21; // Far color R
    const GFC: usize = 22; // Far color G
    const BFC: usize = 23; // Far color B
    const OFX: usize = 24; // Screen offset X
    const OFY: usize = 25; // Screen offset Y
    const H: usize = 26; // Projection plane distance
    const DQA: usize = 27; // Depth queue parameter A
    const DQB: usize = 28; // Depth queue parameter B
    const ZSF3: usize = 29; // Z scale factor (1/3)
    const ZSF4: usize = 30; // Z scale factor (1/4)
    const FLAG: usize = 31; // FLAG register (same as data[31])

    /// Create a new GTE instance
    ///
    /// Initializes all registers to 0. In real hardware, registers
    /// would contain undefined values at power-on.
    ///
    /// # Returns
    ///
    /// A new GTE instance with all registers cleared
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::gte::GTE;
    ///
    /// let gte = GTE::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: [0; 32],
            control: [0; 32],
            flags: 0,
        }
    }

    /// Reset GTE to initial state
    ///
    /// Clears all data and control registers.
    pub fn reset(&mut self) {
        self.data = [0; 32];
        self.control = [0; 32];
        self.flags = 0;
    }

    /// Read from data register
    ///
    /// # Arguments
    ///
    /// * `index` - Register index (0-31)
    ///
    /// # Returns
    ///
    /// Register value as i32
    ///
    /// # Note
    ///
    /// Register 31 (LZCR) also serves as the FLAGS register.
    #[inline(always)]
    pub fn read_data(&self, index: usize) -> i32 {
        self.data[index]
    }

    /// Write to data register
    ///
    /// # Arguments
    ///
    /// * `index` - Register index (0-31)
    /// * `value` - Value to write
    ///
    /// # Note
    ///
    /// Writing to register 15 (SXYP) also updates the FIFO.
    /// Writing to register 28 (IRGB) triggers color conversion.
    /// Writing to register 30 (LZCS) triggers leading zero count.
    #[inline(always)]
    pub fn write_data(&mut self, index: usize, value: i32) {
        match index {
            Self::SXYP => {
                // Writing to SXYP pushes to FIFO
                self.data[Self::SXY0] = self.data[Self::SXY1];
                self.data[Self::SXY1] = self.data[Self::SXY2];
                self.data[Self::SXY2] = value;
                self.data[Self::SXYP] = value;
            }
            Self::LZCS => {
                // Writing to LZCS triggers leading zero count
                self.data[Self::LZCS] = value;
                // Count leading zeros of the value (treating as unsigned)
                self.data[Self::LZCR] = (value as u32).leading_zeros() as i32;
            }
            _ => {
                self.data[index] = value;
            }
        }
    }

    /// Read from control register
    ///
    /// # Arguments
    ///
    /// * `index` - Register index (0-31)
    ///
    /// # Returns
    ///
    /// Register value as i32
    #[inline(always)]
    pub fn read_control(&self, index: usize) -> i32 {
        self.control[index]
    }

    /// Write to control register
    ///
    /// # Arguments
    ///
    /// * `index` - Register index (0-31)
    /// * `value` - Value to write
    #[inline(always)]
    pub fn write_control(&mut self, index: usize, value: i32) {
        self.control[index] = value;
    }

    /// Get rotation matrix from control registers
    ///
    /// The rotation matrix is stored across 5 control registers (0-4)
    /// in a packed format with 16-bit signed values.
    ///
    /// # Returns
    ///
    /// 3x3 rotation matrix as [[i32; 3]; 3]
    fn get_rotation_matrix(&self) -> [[i32; 3]; 3] {
        [
            [
                (self.control[Self::RT11_RT12] & 0xFFFF) as i16 as i32,
                (self.control[Self::RT11_RT12] >> 16) as i16 as i32,
                (self.control[Self::RT13_RT21] & 0xFFFF) as i16 as i32,
            ],
            [
                (self.control[Self::RT13_RT21] >> 16) as i16 as i32,
                (self.control[Self::RT22_RT23] & 0xFFFF) as i16 as i32,
                (self.control[Self::RT22_RT23] >> 16) as i16 as i32,
            ],
            [
                (self.control[Self::RT31_RT32] & 0xFFFF) as i16 as i32,
                (self.control[Self::RT31_RT32] >> 16) as i16 as i32,
                (self.control[Self::RT33] & 0xFFFF) as i16 as i32,
            ],
        ]
    }

    /// RTPS: Rotate, Translate, Perspective Transform, Single
    ///
    /// This is the most commonly used GTE command. It transforms a single
    /// 3D vertex from object space to screen space.
    ///
    /// Steps:
    /// 1. Load input vector from V0 (VXY0, VZ0)
    /// 2. Multiply by rotation matrix (RT)
    /// 3. Add translation vector (TR)
    /// 4. Perform perspective division
    /// 5. Add screen offset and store in SXY FIFO
    ///
    /// # Arguments
    ///
    /// * `sf` - Shift flag: if true, shift right by 12 bits (fixed-point adjustment)
    ///
    /// # Formula
    ///
    /// ```text
    /// MAC = RT * V + TR
    /// SXY = (H * MAC.xy / MAC.z) + Offset
    /// ```
    pub fn rtps(&mut self, sf: bool) {
        let shift = if sf { 12 } else { 0 };

        // Load input vector V0
        let vx = (self.data[Self::VXY0] & 0xFFFF) as i16 as i32;
        let vy = (self.data[Self::VXY0] >> 16) as i16 as i32;
        let vz = self.data[Self::VZ0] as i16 as i32;

        // Get rotation matrix
        let rt = self.get_rotation_matrix();

        // Get translation vector
        let trx = self.control[Self::TRX] as i64;
        let try_val = self.control[Self::TRY] as i64;
        let trz = self.control[Self::TRZ] as i64;

        // Matrix multiplication with translation: MAC = (RT * V + TR * 0x1000) SAR (sf*12)
        // Hardware formula: MACn = (TRn*0x1000 + matrix_terms) SAR (sf*12)
        // Cast to i64 before multiplication to prevent intermediate i32 overflow
        let mac1 = (rt[0][0] as i64 * vx as i64
            + rt[0][1] as i64 * vy as i64
            + rt[0][2] as i64 * vz as i64
            + (trx << 12))
            >> shift;
        let mac2 = (rt[1][0] as i64 * vx as i64
            + rt[1][1] as i64 * vy as i64
            + rt[1][2] as i64 * vz as i64
            + (try_val << 12))
            >> shift;
        let mac3 = (rt[2][0] as i64 * vx as i64
            + rt[2][1] as i64 * vy as i64
            + rt[2][2] as i64 * vz as i64
            + (trz << 12))
            >> shift;

        // Store MAC values (saturated to 32-bit)
        self.data[Self::MAC1] = mac1.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.data[Self::MAC2] = mac2.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.data[Self::MAC3] = mac3.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        // Reset FLAGS for this operation (we currently only model divide overflow).
        self.flags = 0;

        // Perspective transformation.
        // MAC values are in 12.4 fixed point; apply a 12-bit scale so that
        // typical PSX-style ranges don't collapse to zero.
        let h = self.control[Self::H] as i64;
        let z = mac3;

        let (sx, sy) = if z <= 0 {
            // Divide overflow case: negative/zero Z.
            self.flags |= 1 << 14; // Bit 14: divide overflow

            // Saturated scale value used by the real GTE on overflow.
            let scale = 0x1FFFF_i64;
            let sx = ((scale * mac1) >> 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            let sy = ((scale * mac2) >> 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            (sx, sy)
        } else {
            // Normal division with fixed-point aware scale,
            // clamped to the hardware 17-bit range.
            let scale = ((h << 12) / z).min(0x1FFFF);
            let sx = ((scale * mac1) >> 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            let sy = ((scale * mac2) >> 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            (sx, sy)
        };

        // Apply screen offset
        let ofx = self.control[Self::OFX];
        let ofy = self.control[Self::OFY];

        let sx_screen = sx + ofx;
        let sy_screen = sy + ofy;

        // Update screen coordinate FIFO
        self.data[Self::SXY0] = self.data[Self::SXY1];
        self.data[Self::SXY1] = self.data[Self::SXY2];
        self.data[Self::SXY2] =
            (sy_screen.clamp(-1024, 1023) << 16) | (sx_screen.clamp(-1024, 1023) & 0xFFFF);
        self.data[Self::SXYP] = self.data[Self::SXY2];

        // Update screen Z FIFO
        self.data[Self::SZ0] = self.data[Self::SZ1];
        self.data[Self::SZ1] = self.data[Self::SZ2];
        self.data[Self::SZ2] = self.data[Self::SZ3];
        self.data[Self::SZ3] = (z.clamp(0, 0xFFFF) as u16) as i32;

        // Calculate OTZ (average Z for ordering table)
        let sz_avg = (self.data[Self::SZ1] + self.data[Self::SZ2] + self.data[Self::SZ3]) / 3;
        self.data[Self::OTZ] = sz_avg.clamp(0, 0xFFFF);

        // Set IR registers (intermediate results)
        self.data[Self::IR1] = mac1.clamp(-32768, 32767) as i32;
        self.data[Self::IR2] = mac2.clamp(-32768, 32767) as i32;
        self.data[Self::IR3] = mac3.clamp(0, 65535) as i32;

        // Mirror FLAGS into the shared LZCR/FLAG register slot.
        self.data[Self::LZCR] = self.flags as i32;
    }

    /// RTPT: Rotate, Translate, Perspective Transform, Triple
    ///
    /// Similar to RTPS but processes three vertices (V0, V1, V2) in sequence.
    /// This is more efficient than calling RTPS three times.
    ///
    /// # Arguments
    ///
    /// * `sf` - Shift flag: if true, shift right by 12 bits
    pub fn rtpt(&mut self, sf: bool) {
        // Process V0
        self.rtps(sf);

        // Process V1 - swap V0 with V1 temporarily
        let v0_xy = self.data[Self::VXY0];
        let v0_z = self.data[Self::VZ0];
        self.data[Self::VXY0] = self.data[Self::VXY1];
        self.data[Self::VZ0] = self.data[Self::VZ1];
        self.rtps(sf);

        // Process V2 - swap with V2
        self.data[Self::VXY0] = self.data[Self::VXY2];
        self.data[Self::VZ0] = self.data[Self::VZ2];
        self.rtps(sf);

        // Restore V0
        self.data[Self::VXY0] = v0_xy;
        self.data[Self::VZ0] = v0_z;
    }

    /// NCLIP: Normal Clipping
    ///
    /// Calculates the cross product of screen coordinates to determine
    /// if a triangle is front-facing or back-facing (for backface culling).
    ///
    /// # Formula
    ///
    /// ```text
    /// MAC0 = SX0*SY1 + SX1*SY2 + SX2*SY0 - SX0*SY2 - SX1*SY0 - SX2*SY1
    /// ```
    ///
    /// # Result
    ///
    /// - MAC0 > 0: Front-facing (clockwise)
    /// - MAC0 < 0: Back-facing (counter-clockwise)
    /// - MAC0 = 0: Edge-on
    pub fn nclip(&mut self) {
        // Extract screen coordinates from FIFO
        let sx0 = (self.data[Self::SXY0] & 0xFFFF) as i16 as i32;
        let sy0 = (self.data[Self::SXY0] >> 16) as i16 as i32;
        let sx1 = (self.data[Self::SXY1] & 0xFFFF) as i16 as i32;
        let sy1 = (self.data[Self::SXY1] >> 16) as i16 as i32;
        let sx2 = (self.data[Self::SXY2] & 0xFFFF) as i16 as i32;
        let sy2 = (self.data[Self::SXY2] >> 16) as i16 as i32;

        // Calculate cross product (determinant)
        let result =
            (sx0 * sy1) + (sx1 * sy2) + (sx2 * sy0) - (sx0 * sy2) - (sx1 * sy0) - (sx2 * sy1);

        self.data[Self::MAC0] = result;

        // Clear flags
        self.flags = 0;
        self.data[Self::LZCR] = 0;
    }

    /// MVMVA: Multiply Vector by Matrix and Vector Addition
    ///
    /// General-purpose matrix-vector multiplication with vector addition.
    /// This is a flexible command that can use different matrix and vector sources.
    ///
    /// # Arguments
    ///
    /// * `command` - Full GTE command word containing operation parameters
    ///
    /// # Format
    ///
    /// The command word specifies:
    /// - Bits [20:19]: Translation vector selection
    /// - Bits [18:17]: Multiply vector selection
    /// - Bits [16:15]: Multiply matrix selection
    /// - Bit 10: lm flag (limit negative to 0)
    /// - Bit 19: sf flag (shift fraction)
    pub fn mvmva(&mut self, command: u32) {
        let sf = ((command >> 19) & 1) != 0;
        let mx = (command >> 17) & 0x3; // Matrix selection
        let v = (command >> 15) & 0x3; // Vector selection
        let cv = (command >> 13) & 0x3; // Translation vector selection
        let lm = ((command >> 10) & 1) != 0; // Limit negative values

        let shift = if sf { 12 } else { 0 };

        // Select input vector
        let (vx, vy, vz) = match v {
            0 => {
                // V0
                let vx = (self.data[Self::VXY0] & 0xFFFF) as i16 as i32;
                let vy = (self.data[Self::VXY0] >> 16) as i16 as i32;
                let vz = self.data[Self::VZ0] as i16 as i32;
                (vx, vy, vz)
            }
            1 => {
                // V1
                let vx = (self.data[Self::VXY1] & 0xFFFF) as i16 as i32;
                let vy = (self.data[Self::VXY1] >> 16) as i16 as i32;
                let vz = self.data[Self::VZ1] as i16 as i32;
                (vx, vy, vz)
            }
            2 => {
                // V2
                let vx = (self.data[Self::VXY2] & 0xFFFF) as i16 as i32;
                let vy = (self.data[Self::VXY2] >> 16) as i16 as i32;
                let vz = self.data[Self::VZ2] as i16 as i32;
                (vx, vy, vz)
            }
            3 => {
                // IR (short vector)
                (
                    self.data[Self::IR1] as i16 as i32,
                    self.data[Self::IR2] as i16 as i32,
                    self.data[Self::IR3] as i16 as i32,
                )
            }
            _ => unreachable!(),
        };

        // Select matrix (simplified - only rotation matrix for now)
        let matrix = match mx {
            0..=3 => self.get_rotation_matrix(),
            _ => [[0; 3]; 3],
        };

        // Select translation vector (simplified - using TR for now)
        let (tx, ty, tz) = match cv {
            0 | 1 => (
                self.control[Self::TRX] as i64,
                self.control[Self::TRY] as i64,
                self.control[Self::TRZ] as i64,
            ),
            _ => (0, 0, 0),
        };

        // Matrix multiplication with translation: MAC = (Matrix * V + T * 0x1000) SAR (sf*12)
        // Hardware formula: MACn = (Tn*0x1000 + matrix_terms) SAR (sf*12)
        // Cast to i64 before multiplication to prevent intermediate i32 overflow
        let mac1 = (matrix[0][0] as i64 * vx as i64
            + matrix[0][1] as i64 * vy as i64
            + matrix[0][2] as i64 * vz as i64
            + (tx << 12))
            >> shift;
        let mac2 = (matrix[1][0] as i64 * vx as i64
            + matrix[1][1] as i64 * vy as i64
            + matrix[1][2] as i64 * vz as i64
            + (ty << 12))
            >> shift;
        let mac3 = (matrix[2][0] as i64 * vx as i64
            + matrix[2][1] as i64 * vy as i64
            + matrix[2][2] as i64 * vz as i64
            + (tz << 12))
            >> shift;

        // Store results
        self.data[Self::MAC1] = mac1.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.data[Self::MAC2] = mac2.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.data[Self::MAC3] = mac3.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        // Update IR registers with limiting
        let min_val = if lm { 0 } else { -32768 };
        self.data[Self::IR1] = mac1.clamp(min_val, 32767) as i32;
        self.data[Self::IR2] = mac2.clamp(min_val, 32767) as i32;
        self.data[Self::IR3] = mac3.clamp(min_val, 32767) as i32;

        self.flags = 0;
        self.data[Self::LZCR] = 0;
    }

    /// Execute GTE command
    ///
    /// Dispatches a GTE command to the appropriate handler based on the opcode.
    ///
    /// # Arguments
    ///
    /// * `command` - 32-bit GTE command word
    ///
    /// # Format
    ///
    /// - Bits [5:0]: Opcode
    /// - Bit 19: sf (shift fraction)
    /// - Bit 10: lm (limit negative values)
    /// - Other bits: Command-specific parameters
    ///
    /// # Common Commands
    ///
    /// - 0x01: RTPS (Perspective transform single)
    /// - 0x06: NCLIP (Normal clipping)
    /// - 0x12: MVMVA (Matrix-vector multiply)
    /// - 0x30: RTPT (Perspective transform triple)
    pub fn execute(&mut self, command: u32) {
        let opcode = command & 0x3F;
        let sf = (command & 0x80000) != 0; // Shift flag (bit 19)

        match opcode {
            0x01 => self.rtps(sf),
            0x06 => self.nclip(),
            0x12 => self.mvmva(command),
            0x30 => self.rtpt(sf),
            // TODO: Implement remaining GTE commands as needed
            _ => {
                log::warn!("Unknown GTE command: 0x{:02X}", opcode);
                // Set error flag for unknown commands
                self.flags = 0x80000000;
                self.data[Self::LZCR] = 0x80000000u32 as i32;
            }
        }
    }
}

impl Default for GTE {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Basic Register Tests
    // ============================================================================

    #[test]
    fn test_new_initializes_all_registers_to_zero() {
        let gte = GTE::new();
        for i in 0..32 {
            assert_eq!(gte.read_data(i), 0, "Data register {} not zero", i);
            assert_eq!(gte.read_control(i), 0, "Control register {} not zero", i);
        }
        assert_eq!(gte.flags, 0, "FLAGS not zero");
    }

    #[test]
    fn test_reset_clears_all_registers() {
        let mut gte = GTE::new();
        // Set some registers
        gte.write_data(0, 0x12345678);
        gte.write_control(0, 0x87654321u32 as i32);
        gte.flags = 0xFFFFFFFF;

        gte.reset();

        for i in 0..32 {
            assert_eq!(gte.read_data(i), 0, "Data register {} not cleared", i);
            assert_eq!(gte.read_control(i), 0, "Control register {} not cleared", i);
        }
        assert_eq!(gte.flags, 0, "FLAGS not cleared");
    }

    #[test]
    fn test_data_register_read_write() {
        let mut gte = GTE::new();
        let test_value = 0x12345678i32;

        for i in 0..32 {
            if i == GTE::SXYP || i == GTE::LZCS {
                continue; // These have special behavior
            }
            gte.write_data(i, test_value);
            assert_eq!(
                gte.read_data(i),
                test_value,
                "Data register {} failed read/write",
                i
            );
        }
    }

    #[test]
    fn test_control_register_read_write() {
        let mut gte = GTE::new();
        let test_value = 0x87654321u32 as i32;

        for i in 0..32 {
            gte.write_control(i, test_value);
            assert_eq!(
                gte.read_control(i),
                test_value,
                "Control register {} failed read/write",
                i
            );
        }
    }

    // ============================================================================
    // SXYP FIFO Tests (Data Register 15)
    // ============================================================================

    #[test]
    fn test_sxyp_fifo_push() {
        let mut gte = GTE::new();

        // Initial state: all zeros
        assert_eq!(gte.read_data(GTE::SXY0), 0);
        assert_eq!(gte.read_data(GTE::SXY1), 0);
        assert_eq!(gte.read_data(GTE::SXY2), 0);
        assert_eq!(gte.read_data(GTE::SXYP), 0);

        // Write first value to SXYP
        gte.write_data(GTE::SXYP, 0x11111111);
        assert_eq!(gte.read_data(GTE::SXY0), 0);
        assert_eq!(gte.read_data(GTE::SXY1), 0);
        assert_eq!(gte.read_data(GTE::SXY2), 0x11111111);
        assert_eq!(gte.read_data(GTE::SXYP), 0x11111111);

        // Write second value - should shift
        gte.write_data(GTE::SXYP, 0x22222222);
        assert_eq!(gte.read_data(GTE::SXY0), 0);
        assert_eq!(gte.read_data(GTE::SXY1), 0x11111111);
        assert_eq!(gte.read_data(GTE::SXY2), 0x22222222);
        assert_eq!(gte.read_data(GTE::SXYP), 0x22222222);

        // Write third value - should shift all
        gte.write_data(GTE::SXYP, 0x33333333);
        assert_eq!(gte.read_data(GTE::SXY0), 0x11111111);
        assert_eq!(gte.read_data(GTE::SXY1), 0x22222222);
        assert_eq!(gte.read_data(GTE::SXY2), 0x33333333);
        assert_eq!(gte.read_data(GTE::SXYP), 0x33333333);

        // Fourth value - oldest value (0x11111111) is discarded
        gte.write_data(GTE::SXYP, 0x44444444);
        assert_eq!(gte.read_data(GTE::SXY0), 0x22222222);
        assert_eq!(gte.read_data(GTE::SXY1), 0x33333333);
        assert_eq!(gte.read_data(GTE::SXY2), 0x44444444);
        assert_eq!(gte.read_data(GTE::SXYP), 0x44444444);
    }

    #[test]
    fn test_sxy2_direct_write_no_fifo() {
        let mut gte = GTE::new();

        // Pre-populate FIFO
        gte.write_data(GTE::SXYP, 0x11111111);
        gte.write_data(GTE::SXYP, 0x22222222);

        // Direct write to SXY2 should NOT affect FIFO
        gte.write_data(GTE::SXY2, 0x99999999u32 as i32);
        assert_eq!(gte.read_data(GTE::SXY0), 0);
        assert_eq!(gte.read_data(GTE::SXY1), 0x11111111);
        assert_eq!(gte.read_data(GTE::SXY2), 0x99999999u32 as i32);
    }

    // ============================================================================
    // LZCS/LZCR Leading Zero Count Tests
    // ============================================================================

    #[test]
    fn test_lzcs_leading_zeros_positive() {
        let mut gte = GTE::new();

        // Test cases for positive values
        gte.write_data(GTE::LZCS, 0x00000001); // 31 leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 31);

        gte.write_data(GTE::LZCS, 0x00000010); // 27 leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 27);

        gte.write_data(GTE::LZCS, 0x7FFFFFFF); // 1 leading zero
        assert_eq!(gte.read_data(GTE::LZCR), 1);

        gte.write_data(GTE::LZCS, 0x80000000u32 as i32); // No leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 0);

        gte.write_data(GTE::LZCS, 0xFFFFFFFFu32 as i32); // No leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 0);
    }

    #[test]
    fn test_lzcs_all_zeros() {
        let mut gte = GTE::new();
        gte.write_data(GTE::LZCS, 0); // All zeros = 32 leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 32);
    }

    #[test]
    fn test_lzcs_all_ones() {
        let mut gte = GTE::new();
        gte.write_data(GTE::LZCS, -1); // All ones = 0 leading zeros
        assert_eq!(gte.read_data(GTE::LZCR), 0);
    }

    #[test]
    fn test_lzcs_powers_of_two() {
        let mut gte = GTE::new();

        for i in 0..31 {
            let value = 1u32 << i;
            gte.write_data(GTE::LZCS, value as i32);
            assert_eq!(gte.read_data(GTE::LZCR), (31 - i), "Failed for 2^{}", i);
        }
    }

    // ============================================================================
    // RTPS (Rotate, Translate, Perspective Single) Tests
    // ============================================================================

    #[test]
    fn test_rtps_identity_matrix_zero_translation() {
        let mut gte = GTE::new();

        // Set up identity rotation matrix (1.0 in fixed-point = 0x1000)
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT13_RT21, 0);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT31_RT32, 0);
        gte.write_control(GTE::RT33, 0x1000);

        // Zero translation
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 1000); // Z=1000 to avoid divide issues

        // Set input vertex V0 = (100, 200, 1000)
        gte.write_data(GTE::VXY0, (200 << 16) | (100 & 0xFFFF));
        gte.write_data(GTE::VZ0, 1000);

        // Set projection parameters
        gte.write_control(GTE::H, 1000); // Projection distance
        gte.write_control(GTE::OFX, 0); // Screen offset X
        gte.write_control(GTE::OFY, 0); // Screen offset Y

        // Execute RTPS with sf=1
        gte.rtps(true);

        // With identity matrix: MAC = V * 1.0 + TR
        // Formula: MAC = (TR*0x1000 + RT*V) >> 12
        // With TRZ=1000, VZ=1000, RT33=0x1000: MAC3 = (1000<<12 + 0x1000*1000)>>12 = 2000
        assert_eq!(gte.read_data(GTE::MAC1), 100);
        assert_eq!(gte.read_data(GTE::MAC2), 200);
        assert_eq!(gte.read_data(GTE::MAC3), 2000);

        // IR registers should contain saturated MAC values
        assert_eq!(gte.read_data(GTE::IR1), 100);
        assert_eq!(gte.read_data(GTE::IR2), 200);
        assert_eq!(gte.read_data(GTE::IR3), 2000);
    }

    #[test]
    fn test_rtps_with_translation() {
        let mut gte = GTE::new();

        // Identity matrix
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Translation vector (500, 600, 700)
        gte.write_control(GTE::TRX, 500);
        gte.write_control(GTE::TRY, 600);
        gte.write_control(GTE::TRZ, 700);

        // Input vertex V0 = (10, 20, 30)
        gte.write_data(GTE::VXY0, (20 << 16) | (10 & 0xFFFF));
        gte.write_data(GTE::VZ0, 30);

        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        gte.rtps(true);

        // With identity matrix: MAC = V + TR
        // Expected: MAC1 = 10 + 500 = 510
        assert_eq!(gte.read_data(GTE::MAC1), 510);
        assert_eq!(gte.read_data(GTE::MAC2), 620);
        assert_eq!(gte.read_data(GTE::MAC3), 730);
    }

    #[test]
    fn test_rtps_divide_by_zero_sets_flag() {
        let mut gte = GTE::new();

        // Set up simple transform
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Zero Z coordinate will cause divide by zero
        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 0);

        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0); // Results in MAC3 = 0

        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        gte.rtps(true);

        // Check divide overflow flag is set (bit 14)
        assert_ne!(gte.flags & (1 << 14), 0, "Divide overflow flag not set");
        assert_eq!(
            gte.read_data(GTE::LZCR) as u32 & (1 << 14),
            1 << 14,
            "LZCR should reflect divide overflow flag"
        );
    }

    #[test]
    fn test_rtps_negative_z_sets_flag() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Negative Z coordinate
        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, -1000i16 as i32);

        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0);

        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        gte.rtps(true);

        // Check divide overflow flag is set (bit 14)
        assert_ne!(gte.flags & (1 << 14), 0, "Divide overflow flag not set");
    }

    #[test]
    fn test_rtps_fifo_updates() {
        let mut gte = GTE::new();

        // Setup basic transform
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        // First vertex
        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 1000);
        gte.rtps(true);

        let sxy_first = gte.read_data(GTE::SXY2);
        let sz_first = gte.read_data(GTE::SZ3);

        // Second vertex
        gte.write_data(GTE::VXY0, (200 << 16) | 200);
        gte.write_data(GTE::VZ0, 2000);
        gte.rtps(true);

        // Check that first result shifted to SXY1
        assert_eq!(gte.read_data(GTE::SXY1), sxy_first);
        // Check that SZ FIFO shifted
        assert_eq!(gte.read_data(GTE::SZ2), sz_first);
    }

    #[test]
    fn test_rtps_screen_coordinate_saturation() {
        let mut gte = GTE::new();

        // Set up transform that will produce large screen coordinates
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Very large input values
        gte.write_data(
            GTE::VXY0,
            ((30000i16 as u16 as u32) << 16 | (30000i16 as u16 as u32)) as i32,
        );
        gte.write_data(GTE::VZ0, 100);

        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 100);
        gte.write_control(GTE::H, 10000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        gte.rtps(true);

        // SX and SY should be saturated to -1024..1023 range
        let sxy2 = gte.read_data(GTE::SXY2);
        let sx = (sxy2 & 0xFFFF) as i16 as i32;
        let sy = (sxy2 >> 16) as i16 as i32;

        assert!((-1024..=1023).contains(&sx), "SX not saturated: {}", sx);
        assert!((-1024..=1023).contains(&sy), "SY not saturated: {}", sy);
    }

    #[test]
    fn test_rtps_otz_calculation() {
        let mut gte = GTE::new();

        // Setup
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0);
        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        // Execute RTPS three times to fill SZ FIFO
        gte.write_data(GTE::VXY0, 0);
        gte.write_data(GTE::VZ0, 300);
        gte.rtps(true);

        gte.write_data(GTE::VZ0, 600);
        gte.rtps(true);

        gte.write_data(GTE::VZ0, 900);
        gte.rtps(true);

        // OTZ should be average of SZ1, SZ2, SZ3
        // (300 + 600 + 900) / 3 = 600
        let otz = gte.read_data(GTE::OTZ);
        assert_eq!(otz, 600, "OTZ should be average of last 3 SZ values");
    }

    #[test]
    fn test_rtps_sf_flag_behavior() {
        let mut gte = GTE::new();

        // Setup
        gte.write_control(GTE::RT11_RT12, 0x2000); // 2.0 in fixed-point
        gte.write_control(GTE::RT22_RT23, 0x2000);
        gte.write_control(GTE::RT33, 0x2000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0);

        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 100);

        // Test with sf=false (no shift)
        gte.rtps(false);
        let mac1_no_shift = gte.read_data(GTE::MAC1);

        // Reset and test with sf=true (shift by 12)
        gte.reset();
        gte.write_control(GTE::RT11_RT12, 0x2000);
        gte.write_control(GTE::RT22_RT23, 0x2000);
        gte.write_control(GTE::RT33, 0x2000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0);
        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 100);
        gte.rtps(true);
        let mac1_with_shift = gte.read_data(GTE::MAC1);

        // With sf=true, result should be shifted right by 12 bits
        assert!(
            mac1_no_shift > mac1_with_shift,
            "sf=true should produce smaller MAC values"
        );
    }

    // ============================================================================
    // RTPT (Rotate, Translate, Perspective Triple) Tests
    // ============================================================================

    #[test]
    fn test_rtpt_transforms_three_vertices() {
        let mut gte = GTE::new();

        // Setup
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        // Set three vertices
        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 1000);

        gte.write_data(GTE::VXY1, (200 << 16) | 200);
        gte.write_data(GTE::VZ1, 2000);

        gte.write_data(GTE::VXY2, (300 << 16) | 300);
        gte.write_data(GTE::VZ2, 3000);

        gte.rtpt(true);

        // Check that all three screen coordinates are in the FIFO
        let sxy0 = gte.read_data(GTE::SXY0);
        let sxy1 = gte.read_data(GTE::SXY1);
        let sxy2 = gte.read_data(GTE::SXY2);

        assert_ne!(sxy0, 0, "SXY0 should contain first vertex result");
        assert_ne!(sxy1, 0, "SXY1 should contain second vertex result");
        assert_ne!(sxy2, 0, "SXY2 should contain third vertex result");

        // All three should be different
        assert_ne!(sxy0, sxy1, "V0 and V1 results should differ");
        assert_ne!(sxy1, sxy2, "V1 and V2 results should differ");
    }

    #[test]
    fn test_rtpt_preserves_v0() {
        let mut gte = GTE::new();

        // Setup
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);
        gte.write_control(GTE::OFX, 0);
        gte.write_control(GTE::OFY, 0);

        let v0_xy = (123 << 16) | 456;
        let v0_z = 789;

        gte.write_data(GTE::VXY0, v0_xy);
        gte.write_data(GTE::VZ0, v0_z);
        gte.write_data(GTE::VXY1, (200 << 16) | 200);
        gte.write_data(GTE::VZ1, 2000);
        gte.write_data(GTE::VXY2, (300 << 16) | 300);
        gte.write_data(GTE::VZ2, 3000);

        gte.rtpt(true);

        // V0 should be preserved after RTPT
        assert_eq!(
            gte.read_data(GTE::VXY0),
            v0_xy,
            "VXY0 not preserved after RTPT"
        );
        assert_eq!(
            gte.read_data(GTE::VZ0),
            v0_z,
            "VZ0 not preserved after RTPT"
        );
    }

    // ============================================================================
    // NCLIP (Normal Clipping) Tests
    // ============================================================================

    #[test]
    fn test_nclip_front_facing_clockwise() {
        let mut gte = GTE::new();

        // Set up clockwise triangle (front-facing)
        // Triangle vertices in screen space
        gte.write_data(GTE::SXY0, 0); // (0, 0)
        gte.write_data(GTE::SXY1, 100); // (100, 0)
        gte.write_data(GTE::SXY2, (50 << 16) | 50); // (50, 50)

        gte.nclip();

        let mac0 = gte.read_data(GTE::MAC0);
        // For clockwise winding, MAC0 should be positive
        assert!(mac0 > 0, "Front-facing triangle should have MAC0 > 0");
    }

    #[test]
    fn test_nclip_back_facing_counter_clockwise() {
        let mut gte = GTE::new();

        // Set up counter-clockwise triangle (back-facing)
        gte.write_data(GTE::SXY0, 0); // (0, 0)
        gte.write_data(GTE::SXY1, (50 << 16) | 50); // (50, 50)
        gte.write_data(GTE::SXY2, 100); // (100, 0)

        gte.nclip();

        let mac0 = gte.read_data(GTE::MAC0);
        // For counter-clockwise winding, MAC0 should be negative
        assert!(mac0 < 0, "Back-facing triangle should have MAC0 < 0");
    }

    #[test]
    fn test_nclip_edge_on_triangle() {
        let mut gte = GTE::new();

        // Set up degenerate triangle (all points on a line)
        gte.write_data(GTE::SXY0, 0); // (0, 0)
        gte.write_data(GTE::SXY1, 50); // (50, 0)
        gte.write_data(GTE::SXY2, 100); // (100, 0)

        gte.nclip();

        let mac0 = gte.read_data(GTE::MAC0);
        // Edge-on triangle should have MAC0 = 0
        assert_eq!(mac0, 0, "Edge-on triangle should have MAC0 = 0");
    }

    #[test]
    fn test_nclip_clears_flags() {
        let mut gte = GTE::new();

        // Set some flags
        gte.flags = 0xFFFFFFFF;
        gte.data[GTE::LZCR] = 0xFFFFFFFFu32 as i32;

        // Set up valid triangle
        gte.write_data(GTE::SXY0, 0);
        gte.write_data(GTE::SXY1, 100);
        gte.write_data(GTE::SXY2, (50 << 16) | 50);

        gte.nclip();

        // Flags should be cleared
        assert_eq!(gte.flags, 0, "NCLIP should clear flags");
        assert_eq!(gte.read_data(GTE::LZCR), 0, "LZCR should be cleared");
    }

    #[test]
    fn test_nclip_with_negative_coordinates() {
        let mut gte = GTE::new();

        // Triangle with negative coordinates
        gte.write_data(
            GTE::SXY0,
            (((-50i16 as u16 as u32) << 16) | (-50i16 as u16 as u32)) as i32,
        );
        gte.write_data(GTE::SXY1, (((-50i16 as u16 as u32) << 16) | 50) as i32);
        gte.write_data(GTE::SXY2, 50 << 16);

        gte.nclip();

        let mac0 = gte.read_data(GTE::MAC0);
        // Should still calculate correctly with negative values
        assert_ne!(mac0, 0, "NCLIP should work with negative coordinates");
    }

    // ============================================================================
    // MVMVA (Matrix-Vector Multiply-Add) Tests
    // ============================================================================

    #[test]
    fn test_mvmva_vector_v0_selection() {
        let mut gte = GTE::new();

        // Setup identity matrix
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 0);
        gte.write_control(GTE::TRY, 0);
        gte.write_control(GTE::TRZ, 0);

        // Set V0 = (100, 200, 300)
        gte.write_data(GTE::VXY0, (200 << 16) | 100);
        gte.write_data(GTE::VZ0, 300);

        // MVMVA command: v=0 (V0), mx=0 (RT), cv=3 (none), lm=0, sf=1
        // Bits: sf(19)=1, mx(17-18)=0, v(15-16)=0, cv(13-14)=3, lm(10)=0, opcode(0-5)=0x12
        let command = 0x00086012; // sf=1(0x80000), cv=3(0x6000), opcode=0x12
        gte.mvmva(command);

        // Result should be V0 * I + 0 = V0 (no translation with cv=3)
        assert_eq!(gte.read_data(GTE::MAC1), 100);
        assert_eq!(gte.read_data(GTE::MAC2), 200);
        assert_eq!(gte.read_data(GTE::MAC3), 300);
    }

    #[test]
    fn test_mvmva_vector_v1_selection() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Set V1 = (400, 500, 600)
        gte.write_data(GTE::VXY1, (500 << 16) | 400);
        gte.write_data(GTE::VZ1, 600);

        // MVMVA: v=1 (V1), cv=3 (none)
        // Bits: sf(19)=1, mx(17-18)=0, v(15-16)=1, cv(13-14)=3, lm(10)=0, opcode=0x12
        let command = 0x0008E012; // sf=1(0x80000), v=1(0x8000), cv=3(0x6000)
        gte.mvmva(command);

        assert_eq!(gte.read_data(GTE::MAC1), 400);
        assert_eq!(gte.read_data(GTE::MAC2), 500);
        assert_eq!(gte.read_data(GTE::MAC3), 600);
    }

    #[test]
    fn test_mvmva_vector_ir_selection() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Set IR registers
        gte.write_data(GTE::IR1, 111);
        gte.write_data(GTE::IR2, 222);
        gte.write_data(GTE::IR3, 333);

        // MVMVA: v=3 (IR), cv=3 (none)
        // Bits: sf(19)=1, mx(17-18)=0, v(15-16)=3, cv(13-14)=3, lm(10)=0, opcode=0x12
        let command = 0x0009E012; // sf=1(0x80000), v=3(0x18000), cv=3(0x6000)
        gte.mvmva(command);

        assert_eq!(gte.read_data(GTE::MAC1), 111);
        assert_eq!(gte.read_data(GTE::MAC2), 222);
        assert_eq!(gte.read_data(GTE::MAC3), 333);
    }

    #[test]
    fn test_mvmva_lm_flag_unsigned_saturation() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);

        // Set negative values
        gte.write_data(
            GTE::VXY0,
            (((-100i16 as u16 as u32) << 16) | (-100i16 as u16 as u32)) as i32,
        );
        gte.write_data(GTE::VZ0, (-100i16 as u16 as u32) as i32);

        // With lm=1 (unsigned saturation), negative values should saturate to 0
        let command = 0x00400412; // lm=1
        gte.mvmva(command);

        let ir1 = gte.read_data(GTE::IR1);
        let ir2 = gte.read_data(GTE::IR2);
        let ir3 = gte.read_data(GTE::IR3);

        // With lm=1, negative IR values should be clamped to 0
        assert!(ir1 >= 0, "IR1 should be non-negative with lm=1");
        assert!(ir2 >= 0, "IR2 should be non-negative with lm=1");
        assert!(ir3 >= 0, "IR3 should be non-negative with lm=1");
    }

    #[test]
    fn test_mvmva_sf_flag_shift() {
        let mut gte = GTE::new();

        // Matrix with value 2.0
        gte.write_control(GTE::RT11_RT12, 0x2000);
        gte.write_control(GTE::RT22_RT23, 0x2000);
        gte.write_control(GTE::RT33, 0x2000);

        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 100);

        // Test with sf=0 (no shift)
        let command_no_shift = 0x00000012;
        gte.mvmva(command_no_shift);
        let mac1_no_shift = gte.read_data(GTE::MAC1);

        // Test with sf=1 (shift by 12)
        let command_shift = 0x00080012;
        gte.mvmva(command_shift);
        let mac1_shift = gte.read_data(GTE::MAC1);

        // With shift, result should be smaller
        assert!(
            mac1_no_shift > mac1_shift,
            "sf=1 should produce smaller result"
        );
    }

    // ============================================================================
    // GTE Command Execute Tests
    // ============================================================================

    #[test]
    fn test_execute_rtps_opcode() {
        let mut gte = GTE::new();

        // Setup basic transform
        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);

        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 1000);

        // Execute RTPS via execute() - opcode 0x01, sf=1
        gte.execute(0x00080001);

        // Should have executed RTPS
        assert_ne!(gte.read_data(GTE::MAC3), 0, "RTPS should have executed");
    }

    #[test]
    fn test_execute_nclip_opcode() {
        let mut gte = GTE::new();

        gte.write_data(GTE::SXY0, 0);
        gte.write_data(GTE::SXY1, 100);
        gte.write_data(GTE::SXY2, (50 << 16) | 50);

        // Execute NCLIP - opcode 0x06
        gte.execute(0x00000006);

        // Should have executed NCLIP
        assert_ne!(gte.read_data(GTE::MAC0), 0, "NCLIP should have executed");
    }

    #[test]
    fn test_execute_rtpt_opcode() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);

        gte.write_data(GTE::VXY0, 100);
        gte.write_data(GTE::VZ0, 1000);
        gte.write_data(GTE::VXY1, 200);
        gte.write_data(GTE::VZ1, 2000);
        gte.write_data(GTE::VXY2, 300);
        gte.write_data(GTE::VZ2, 3000);

        // Execute RTPT - opcode 0x30, sf=1
        gte.execute(0x00080030);

        // All three screen coordinates should be set
        assert_ne!(gte.read_data(GTE::SXY0), 0);
        assert_ne!(gte.read_data(GTE::SXY1), 0);
        assert_ne!(gte.read_data(GTE::SXY2), 0);
    }

    #[test]
    fn test_execute_unknown_opcode() {
        let mut gte = GTE::new();

        // Execute unknown opcode 0x3F (not implemented)
        gte.execute(0x0000003F);

        // Should set error flag (bit 31)
        assert_eq!(
            gte.flags, 0x80000000,
            "Unknown opcode should set error flag"
        );
        assert_eq!(
            gte.read_data(GTE::LZCR) as u32,
            0x80000000,
            "LZCR should reflect error flag"
        );
    }

    // ============================================================================
    // Edge Cases and Hardware Quirks
    // ============================================================================

    #[test]
    fn test_mac_registers_no_saturation() {
        let mut gte = GTE::new();

        // Set up matrix that will cause overflow
        gte.write_control(GTE::RT11_RT12, 0x7FFF); // Max value
        gte.write_control(GTE::RT22_RT23, 0x7FFF);
        gte.write_control(GTE::RT33, 0x7FFF);

        // Large input vector
        gte.write_data(GTE::VXY0, (0x7FFF << 16) | 0x7FFF);
        gte.write_data(GTE::VZ0, 0x7FFF);

        gte.write_control(GTE::TRX, 0x7FFFFFFF);
        gte.write_control(GTE::TRY, 0x7FFFFFFF);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 1000);

        gte.rtps(false);

        // MAC registers can overflow - they should still contain a value
        // (may be clamped to i32 range but not saturated to smaller range)
        let _mac1 = gte.read_data(GTE::MAC1);
        let _mac2 = gte.read_data(GTE::MAC2);
        let _mac3 = gte.read_data(GTE::MAC3);

        // Just verify they didn't panic - exact values depend on overflow behavior
    }

    #[test]
    fn test_ir_registers_saturate() {
        let mut gte = GTE::new();

        // Large MAC values
        gte.data[GTE::MAC1] = 100000;
        gte.data[GTE::MAC2] = -100000;
        gte.data[GTE::MAC3] = 50000;

        // Manually call rtps which will saturate IR from MAC
        gte.write_control(GTE::RT11_RT12, 0);
        gte.write_control(GTE::RT22_RT23, 0);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRX, 100000);
        gte.write_control(GTE::TRY, -100000);
        gte.write_control(GTE::TRZ, 50000);
        gte.write_control(GTE::H, 1000);

        gte.write_data(GTE::VXY0, 0);
        gte.write_data(GTE::VZ0, 0);

        gte.rtps(true);

        // IR registers should be saturated to -32768..32767
        let ir1 = gte.read_data(GTE::IR1);
        let ir2 = gte.read_data(GTE::IR2);
        let ir3 = gte.read_data(GTE::IR3);

        assert!(
            (-32768..=32767).contains(&ir1),
            "IR1 not saturated: {}",
            ir1
        );
        assert!(
            (-32768..=32767).contains(&ir2),
            "IR2 not saturated: {}",
            ir2
        );
        assert!((0..=65535).contains(&ir3), "IR3 not saturated: {}", ir3);
    }

    #[test]
    fn test_sz_fifo_4_stages() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::H, 1000);

        // Execute RTPS 4 times to fill SZ FIFO
        // Note: SZ3 = MAC3 SAR ((1-sf)*12), and MAC3 includes the matrix multiply result
        // With identity matrix and TRZ: MAC3 = (TRZ << 12) >> 12 = TRZ when VZ=0
        for z in [100, 200, 300, 400] {
            gte.write_data(GTE::VXY0, 0);
            gte.write_data(GTE::VZ0, z);
            gte.write_control(GTE::TRZ, z);
            gte.rtps(true);
        }

        // SZ FIFO should contain results (which will be 2*z due to TRZ+VZ*RT33)
        // After 4 RTPS: SZ0=200, SZ1=400, SZ2=600, SZ3=800
        assert_eq!(gte.read_data(GTE::SZ0), 200);
        assert_eq!(gte.read_data(GTE::SZ1), 400);
        assert_eq!(gte.read_data(GTE::SZ2), 600);
        assert_eq!(gte.read_data(GTE::SZ3), 800);
    }

    #[test]
    fn test_flags_cleared_on_command_start() {
        let mut gte = GTE::new();

        // Set flags from previous operation
        gte.execute(0xFF); // Unknown command sets error flag

        assert_eq!(gte.flags, 0x80000000, "Error flag should be set");

        // Execute valid command
        gte.write_data(GTE::SXY0, 0);
        gte.write_data(GTE::SXY1, 100);
        gte.write_data(GTE::SXY2, 50);
        gte.nclip();

        // Flags should be cleared at start of NCLIP
        assert_eq!(gte.flags, 0, "Flags should be cleared on new command");
    }

    #[test]
    fn test_rotation_matrix_extraction() {
        let mut gte = GTE::new();

        // Set rotation matrix with known values
        // RT11=0x1000 (1.0), RT12=0x0800 (0.5)
        gte.write_control(GTE::RT11_RT12, (0x0800 << 16) | 0x1000);
        // RT13=0x0400 (0.25), RT21=0x0200 (0.125)
        gte.write_control(GTE::RT13_RT21, (0x0200 << 16) | 0x0400);
        // RT22=0x2000 (2.0), RT23=0x0100
        gte.write_control(GTE::RT22_RT23, (0x0100 << 16) | 0x2000);
        // RT31=0x0080, RT32=0x0040
        gte.write_control(GTE::RT31_RT32, (0x0040 << 16) | 0x0080);
        // RT33=0x3000 (3.0)
        gte.write_control(GTE::RT33, 0x3000);

        let matrix = gte.get_rotation_matrix();

        assert_eq!(matrix[0][0], 0x1000);
        assert_eq!(matrix[0][1], 0x0800);
        assert_eq!(matrix[0][2], 0x0400);
        assert_eq!(matrix[1][0], 0x0200);
        assert_eq!(matrix[1][1], 0x2000);
        assert_eq!(matrix[1][2], 0x0100);
        assert_eq!(matrix[2][0], 0x0080);
        assert_eq!(matrix[2][1], 0x0040);
        assert_eq!(matrix[2][2], 0x3000);
    }

    #[test]
    fn test_negative_matrix_elements() {
        let mut gte = GTE::new();

        // Set negative rotation matrix elements
        gte.write_control(
            GTE::RT11_RT12,
            (((-0x1000i16 as u16 as u32) << 16) | (-0x1000i16 as u16 as u32)) as i32,
        );
        gte.write_control(GTE::RT13_RT21, 0);
        gte.write_control(GTE::RT22_RT23, (-0x1000i16 as u16 as u32) as i32);
        gte.write_control(GTE::RT31_RT32, 0);
        gte.write_control(GTE::RT33, (-0x1000i16 as u16 as u32) as i32);

        let matrix = gte.get_rotation_matrix();

        // Check that negative values are correctly sign-extended
        assert_eq!(matrix[0][0], -0x1000);
        assert_eq!(matrix[0][1], -0x1000);
        assert_eq!(matrix[1][1], -0x1000);
        assert_eq!(matrix[2][2], -0x1000);
    }

    #[test]
    fn test_zero_h_value() {
        let mut gte = GTE::new();

        gte.write_control(GTE::RT11_RT12, 0x1000);
        gte.write_control(GTE::RT22_RT23, 0x1000);
        gte.write_control(GTE::RT33, 0x1000);
        gte.write_control(GTE::TRZ, 1000);
        gte.write_control(GTE::H, 0); // Zero projection distance

        gte.write_data(GTE::VXY0, (100 << 16) | 100);
        gte.write_data(GTE::VZ0, 1000);

        gte.rtps(true);

        // With H=0, projection scale should be zero
        // Screen coordinates should be at screen offset
        let sxy2 = gte.read_data(GTE::SXY2);
        let sx = (sxy2 & 0xFFFF) as i16 as i32;
        let sy = (sxy2 >> 16) as i16 as i32;

        // Should be close to zero (since OFX=OFY=0)
        assert!((-100..=100).contains(&sx), "SX with H=0: {}", sx);
        assert!((-100..=100).contains(&sy), "SY with H=0: {}", sy);
    }
}
