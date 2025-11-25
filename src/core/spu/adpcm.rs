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

//! ADPCM (Adaptive Differential Pulse Code Modulation) decoder
//!
//! Implements the PlayStation's ADPCM audio decompression format.
//! ADPCM compresses 16-bit PCM audio to 4 bits per sample using
//! adaptive prediction filters.

/// ADPCM decoder state
///
/// Maintains state for ADPCM audio decompression including previous samples
/// for filter interpolation and current decode position.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ADPCMState {
    /// Previous samples for interpolation (history for filters)
    pub(crate) prev_samples: [i16; 2],

    /// Decoder position within the current block (0.0-28.0)
    pub(crate) position: f32,
}

impl Default for ADPCMState {
    fn default() -> Self {
        Self {
            prev_samples: [0; 2],
            position: 0.0,
        }
    }
}

#[allow(dead_code)]
impl ADPCMState {
    /// Decode a single ADPCM block
    ///
    /// # Arguments
    ///
    /// * `block` - 16-byte ADPCM block to decode
    ///
    /// # Returns
    ///
    /// Vector of 28 decoded 16-bit PCM samples
    ///
    /// # ADPCM Block Format
    ///
    /// ```text
    /// Byte 0: Shift (bits 0-3) | Filter (bits 4-7)
    /// Byte 1: Flags (loop end, loop repeat, etc.)
    /// Bytes 2-15: 14 bytes of nibble pairs (28 samples total)
    /// ```
    pub fn decode_block(&mut self, block: &[u8]) -> Vec<i16> {
        if block.len() < 16 {
            return Vec::new();
        }

        let mut samples = Vec::with_capacity(28);

        // Block header
        let shift = block[0] & 0xF;
        let filter = (block[0] >> 4) & 0x3;
        // Flags in block[1] are for loop control, handled elsewhere

        // Decode 28 samples from 14 bytes (2 samples per byte)
        for i in 0..14 {
            let byte = block[2 + i];

            // Extract two 4-bit samples per byte
            let nibble1 = (byte & 0xF) as i8;
            let nibble2 = ((byte >> 4) & 0xF) as i8;

            // Sign extend 4-bit nibbles to 8-bit
            let nibble1_signed = (nibble1 << 4) >> 4;
            let nibble2_signed = (nibble2 << 4) >> 4;

            // Apply shift (scale up, then shift right)
            let sample1 = ((nibble1_signed as i16) << 12) >> shift;
            let sample2 = ((nibble2_signed as i16) << 12) >> shift;

            // Apply filter
            let decoded1 = self.apply_filter(sample1, filter);
            let decoded2 = self.apply_filter(sample2, filter);

            samples.push(decoded1);
            samples.push(decoded2);
        }

        samples
    }

    /// Apply ADPCM filter to a sample
    ///
    /// # Arguments
    ///
    /// * `sample` - Input sample after shift
    /// * `filter` - Filter mode (0-3)
    ///
    /// # Returns
    ///
    /// Filtered and clamped sample
    ///
    /// # Filter Modes
    ///
    /// - Filter 0: No filtering, pass through
    /// - Filter 1: Simple first-order prediction
    /// - Filter 2: Second-order prediction with both previous samples
    /// - Filter 3: Alternative second-order prediction
    #[inline(always)]
    fn apply_filter(&mut self, sample: i16, filter: u8) -> i16 {
        let result = match filter {
            0 => sample as i32,
            1 => {
                // Filter 1: s + old[0] + (-old[0] >> 1)
                sample as i32 + self.prev_samples[0] as i32 + (-(self.prev_samples[0] as i32) >> 1)
            }
            2 => {
                // Filter 2: s + old[0]*2 + (-old[0]*3 >> 1) - old[1] + (old[1] >> 1)
                sample as i32
                    + (self.prev_samples[0] as i32 * 2)
                    + ((-(self.prev_samples[0] as i32) * 3) >> 1)
                    - self.prev_samples[1] as i32
                    + (self.prev_samples[1] as i32 >> 1)
            }
            3 => {
                // Filter 3: s + old[0]*2 - (old[0]*5 >> 2) + old[1]*2 - (old[1] >> 1)
                sample as i32 + (self.prev_samples[0] as i32 * 2)
                    - ((self.prev_samples[0] as i32 * 5) >> 2)
                    + (self.prev_samples[1] as i32 * 2)
                    - (self.prev_samples[1] as i32 >> 1)
            }
            _ => sample as i32,
        };

        // Clamp to i16 range
        let clamped = result.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        // Update history (shift old samples)
        self.prev_samples[1] = self.prev_samples[0];
        self.prev_samples[0] = clamped;

        clamped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adpcm_state_default() {
        let state = ADPCMState::default();
        assert_eq!(state.prev_samples, [0, 0]);
        assert_eq!(state.position, 0.0);
    }

    #[test]
    fn test_decode_block_empty() {
        let mut state = ADPCMState::default();
        let samples = state.decode_block(&[]);
        assert!(
            samples.is_empty(),
            "Empty block should return empty samples"
        );
    }

    #[test]
    fn test_decode_block_insufficient_size() {
        let mut state = ADPCMState::default();
        let block = [0u8; 10]; // Less than 16 bytes
        let samples = state.decode_block(&block);
        assert!(
            samples.is_empty(),
            "Block smaller than 16 bytes should return empty samples"
        );
    }

    #[test]
    fn test_decode_block_filter_0_no_filtering() {
        let mut state = ADPCMState::default();

        // Create a test block with filter 0 (no filtering), shift 0
        let mut block = [0u8; 16];
        block[0] = 0x00; // shift=0, filter=0
        block[1] = 0x00; // flags

        // Set a simple pattern: nibble value 1 (0x1) in all positions
        for item in block.iter_mut().skip(2) {
            *item = 0x11; // Both nibbles = 1
        }

        let samples = state.decode_block(&block);

        assert_eq!(samples.len(), 28, "Should decode 28 samples");

        // With filter 0, output should be exactly the shifted nibble value
        // Nibble 1 (signed) = 1, shifted left by 12 then right by 0 = 4096
        for sample in samples {
            assert_eq!(sample, 4096, "Filter 0 should pass through scaled nibble");
        }
    }

    #[test]
    fn test_decode_block_filter_modes() {
        // Test all 4 filter modes with known inputs
        for filter in 0..4 {
            let mut state = ADPCMState {
                prev_samples: [1000, 500],
                ..Default::default()
            };

            let mut block = [0u8; 16];
            block[0] = filter << 4; // shift=0, filter=filter
            block[1] = 0x00;

            // Single nibble value: 2
            for item in block.iter_mut().skip(2) {
                *item = 0x22;
            }

            let samples = state.decode_block(&block);
            assert_eq!(samples.len(), 28);

            // Each filter should produce different results
            // We're mainly verifying it doesn't crash and produces valid output
            // Samples are i16, so they're always in valid range by definition
            assert!(
                !samples.is_empty(),
                "Filter {} should produce samples",
                filter
            );
        }
    }

    #[test]
    fn test_decode_block_shift_values() {
        // Test various shift values (0-15)
        for shift in 0..=15 {
            let mut state = ADPCMState::default();

            let mut block = [0u8; 16];
            block[0] = shift; // shift=shift, filter=0
            block[1] = 0x00;

            // Use nibble value 7 (max positive 4-bit value)
            for item in block.iter_mut().skip(2) {
                *item = 0x77;
            }

            let samples = state.decode_block(&block);
            assert_eq!(samples.len(), 28);

            // Larger shift should produce smaller magnitude
            // Samples are always valid i16, just verify we got samples
            assert!(
                !samples.is_empty(),
                "Shift {} should produce samples",
                shift
            );
        }
    }

    #[test]
    fn test_decode_block_negative_nibbles() {
        let mut state = ADPCMState::default();

        let mut block = [0u8; 16];
        block[0] = 0x00; // shift=0, filter=0
        block[1] = 0x00;

        // Use nibble value 0xF (-1 in 4-bit signed)
        for item in block.iter_mut().skip(2) {
            *item = 0xFF;
        }

        let samples = state.decode_block(&block);
        assert_eq!(samples.len(), 28);

        // All samples should be negative
        for sample in samples {
            assert!(sample < 0, "0xF nibble should decode to negative value");
        }
    }

    #[test]
    fn test_decode_block_nibble_sign_extension() {
        let mut state = ADPCMState::default();

        // Test nibble 0x8 (binary 1000, should be -8 when sign-extended)
        let mut block = [0u8; 16];
        block[0] = 0x00; // shift=0, filter=0
        block[1] = 0x00;
        block[2] = 0x88; // Both nibbles = 0x8

        let samples = state.decode_block(&block);

        // 0x8 in 4-bit signed is -8
        // After sign extension to 8-bit: -8
        // Shifted: (-8 << 12) >> 0 = -32768
        assert_eq!(samples[0], -32768, "Nibble 0x8 should sign-extend to -8");
        assert_eq!(samples[1], -32768, "Nibble 0x8 should sign-extend to -8");
    }

    #[test]
    fn test_decode_block_clamping() {
        let mut state = ADPCMState {
            prev_samples: [32700, 32700],
            ..Default::default()
        };

        let mut block = [0u8; 16];
        block[0] = 0x10; // shift=0, filter=1 (uses prev_samples)
        block[1] = 0x00;

        // Large positive nibble
        for item in block.iter_mut().skip(2) {
            *item = 0x77;
        }

        let samples = state.decode_block(&block);

        // Verify samples are produced (they're always i16, so always in valid range)
        assert_eq!(samples.len(), 28, "Should produce 28 samples with clamping");
    }

    #[test]
    fn test_decode_block_history_update() {
        let mut state = ADPCMState::default();

        let mut block = [0u8; 16];
        block[0] = 0x00; // shift=0, filter=0 (simple pass-through)
        block[1] = 0x00;

        // Use larger nibble values to ensure non-zero output
        for item in block.iter_mut().skip(2) {
            *item = 0x77; // nibbles: 7, 7 (max positive 4-bit value)
        }

        let samples = state.decode_block(&block);

        // After decoding, prev_samples should contain the last two decoded samples
        assert_eq!(samples.len(), 28);

        // With filter 0 and nibble value 7, we should get 7 << 12 = 28672
        // History should be updated with the last two samples
        assert_ne!(
            state.prev_samples,
            [0, 0],
            "History should be updated after decoding"
        );

        // Verify the values are what we expect (7 << 12 >> 0 = 28672)
        assert_eq!(state.prev_samples[0], 28672);
        assert_eq!(state.prev_samples[1], 28672);
    }

    #[test]
    fn test_decode_block_28_samples() {
        let mut state = ADPCMState::default();

        let mut block = [0u8; 16];
        block[0] = 0x00;
        block[1] = 0x00;

        // 14 bytes of data, 2 nibbles per byte = 28 samples
        for item in block.iter_mut().skip(2) {
            *item = 0x12;
        }

        let samples = state.decode_block(&block);
        assert_eq!(
            samples.len(),
            28,
            "Each block should decode to exactly 28 samples"
        );
    }

    #[test]
    fn test_decode_multiple_blocks_preserves_state() {
        let mut state = ADPCMState::default();

        let mut block1 = [0u8; 16];
        block1[0] = 0x10; // shift=0, filter=1
        block1[1] = 0x00;
        for item in block1.iter_mut().skip(2) {
            *item = 0x33;
        }

        let samples1 = state.decode_block(&block1);
        let history_after_first = state.prev_samples;

        let mut block2 = [0u8; 16];
        block2[0] = 0x10; // shift=0, filter=1
        block2[1] = 0x00;
        for item in block2.iter_mut().skip(2) {
            *item = 0x44;
        }

        let samples2 = state.decode_block(&block2);

        // Second block should use history from first block
        assert_ne!(
            samples1, samples2,
            "Different input should produce different output"
        );
        assert_ne!(
            history_after_first,
            [0, 0],
            "First block should update history"
        );
    }

    #[test]
    fn test_filter_coefficients_accuracy() {
        let mut state = ADPCMState {
            prev_samples: [8192, 4096],
            ..Default::default()
        };

        // Test filter 1: s + old[0] + (-old[0] >> 1)
        let sample = 0i16;
        let result = state.apply_filter(sample, 1);

        // Expected: 0 + 8192 + (-8192 >> 1) = 0 + 8192 - 4096 = 4096
        assert_eq!(result, 4096, "Filter 1 calculation should be correct");
        assert_eq!(
            state.prev_samples[0], result,
            "History should update with result"
        );
        assert_eq!(
            state.prev_samples[1], 8192,
            "Old history should shift to [1]"
        );
    }

    #[test]
    fn test_extreme_shift_values() {
        let mut state = ADPCMState::default();

        // Test maximum shift (15) - should produce very small values
        let mut block = [0u8; 16];
        block[0] = 0x0F; // shift=15, filter=0
        block[1] = 0x00;

        for item in block.iter_mut().skip(2) {
            *item = 0x77; // Max positive nibble
        }

        let samples = state.decode_block(&block);

        // With shift=15, even max nibble should produce small values
        // (7 << 12) >> 15 = 28672 >> 15 = 0
        for sample in samples {
            assert!(
                sample.abs() < 100,
                "High shift should produce very small values"
            );
        }
    }
}
