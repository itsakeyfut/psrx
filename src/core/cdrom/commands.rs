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

//! CD-ROM command implementations
//!
//! This module contains implementations of all CD-ROM commands
//! (GetStat, SetLoc, ReadN, etc.)
//!
//! Commands are now executed via timing events with proper delays:
//! 1. CPU writes command -> queued for scheduling
//! 2. After ACK delay -> execute_command_callback() sends INT3
//! 3. For multi-stage commands -> queue second response
//! 4. After completion delay -> execute_second_response_callback() sends INT2

use super::{bcd_to_dec, CDPosition, CDState, SecondResponseType, CDROM};
use crate::core::timing::{TickCount, TimingEventManager};

impl CDROM {
    /// Execute CD-ROM command
    ///
    /// Executes the specified command byte, consuming parameters from
    /// the parameter FIFO and generating responses in the response FIFO.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command byte (0x00-0xFF)
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::core::cdrom::CDROM;
    ///
    /// let mut cdrom = CDROM::new();
    /// cdrom.execute_command(0x01); // GetStat
    /// assert!(!cdrom.response_empty());
    /// ```
    pub fn execute_command(&mut self, cmd: u8) {
        log::debug!("CD-ROM command: 0x{:02X}", cmd);

        match cmd {
            0x01 => self.cmd_getstat(),
            0x02 => self.cmd_setloc(),
            0x06 => self.cmd_readn(),
            0x09 => self.cmd_pause(),
            0x0A => self.cmd_init(),
            0x0E => self.cmd_setmode(),
            0x15 => self.cmd_seekl(),
            0x19 => self.cmd_test(),
            0x1A => self.cmd_getid(),
            0x1B => self.cmd_reads(),
            0x1E => self.cmd_readtoc(),
            _ => {
                log::warn!("Unknown CD-ROM command: 0x{:02X}", cmd);
                self.error_response();
            }
        }
    }

    /// Command 0x01: GetStat
    ///
    /// Returns the current drive status byte.
    pub(super) fn cmd_getstat(&mut self) {
        log::trace!("CD-ROM: GetStat");
        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)
    }

    /// Command 0x02: SetLoc
    ///
    /// Sets the seek target position from 3 parameter bytes (MM:SS:FF in BCD).
    pub(super) fn cmd_setloc(&mut self) {
        if self.param_fifo.len() < 3 {
            log::warn!("CD-ROM: SetLoc with insufficient parameters");
            self.error_response();
            return;
        }

        let minute = self.param_fifo.pop_front().unwrap();
        let second = self.param_fifo.pop_front().unwrap();
        let sector = self.param_fifo.pop_front().unwrap();

        self.seek_target = Some(CDPosition::new(
            bcd_to_dec(minute),
            bcd_to_dec(second),
            bcd_to_dec(sector),
        ));

        log::debug!(
            "CD-ROM: SetLoc to {:02}:{:02}:{:02}",
            bcd_to_dec(minute),
            bcd_to_dec(second),
            bcd_to_dec(sector)
        );

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)
    }

    /// Command 0x06: ReadN
    ///
    /// Start reading data sectors at current position.
    pub(super) fn cmd_readn(&mut self) {
        log::debug!("CD-ROM: ReadN");
        self.state = CDState::Reading;
        self.status.reading = true;
        self.read_ticks = 0; // Reset read timer

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Actual sector data will be read by tick() after appropriate timing
        // INT1 interrupts will be triggered when each sector is ready
    }

    /// Command 0x09: Pause
    ///
    /// Pause reading or audio playback.
    pub(super) fn cmd_pause(&mut self) {
        log::debug!("CD-ROM: Pause");

        self.state = CDState::Idle;
        self.status.reading = false;
        self.status.playing = false;

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Second response after pause completes
        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(2); // INT2 (complete)
    }

    /// Command 0x0A: Init
    ///
    /// Initialize the drive (motor on, reset state).
    pub(super) fn cmd_init(&mut self) {
        log::debug!("CD-ROM: Init");

        self.status.motor_on = true;
        self.state = CDState::Idle;
        self.status.reading = false;
        self.status.seeking = false;
        self.status.playing = false;

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Second response after init completes
        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(2); // INT2 (complete)
    }

    /// Command 0x0E: SetMode
    ///
    /// Set drive mode (speed, sector size, etc).
    ///
    /// # Mode byte format (parameter)
    ///
    /// ```text
    /// Bit 0: CD-DA mode (0=Off, 1=On)
    /// Bit 1: Auto Pause (0=Off, 1=On)
    /// Bit 2: Report (0=Off, 1=Report interrupts for all sectors)
    /// Bit 3: XA-Filter (0=Off, 1=Process only XA-ADPCM sectors)
    /// Bit 4: Ignore Bit (0=Off, 1=Ignore sector size and setloc position)
    /// Bit 5: Sector Size (0=2048 bytes, 1=2340 bytes)
    /// Bit 6: XA-ADPCM (0=Off, 1=Send XA-ADPCM to SPU)
    /// Bit 7: Double Speed (0=Off, 1=On, 2x speed)
    /// ```
    pub(super) fn cmd_setmode(&mut self) {
        if self.param_fifo.is_empty() {
            log::warn!("CD-ROM: SetMode with no parameters");
            self.error_response();
            return;
        }

        let mode_byte = self.param_fifo.pop_front().unwrap();
        log::debug!("CD-ROM: SetMode = 0x{:02X}", mode_byte);

        // Parse mode byte and update mode settings
        self.mode.cdda_report = (mode_byte & 0x01) != 0;
        self.mode.auto_pause = (mode_byte & 0x02) != 0;
        self.mode.report_all = (mode_byte & 0x04) != 0;
        self.mode.xa_filter = (mode_byte & 0x08) != 0;
        self.mode.ignore_bit = (mode_byte & 0x10) != 0;
        self.mode.size_2340 = (mode_byte & 0x20) != 0;
        self.mode.xa_adpcm = (mode_byte & 0x40) != 0;
        self.mode.double_speed = (mode_byte & 0x80) != 0;

        log::trace!(
            "CD-ROM: Mode settings - Speed: {}x, Size: {} bytes, XA-ADPCM: {}, Report All: {}",
            if self.mode.double_speed { 2 } else { 1 },
            if self.mode.size_2340 { 2340 } else { 2048 },
            self.mode.xa_adpcm,
            self.mode.report_all
        );

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)
    }

    /// Command 0x15: SeekL
    ///
    /// Seek to target position (data mode).
    pub(super) fn cmd_seekl(&mut self) {
        log::debug!("CD-ROM: SeekL");

        if self.seek_target.is_some() {
            self.state = CDState::Seeking;
            self.status.seeking = true;
            self.seek_ticks = 0; // Reset seek timer

            self.response_fifo.push_back(self.get_status_byte());
            self.trigger_interrupt(3); // INT3 (acknowledge)

        // The actual seek will complete in tick() after the appropriate delay
        // INT2 will be triggered when the seek completes
        } else {
            log::warn!("CD-ROM: SeekL with no target set");
            self.error_response();
        }
    }

    /// Command 0x19: Test
    ///
    /// Test/diagnostic commands with various sub-functions.
    ///
    /// # Sub-functions (first parameter byte)
    ///
    /// - 0x20: Get BIOS date/version (returns 4 bytes: YY, MM, DD, Version in BCD)
    /// - 0x04: Get CD controller chip ID (returns 5 bytes)
    /// - Other sub-functions are hardware diagnostic tests
    ///
    /// # Response
    ///
    /// Varies by sub-function. Most return status byte and test results.
    pub(super) fn cmd_test(&mut self) {
        if self.param_fifo.is_empty() {
            log::warn!("CD-ROM: Test with no parameters");
            self.error_response();
            return;
        }

        let subfunction = self.param_fifo.pop_front().unwrap();
        log::debug!("CD-ROM: Test sub-function 0x{:02X}", subfunction);

        match subfunction {
            0x20 => {
                // Get BIOS date/version
                // Real hardware returns: YY, MM, DD, Version (4 bytes)
                // For emulation, return a fixed date: 1998/08/07 (SCPH-1001 date)
                self.response_fifo.push_back(0x98); // Year (98 = 1998)
                self.response_fifo.push_back(0x08); // Month
                self.response_fifo.push_back(0x07); // Day
                self.response_fifo.push_back(0xC3); // Version byte

                log::trace!("CD-ROM: Test 0x20 - Returned BIOS date 1998/08/07");
                self.trigger_interrupt(3); // INT3 (acknowledge)
            }
            0x04 => {
                // Get CD controller chip ID/version
                // Return actual drive status and fixed chip ID for emulation
                self.response_fifo.push_back(self.get_status_byte());
                self.response_fifo.push_back(0x00); // Chip ID byte 1
                self.response_fifo.push_back(0x00); // Chip ID byte 2
                self.response_fifo.push_back(0x00); // Chip ID byte 3
                self.response_fifo.push_back(0x00); // Chip ID byte 4

                log::trace!("CD-ROM: Test 0x04 - Returned chip ID");
                self.trigger_interrupt(3); // INT3 (acknowledge)
            }
            _ => {
                log::warn!("CD-ROM: Unknown Test sub-function 0x{:02X}", subfunction);
                // For unknown test commands, return status byte
                self.response_fifo.push_back(self.get_status_byte());
                self.trigger_interrupt(3); // INT3 (acknowledge)
            }
        }
    }

    /// Command 0x1A: GetID
    ///
    /// Get disc identification (region, disc type, etc).
    pub(super) fn cmd_getid(&mut self) {
        log::debug!("CD-ROM: GetID");

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Second response with disc info
        if self.disc.is_some() {
            self.response_fifo.push_back(self.get_status_byte());
            self.response_fifo.push_back(0x00); // Licensed
            self.response_fifo.push_back(0x20); // Audio+CDROM
            self.response_fifo.push_back(0x00); // SCEx region string (unused)
            self.response_fifo.push_back(b'S'); // SCEx region
            self.response_fifo.push_back(b'C');
            self.response_fifo.push_back(b'E');
            self.response_fifo.push_back(b'A'); // SCEA (America)
            self.trigger_interrupt(2); // INT2 (complete)
        } else {
            // No disc
            self.status.id_error = true;
            self.error_response();
        }
    }

    /// Command 0x1B: ReadS
    ///
    /// Start reading sectors with retry on errors.
    pub(super) fn cmd_reads(&mut self) {
        log::debug!("CD-ROM: ReadS");

        self.state = CDState::Reading;
        self.status.reading = true;
        self.read_ticks = 0; // Reset read timer

        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Actual sector data will be read by tick() after appropriate timing
        // INT1 interrupts will be triggered when each sector is ready
    }

    /// Command 0x1E: ReadTOC
    ///
    /// Read table of contents from disc.
    ///
    /// This command reads the disc's TOC (track information) and stores it
    /// internally. The TOC is used by subsequent commands like GetTD (Get Track Duration).
    ///
    /// # Response
    ///
    /// First response (INT3): Status byte
    /// Second response (INT2): Status byte (after TOC read completes)
    ///
    /// # Timing
    ///
    /// The TOC read takes approximately 1 second on real hardware.
    /// For now, we respond immediately.
    pub(super) fn cmd_readtoc(&mut self) {
        log::debug!("CD-ROM: ReadTOC");

        if self.disc.is_none() {
            log::warn!("CD-ROM: ReadTOC with no disc loaded");
            self.status.id_error = true;
            self.error_response();
            return;
        }

        // First response: acknowledge
        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(3); // INT3 (acknowledge)

        // Log TOC information for debugging
        if let Some(ref disc) = self.disc {
            let track_count = disc.track_count();
            log::debug!("CD-ROM: ReadTOC - {} tracks on disc", track_count);

            for i in 1..=track_count {
                if let Some(track) = disc.get_track(i as u8) {
                    log::trace!(
                        "CD-ROM: Track {} - Type: {:?}, Start: {:02}:{:02}:{:02}, Length: {} sectors",
                        track.number,
                        track.track_type,
                        track.start_position.minute,
                        track.start_position.second,
                        track.start_position.sector,
                        track.length_sectors
                    );
                }
            }
        }

        // Second response: TOC read complete
        self.response_fifo.push_back(self.get_status_byte());
        self.trigger_interrupt(2); // INT2 (complete)
    }
}

// ============================================================================
// Timing Event Callbacks
// ============================================================================

impl CDROM {
    /// Execute command callback (called when command_event fires)
    ///
    /// Executes the pending command after ACK delay.
    /// Sends INT3 (acknowledge) and may queue second response.
    ///
    /// # Arguments
    ///
    /// * `timing` - Timing event manager
    pub(super) fn execute_command_callback(&mut self, timing: &mut TimingEventManager) {
        let Some(cmd) = self.pending_command.take() else {
            return;
        };

        log::debug!("CD-ROM: Executing command 0x{:02X} after ACK delay", cmd);

        // Execute command-specific logic
        match cmd {
            0x01 => {
                // GetStat: Single response, no second response needed
                self.send_ack_and_stat();
                log::trace!("CD-ROM: GetStat command complete");
            }
            0x02 => {
                // SetLoc: Parse parameters and set seek target
                self.send_ack_and_stat();
                if self.param_fifo.len() >= 3 {
                    let minute = self.param_fifo.pop_front().unwrap();
                    let second = self.param_fifo.pop_front().unwrap();
                    let sector = self.param_fifo.pop_front().unwrap();

                    self.seek_target = Some(CDPosition::new(
                        bcd_to_dec(minute),
                        bcd_to_dec(second),
                        bcd_to_dec(sector),
                    ));
                    log::debug!(
                        "CD-ROM: SetLoc to {:02}:{:02}:{:02}",
                        bcd_to_dec(minute),
                        bcd_to_dec(second),
                        bcd_to_dec(sector)
                    );
                }
            }
            0x06 | 0x1B => {
                // ReadN / ReadS: Start reading
                self.send_ack_and_stat();
                self.state = CDState::Reading;
                self.status.reading = true;
                // Sector reading will be handled by sector_read_event
            }
            0x09 => {
                // Pause: Stop reading, queue second response
                self.send_ack_and_stat();
                self.state = CDState::Idle;
                self.status.reading = false;
                self.status.playing = false;
                self.queue_second_response(SecondResponseType::Pause, timing);
            }
            0x0A => {
                // Init: Initialize drive, queue second response
                self.send_ack_and_stat();
                self.status.motor_on = true;
                self.state = CDState::Idle;
                self.status.reading = false;
                self.status.seeking = false;
                self.status.playing = false;
                self.queue_second_response(SecondResponseType::Init, timing);
            }
            0x0E => {
                // SetMode: Parse mode parameter
                self.send_ack_and_stat();
                if let Some(mode_byte) = self.param_fifo.pop_front() {
                    self.mode.cdda_report = (mode_byte & 0x01) != 0;
                    self.mode.auto_pause = (mode_byte & 0x02) != 0;
                    self.mode.report_all = (mode_byte & 0x04) != 0;
                    self.mode.xa_filter = (mode_byte & 0x08) != 0;
                    self.mode.ignore_bit = (mode_byte & 0x10) != 0;
                    self.mode.size_2340 = (mode_byte & 0x20) != 0;
                    self.mode.xa_adpcm = (mode_byte & 0x40) != 0;
                    self.mode.double_speed = (mode_byte & 0x80) != 0;
                    log::debug!("CD-ROM: SetMode = 0x{:02X}", mode_byte);
                }
            }
            0x15 => {
                // SeekL: Start seeking, queue second response
                self.send_ack_and_stat();
                if self.seek_target.is_some() {
                    self.state = CDState::Seeking;
                    self.status.seeking = true;
                    self.seek_ticks = 0;
                    self.queue_second_response(SecondResponseType::Seek, timing);
                } else {
                    log::warn!("CD-ROM: SeekL with no target set");
                    self.error_response();
                }
            }
            0x19 => {
                // Test: Handle test sub-functions
                if let Some(subfunction) = self.param_fifo.pop_front() {
                    match subfunction {
                        0x20 => {
                            // Get BIOS date
                            self.response_fifo.push_back(0x98); // Year
                            self.response_fifo.push_back(0x08); // Month
                            self.response_fifo.push_back(0x07); // Day
                            self.response_fifo.push_back(0xC3); // Version
                            self.trigger_interrupt(3); // INT3
                        }
                        0x04 => {
                            // Get chip ID
                            self.response_fifo.push_back(self.get_status_byte());
                            self.response_fifo.push_back(0x00);
                            self.response_fifo.push_back(0x00);
                            self.response_fifo.push_back(0x00);
                            self.response_fifo.push_back(0x00);
                            self.trigger_interrupt(3); // INT3
                        }
                        _ => {
                            self.send_ack_and_stat();
                        }
                    }
                } else {
                    self.send_ack_and_stat();
                }
            }
            0x1A => {
                // GetID: Queue second response with disc info
                self.send_ack_and_stat();
                self.queue_second_response(SecondResponseType::GetID, timing);
            }
            0x1E => {
                // ReadTOC: Queue second response
                if self.disc.is_some() {
                    self.send_ack_and_stat();
                    self.queue_second_response(SecondResponseType::ReadTOC, timing);
                } else {
                    self.status.id_error = true;
                    self.error_response();
                }
            }
            _ => {
                log::warn!("CD-ROM: Unknown command 0x{:02X}", cmd);
                self.error_response();
            }
        }
    }

    /// Execute GetID second response
    ///
    /// Populates response FIFO with disc identification information.
    fn do_getid_read(&mut self) {
        if self.disc.is_some() {
            self.async_response_fifo.push_back(self.get_status_byte());
            self.async_response_fifo.push_back(0x00); // Licensed
            self.async_response_fifo.push_back(0x20); // Audio+CDROM
            self.async_response_fifo.push_back(0x00); // SCEx string (unused)
            self.async_response_fifo.push_back(b'S'); // SCEx region
            self.async_response_fifo.push_back(b'C');
            self.async_response_fifo.push_back(b'E');
            self.async_response_fifo.push_back(b'A'); // SCEA
        } else {
            self.status.id_error = true;
            self.async_response_fifo
                .push_back(self.get_status_byte() | 0x01);
            self.async_response_fifo.push_back(0x80); // Error code
        }
    }

    /// Execute ReadTOC second response
    ///
    /// Completes TOC reading operation.
    fn do_toc_read(&mut self) {
        self.async_response_fifo.push_back(self.get_status_byte());

        // Log TOC information for debugging
        if let Some(ref disc) = self.disc {
            let track_count = disc.track_count();
            log::debug!("CD-ROM: ReadTOC complete - {} tracks", track_count);
        }
    }

    /// Execute Init second response
    ///
    /// Completes drive initialization.
    fn do_init_complete(&mut self) {
        self.async_response_fifo.push_back(self.get_status_byte());
    }

    /// Execute Pause second response
    ///
    /// Completes pause operation.
    fn do_pause_complete(&mut self) {
        self.async_response_fifo.push_back(self.get_status_byte());
    }

    /// Execute Seek second response
    ///
    /// Completes seek operation and updates position.
    fn do_seek_complete(&mut self) {
        // Complete seek
        if let Some(target) = self.seek_target {
            self.position = target;
            self.state = CDState::Idle;
            self.status.seeking = false;
            log::debug!(
                "CD-ROM: Seek complete to {:02}:{:02}:{:02}",
                self.position.minute,
                self.position.second,
                self.position.sector
            );
        }
        self.async_response_fifo.push_back(self.get_status_byte());
    }

    /// Execute second response callback
    ///
    /// Delivers the second response for commands that need it.
    /// Schedules async interrupt delivery.
    ///
    /// # Arguments
    ///
    /// * `timing` - Timing event manager
    pub(super) fn execute_second_response_callback(&mut self, timing: &mut TimingEventManager) {
        let Some(response_type) = self.pending_second_response.take() else {
            return;
        };

        log::trace!("CD-ROM: Executing second response {:?}", response_type);

        // Prepare response based on type
        match response_type {
            SecondResponseType::None => (),
            SecondResponseType::GetID => {
                self.do_getid_read();
                let int_level = if self.disc.is_some() { 2 } else { 5 };
                self.schedule_async_interrupt(int_level, timing);
            }
            SecondResponseType::ReadTOC => {
                self.do_toc_read();
                self.schedule_async_interrupt(2, timing); // INT2
            }
            SecondResponseType::Init => {
                self.do_init_complete();
                self.schedule_async_interrupt(2, timing); // INT2
            }
            SecondResponseType::Pause => {
                self.do_pause_complete();
                self.schedule_async_interrupt(2, timing); // INT2
            }
            SecondResponseType::Seek => {
                self.do_seek_complete();
                self.schedule_async_interrupt(2, timing); // INT2
            }
        }
    }

    /// Deliver async interrupt callback
    ///
    /// Delivers a pending async interrupt to the CPU.
    /// Moves data from async_response_fifo to main response_fifo.
    ///
    /// # Arguments
    ///
    /// * `timing` - Timing event manager
    pub(super) fn deliver_async_interrupt_callback(&mut self, timing: &mut TimingEventManager) {
        if self.pending_async_interrupt == 0 {
            return;
        }

        // Move async response to main response FIFO
        while let Some(byte) = self.async_response_fifo.pop_front() {
            self.response_fifo.push_back(byte);
        }

        // Trigger the interrupt
        let interrupt_level = self.pending_async_interrupt;
        self.trigger_interrupt(interrupt_level);
        self.pending_async_interrupt = 0;

        // Update last interrupt time
        self.last_interrupt_time = timing.global_tick_counter as TickCount;

        log::trace!("CD-ROM: Delivered async INT{}", interrupt_level);
    }

    /// Read sector callback (called by sector_read_event)
    ///
    /// Reads one sector and triggers INT1 (data ready).
    pub(super) fn read_sector_callback(&mut self, _timing: &mut TimingEventManager) {
        if self.state != CDState::Reading {
            return;
        }

        // Read sector from disc
        if let Some(data) = self.read_current_sector() {
            self.data_buffer = data;
            self.data_index = 0;

            log::trace!(
                "CD-ROM: Read sector at {:02}:{:02}:{:02}",
                self.position.minute,
                self.position.second,
                self.position.sector
            );

            // Advance to next sector
            self.advance_position();

            // Trigger INT1 (data ready) immediately
            // Note: In real hardware there's a small delay, but we'll deliver immediately
            self.response_fifo.push_back(self.get_status_byte());
            self.trigger_interrupt(1); // INT1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cdrom::{DiscImage, CDROM};

    /// Helper to convert decimal to BCD
    fn dec_to_bcd(decimal: u8) -> u8 {
        ((decimal / 10) << 4) | (decimal % 10)
    }

    #[test]
    fn test_cmd_getstat_returns_status() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_getstat();

        // Should have one response byte
        assert_eq!(cdrom.response_fifo.len(), 1);
        // Should have triggered INT3 (bit 2 set = value 4)
        assert_eq!(cdrom.interrupt_flag(), 4);
    }

    #[test]
    fn test_cmd_setloc_with_valid_parameters() {
        let mut cdrom = CDROM::new();

        // Set parameters: MM=00, SS=02, FF=00 (BCD format)
        cdrom.param_fifo.push_back(dec_to_bcd(0)); // minute
        cdrom.param_fifo.push_back(dec_to_bcd(2)); // second
        cdrom.param_fifo.push_back(dec_to_bcd(0)); // sector

        cdrom.cmd_setloc();

        // Should set seek_target
        assert!(cdrom.seek_target.is_some());
        let target = cdrom.seek_target.unwrap();
        assert_eq!(target.minute, 0);
        assert_eq!(target.second, 2);
        assert_eq!(target.sector, 0);

        // Should have response and interrupt
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4

        // Parameters should be consumed
        assert!(cdrom.param_fifo.is_empty());
    }

    #[test]
    fn test_cmd_setloc_with_insufficient_parameters() {
        let mut cdrom = CDROM::new();

        // Only 2 parameters instead of 3
        cdrom.param_fifo.push_back(0x00);
        cdrom.param_fifo.push_back(0x02);

        cdrom.cmd_setloc();

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_cmd_setloc_with_no_parameters() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_setloc();

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_cmd_setloc_bcd_conversion() {
        let mut cdrom = CDROM::new();

        // Test BCD values: MM=12, SS=34, FF=56
        cdrom.param_fifo.push_back(0x12); // 12 in BCD = 12 decimal
        cdrom.param_fifo.push_back(0x34); // 34 in BCD = 34 decimal
        cdrom.param_fifo.push_back(0x56); // 56 in BCD = 56 decimal

        cdrom.cmd_setloc();

        let target = cdrom.seek_target.unwrap();
        assert_eq!(target.minute, 12);
        assert_eq!(target.second, 34);
        assert_eq!(target.sector, 56);
    }

    #[test]
    fn test_cmd_readn_sets_reading_state() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_readn();

        // Should be in reading state
        assert_eq!(cdrom.state, CDState::Reading);
        assert!(cdrom.status.reading);

        // Should have response and INT3
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_reads_sets_reading_state() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_reads();

        // Should be in reading state
        assert_eq!(cdrom.state, CDState::Reading);
        assert!(cdrom.status.reading);

        // Should have response and INT3
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_pause_stops_reading() {
        let mut cdrom = CDROM::new();

        // Start reading first
        cdrom.state = CDState::Reading;
        cdrom.status.reading = true;

        cdrom.cmd_pause();

        // Should stop reading
        assert_eq!(cdrom.state, CDState::Idle);
        assert!(!cdrom.status.reading);

        // Should have two responses (first and second)
        assert_eq!(cdrom.response_fifo.len(), 2);

        // Should trigger INT3 and INT2
        // Note: The implementation triggers both immediately
    }

    #[test]
    fn test_cmd_pause_stops_audio() {
        let mut cdrom = CDROM::new();

        // Start audio playback
        cdrom.status.playing = true;

        cdrom.cmd_pause();

        // Should stop playback
        assert!(!cdrom.status.playing);
    }

    #[test]
    fn test_cmd_init_resets_state() {
        let mut cdrom = CDROM::new();

        // Set some state
        cdrom.status.reading = true;
        cdrom.status.seeking = true;
        cdrom.status.playing = true;

        cdrom.cmd_init();

        // Should reset state
        assert!(cdrom.status.motor_on);
        assert_eq!(cdrom.state, CDState::Idle);
        assert!(!cdrom.status.reading);
        assert!(!cdrom.status.seeking);
        assert!(!cdrom.status.playing);

        // Should have two responses
        assert_eq!(cdrom.response_fifo.len(), 2);
    }

    #[test]
    fn test_cmd_setmode_with_no_parameters() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_setmode();

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_cmd_setmode_parses_mode_byte() {
        let mut cdrom = CDROM::new();

        // Set mode: bit 7 (double speed) and bit 5 (sector size 2340)
        let mode_byte = 0b1010_0000;
        cdrom.param_fifo.push_back(mode_byte);

        cdrom.cmd_setmode();

        // Check mode flags
        assert!(cdrom.mode.double_speed);
        assert!(cdrom.mode.size_2340);
        assert!(!cdrom.mode.cdda_report);
        assert!(!cdrom.mode.auto_pause);

        // Should have response
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_setmode_all_flags() {
        // Test each individual flag
        let test_cases = vec![
            (0x01, "cdda_report"),
            (0x02, "auto_pause"),
            (0x04, "report_all"),
            (0x08, "xa_filter"),
            (0x10, "ignore_bit"),
            (0x20, "size_2340"),
            (0x40, "xa_adpcm"),
            (0x80, "double_speed"),
        ];

        for (mode_byte, flag_name) in test_cases {
            let mut cdrom = CDROM::new();
            cdrom.param_fifo.push_back(mode_byte);
            cdrom.cmd_setmode();

            // Verify the correct flag was set
            match flag_name {
                "cdda_report" => assert!(cdrom.mode.cdda_report),
                "auto_pause" => assert!(cdrom.mode.auto_pause),
                "report_all" => assert!(cdrom.mode.report_all),
                "xa_filter" => assert!(cdrom.mode.xa_filter),
                "ignore_bit" => assert!(cdrom.mode.ignore_bit),
                "size_2340" => assert!(cdrom.mode.size_2340),
                "xa_adpcm" => assert!(cdrom.mode.xa_adpcm),
                "double_speed" => assert!(cdrom.mode.double_speed),
                _ => panic!("Unknown flag"),
            }
        }
    }

    #[test]
    fn test_cmd_seekl_with_target_set() {
        let mut cdrom = CDROM::new();

        // Set seek target first
        cdrom.seek_target = Some(CDPosition::new(0, 10, 0));

        cdrom.cmd_seekl();

        // Should be in seeking state
        assert_eq!(cdrom.state, CDState::Seeking);
        assert!(cdrom.status.seeking);

        // Should have response and INT3
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_seekl_without_target() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_seekl();

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_cmd_test_subfunction_0x20_bios_version() {
        let mut cdrom = CDROM::new();

        cdrom.param_fifo.push_back(0x20);

        cdrom.cmd_test();

        // Should return 4 bytes: YY, MM, DD, Version
        assert_eq!(cdrom.response_fifo.len(), 4);
        assert_eq!(cdrom.response_fifo[0], 0x98); // Year 98
        assert_eq!(cdrom.response_fifo[1], 0x08); // Month 08
        assert_eq!(cdrom.response_fifo[2], 0x07); // Day 07
        assert_eq!(cdrom.response_fifo[3], 0xC3); // Version

        // Should trigger INT3
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_test_subfunction_0x04_chip_id() {
        let mut cdrom = CDROM::new();

        cdrom.param_fifo.push_back(0x04);

        cdrom.cmd_test();

        // Should return 5 bytes: status + 4 chip ID bytes
        assert_eq!(cdrom.response_fifo.len(), 5);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_test_unknown_subfunction() {
        let mut cdrom = CDROM::new();

        cdrom.param_fifo.push_back(0xFF);

        cdrom.cmd_test();

        // Should return status byte
        assert_eq!(cdrom.response_fifo.len(), 1);
        assert_eq!(cdrom.interrupt_flag(), 4); // INT3 = bit 2 = value 4
    }

    #[test]
    fn test_cmd_test_without_parameters() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_test();

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_cmd_getid_with_disc() {
        let mut cdrom = CDROM::new();

        // Load a dummy disc
        cdrom.disc = Some(DiscImage::new_dummy());

        cdrom.cmd_getid();

        // Should have first response (status byte)
        assert!(!cdrom.response_fifo.is_empty());

        // Clear first response
        cdrom.response_fifo.clear();
        cdrom.interrupt_flag = 0;

        // Note: Second response would be generated by timing system
        // For now we just verify first response worked
    }

    #[test]
    fn test_cmd_getid_without_disc() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_getid();

        // Should have response
        assert!(!cdrom.response_fifo.is_empty());

        // Should set id_error flag
        assert!(cdrom.status.id_error);
    }

    #[test]
    fn test_cmd_readtoc_with_disc() {
        let mut cdrom = CDROM::new();

        // Load a dummy disc
        cdrom.disc = Some(DiscImage::new_dummy());

        cdrom.cmd_readtoc();

        // Should have responses
        assert!(!cdrom.response_fifo.is_empty());

        // Should trigger INT3 and INT2
        // Note: The implementation triggers both immediately
        assert_eq!(cdrom.response_fifo.len(), 2);
    }

    #[test]
    fn test_cmd_readtoc_without_disc() {
        let mut cdrom = CDROM::new();

        cdrom.cmd_readtoc();

        // Should have error response
        assert!(!cdrom.response_fifo.is_empty());

        // Should set id_error flag
        assert!(cdrom.status.id_error);
    }

    #[test]
    fn test_execute_command_unknown_command() {
        let mut cdrom = CDROM::new();

        cdrom.execute_command(0xFF);

        // Should trigger error response
        assert!(!cdrom.response_fifo.is_empty());
    }

    #[test]
    fn test_execute_command_dispatches_correctly() {
        let test_cases = vec![
            (0x01, "GetStat"),
            (0x02, "SetLoc"),
            (0x06, "ReadN"),
            (0x09, "Pause"),
            (0x0A, "Init"),
            (0x0E, "SetMode"),
            (0x15, "SeekL"),
            (0x19, "Test"),
            (0x1A, "GetID"),
            (0x1B, "ReadS"),
            (0x1E, "ReadTOC"),
        ];

        for (cmd_byte, cmd_name) in test_cases {
            let mut cdrom = CDROM::new();

            // Provide dummy disc for commands that need it
            if cmd_byte == 0x1E {
                cdrom.disc = Some(DiscImage::new_dummy());
            }

            // Provide parameters for commands that need them
            if cmd_byte == 0x02 {
                // SetLoc needs 3 parameters
                cdrom.param_fifo.push_back(0x00);
                cdrom.param_fifo.push_back(0x02);
                cdrom.param_fifo.push_back(0x00);
            } else if cmd_byte == 0x0E {
                // SetMode needs 1 parameter
                cdrom.param_fifo.push_back(0x00);
            } else if cmd_byte == 0x19 {
                // Test needs 1 parameter
                cdrom.param_fifo.push_back(0x20);
            }

            cdrom.execute_command(cmd_byte);

            // All commands should produce some response
            assert!(
                !cdrom.response_fifo.is_empty(),
                "Command {} ({}) should produce response",
                cmd_name,
                cmd_byte
            );
        }
    }

    #[test]
    fn test_multiple_commands_preserve_order() {
        let mut cdrom = CDROM::new();

        // Execute GetStat
        cdrom.cmd_getstat();
        let first_response_len = cdrom.response_fifo.len();

        // Clear responses
        cdrom.response_fifo.clear();
        cdrom.interrupt_flag = 0;

        // Execute SetMode
        cdrom.param_fifo.push_back(0x80);
        cdrom.cmd_setmode();
        let second_response_len = cdrom.response_fifo.len();

        // Both should have produced responses
        assert_eq!(first_response_len, 1);
        assert_eq!(second_response_len, 1);
    }

    #[test]
    fn test_status_byte_reflects_state() {
        let mut cdrom = CDROM::new();

        // Initial state (motor off at start)
        let status = cdrom.get_status_byte();
        // Motor is initially off
        assert_eq!(status & 0x02, 0x00);

        // Start reading
        cdrom.status.reading = true;
        let status = cdrom.get_status_byte();
        assert_eq!(status & 0x20, 0x20); // Read bit should be set

        // Start seeking
        cdrom.status.reading = false;
        cdrom.status.seeking = true;
        let status = cdrom.get_status_byte();
        assert_eq!(status & 0x40, 0x40); // Seek bit should be set
    }

    #[test]
    fn test_bcd_to_dec_conversion() {
        assert_eq!(bcd_to_dec(0x00), 0);
        assert_eq!(bcd_to_dec(0x09), 9);
        assert_eq!(bcd_to_dec(0x10), 10);
        assert_eq!(bcd_to_dec(0x19), 19);
        assert_eq!(bcd_to_dec(0x99), 99);
        assert_eq!(bcd_to_dec(0x45), 45);
    }

    #[test]
    fn test_command_consumes_parameters() {
        let mut cdrom = CDROM::new();

        // Add parameters
        cdrom.param_fifo.push_back(0x00);
        cdrom.param_fifo.push_back(0x02);
        cdrom.param_fifo.push_back(0x00);

        assert_eq!(cdrom.param_fifo.len(), 3);

        cdrom.cmd_setloc();

        // Parameters should be consumed
        assert_eq!(cdrom.param_fifo.len(), 0);
    }

    #[test]
    fn test_error_response_on_invalid_state() {
        let mut cdrom = CDROM::new();

        // Try to seek without setting target first
        cdrom.cmd_seekl();

        // Should generate error
        assert!(!cdrom.response_fifo.is_empty());
    }
}
