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

//! SPU register definitions and types

/// SPU control register
///
/// Controls SPU operation including enable/mute, DMA transfer mode,
/// and audio input settings.
pub struct SPUControl {
    pub enabled: bool,
    pub unmute: bool,
    pub noise_clock: u8,
    pub noise_step: u8,
    pub reverb_enabled: bool,
    pub irq_enabled: bool,
    pub transfer_mode: TransferMode,
    pub external_audio_reverb: bool,
    pub cd_audio_reverb: bool,
    pub external_audio_enabled: bool,
    pub cd_audio_enabled: bool,
}

/// SPU status register
///
/// Provides status information about SPU operation including
/// IRQ flags, DMA status, and capture readiness.
#[derive(Default)]
pub struct SPUStatus {
    #[allow(dead_code)]
    pub mode: u16,
    pub irq_flag: bool,
    #[allow(dead_code)]
    pub dma_request: bool,
    pub dma_busy: bool,
    #[allow(dead_code)]
    pub capture_ready: bool,
}

/// SPU data transfer mode
///
/// Specifies how data is transferred to/from SPU RAM.
#[derive(Debug, Clone, Copy)]
pub enum TransferMode {
    /// No transfer
    Stop,
    /// Manual write via FIFO
    ManualWrite,
    /// DMA write to SPU RAM
    DMAWrite,
    /// DMA read from SPU RAM
    DMARead,
}

impl Default for SPUControl {
    fn default() -> Self {
        Self {
            enabled: false,
            unmute: false,
            noise_clock: 0,
            noise_step: 0,
            reverb_enabled: false,
            irq_enabled: false,
            transfer_mode: TransferMode::Stop,
            external_audio_reverb: false,
            cd_audio_reverb: false,
            external_audio_enabled: false,
            cd_audio_enabled: false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_spu_control_default() {
        let ctrl = SPUControl::default();
        assert!(!ctrl.enabled);
        assert!(!ctrl.unmute);
        assert_eq!(ctrl.noise_clock, 0);
        assert_eq!(ctrl.noise_step, 0);
        assert!(!ctrl.reverb_enabled);
        assert!(!ctrl.irq_enabled);
        assert!(matches!(ctrl.transfer_mode, TransferMode::Stop));
        assert!(!ctrl.external_audio_reverb);
        assert!(!ctrl.cd_audio_reverb);
        assert!(!ctrl.external_audio_enabled);
        assert!(!ctrl.cd_audio_enabled);
    }

    #[test]
    fn test_spu_status_default() {
        let status = SPUStatus::default();
        assert_eq!(status.mode, 0);
        assert!(!status.irq_flag);
        assert!(!status.dma_request);
        assert!(!status.dma_busy);
        assert!(!status.capture_ready);
    }

    #[test]
    fn test_transfer_mode_variants() {
        // Verify all transfer mode variants exist
        let modes = [
            TransferMode::Stop,
            TransferMode::ManualWrite,
            TransferMode::DMAWrite,
            TransferMode::DMARead,
        ];

        for mode in &modes {
            // Test Debug formatting works
            let _debug_str = format!("{:?}", mode);
        }
    }

    #[test]
    fn test_spu_control_all_enabled() {
        let mut ctrl = SPUControl::default();
        ctrl.enabled = true;
        ctrl.unmute = true;
        ctrl.noise_clock = 0xF;
        ctrl.noise_step = 0x3;
        ctrl.reverb_enabled = true;
        ctrl.irq_enabled = true;
        ctrl.transfer_mode = TransferMode::DMAWrite;
        ctrl.external_audio_reverb = true;
        ctrl.cd_audio_reverb = true;
        ctrl.external_audio_enabled = true;
        ctrl.cd_audio_enabled = true;

        assert!(ctrl.enabled);
        assert!(ctrl.unmute);
        assert_eq!(ctrl.noise_clock, 0xF);
        assert_eq!(ctrl.noise_step, 0x3);
        assert!(ctrl.reverb_enabled);
        assert!(ctrl.irq_enabled);
        assert!(matches!(ctrl.transfer_mode, TransferMode::DMAWrite));
        assert!(ctrl.external_audio_reverb);
        assert!(ctrl.cd_audio_reverb);
        assert!(ctrl.external_audio_enabled);
        assert!(ctrl.cd_audio_enabled);
    }

    #[test]
    fn test_spu_status_all_flags_set() {
        let mut status = SPUStatus::default();
        status.mode = 0xFFFF;
        status.irq_flag = true;
        status.dma_request = true;
        status.dma_busy = true;
        status.capture_ready = true;

        assert_eq!(status.mode, 0xFFFF);
        assert!(status.irq_flag);
        assert!(status.dma_request);
        assert!(status.dma_busy);
        assert!(status.capture_ready);
    }

    #[test]
    fn test_transfer_mode_clone() {
        let mode1 = TransferMode::DMAWrite;
        let mode2 = mode1;

        assert!(matches!(mode1, TransferMode::DMAWrite));
        assert!(matches!(mode2, TransferMode::DMAWrite));
    }

    #[test]
    fn test_noise_parameters() {
        let mut ctrl = SPUControl::default();

        // Test noise clock range (0-15)
        for clock in 0..=15u8 {
            ctrl.noise_clock = clock;
            assert_eq!(ctrl.noise_clock, clock);
        }

        // Test noise step range (0-3)
        for step in 0..=3u8 {
            ctrl.noise_step = step;
            assert_eq!(ctrl.noise_step, step);
        }
    }

    #[test]
    fn test_transfer_modes() {
        let mut ctrl = SPUControl::default();

        let modes = [
            TransferMode::Stop,
            TransferMode::ManualWrite,
            TransferMode::DMAWrite,
            TransferMode::DMARead,
        ];

        for mode in &modes {
            ctrl.transfer_mode = *mode;
            assert!(
                matches!(ctrl.transfer_mode, _m if std::mem::discriminant(&ctrl.transfer_mode) == std::mem::discriminant(mode))
            );
        }
    }

    #[test]
    fn test_spu_status_independent_flags() {
        let mut status = SPUStatus::default();

        // Test each flag independently
        status.irq_flag = true;
        assert!(status.irq_flag);
        assert!(!status.dma_busy);

        status.irq_flag = false;
        status.dma_busy = true;
        assert!(!status.irq_flag);
        assert!(status.dma_busy);
    }
}
