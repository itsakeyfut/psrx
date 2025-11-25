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

//! SPU voice (audio channel) implementation
//!
//! Each voice can play back ADPCM-compressed audio samples with
//! independent volume, pitch, and ADSR envelope control.

use super::adpcm::ADPCMState;
use super::adsr::{ADSREnvelope, ADSRPhase};

/// Individual voice channel
///
/// Each voice can play back ADPCM-compressed audio samples with
/// independent volume, pitch, and ADSR envelope control.
#[allow(dead_code)]
pub struct Voice {
    /// Voice number (0-23)
    id: u8,

    /// Current volume (left/right)
    pub(crate) volume_left: i16,
    pub(crate) volume_right: i16,

    /// ADSR state
    pub(crate) adsr: ADSREnvelope,

    /// Current sample rate (pitch)
    pub(crate) sample_rate: u16,

    /// Start address in SPU RAM (multiply by 8 for byte address)
    pub(crate) start_address: u16,

    /// Repeat address (loop point, multiply by 8 for byte address)
    pub(crate) repeat_address: u16,

    /// Current address
    pub(crate) current_address: u32,

    /// ADPCM decoder state
    pub(crate) adpcm_state: ADPCMState,

    /// Decoded samples buffer (28 samples per ADPCM block)
    pub(crate) decoded_samples: Vec<i16>,

    /// Voice enabled
    pub(crate) enabled: bool,

    /// Key on flag
    key_on: bool,

    /// Key off flag
    key_off: bool,

    /// Loop flag (set when loop end flag is encountered)
    pub(crate) loop_flag: bool,

    /// Final block flag (set when a non-repeating end block is encountered)
    /// Playback will stop after this block is fully consumed
    pub(crate) final_block: bool,

    /// Noise mode enabled
    pub(crate) noise_enabled: bool,
}

#[allow(dead_code)]
impl Voice {
    /// Create a new voice instance
    ///
    /// # Arguments
    ///
    /// * `id` - Voice number (0-23)
    ///
    /// # Returns
    ///
    /// Initialized voice
    pub fn new(id: u8) -> Self {
        Self {
            id,
            volume_left: 0,
            volume_right: 0,
            adsr: ADSREnvelope::default(),
            sample_rate: 0,
            start_address: 0,
            repeat_address: 0,
            current_address: 0,
            adpcm_state: ADPCMState::default(),
            decoded_samples: Vec::new(),
            enabled: false,
            key_on: false,
            key_off: false,
            loop_flag: false,
            final_block: false,
            noise_enabled: false,
        }
    }

    /// Trigger key-on for this voice
    ///
    /// Starts playback from the start address and begins the attack phase
    /// of the ADSR envelope.
    pub fn key_on(&mut self) {
        self.enabled = true;
        self.key_on = true;
        self.current_address = (self.start_address as u32) * 8;
        self.adpcm_state = ADPCMState::default();
        self.decoded_samples.clear();
        self.loop_flag = false;
        self.final_block = false;
        self.key_off = false;
        self.adsr.phase = ADSRPhase::Attack;
        self.adsr.level = 0;

        log::trace!("Voice {} key on", self.id);
    }

    /// Trigger key-off for this voice
    ///
    /// Begins the release phase of the ADSR envelope.
    pub fn key_off(&mut self) {
        self.key_off = true;
        self.adsr.phase = ADSRPhase::Release;

        log::trace!("Voice {} key off", self.id);
    }

    /// Render a single stereo sample from this voice
    ///
    /// # Arguments
    ///
    /// * `spu_ram` - Reference to SPU RAM for ADPCM data access
    /// * `noise` - Mutable reference to noise generator
    ///
    /// # Returns
    ///
    /// Tuple of (left, right) 16-bit audio samples
    #[inline(always)]
    pub fn render_sample(
        &mut self,
        spu_ram: &[u8],
        noise: &mut super::noise::NoiseGenerator,
    ) -> (i16, i16) {
        if !self.enabled || self.adsr.phase == ADSRPhase::Off {
            return (0, 0);
        }

        // Get sample (ADPCM or noise)
        let sample = if self.noise_enabled {
            noise.generate()
        } else {
            // Check if we need to decode a new ADPCM block
            if self.needs_decode() {
                self.decode_block(spu_ram);
            }

            // Get interpolated sample at current position
            self.interpolate_sample()
        };

        // Apply ADSR envelope
        let enveloped = self.apply_envelope(sample);

        // Apply volume (fixed-point multiply with 15-bit fraction)
        let left = ((enveloped as i32 * self.volume_left as i32) >> 15) as i16;
        let right = ((enveloped as i32 * self.volume_right as i32) >> 15) as i16;

        // Advance playback position (only for non-noise samples)
        if !self.noise_enabled {
            self.advance_position();
        }

        (left, right)
    }

    /// Check if a new ADPCM block needs to be decoded
    ///
    /// # Returns
    ///
    /// True if the decoded samples buffer is empty or the read position has
    /// advanced past the current 28-sample ADPCM block
    fn needs_decode(&self) -> bool {
        self.decoded_samples.is_empty() || self.adpcm_state.position >= 28.0
    }

    /// Decode the current ADPCM block from SPU RAM
    ///
    /// # Arguments
    ///
    /// * `spu_ram` - Reference to SPU RAM for ADPCM data access
    pub(crate) fn decode_block(&mut self, spu_ram: &[u8]) {
        // Ensure SPU RAM size is power of 2 for bitwise masking
        debug_assert!(
            spu_ram.len().is_power_of_two(),
            "SPU RAM size must be power of 2 for efficient address masking"
        );

        // Calculate block address (each block is 16 bytes)
        // Using bitwise AND for performance (valid when len is power of 2)
        let block_addr = (self.current_address as usize) & (spu_ram.len() - 1);

        // Ensure we have a full block available
        if block_addr + 16 > spu_ram.len() {
            // Treat out-of-bounds access as terminal to avoid spinning on bad block
            self.decoded_samples.clear();
            self.enabled = false;
            self.adsr.phase = ADSRPhase::Off;
            self.final_block = true;
            self.adpcm_state.position = 28.0;
            return;
        }

        let block = &spu_ram[block_addr..block_addr + 16];

        // Check loop flags in block header
        let flags = block[1];
        let loop_end = (flags & 0x01) != 0;
        let loop_repeat = (flags & 0x02) != 0;

        // Decode the block
        self.decoded_samples = self.adpcm_state.decode_block(block);

        // Handle loop flags
        if loop_end {
            // Remember that this block had a loop-end flag
            self.loop_flag = true;
            if loop_repeat {
                // Next block will start from repeat address; we must not
                // auto-increment current_address again when we finish this
                // block, or we'd skip the first loop block
                self.current_address = (self.repeat_address as u32) * 8;
            } else {
                // Mark that this is the final block; playback will stop after
                // this block is fully consumed.
                // (Actual disabling is deferred until advance_position detects
                // position >= 28.0 on a final_block)
                self.final_block = true;
            }
        } else {
            self.loop_flag = false;
        }

        // Reset position to start of new block
        self.adpcm_state.position = 0.0;
    }

    /// Get interpolated sample at current position
    ///
    /// Uses simple linear interpolation for smooth pitch shifting.
    ///
    /// # Returns
    ///
    /// Interpolated 16-bit sample
    #[inline(always)]
    pub(crate) fn interpolate_sample(&self) -> i16 {
        if self.decoded_samples.is_empty() {
            return 0;
        }

        let pos = self.adpcm_state.position;
        let index = pos as usize;

        // Simple linear interpolation (Gaussian would be more accurate but slower)
        if index + 1 < self.decoded_samples.len() {
            let s0 = self.decoded_samples[index] as f32;
            let s1 = self.decoded_samples[index + 1] as f32;
            let frac = pos - index as f32;
            (s0 + (s1 - s0) * frac) as i16
        } else if index < self.decoded_samples.len() {
            self.decoded_samples[index]
        } else {
            0
        }
    }

    /// Apply ADSR envelope to a sample
    ///
    /// # Arguments
    ///
    /// * `sample` - Input sample
    ///
    /// # Returns
    ///
    /// Sample with envelope applied
    #[inline(always)]
    fn apply_envelope(&mut self, sample: i16) -> i16 {
        // Update ADSR envelope
        self.adsr.tick();

        // Apply envelope level (fixed-point multiply)
        ((sample as i32 * self.adsr.level as i32) >> 15) as i16
    }

    /// Advance the playback position
    ///
    /// Updates position based on sample rate and handles block transitions.
    pub(crate) fn advance_position(&mut self) {
        // Calculate step based on sample rate
        // Sample rate is in 4.12 fixed point format
        // Base sample rate is 44100 Hz
        let step = (self.sample_rate as f32) / 4096.0;

        self.adpcm_state.position += step;

        // Check if we've advanced past the current block
        if self.adpcm_state.position >= 28.0 {
            // If this was a non-repeating end block, stop playback now
            // that all samples have been consumed
            if self.final_block {
                self.enabled = false;
                self.adsr.phase = ADSRPhase::Off;
                self.final_block = false;
            }

            // Move to next block (16 bytes per block) unless we've just
            // processed a loop-end block that already updated current_address
            if !self.loop_flag {
                self.current_address += 16;
            }

            // Position will be reset when decode_block is called.
            // Loop end/repeat behavior is handled via loop_flag above.
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::core::spu::noise::NoiseGenerator;

    #[test]
    fn test_voice_creation() {
        let voice = Voice::new(5);
        assert_eq!(voice.id, 5);
        assert!(!voice.enabled);
        assert_eq!(voice.volume_left, 0);
        assert_eq!(voice.volume_right, 0);
        assert_eq!(voice.sample_rate, 0);
        assert_eq!(voice.current_address, 0);
    }

    #[test]
    fn test_voice_key_on() {
        let mut voice = Voice::new(0);
        voice.start_address = 0x100;

        voice.key_on();

        assert!(voice.enabled);
        assert_eq!(voice.current_address, 0x100 * 8);
        assert_eq!(voice.adsr.phase, ADSRPhase::Attack);
        assert_eq!(voice.adsr.level, 0);
        assert!(!voice.loop_flag);
        assert!(!voice.final_block);
    }

    #[test]
    fn test_voice_key_off() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.adsr.phase = ADSRPhase::Sustain;
        voice.adsr.level = 10000;

        voice.key_off();

        assert_eq!(voice.adsr.phase, ADSRPhase::Release);
        assert_eq!(voice.adsr.level, 10000); // Level preserved
    }

    #[test]
    fn test_voice_render_disabled() {
        let mut voice = Voice::new(0);
        voice.enabled = false;

        let spu_ram = vec![0u8; 512 * 1024];
        let mut noise = NoiseGenerator::new();

        let (left, right) = voice.render_sample(&spu_ram, &mut noise);

        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_voice_render_noise_mode() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.noise_enabled = true;
        voice.adsr.phase = ADSRPhase::Sustain;
        voice.adsr.level = 32767;
        voice.volume_left = 0x4000;
        voice.volume_right = 0x4000;

        let spu_ram = vec![0u8; 512 * 1024];
        let mut noise = NoiseGenerator::new();
        noise.set_frequency(0, 1);

        let (_left, _right) = voice.render_sample(&spu_ram, &mut noise);

        // Noise mode should produce output
        // Output depends on noise state and is always valid i16
        // Just verify it doesn't crash
    }

    #[test]
    fn test_voice_decode_block_basic() {
        let mut voice = Voice::new(0);
        let mut spu_ram = vec![0u8; 512 * 1024];

        // Create a simple ADPCM block
        let block_addr: usize = 0x1000;
        voice.current_address = block_addr as u32;

        // Block header: shift=0, filter=0
        spu_ram[block_addr] = 0x00;
        spu_ram[block_addr + 1] = 0x00; // No loop flags

        // Fill with simple pattern
        for i in 2..16 {
            spu_ram[block_addr + i] = 0x11;
        }

        voice.decode_block(&spu_ram);

        // Should have 28 decoded samples
        assert_eq!(voice.decoded_samples.len(), 28);
        assert_eq!(voice.adpcm_state.position, 0.0);
    }

    #[test]
    fn test_voice_decode_block_loop_repeat() {
        let mut voice = Voice::new(0);
        let mut spu_ram = vec![0u8; 512 * 1024];

        voice.current_address = 0x1000;
        voice.repeat_address = 0x200;

        // Block with loop end and repeat flags
        spu_ram[0x1000] = 0x00;
        spu_ram[0x1001] = 0x03; // Loop end | Loop repeat

        for i in 2..16 {
            spu_ram[0x1000 + i] = 0x22;
        }

        voice.decode_block(&spu_ram);

        // Should set loop flag and jump to repeat address
        assert!(voice.loop_flag);
        assert_eq!(voice.current_address, 0x200 * 8);
    }

    #[test]
    fn test_voice_decode_block_final() {
        let mut voice = Voice::new(0);
        let mut spu_ram = vec![0u8; 512 * 1024];

        voice.current_address = 0x1000;

        // Block with loop end but no repeat (final block)
        spu_ram[0x1000] = 0x00;
        spu_ram[0x1001] = 0x01; // Loop end only

        for i in 2..16 {
            spu_ram[0x1000 + i] = 0x33;
        }

        voice.decode_block(&spu_ram);

        // Should mark as final block
        assert!(voice.final_block);
        assert_eq!(voice.decoded_samples.len(), 28);
    }

    #[test]
    fn test_voice_decode_out_of_bounds() {
        let mut voice = Voice::new(0);
        let spu_ram = vec![0u8; 512 * 1024];

        // Set address beyond RAM
        voice.current_address = 512 * 1024 - 8;

        voice.decode_block(&spu_ram);

        // Should handle gracefully
        assert!(voice.decoded_samples.is_empty());
        assert!(!voice.enabled);
        assert_eq!(voice.adsr.phase, ADSRPhase::Off);
    }

    #[test]
    fn test_voice_interpolate_sample() {
        let mut voice = Voice::new(0);
        voice.decoded_samples = vec![100, 200, 300, 400];
        voice.adpcm_state.position = 1.5;

        let sample = voice.interpolate_sample();

        // Should interpolate between samples[1] and samples[2]
        // Expected: 200 + (300 - 200) * 0.5 = 250
        assert_eq!(sample, 250);
    }

    #[test]
    fn test_voice_interpolate_at_boundary() {
        let mut voice = Voice::new(0);
        voice.decoded_samples = vec![100, 200, 300];
        voice.adpcm_state.position = 0.0;

        let sample = voice.interpolate_sample();

        // At position 0.0, should return first sample
        assert_eq!(sample, 100);
    }

    #[test]
    fn test_voice_interpolate_empty() {
        let voice = Voice::new(0);

        let sample = voice.interpolate_sample();

        // Empty samples should return 0
        assert_eq!(sample, 0);
    }

    #[test]
    fn test_voice_advance_position() {
        let mut voice = Voice::new(0);
        voice.sample_rate = 4096; // 1.0 in 4.12 fixed point
        voice.adpcm_state.position = 10.0;
        voice.decoded_samples = vec![0i16; 28];

        voice.advance_position();

        // Position should advance by 1.0
        assert_eq!(voice.adpcm_state.position, 11.0);
    }

    #[test]
    fn test_voice_advance_position_block_transition() {
        let mut voice = Voice::new(0);
        voice.sample_rate = 4096;
        voice.adpcm_state.position = 27.5;
        voice.current_address = 0x1000;
        voice.decoded_samples = vec![0i16; 28];
        voice.loop_flag = false;

        voice.advance_position();

        // Should advance past block boundary
        assert!(voice.adpcm_state.position >= 28.0);
        // Current address should advance by 16 bytes
        assert_eq!(voice.current_address, 0x1000 + 16);
    }

    #[test]
    fn test_voice_advance_position_final_block() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.sample_rate = 4096;
        voice.adpcm_state.position = 27.5;
        voice.final_block = true;
        voice.decoded_samples = vec![0i16; 28];

        voice.advance_position();

        // Should disable voice on final block completion
        assert!(!voice.enabled);
        assert_eq!(voice.adsr.phase, ADSRPhase::Off);
    }

    #[test]
    fn test_voice_volume_application() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.noise_enabled = false;
        voice.volume_left = 0x4000; // 0.5 in fixed-point
        voice.volume_right = 0x2000; // 0.25 in fixed-point
        voice.adsr.phase = ADSRPhase::Sustain;
        voice.adsr.level = 32767;
        voice.sample_rate = 4096;

        // Setup decoded samples
        voice.decoded_samples = vec![10000i16; 28];
        voice.adpcm_state.position = 0.0;

        let spu_ram = vec![0u8; 512 * 1024];
        let mut noise = NoiseGenerator::new();

        let (left, right) = voice.render_sample(&spu_ram, &mut noise);

        // Left should be roughly half of right (due to volume difference)
        assert!(left.abs() > right.abs());
    }

    #[test]
    fn test_voice_adsr_envelope_application() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.volume_left = 0x7FFF;
        voice.volume_right = 0x7FFF;
        voice.adsr.phase = ADSRPhase::Sustain;
        voice.adsr.level = 16383; // Half of max
        voice.sample_rate = 4096;

        voice.decoded_samples = vec![10000i16; 28];
        voice.adpcm_state.position = 0.0;

        let spu_ram = vec![0u8; 512 * 1024];
        let mut noise = NoiseGenerator::new();

        let (left, _right) = voice.render_sample(&spu_ram, &mut noise);

        // Output should be scaled by ADSR level
        // Expected: roughly 10000 * (16383 / 32767) ≈ 5000
        assert!(left.abs() < 10000); // Should be less than full amplitude
    }

    #[test]
    fn test_voice_multiple_voices_independent() {
        let mut voice1 = Voice::new(0);
        let mut voice2 = Voice::new(1);

        voice1.volume_left = 0x4000;
        voice2.volume_left = 0x2000;

        voice1.start_address = 0x100;
        voice2.start_address = 0x200;

        voice1.key_on();
        voice2.key_on();

        // Voices should be independent
        assert_eq!(voice1.current_address, 0x100 * 8);
        assert_eq!(voice2.current_address, 0x200 * 8);
        assert_ne!(voice1.volume_left, voice2.volume_left);
    }

    #[test]
    fn test_voice_sample_rate_range() {
        let mut voice = Voice::new(0);
        voice.decoded_samples = vec![100i16; 28];
        voice.adpcm_state.position = 0.0;

        // Test various sample rates
        let rates = [0u16, 4096, 8192, 16384, 65535];

        for rate in &rates {
            voice.sample_rate = *rate;
            let initial_pos = voice.adpcm_state.position;

            voice.advance_position();

            if *rate == 0 {
                // Zero rate should not advance
                assert_eq!(voice.adpcm_state.position, initial_pos);
            } else {
                // Non-zero rate should advance
                assert!(voice.adpcm_state.position > initial_pos);
            }

            voice.adpcm_state.position = 0.0; // Reset for next test
        }
    }

    #[test]
    fn test_voice_loop_address_handling() {
        let mut voice = Voice::new(0);
        voice.start_address = 0x100;
        voice.repeat_address = 0x200;

        voice.key_on();
        assert_eq!(voice.current_address, 0x100 * 8);

        // Simulate reaching loop point
        let mut spu_ram = vec![0u8; 512 * 1024];
        spu_ram[0x100 * 8] = 0x00;
        spu_ram[0x100 * 8 + 1] = 0x03; // Loop end | Loop repeat

        voice.decode_block(&spu_ram);

        // Should jump to repeat address
        assert_eq!(voice.current_address, 0x200 * 8);
    }

    #[test]
    fn test_voice_adsr_off_stops_output() {
        let mut voice = Voice::new(0);
        voice.enabled = true;
        voice.adsr.phase = ADSRPhase::Off;
        voice.volume_left = 0x7FFF;
        voice.volume_right = 0x7FFF;
        voice.decoded_samples = vec![10000i16; 28];

        let spu_ram = vec![0u8; 512 * 1024];
        let mut noise = NoiseGenerator::new();

        let (left, right) = voice.render_sample(&spu_ram, &mut noise);

        // ADSR Off should produce silence
        assert_eq!(left, 0);
        assert_eq!(right, 0);
    }

    #[test]
    fn test_voice_needs_decode() {
        let mut voice = Voice::new(0);

        // Empty samples should need decode
        assert!(voice.needs_decode());

        // Position past block should need decode
        voice.decoded_samples = vec![0i16; 28];
        voice.adpcm_state.position = 28.0;
        assert!(voice.needs_decode());

        // Valid position should not need decode
        voice.adpcm_state.position = 10.0;
        assert!(!voice.needs_decode());
    }

    #[test]
    fn test_voice_key_on_resets_state() {
        let mut voice = Voice::new(0);
        voice.start_address = 0x100;
        voice.adpcm_state.prev_samples = [100, 200];
        voice.decoded_samples = vec![1, 2, 3];
        voice.loop_flag = true;
        voice.final_block = true;

        voice.key_on();

        // Key on should reset all state
        assert_eq!(voice.adpcm_state.prev_samples, [0, 0]);
        assert!(voice.decoded_samples.is_empty());
        assert!(!voice.loop_flag);
        assert!(!voice.final_block);
        assert_eq!(voice.adsr.phase, ADSRPhase::Attack);
    }
}
