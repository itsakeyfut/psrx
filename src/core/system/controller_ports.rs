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

//! PlayStation Controller Port Registers
//!
//! This module manages the memory-mapped I/O registers for controller communication.

use super::super::controller::Controller;

/// PlayStation Controller Port Registers
///
/// Manages the memory-mapped I/O registers for controller communication.
///
/// # Register Map
/// - 0x1F801040: JOY_TX_DATA / JOY_RX_DATA (read/write)
/// - 0x1F801044: JOY_STAT (Status register)
/// - 0x1F801048: JOY_MODE (Mode register)
/// - 0x1F80104A: JOY_CTRL (Control register)
/// - 0x1F80104E: JOY_BAUD (Baud rate)
///
/// # Protocol
/// The controller uses a synchronous serial protocol:
/// 1. Write to JOY_CTRL to select controller
/// 2. Write bytes to JOY_TX_DATA
/// 3. Read responses from JOY_RX_DATA
/// 4. Write to JOY_CTRL to deselect controller
pub struct ControllerPorts {
    /// JOY_TX_DATA (0x1F801040) - Transmit data
    tx_data: u8,

    /// JOY_RX_DATA (0x1F801040) - Receive data (same register)
    rx_data: u8,

    /// JOY_STAT (0x1F801044) - Status register
    stat: u32,

    /// JOY_MODE (0x1F801048) - Mode register
    mode: u16,

    /// JOY_CTRL (0x1F80104A) - Control register
    ctrl: u16,

    /// JOY_BAUD (0x1F80104E) - Baud rate
    baud: u16,

    /// Connected controllers (port 1 and 2)
    controllers: [Option<Controller>; 2],

    /// Currently selected port (0 or 1)
    selected_port: Option<usize>,
}

impl ControllerPorts {
    /// Create new controller ports with default state
    ///
    /// Initializes with one controller connected to port 1.
    pub fn new() -> Self {
        Self {
            tx_data: 0xFF,
            rx_data: 0xFF,
            stat: 0x05, // TX ready (bit 0), RX ready (bit 2)
            mode: 0x000D,
            ctrl: 0,
            baud: 0,
            controllers: [Some(Controller::new()), None], // Port 1 has controller
            selected_port: None,
        }
    }

    /// Write to TX_DATA register (0x1F801040)
    ///
    /// Transmits a byte to the selected controller and receives a response byte.
    ///
    /// # Arguments
    ///
    /// * `value` - Byte to transmit
    pub fn write_tx_data(&mut self, value: u8) {
        self.tx_data = value;

        // If controller is selected, perform transfer
        if let Some(port) = self.selected_port {
            if let Some(controller) = &mut self.controllers[port] {
                self.rx_data = controller.transfer(value);
            } else {
                self.rx_data = 0xFF; // No controller
            }
        } else {
            self.rx_data = 0xFF;
        }

        // Set RX ready flag (bit 1)
        self.stat |= 0x02;
    }

    /// Read from RX_DATA register (0x1F801040)
    ///
    /// Returns the last received byte from the controller.
    ///
    /// # Returns
    ///
    /// Received byte
    pub fn read_rx_data(&mut self) -> u8 {
        // Clear RX ready flag
        self.stat &= !0x02;
        self.rx_data
    }

    /// Write to CTRL register (0x1F80104A)
    ///
    /// Controls controller selection and interrupt acknowledgment.
    ///
    /// # Arguments
    ///
    /// * `value` - Control register value
    pub fn write_ctrl(&mut self, value: u16) {
        self.ctrl = value;

        // Check for controller select (bit 1)
        if (value & 0x0002) != 0 {
            // Determine which port based on DTR bits
            let port = if (value & 0x2000) != 0 { 1 } else { 0 };
            self.selected_port = Some(port);

            if let Some(controller) = &mut self.controllers[port] {
                controller.select();
            }

            log::trace!("Controller port {} selected", port + 1);
        } else {
            // Deselect
            if let Some(port) = self.selected_port {
                if let Some(controller) = &mut self.controllers[port] {
                    controller.deselect();
                }
                log::trace!("Controller port {} deselected", port + 1);
            }
            self.selected_port = None;
        }

        // Acknowledge interrupt (bit 4)
        if (value & 0x0010) != 0 {
            self.stat &= !0x0200; // Clear IRQ flag
        }
    }

    /// Read STAT register (0x1F801044)
    ///
    /// Returns the controller port status.
    ///
    /// # Returns
    ///
    /// Status register value
    #[inline]
    pub fn read_stat(&self) -> u32 {
        self.stat
    }

    /// Read MODE register (0x1F801048)
    ///
    /// # Returns
    ///
    /// Mode register value
    #[inline]
    pub fn read_mode(&self) -> u16 {
        self.mode
    }

    /// Write MODE register (0x1F801048)
    ///
    /// # Arguments
    ///
    /// * `value` - Mode register value
    #[inline]
    pub fn write_mode(&mut self, value: u16) {
        self.mode = value;
    }

    /// Read CTRL register (0x1F80104A)
    ///
    /// # Returns
    ///
    /// Control register value
    #[inline]
    pub fn read_ctrl(&self) -> u16 {
        self.ctrl
    }

    /// Read BAUD register (0x1F80104E)
    ///
    /// # Returns
    ///
    /// Baud rate register value
    #[inline]
    pub fn read_baud(&self) -> u16 {
        self.baud
    }

    /// Write BAUD register (0x1F80104E)
    ///
    /// # Arguments
    ///
    /// * `value` - Baud rate value
    #[inline]
    pub fn write_baud(&mut self, value: u16) {
        self.baud = value;
    }

    /// Get mutable reference to controller at port (0 or 1)
    ///
    /// # Arguments
    ///
    /// * `port` - Port number (0 = port 1, 1 = port 2)
    ///
    /// # Returns
    ///
    /// Optional mutable reference to controller
    pub fn get_controller_mut(&mut self, port: usize) -> Option<&mut Controller> {
        self.controllers.get_mut(port).and_then(|c| c.as_mut())
    }
}

impl Default for ControllerPorts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_ports_creation() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.tx_data, 0xFF);
        assert_eq!(ports.rx_data, 0xFF);
        assert_eq!(ports.stat, 0x05); // TX ready | RX ready
        assert_eq!(ports.mode, 0x000D);
        assert_eq!(ports.ctrl, 0);
        assert_eq!(ports.baud, 0);
        assert!(
            ports.controllers[0].is_some(),
            "Port 1 should have controller"
        );
        assert!(ports.controllers[1].is_none(), "Port 2 should be empty");
        assert!(ports.selected_port.is_none(), "No port should be selected");
    }

    #[test]
    fn test_controller_ports_default() {
        let ports1 = ControllerPorts::new();
        let ports2 = ControllerPorts::default();

        assert_eq!(ports1.stat, ports2.stat);
        assert_eq!(ports1.mode, ports2.mode);
        assert_eq!(ports1.ctrl, ports2.ctrl);
    }

    #[test]
    fn test_read_stat_register() {
        let ports = ControllerPorts::new();
        let stat = ports.read_stat();
        assert_eq!(stat, 0x05);

        // Verify TX ready (bit 0) is set
        assert_eq!(stat & 0x01, 0x01, "TX ready flag should be set");
        // Verify RX ready (bit 2) is set
        assert_eq!(stat & 0x04, 0x04, "RX ready flag should be set");
    }

    #[test]
    fn test_read_mode_register() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.read_mode(), 0x000D);
    }

    #[test]
    fn test_write_mode_register() {
        let mut ports = ControllerPorts::new();
        ports.write_mode(0x1234);
        assert_eq!(ports.read_mode(), 0x1234);
    }

    #[test]
    fn test_read_ctrl_register() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.read_ctrl(), 0);
    }

    #[test]
    fn test_read_baud_register() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.read_baud(), 0);
    }

    #[test]
    fn test_write_baud_register() {
        let mut ports = ControllerPorts::new();
        ports.write_baud(0x5678);
        assert_eq!(ports.read_baud(), 0x5678);
    }

    #[test]
    fn test_write_tx_data_no_controller_selected() {
        let mut ports = ControllerPorts::new();

        ports.write_tx_data(0x42);

        // Should return 0xFF when no controller selected
        assert_eq!(ports.rx_data, 0xFF);
        // RX ready flag (bit 1) should be set
        assert_eq!(ports.stat & 0x02, 0x02);
    }

    #[test]
    fn test_write_tx_data_with_controller_selected() {
        let mut ports = ControllerPorts::new();

        // Select port 1 (bit 1 set, bit 13 clear)
        ports.write_ctrl(0x0002);
        assert_eq!(ports.selected_port, Some(0));

        // Write data to controller
        ports.write_tx_data(0x01);

        // Should receive response from controller (not 0xFF)
        assert_eq!(ports.rx_data, 0xFF); // Controller initial state returns 0xFF
                                         // RX ready flag should be set
        assert_eq!(ports.stat & 0x02, 0x02);
    }

    #[test]
    fn test_write_tx_data_port_2_no_controller() {
        let mut ports = ControllerPorts::new();

        // Select port 2 (bit 1 set, bit 13 set)
        ports.write_ctrl(0x2002);
        assert_eq!(ports.selected_port, Some(1));

        // Write data when no controller in port 2
        ports.write_tx_data(0x42);

        // Should return 0xFF (no controller)
        assert_eq!(ports.rx_data, 0xFF);
    }

    #[test]
    fn test_read_rx_data_clears_flag() {
        let mut ports = ControllerPorts::new();

        // Write to set RX ready flag
        ports.write_tx_data(0x42);
        assert_eq!(ports.stat & 0x02, 0x02, "RX ready should be set");

        // Read RX data
        let data = ports.read_rx_data();
        assert_eq!(data, 0xFF);

        // RX ready flag should be cleared
        assert_eq!(
            ports.stat & 0x02,
            0,
            "RX ready should be cleared after read"
        );
    }

    #[test]
    fn test_controller_select_port_1() {
        let mut ports = ControllerPorts::new();

        // Select port 1 (bit 1 set, bit 13 clear)
        ports.write_ctrl(0x0002);

        assert_eq!(ports.selected_port, Some(0), "Port 1 should be selected");
        assert_eq!(ports.ctrl, 0x0002);
    }

    #[test]
    fn test_controller_select_port_2() {
        let mut ports = ControllerPorts::new();

        // Select port 2 (bit 1 set, bit 13 set)
        ports.write_ctrl(0x2002);

        assert_eq!(ports.selected_port, Some(1), "Port 2 should be selected");
        assert_eq!(ports.ctrl, 0x2002);
    }

    #[test]
    fn test_controller_deselect() {
        let mut ports = ControllerPorts::new();

        // Select port 1
        ports.write_ctrl(0x0002);
        assert_eq!(ports.selected_port, Some(0));

        // Deselect (bit 1 clear)
        ports.write_ctrl(0x0000);

        assert!(ports.selected_port.is_none(), "No port should be selected");
    }

    #[test]
    fn test_acknowledge_interrupt() {
        let mut ports = ControllerPorts::new();

        // Set IRQ flag manually
        ports.stat |= 0x0200;
        assert_eq!(ports.stat & 0x0200, 0x0200, "IRQ flag should be set");

        // Write ACK bit (bit 4)
        ports.write_ctrl(0x0010);

        // IRQ flag should be cleared
        assert_eq!(ports.stat & 0x0200, 0, "IRQ flag should be cleared");
    }

    #[test]
    fn test_multiple_transfers() {
        let mut ports = ControllerPorts::new();

        // Select port 1
        ports.write_ctrl(0x0002);

        // Transfer multiple bytes
        ports.write_tx_data(0x01);
        let _rx1 = ports.read_rx_data();

        ports.write_tx_data(0x42);
        let _rx2 = ports.read_rx_data();

        ports.write_tx_data(0x00);
        let _rx3 = ports.read_rx_data();

        // Verify port still selected
        assert_eq!(ports.selected_port, Some(0));
    }

    #[test]
    fn test_get_controller_mut_port_1() {
        let mut ports = ControllerPorts::new();

        let controller = ports.get_controller_mut(0);
        assert!(controller.is_some(), "Port 1 should have controller");
    }

    #[test]
    fn test_get_controller_mut_port_2() {
        let mut ports = ControllerPorts::new();

        let controller = ports.get_controller_mut(1);
        assert!(controller.is_none(), "Port 2 should not have controller");
    }

    #[test]
    fn test_get_controller_mut_invalid_port() {
        let mut ports = ControllerPorts::new();

        let controller = ports.get_controller_mut(2);
        assert!(controller.is_none(), "Invalid port should return None");
    }

    #[test]
    fn test_stat_register_persistence() {
        let mut ports = ControllerPorts::new();

        // Write data to set RX ready
        ports.write_tx_data(0x42);
        let stat1 = ports.read_stat();

        // Read RX data to clear flag
        let _rx = ports.read_rx_data();
        let stat2 = ports.read_stat();

        // Verify flag was cleared
        assert_ne!(stat1 & 0x02, stat2 & 0x02, "RX ready flag should change");
    }

    #[test]
    fn test_ctrl_register_select_bit_combinations() {
        let mut ports = ControllerPorts::new();

        // Test various bit combinations
        let test_values = [
            (0x0000, None),    // No select
            (0x0002, Some(0)), // Port 1
            (0x2002, Some(1)), // Port 2
            (0x0003, Some(0)), // Port 1 with extra bits
            (0x2003, Some(1)), // Port 2 with extra bits
        ];

        for (ctrl_value, expected_port) in &test_values {
            ports.write_ctrl(*ctrl_value);
            assert_eq!(
                ports.selected_port, *expected_port,
                "CTRL value 0x{:04X} should select port {:?}",
                ctrl_value, expected_port
            );
        }
    }

    #[test]
    fn test_rx_data_initial_value() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.rx_data, 0xFF, "RX data should initialize to 0xFF");
    }

    #[test]
    fn test_tx_data_initial_value() {
        let ports = ControllerPorts::new();
        assert_eq!(ports.tx_data, 0xFF, "TX data should initialize to 0xFF");
    }

    #[test]
    fn test_controller_transfer_sequence() {
        let mut ports = ControllerPorts::new();

        // Standard controller query sequence
        ports.write_ctrl(0x0002); // Select port 1

        // Send ID command
        ports.write_tx_data(0x01);
        let _response = ports.read_rx_data();

        // Send button request
        ports.write_tx_data(0x42);
        let _response = ports.read_rx_data();

        // Deselect
        ports.write_ctrl(0x0000);

        assert!(ports.selected_port.is_none());
    }

    #[test]
    fn test_baud_rate_independence() {
        let mut ports = ControllerPorts::new();

        // Baud rate should not affect other operations
        ports.write_baud(0xFFFF);

        ports.write_ctrl(0x0002);
        assert_eq!(ports.selected_port, Some(0));

        ports.write_tx_data(0x42);
        let _rx = ports.read_rx_data();

        // Baud rate should remain unchanged
        assert_eq!(ports.read_baud(), 0xFFFF);
    }

    #[test]
    fn test_mode_register_independence() {
        let mut ports = ControllerPorts::new();

        // Mode should not affect controller selection
        ports.write_mode(0xABCD);
        ports.write_ctrl(0x0002);

        assert_eq!(ports.selected_port, Some(0));
        assert_eq!(ports.read_mode(), 0xABCD);
    }
}
