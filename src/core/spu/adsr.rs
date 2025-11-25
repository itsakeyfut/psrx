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

//! ADSR (Attack, Decay, Sustain, Release) envelope generator
//!
//! Controls the volume envelope for each voice over time.
//! The envelope has four phases:
//! - Attack: Volume rises from 0 to maximum
//! - Decay: Volume falls from maximum to sustain level
//! - Sustain: Volume holds at sustain level
//! - Release: Volume falls from current level to 0

/// ADSR (Attack, Decay, Sustain, Release) envelope generator
///
/// Controls the volume envelope for each voice over time.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ADSREnvelope {
    pub attack_rate: u8,
    pub attack_mode: AttackMode,
    pub decay_rate: u8,
    pub sustain_level: u8,
    pub sustain_rate: u8,
    pub sustain_mode: SustainMode,
    pub release_rate: u8,
    pub release_mode: ReleaseMode,

    /// Current ADSR phase
    pub phase: ADSRPhase,

    /// Current envelope level (0-32767)
    pub level: i16,
}

/// ADSR envelope phase
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ADSRPhase {
    /// Attack phase: volume rising
    Attack,
    /// Decay phase: volume falling to sustain level
    Decay,
    /// Sustain phase: volume held at sustain level
    Sustain,
    /// Release phase: volume falling to zero
    Release,
    /// Off: voice is silent
    Off,
}

/// Attack mode (linear or exponential)
#[derive(Debug, Clone, Copy)]
pub enum AttackMode {
    Linear,
    Exponential,
}

/// Sustain mode (linear or exponential)
#[derive(Debug, Clone, Copy)]
pub enum SustainMode {
    Linear,
    Exponential,
}

/// Release mode (linear or exponential)
#[derive(Debug, Clone, Copy)]
pub enum ReleaseMode {
    Linear,
    Exponential,
}

#[allow(dead_code)]
impl ADSREnvelope {
    /// Advance the ADSR envelope by one sample
    ///
    /// Updates the current level based on the current phase and configured rates.
    /// Called once per audio sample (44100 Hz).
    pub fn tick(&mut self) {
        match self.phase {
            ADSRPhase::Attack => self.tick_attack(),
            ADSRPhase::Decay => self.tick_decay(),
            ADSRPhase::Sustain => self.tick_sustain(),
            ADSRPhase::Release => self.tick_release(),
            ADSRPhase::Off => {}
        }
    }

    /// Process attack phase
    fn tick_attack(&mut self) {
        let rate = self.attack_rate_to_step();

        match self.attack_mode {
            AttackMode::Linear => {
                self.level = self.level.saturating_add(rate);
            }
            AttackMode::Exponential => {
                // Exponential: increase rate scales with distance from max.
                // When we're very close to max, the computed step can round to 0,
                // which would otherwise leave us stuck in the attack phase.
                let step = ((rate as i32 * (32767 - self.level as i32)) >> 15) as i16;
                if step > 0 {
                    self.level = self.level.saturating_add(step);
                } else {
                    self.level = 32767;
                }
            }
        }

        if self.level == 32767 {
            self.phase = ADSRPhase::Decay;
        }
    }

    /// Process decay phase
    fn tick_decay(&mut self) {
        let rate = self.decay_rate_to_step();
        let sustain_level = ((self.sustain_level as i32 + 1) << 11).min(32767) as i16;

        // Decay is always exponential in hardware
        let step = ((rate as i32 * self.level as i32) >> 15) as i16;
        self.level = self.level.saturating_sub(step);

        if self.level <= sustain_level {
            self.level = sustain_level;
            self.phase = ADSRPhase::Sustain;
        }
    }

    /// Process sustain phase
    fn tick_sustain(&mut self) {
        let rate = self.sustain_rate_to_step();

        match self.sustain_mode {
            SustainMode::Linear => {
                self.level = self.level.saturating_sub(rate);
            }
            SustainMode::Exponential => {
                let step = ((rate as i32 * self.level as i32) >> 15) as i16;
                self.level = self.level.saturating_sub(step);
            }
        }

        if self.level <= 0 {
            self.level = 0;
            self.phase = ADSRPhase::Off;
        }
    }

    /// Process release phase
    fn tick_release(&mut self) {
        let rate = self.release_rate_to_step();

        match self.release_mode {
            ReleaseMode::Linear => {
                self.level = self.level.saturating_sub(rate);
            }
            ReleaseMode::Exponential => {
                let step = ((rate as i32 * self.level as i32) >> 15) as i16;
                self.level = self.level.saturating_sub(step);
            }
        }

        if self.level <= 0 {
            self.level = 0;
            self.phase = ADSRPhase::Off;
        }
    }

    /// Convert attack rate to step value
    ///
    /// # Returns
    ///
    /// Step value to add per sample during attack phase
    fn attack_rate_to_step(&self) -> i16 {
        // PSX attack rate formula: simplified approximation
        // Real hardware uses complex cycle counters
        if self.attack_rate == 0 {
            return 0;
        }

        // Higher rate = faster attack
        // Rate 127 should reach max in ~1ms (44 samples)
        // Rate 0 = infinite attack
        let rate = self.attack_rate as i32;
        ((32767 * rate) / (128 * 50)) as i16
    }

    /// Convert decay rate to step value
    ///
    /// # Returns
    ///
    /// Step value for decay phase
    fn decay_rate_to_step(&self) -> i16 {
        if self.decay_rate == 0 {
            return 0;
        }

        let rate = self.decay_rate as i32;
        ((32767 * rate) / (16 * 200)) as i16
    }

    /// Convert sustain rate to step value
    ///
    /// # Returns
    ///
    /// Step value for sustain phase
    fn sustain_rate_to_step(&self) -> i16 {
        if self.sustain_rate == 0 {
            return 0;
        }

        let rate = self.sustain_rate as i32;
        ((32767 * rate) / (128 * 200)) as i16
    }

    /// Convert release rate to step value
    ///
    /// # Returns
    ///
    /// Step value for release phase
    fn release_rate_to_step(&self) -> i16 {
        if self.release_rate == 0 {
            return 0;
        }

        let rate = self.release_rate as i32;
        ((32767 * rate) / (32 * 200)) as i16
    }

    /// Convert ADSR configuration to register format (word 1)
    ///
    /// # Returns
    ///
    /// Lower 16 bits of ADSR configuration
    ///
    /// # Format
    ///
    /// ```text
    /// Bits  0-3:  Sustain Level
    /// Bit   4:    Decay Rate (bit 0)
    /// Bits  5-7:  Decay Rate (bits 1-3)
    /// Bits  8-14: Attack Rate
    /// Bit   15:   Attack Mode (0=Linear, 1=Exponential)
    /// ```
    pub fn to_word_1(&self) -> u16 {
        let mut value = 0u16;

        value |= (self.sustain_level as u16) & 0xF;
        value |= ((self.decay_rate as u16) & 0xF) << 4;
        value |= ((self.attack_rate as u16) & 0x7F) << 8;
        value |= if matches!(self.attack_mode, AttackMode::Exponential) {
            1 << 15
        } else {
            0
        };

        value
    }

    /// Convert ADSR configuration to register format (word 2)
    ///
    /// # Returns
    ///
    /// Upper 16 bits of ADSR configuration
    ///
    /// # Format
    ///
    /// ```text
    /// Bits  0-4:  Release Rate
    /// Bit   5:    Release Mode (0=Linear, 1=Exponential)
    /// Bits  6-12: Sustain Rate
    /// Bit   13:   (unused, always 0)
    /// Bit   14:   Sustain Direction (0=Increase, 1=Decrease)
    /// Bit   15:   Sustain Mode (0=Linear, 1=Exponential)
    /// ```
    pub fn to_word_2(&self) -> u16 {
        let mut value = 0u16;

        value |= (self.release_rate as u16) & 0x1F;
        value |= if matches!(self.release_mode, ReleaseMode::Exponential) {
            1 << 5
        } else {
            0
        };
        value |= ((self.sustain_rate as u16) & 0x7F) << 6;
        value |= if matches!(self.sustain_mode, SustainMode::Exponential) {
            1 << 15
        } else {
            0
        };

        value
    }

    /// Load ADSR configuration from register format (word 1)
    ///
    /// # Arguments
    ///
    /// * `value` - Lower 16 bits of ADSR configuration
    pub fn set_word_1(&mut self, value: u16) {
        self.sustain_level = (value & 0xF) as u8;
        self.decay_rate = ((value >> 4) & 0xF) as u8;
        self.attack_rate = ((value >> 8) & 0x7F) as u8;
        self.attack_mode = if (value & (1 << 15)) != 0 {
            AttackMode::Exponential
        } else {
            AttackMode::Linear
        };
    }

    /// Load ADSR configuration from register format (word 2)
    ///
    /// # Arguments
    ///
    /// * `value` - Upper 16 bits of ADSR configuration
    pub fn set_word_2(&mut self, value: u16) {
        self.release_rate = (value & 0x1F) as u8;
        self.release_mode = if (value & (1 << 5)) != 0 {
            ReleaseMode::Exponential
        } else {
            ReleaseMode::Linear
        };
        self.sustain_rate = ((value >> 6) & 0x7F) as u8;
        self.sustain_mode = if (value & (1 << 15)) != 0 {
            SustainMode::Exponential
        } else {
            SustainMode::Linear
        };
    }
}

impl Default for ADSREnvelope {
    fn default() -> Self {
        Self {
            attack_rate: 0,
            attack_mode: AttackMode::Linear,
            decay_rate: 0,
            sustain_level: 0,
            sustain_rate: 0,
            sustain_mode: SustainMode::Linear,
            release_rate: 0,
            release_mode: ReleaseMode::Linear,
            phase: ADSRPhase::Off,
            level: 0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_adsr_envelope_default() {
        let env = ADSREnvelope::default();
        assert_eq!(env.level, 0);
        assert_eq!(env.phase, ADSRPhase::Off);
        assert_eq!(env.attack_rate, 0);
        assert_eq!(env.decay_rate, 0);
        assert_eq!(env.sustain_level, 0);
        assert_eq!(env.sustain_rate, 0);
        assert_eq!(env.release_rate, 0);
    }

    #[test]
    fn test_adsr_phase_transitions() {
        let mut env = ADSREnvelope::default();

        // Start in Off phase
        assert_eq!(env.phase, ADSRPhase::Off);

        // Transition to Attack
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 127;
        env.attack_mode = AttackMode::Linear;

        // Tick until max level
        while env.level < 32767 {
            env.tick();
        }

        // Should transition to Decay
        assert_eq!(env.phase, ADSRPhase::Decay);

        // Decay to sustain level
        env.decay_rate = 15;
        env.sustain_level = 8;
        while env.phase == ADSRPhase::Decay {
            env.tick();
        }

        // Should be in Sustain
        assert_eq!(env.phase, ADSRPhase::Sustain);
    }

    #[test]
    fn test_attack_linear_mode() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 64;
        env.attack_mode = AttackMode::Linear;
        env.level = 0;

        let initial_level = env.level;
        env.tick();

        // Linear mode should add constant step
        assert!(env.level > initial_level, "Level should increase");
        assert_eq!(env.phase, ADSRPhase::Attack, "Should stay in Attack");
    }

    #[test]
    fn test_attack_exponential_mode() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 64;
        env.attack_mode = AttackMode::Exponential;
        env.level = 1000;

        let initial_level = env.level;
        env.tick();

        // Exponential mode should increase by scaled step
        assert!(
            env.level > initial_level,
            "Level should increase in exponential attack"
        );
    }

    #[test]
    fn test_attack_reaches_maximum() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 127; // Maximum rate
        env.attack_mode = AttackMode::Linear;
        env.level = 0;

        // Tick until we reach max or transition to Decay
        for _ in 0..10000 {
            if env.phase != ADSRPhase::Attack {
                break;
            }
            env.tick();
        }

        // Should have transitioned to Decay
        assert_eq!(env.phase, ADSRPhase::Decay);
        assert_eq!(env.level, 32767, "Should reach maximum level");
    }

    #[test]
    fn test_attack_exponential_completes_near_max() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 32;
        env.attack_mode = AttackMode::Exponential;
        env.level = 32760; // Very close to max

        // When step rounds to 0, should jump to max
        env.tick();

        assert_eq!(env.level, 32767, "Should complete to max level");
        assert_eq!(env.phase, ADSRPhase::Decay, "Should transition to Decay");
    }

    #[test]
    fn test_decay_phase() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Decay;
        env.level = 32767;
        env.decay_rate = 8;
        env.sustain_level = 10;

        let initial_level = env.level;
        env.tick();

        // Decay should decrease level
        assert!(env.level < initial_level, "Level should decrease in decay");
    }

    #[test]
    fn test_decay_reaches_sustain_level() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Decay;
        env.level = 32767;
        env.decay_rate = 15;
        env.sustain_level = 8; // Target level: (8+1) << 11 = 18432

        // Tick until we reach sustain or timeout
        for _ in 0..100000 {
            if env.phase != ADSRPhase::Decay {
                break;
            }
            env.tick();
        }

        // Should have transitioned to Sustain
        assert_eq!(env.phase, ADSRPhase::Sustain);

        let expected_sustain = ((env.sustain_level as i32 + 1) << 11).min(32767) as i16;
        assert_eq!(env.level, expected_sustain, "Should reach sustain level");
    }

    #[test]
    fn test_sustain_linear_decrease() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Sustain;
        env.level = 10000;
        env.sustain_rate = 16;
        env.sustain_mode = SustainMode::Linear;

        let initial_level = env.level;
        env.tick();

        // Linear sustain should decrease by constant step
        assert!(
            env.level < initial_level,
            "Level should decrease in linear sustain"
        );
    }

    #[test]
    fn test_sustain_exponential_decrease() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Sustain;
        env.level = 10000;
        env.sustain_rate = 16;
        env.sustain_mode = SustainMode::Exponential;

        let initial_level = env.level;
        env.tick();

        // Exponential sustain should decrease proportionally
        assert!(
            env.level < initial_level,
            "Level should decrease in exponential sustain"
        );
    }

    #[test]
    fn test_sustain_reaches_zero() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Sustain;
        env.level = 100;
        env.sustain_rate = 64;
        env.sustain_mode = SustainMode::Linear;

        // Tick until we reach zero or timeout
        for _ in 0..10000 {
            if env.phase == ADSRPhase::Off {
                break;
            }
            env.tick();
        }

        // Should have transitioned to Off
        assert_eq!(env.phase, ADSRPhase::Off);
        assert_eq!(env.level, 0, "Level should reach zero");
    }

    #[test]
    fn test_release_linear_mode() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Release;
        env.level = 10000;
        env.release_rate = 16;
        env.release_mode = ReleaseMode::Linear;

        let initial_level = env.level;
        env.tick();

        // Release should decrease level
        assert!(
            env.level < initial_level,
            "Level should decrease in linear release"
        );
    }

    #[test]
    fn test_release_exponential_mode() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Release;
        env.level = 10000;
        env.release_rate = 16;
        env.release_mode = ReleaseMode::Exponential;

        let initial_level = env.level;
        env.tick();

        // Exponential release should decrease proportionally
        assert!(
            env.level < initial_level,
            "Level should decrease in exponential release"
        );
    }

    #[test]
    fn test_release_reaches_zero() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Release;
        env.level = 100;
        env.release_rate = 31; // Max rate
        env.release_mode = ReleaseMode::Linear;

        // Tick until we reach zero or timeout
        for _ in 0..10000 {
            if env.phase == ADSRPhase::Off {
                break;
            }
            env.tick();
        }

        // Should have transitioned to Off
        assert_eq!(env.phase, ADSRPhase::Off);
        assert_eq!(env.level, 0, "Level should reach zero");
    }

    #[test]
    fn test_off_phase_no_change() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Off;
        env.level = 0;

        env.tick();

        // Off phase should not change anything
        assert_eq!(env.phase, ADSRPhase::Off);
        assert_eq!(env.level, 0);
    }

    #[test]
    fn test_rate_zero_no_change() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 0; // Zero rate
        env.attack_mode = AttackMode::Linear;
        env.level = 0;

        env.tick();

        // Zero rate should not change level
        assert_eq!(env.level, 0, "Zero rate should not change level");
        assert_eq!(env.phase, ADSRPhase::Attack, "Should stay in same phase");
    }

    #[test]
    fn test_word_1_serialization() {
        let mut env = ADSREnvelope::default();
        env.sustain_level = 0xA;
        env.decay_rate = 0x7;
        env.attack_rate = 0x3F;
        env.attack_mode = AttackMode::Exponential;

        let word = env.to_word_1();

        // Verify bit layout
        assert_eq!(word & 0xF, 0xA, "Bits 0-3: Sustain Level");
        assert_eq!((word >> 4) & 0xF, 0x7, "Bits 4-7: Decay Rate");
        assert_eq!((word >> 8) & 0x7F, 0x3F, "Bits 8-14: Attack Rate");
        assert_eq!((word >> 15) & 0x1, 1, "Bit 15: Attack Mode");
    }

    #[test]
    fn test_word_2_serialization() {
        let mut env = ADSREnvelope::default();
        env.release_rate = 0x15;
        env.release_mode = ReleaseMode::Exponential;
        env.sustain_rate = 0x40;
        env.sustain_mode = SustainMode::Exponential;

        let word = env.to_word_2();

        // Verify bit layout
        assert_eq!(word & 0x1F, 0x15, "Bits 0-4: Release Rate");
        assert_eq!((word >> 5) & 0x1, 1, "Bit 5: Release Mode");
        assert_eq!((word >> 6) & 0x7F, 0x40, "Bits 6-12: Sustain Rate");
        assert_eq!((word >> 15) & 0x1, 1, "Bit 15: Sustain Mode");
    }

    #[test]
    fn test_word_1_deserialization() {
        let mut env = ADSREnvelope::default();

        // Word with known values
        let word: u16 = (1 << 15) | (0x3F << 8) | (0x7 << 4) | 0xA;

        env.set_word_1(word);

        assert_eq!(env.sustain_level, 0xA);
        assert_eq!(env.decay_rate, 0x7);
        assert_eq!(env.attack_rate, 0x3F);
        assert!(matches!(env.attack_mode, AttackMode::Exponential));
    }

    #[test]
    fn test_word_2_deserialization() {
        let mut env = ADSREnvelope::default();

        // Word with known values
        let word: u16 = (1 << 15) | (0x40 << 6) | (1 << 5) | 0x15;

        env.set_word_2(word);

        assert_eq!(env.release_rate, 0x15);
        assert!(matches!(env.release_mode, ReleaseMode::Exponential));
        assert_eq!(env.sustain_rate, 0x40);
        assert!(matches!(env.sustain_mode, SustainMode::Exponential));
    }

    #[test]
    fn test_serialization_round_trip() {
        let mut env1 = ADSREnvelope::default();
        env1.sustain_level = 12;
        env1.decay_rate = 9;
        env1.attack_rate = 100;
        env1.attack_mode = AttackMode::Linear;
        env1.release_rate = 20;
        env1.release_mode = ReleaseMode::Exponential;
        env1.sustain_rate = 50;
        env1.sustain_mode = SustainMode::Linear;

        let word1 = env1.to_word_1();
        let word2 = env1.to_word_2();

        let mut env2 = ADSREnvelope::default();
        env2.set_word_1(word1);
        env2.set_word_2(word2);

        assert_eq!(env2.sustain_level, env1.sustain_level);
        assert_eq!(env2.decay_rate, env1.decay_rate);
        assert_eq!(env2.attack_rate, env1.attack_rate);
        assert_eq!(env2.release_rate, env1.release_rate);
        assert_eq!(env2.sustain_rate, env1.sustain_rate);
    }

    #[test]
    fn test_sustain_level_calculation() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Decay;
        env.level = 32767;
        env.decay_rate = 15;

        // Test various sustain levels
        for sustain_level in 0..=15u8 {
            env.sustain_level = sustain_level;
            env.level = 32767;
            env.phase = ADSRPhase::Decay;

            // Decay to sustain
            for _ in 0..100000 {
                if env.phase != ADSRPhase::Decay {
                    break;
                }
                env.tick();
            }

            let expected = ((sustain_level as i32 + 1) << 11).min(32767) as i16;
            assert_eq!(
                env.level, expected,
                "Sustain level {} should reach {}",
                sustain_level, expected
            );
        }
    }

    #[test]
    fn test_level_saturation() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Attack;
        env.attack_rate = 127;
        env.attack_mode = AttackMode::Linear;
        env.level = 32760;

        // Tick multiple times
        for _ in 0..100 {
            env.tick();
            // Level is i16, always in valid range
            // Just verify it reaches the attack phase completion
            if env.phase != ADSRPhase::Attack {
                break;
            }
        }

        // Should have transitioned to Decay
        assert_eq!(env.phase, ADSRPhase::Decay, "Should reach Decay phase");
    }

    #[test]
    fn test_level_floor_zero() {
        let mut env = ADSREnvelope::default();
        env.phase = ADSRPhase::Release;
        env.release_rate = 31;
        env.release_mode = ReleaseMode::Linear;
        env.level = 10;

        // Tick multiple times
        for _ in 0..1000 {
            env.tick();
            // Level should never go below zero
            assert!(env.level >= 0, "Level should not go below zero");
            if env.phase == ADSRPhase::Off {
                break;
            }
        }
    }
}
