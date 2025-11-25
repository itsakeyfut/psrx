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

//! SPU noise generator
//!
//! The PlayStation SPU includes a noise generator for creating sound effects
//! like explosions, wind, or other non-tonal sounds. It uses a Linear Feedback
//! Shift Register (LFSR) to generate pseudo-random noise.

/// Noise generator using LFSR
///
/// Generates pseudo-random noise samples using a Galois LFSR
/// with configurable frequency.
pub struct NoiseGenerator {
    /// LFSR (Linear Feedback Shift Register) state
    lfsr: u32,

    /// Noise clock frequency shift (0-15)
    clock_shift: u8,

    /// Noise clock step (0-3)
    clock_step: u8,

    /// Current counter for frequency divider
    counter: u32,
}

impl NoiseGenerator {
    /// Create a new noise generator
    ///
    /// # Returns
    ///
    /// Initialized noise generator with default state
    pub fn new() -> Self {
        Self {
            lfsr: 0x0001,
            clock_shift: 0,
            clock_step: 0,
            counter: 0,
        }
    }

    /// Set noise frequency parameters
    ///
    /// The frequency is determined by: freq = step_value >> shift
    /// where step_value depends on clock_step:
    /// - 0: disabled (outputs silence, no contribution to audio output)
    /// - 1: 0x8000
    /// - 2: 0x10000
    /// - 3: 0x20000
    ///
    /// # Arguments
    ///
    /// * `shift` - Frequency shift value (0-15)
    /// * `step` - Frequency step selector (0-3)
    pub fn set_frequency(&mut self, shift: u8, step: u8) {
        self.clock_shift = shift & 0xF;
        self.clock_step = step & 0x3;
    }

    /// Generate next noise sample
    ///
    /// # Returns
    ///
    /// 16-bit noise sample (either 0x7FFF or -0x8000), or 0 when disabled
    #[inline(always)]
    pub fn generate(&mut self) -> i16 {
        // When clock_step is 0, noise is disabled and outputs silence
        if self.clock_step == 0 {
            return 0;
        }

        // Calculate clock divider
        let freq = match self.clock_step {
            1 => 0x8000 >> self.clock_shift,
            2 => 0x10000 >> self.clock_shift,
            3 => 0x20000 >> self.clock_shift,
            _ => 0,
        };

        self.counter += 1;

        if freq > 0 && self.counter >= freq {
            self.counter = 0;
            self.step_lfsr();
        }

        // Output is based on the bottom bit (where feedback is inserted)
        if (self.lfsr & 0x0001) != 0 {
            0x7FFF
        } else {
            -0x8000
        }
    }

    /// Step the LFSR by one tick
    ///
    /// Uses a Galois LFSR with taps at bits 15, 12, 11, and 10
    /// to generate pseudo-random sequences.
    pub(crate) fn step_lfsr(&mut self) {
        // Galois LFSR with taps at bits 15, 12, 11, and 10
        let feedback =
            ((self.lfsr >> 15) ^ (self.lfsr >> 12) ^ (self.lfsr >> 11) ^ (self.lfsr >> 10)) & 1;
        self.lfsr = (self.lfsr << 1) | feedback;
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_generator_creation() {
        let noise = NoiseGenerator::new();
        assert_eq!(noise.lfsr, 0x0001);
        assert_eq!(noise.clock_shift, 0);
        assert_eq!(noise.clock_step, 0);
    }

    #[test]
    fn test_noise_generator_frequency() {
        let mut noise = NoiseGenerator::new();
        noise.set_frequency(0, 1); // shift=0, step=1, freq=0x8000

        // Generate enough samples to trigger at least one LFSR step
        // With shift=0 and step=1, we need 0x8000 (32768) calls
        let samples: Vec<i16> = (0..40000).map(|_| noise.generate()).collect();

        // Verify noise is not constant
        let all_same = samples.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "Noise should not be constant");
    }

    #[test]
    fn test_lfsr_sequence() {
        let mut noise = NoiseGenerator::new();

        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            noise.step_lfsr();
            seen.insert(noise.lfsr);
        }

        // LFSR should produce varied values
        assert!(
            seen.len() > 100,
            "LFSR should produce many different values"
        );
    }

    #[test]
    fn test_noise_output_values() {
        let mut noise = NoiseGenerator::new();
        noise.set_frequency(0, 1);

        for _ in 0..100 {
            let sample = noise.generate();
            // Output should only be 0x7FFF or -0x8000
            assert!(
                sample == 0x7FFF || sample == -0x8000,
                "Noise output should be max or min"
            );
        }
    }

    #[test]
    fn test_noise_disabled_outputs_silence() {
        let mut noise = NoiseGenerator::new();
        noise.set_frequency(0, 0); // clock_step = 0 disables noise

        for _ in 0..1000 {
            let sample = noise.generate();
            assert_eq!(sample, 0, "Disabled noise should output silence");
        }
    }

    #[test]
    fn test_noise_clock_step_values() {
        // Test all clock_step values (0-3)
        for step in 0..=3 {
            let mut noise = NoiseGenerator::new();
            noise.set_frequency(0, step);

            if step == 0 {
                // Step 0 should output silence
                assert_eq!(noise.generate(), 0);
            } else {
                // Other steps should output noise
                let sample = noise.generate();
                assert!(
                    sample == 0x7FFF || sample == -0x8000,
                    "Step {} should produce noise",
                    step
                );
            }
        }
    }

    #[test]
    fn test_noise_clock_shift_values() {
        // Test various shift values (0-15)
        for shift in 0..=15 {
            let mut noise = NoiseGenerator::new();
            noise.set_frequency(shift, 1);

            // Should generate noise without crashing
            for _ in 0..100 {
                let sample = noise.generate();
                assert!(
                    sample == 0x7FFF || sample == -0x8000 || sample == 0,
                    "Shift {} should produce valid output",
                    shift
                );
            }
        }
    }

    #[test]
    fn test_noise_frequency_calculation() {
        let mut noise = NoiseGenerator::new();

        // Test step 1: freq = 0x8000 >> shift
        noise.set_frequency(0, 1);
        assert_eq!(noise.clock_step, 1);
        assert_eq!(noise.clock_shift, 0);

        // Test step 2: freq = 0x10000 >> shift
        noise.set_frequency(1, 2);
        assert_eq!(noise.clock_step, 2);
        assert_eq!(noise.clock_shift, 1);

        // Test step 3: freq = 0x20000 >> shift
        noise.set_frequency(2, 3);
        assert_eq!(noise.clock_step, 3);
        assert_eq!(noise.clock_shift, 2);
    }

    #[test]
    fn test_noise_counter_behavior() {
        let mut noise = NoiseGenerator::new();
        noise.set_frequency(10, 1); // High shift = lower frequency

        let initial_lfsr = noise.lfsr;

        // Generate samples until LFSR changes
        for _ in 0..100000 {
            noise.generate();
            if noise.lfsr != initial_lfsr {
                break;
            }
        }

        // Should eventually change (counter should reach threshold)
        assert_ne!(
            noise.lfsr, initial_lfsr,
            "LFSR should change after sufficient samples"
        );
    }

    #[test]
    fn test_lfsr_non_zero() {
        let mut noise = NoiseGenerator::new();

        // LFSR should never become zero (would lock up)
        for _ in 0..10000 {
            noise.step_lfsr();
            assert_ne!(noise.lfsr, 0, "LFSR should never be zero");
        }
    }

    #[test]
    fn test_lfsr_periodicity() {
        let mut noise = NoiseGenerator::new();
        let initial_lfsr = noise.lfsr;

        let mut seen = std::collections::HashSet::new();
        seen.insert(initial_lfsr);

        // Step LFSR and check for reasonable period
        for i in 1..=10000 {
            noise.step_lfsr();

            if noise.lfsr == initial_lfsr {
                // Found the period
                assert!(
                    i > 100,
                    "LFSR period should be reasonably long, found {}",
                    i
                );
                return;
            }

            seen.insert(noise.lfsr);
        }

        // If we didn't find period in 10000 steps, that's also acceptable
        // (period might be longer than 10000)
        assert!(seen.len() > 1000, "LFSR should produce many unique values");
    }

    #[test]
    fn test_noise_masking_behavior() {
        let mut noise = NoiseGenerator::new();

        // Test that shift and step are masked correctly
        noise.set_frequency(0xFF, 0xFF); // Values that exceed valid range
        assert_eq!(noise.clock_shift, 0xF, "Shift should be masked to 4 bits");
        assert_eq!(noise.clock_step, 0x3, "Step should be masked to 2 bits");
    }

    #[test]
    fn test_noise_generator_default() {
        let noise = NoiseGenerator::default();
        assert_eq!(noise.lfsr, 0x0001, "Default LFSR should be 1");
        assert_eq!(noise.clock_shift, 0);
        assert_eq!(noise.clock_step, 0);
        assert_eq!(noise.counter, 0);
    }

    #[test]
    fn test_lfsr_tap_bits() {
        let mut noise = NoiseGenerator::new();
        noise.lfsr = 0b1100110000000000; // Known pattern with bits at tap positions

        let initial = noise.lfsr;
        noise.step_lfsr();

        // LFSR should have shifted and XOR'd feedback bit
        assert_ne!(noise.lfsr, initial, "LFSR should change");
        // LFSR is u32 internally, so it can go beyond 16 bits after shift
        // The implementation uses all 32 bits for the LFSR state
    }

    #[test]
    fn test_noise_deterministic() {
        let mut noise1 = NoiseGenerator::new();
        let mut noise2 = NoiseGenerator::new();

        noise1.set_frequency(5, 2);
        noise2.set_frequency(5, 2);

        // Both generators with same state should produce same sequence
        for _ in 0..100 {
            let sample1 = noise1.generate();
            let sample2 = noise2.generate();
            assert_eq!(
                sample1, sample2,
                "Identical noise generators should produce identical output"
            );
        }
    }

    #[test]
    fn test_noise_step_independence() {
        // Test that different step values produce different frequencies
        // Higher step = higher base frequency value
        let mut noise_step1 = NoiseGenerator::new();
        let mut noise_step2 = NoiseGenerator::new();
        let mut noise_step3 = NoiseGenerator::new();

        noise_step1.set_frequency(0, 1); // freq = 0x8000
        noise_step2.set_frequency(0, 2); // freq = 0x10000
        noise_step3.set_frequency(0, 3); // freq = 0x20000

        // Verify that the frequency divisors are different
        // Step 1: 0x8000, Step 2: 0x10000, Step 3: 0x20000
        // Lower divisor = higher update frequency (counter reaches threshold faster)
        // So step 1 should update most frequently, then 2, then 3

        // Generate enough samples to see differences
        let mut updates1 = 0;
        let mut updates2 = 0;
        let mut updates3 = 0;

        let initial_lfsr1 = noise_step1.lfsr;
        let initial_lfsr2 = noise_step2.lfsr;
        let initial_lfsr3 = noise_step3.lfsr;

        for _ in 0..300000 {
            noise_step1.generate();
            noise_step2.generate();
            noise_step3.generate();

            if noise_step1.lfsr != initial_lfsr1 {
                updates1 += 1;
            }
            if noise_step2.lfsr != initial_lfsr2 {
                updates2 += 1;
            }
            if noise_step3.lfsr != initial_lfsr3 {
                updates3 += 1;
            }
        }

        // Step 1 has smallest divisor, should update most frequently
        // Step 3 has largest divisor, should update least frequently
        assert!(
            updates1 > updates2,
            "Step 1 (0x8000) should update more than step 2 (0x10000): {} vs {}",
            updates1,
            updates2
        );
        assert!(
            updates2 > updates3,
            "Step 2 (0x10000) should update more than step 3 (0x20000): {} vs {}",
            updates2,
            updates3
        );
    }
}
