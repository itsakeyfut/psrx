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

//! Disc image loading and management
//!
//! This module handles loading CD-ROM disc images from .cue/.bin files
//! and provides sector reading functionality.

use super::CDPosition;
use crate::core::error::CdRomError;

/// Disc image loaded from .bin/.cue files
///
/// Represents a CD-ROM disc image with tracks and raw sector data.
/// Supports reading sectors in MSF format.
///
/// # Example
///
/// ```no_run
/// use psrx::core::cdrom::DiscImage;
///
/// let disc = DiscImage::load("game.cue").unwrap();
/// let position = psrx::core::cdrom::CDPosition::new(0, 2, 0);
/// let sector_data = disc.read_sector(&position);
/// ```
#[derive(Debug)]
pub struct DiscImage {
    /// Tracks on the disc
    tracks: Vec<Track>,

    /// Raw sector data from .bin file
    data: Vec<u8>,
}

/// CD-ROM track information
///
/// Represents a single track on a CD-ROM disc, including its type,
/// position, and location in the .bin file.
#[derive(Debug, Clone)]
pub struct Track {
    /// Track number (1-99)
    pub number: u8,

    /// Track type (Mode1/2352, Mode2/2352, Audio)
    pub track_type: TrackType,

    /// Start position (MSF)
    pub start_position: CDPosition,

    /// Length in sectors
    pub length_sectors: u32,

    /// Byte offset in .bin file
    pub file_offset: u64,
}

/// CD-ROM track type
///
/// Specifies the format of data stored in a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    /// Data track, 2352 bytes per sector (Mode 1)
    Mode1_2352,
    /// XA track, 2352 bytes per sector (Mode 2)
    Mode2_2352,
    /// CD-DA audio, 2352 bytes per sector
    Audio,
}

impl DiscImage {
    /// Load a disc image from a .cue file
    ///
    /// Parses the .cue file to extract track information and loads
    /// the corresponding .bin file containing raw sector data.
    ///
    /// # Arguments
    ///
    /// * `cue_path` - Path to the .cue file
    ///
    /// # Returns
    ///
    /// - `Ok(DiscImage)` if loading succeeded
    /// - `Err(CdRomError)` if loading failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use psrx::core::cdrom::DiscImage;
    ///
    /// let disc = DiscImage::load("game.cue").unwrap();
    /// ```
    pub fn load(cue_path: &str) -> Result<Self, CdRomError> {
        let cue_data = std::fs::read_to_string(cue_path)?;
        let bin_path = Self::get_bin_path_from_cue(cue_path, &cue_data)?;

        let mut tracks = Self::parse_cue(&cue_data)?;
        let data = std::fs::read(&bin_path).map_err(|e| {
            CdRomError::DiscLoadError(format!("Failed to read bin file '{}': {}", bin_path, e))
        })?;

        // Calculate track lengths based on file size and positions
        Self::calculate_track_lengths(&mut tracks, data.len());

        log::info!(
            "Loaded disc image: {} tracks, {} MB",
            tracks.len(),
            data.len() / 1024 / 1024
        );

        Ok(Self { tracks, data })
    }

    /// Extract .bin file path from .cue file path and content
    ///
    /// Searches for FILE directive in .cue content to determine .bin filename.
    ///
    /// # Arguments
    ///
    /// * `cue_path` - Path to the .cue file
    /// * `cue_data` - Content of the .cue file
    ///
    /// # Returns
    ///
    /// Full path to the .bin file
    fn get_bin_path_from_cue(cue_path: &str, cue_data: &str) -> Result<String, CdRomError> {
        // Find FILE directive
        for line in cue_data.lines() {
            let line = line.trim();
            if line.starts_with("FILE") {
                // Extract filename from quotes
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let bin_filename = &line[start + 1..start + 1 + end];

                        // Construct full path by replacing .cue filename with .bin filename
                        let cue_path_obj = std::path::Path::new(cue_path);
                        let bin_path = if let Some(parent) = cue_path_obj.parent() {
                            parent.join(bin_filename)
                        } else {
                            std::path::PathBuf::from(bin_filename)
                        };

                        return Ok(bin_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Err(CdRomError::DiscLoadError(
            "No FILE directive found in .cue file".to_string(),
        ))
    }

    /// Parse .cue file content to extract track information
    ///
    /// # Arguments
    ///
    /// * `cue_data` - Content of the .cue file
    ///
    /// # Returns
    ///
    /// Vector of tracks parsed from the .cue file
    pub(super) fn parse_cue(cue_data: &str) -> Result<Vec<Track>, CdRomError> {
        let mut tracks = Vec::new();
        let mut current_track: Option<Track> = None;

        for line in cue_data.lines() {
            let line = line.trim();

            if line.starts_with("TRACK") {
                // Save previous track
                if let Some(track) = current_track.take() {
                    tracks.push(track);
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                let track_num = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                let track_type_str = parts.get(2).unwrap_or(&"MODE2/2352");

                current_track = Some(Track {
                    number: track_num,
                    track_type: Self::parse_track_type(track_type_str),
                    start_position: CDPosition::new(0, 0, 0),
                    length_sectors: 0,
                    file_offset: 0,
                });
            } else if line.starts_with("INDEX 01") {
                if let Some(ref mut track) = current_track {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(time_str) = parts.get(2) {
                        track.start_position = Self::parse_msf(time_str)?;
                        // Calculate file offset from MSF position
                        track.file_offset =
                            Self::msf_to_sector(&track.start_position) as u64 * 2352;
                    }
                }
            }
        }

        // Save last track
        if let Some(track) = current_track {
            tracks.push(track);
        }

        Ok(tracks)
    }

    /// Parse MSF time string (MM:SS:FF)
    ///
    /// # Arguments
    ///
    /// * `msf` - MSF string in format "MM:SS:FF"
    ///
    /// # Returns
    ///
    /// CDPosition parsed from the string
    pub(super) fn parse_msf(msf: &str) -> Result<CDPosition, CdRomError> {
        let parts: Vec<&str> = msf.split(':').collect();
        if parts.len() != 3 {
            return Err(CdRomError::DiscLoadError(format!(
                "Invalid MSF format: '{}'",
                msf
            )));
        }

        let minute = parts[0]
            .parse()
            .map_err(|_| CdRomError::DiscLoadError(format!("Invalid minute in MSF: '{}'", msf)))?;
        let second = parts[1]
            .parse()
            .map_err(|_| CdRomError::DiscLoadError(format!("Invalid second in MSF: '{}'", msf)))?;
        let sector = parts[2]
            .parse()
            .map_err(|_| CdRomError::DiscLoadError(format!("Invalid sector in MSF: '{}'", msf)))?;

        Ok(CDPosition {
            minute,
            second,
            sector,
        })
    }

    /// Parse track type string from .cue file
    ///
    /// # Arguments
    ///
    /// * `s` - Track type string (e.g., "MODE1/2352", "AUDIO")
    ///
    /// # Returns
    ///
    /// Corresponding TrackType enum value
    pub(super) fn parse_track_type(s: &str) -> TrackType {
        match s {
            "MODE1/2352" => TrackType::Mode1_2352,
            "MODE2/2352" => TrackType::Mode2_2352,
            "AUDIO" => TrackType::Audio,
            _ => TrackType::Mode2_2352, // Default to Mode2
        }
    }

    /// Calculate track lengths based on file size and start positions
    ///
    /// # Arguments
    ///
    /// * `tracks` - Mutable vector of tracks to update
    /// * `file_size` - Total size of the .bin file in bytes
    pub(super) fn calculate_track_lengths(tracks: &mut [Track], file_size: usize) {
        for i in 0..tracks.len() {
            if i + 1 < tracks.len() {
                // Calculate length as difference between this track and next track
                let next_offset = tracks[i + 1].file_offset;
                let this_offset = tracks[i].file_offset;
                tracks[i].length_sectors = ((next_offset - this_offset) / 2352) as u32;
            } else {
                // Last track: calculate from remaining file size
                let this_offset = tracks[i].file_offset;
                tracks[i].length_sectors = ((file_size as u64 - this_offset) / 2352) as u32;
            }
        }
    }

    /// Read a sector from the disc at the specified MSF position
    ///
    /// # Arguments
    ///
    /// * `position` - MSF position to read from
    ///
    /// # Returns
    ///
    /// - `Some(&[u8])` - Sector data (2352 bytes)
    /// - `None` - Position out of bounds
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use psrx::core::cdrom::{DiscImage, CDPosition};
    /// # let disc = DiscImage::load("game.cue").unwrap();
    /// let position = CDPosition::new(0, 2, 0);
    /// if let Some(data) = disc.read_sector(&position) {
    ///     println!("Read {} bytes", data.len());
    /// }
    /// ```
    pub fn read_sector(&self, position: &CDPosition) -> Option<&[u8]> {
        let sector_num = Self::msf_to_sector(position);
        let offset = sector_num * 2352;

        if offset + 2352 <= self.data.len() {
            Some(&self.data[offset..offset + 2352])
        } else {
            None
        }
    }

    /// Convert MSF position to sector number
    ///
    /// # Arguments
    ///
    /// * `pos` - MSF position
    ///
    /// # Returns
    ///
    /// Sector number (0-based, accounting for 2-second pregap)
    pub(super) fn msf_to_sector(pos: &CDPosition) -> usize {
        let total = (pos.minute as u32 * 60 * 75) + (pos.second as u32 * 75) + pos.sector as u32;
        total.saturating_sub(150) as usize
    }

    /// Get the number of tracks on the disc
    ///
    /// # Returns
    ///
    /// Number of tracks
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Get track information by track number
    ///
    /// # Arguments
    ///
    /// * `track_num` - Track number (1-99)
    ///
    /// # Returns
    ///
    /// Optional reference to track information
    pub fn get_track(&self, track_num: u8) -> Option<&Track> {
        self.tracks.iter().find(|t| t.number == track_num)
    }

    /// Create a dummy disc image for testing
    ///
    /// Creates a minimal valid disc image with a single data track.
    /// Used in tests where a disc needs to be present but the actual data
    /// doesn't matter.
    ///
    /// # Returns
    ///
    /// A minimal disc image with one track
    #[cfg(test)]
    pub fn new_dummy() -> Self {
        let track = Track {
            number: 1,
            track_type: TrackType::Mode2_2352,
            start_position: CDPosition::new(0, 2, 0),
            length_sectors: 100,
            file_offset: 0,
        };

        // Create minimal dummy data (100 sectors * 2352 bytes)
        let data = vec![0u8; 100 * 2352];

        Self {
            tracks: vec![track],
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    /// Helper to create a test .cue file
    fn create_test_cue_file(content: &str) -> (NamedTempFile, TempDir) {
        let dir = TempDir::new().unwrap();
        let cue_file = NamedTempFile::new_in(dir.path()).unwrap();
        std::fs::write(cue_file.path(), content).unwrap();
        (cue_file, dir)
    }

    /// Helper to create a test .bin file with specific size
    fn create_test_bin_file(dir: &TempDir, filename: &str, sectors: usize) -> std::path::PathBuf {
        let bin_path = dir.path().join(filename);
        let data = vec![0u8; sectors * 2352];
        std::fs::write(&bin_path, data).unwrap();
        bin_path
    }

    #[test]
    fn test_parse_msf_valid() {
        let result = DiscImage::parse_msf("00:02:00");
        assert!(result.is_ok());

        let pos = result.unwrap();
        assert_eq!(pos.minute, 0);
        assert_eq!(pos.second, 2);
        assert_eq!(pos.sector, 0);
    }

    #[test]
    fn test_parse_msf_with_large_values() {
        let result = DiscImage::parse_msf("99:59:74");
        assert!(result.is_ok());

        let pos = result.unwrap();
        assert_eq!(pos.minute, 99);
        assert_eq!(pos.second, 59);
        assert_eq!(pos.sector, 74);
    }

    #[test]
    fn test_parse_msf_invalid_format() {
        // Missing component
        assert!(DiscImage::parse_msf("00:02").is_err());

        // Too many components
        assert!(DiscImage::parse_msf("00:02:00:00").is_err());

        // Invalid separator
        assert!(DiscImage::parse_msf("00-02-00").is_err());
    }

    #[test]
    fn test_parse_msf_invalid_numbers() {
        // Non-numeric values
        assert!(DiscImage::parse_msf("AA:BB:CC").is_err());

        // Empty string
        assert!(DiscImage::parse_msf("").is_err());
    }

    #[test]
    fn test_parse_track_type() {
        assert_eq!(
            DiscImage::parse_track_type("MODE1/2352"),
            TrackType::Mode1_2352
        );
        assert_eq!(
            DiscImage::parse_track_type("MODE2/2352"),
            TrackType::Mode2_2352
        );
        assert_eq!(DiscImage::parse_track_type("AUDIO"), TrackType::Audio);

        // Unknown types default to Mode2
        assert_eq!(
            DiscImage::parse_track_type("UNKNOWN"),
            TrackType::Mode2_2352
        );
    }

    #[test]
    fn test_parse_cue_single_track() {
        let cue_content = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;

        let result = DiscImage::parse_cue(cue_content);
        assert!(result.is_ok());

        let tracks = result.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].number, 1);
        assert_eq!(tracks[0].track_type, TrackType::Mode2_2352);
        assert_eq!(tracks[0].start_position.minute, 0);
        assert_eq!(tracks[0].start_position.second, 0);
        assert_eq!(tracks[0].start_position.sector, 0);
    }

    #[test]
    fn test_parse_cue_multiple_tracks() {
        let cue_content = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 01 10:30:15
  TRACK 03 AUDIO
    INDEX 01 15:45:30
"#;

        let result = DiscImage::parse_cue(cue_content);
        assert!(result.is_ok());

        let tracks = result.unwrap();
        assert_eq!(tracks.len(), 3);

        // Track 1
        assert_eq!(tracks[0].number, 1);
        assert_eq!(tracks[0].track_type, TrackType::Mode2_2352);

        // Track 2
        assert_eq!(tracks[1].number, 2);
        assert_eq!(tracks[1].track_type, TrackType::Audio);
        assert_eq!(tracks[1].start_position.minute, 10);
        assert_eq!(tracks[1].start_position.second, 30);
        assert_eq!(tracks[1].start_position.sector, 15);

        // Track 3
        assert_eq!(tracks[2].number, 3);
        assert_eq!(tracks[2].track_type, TrackType::Audio);
    }

    #[test]
    fn test_parse_cue_empty() {
        let cue_content = "";

        let result = DiscImage::parse_cue(cue_content);
        assert!(result.is_ok());

        let tracks = result.unwrap();
        assert_eq!(tracks.len(), 0);
    }

    #[test]
    fn test_parse_cue_without_index() {
        let cue_content = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
"#;

        let result = DiscImage::parse_cue(cue_content);
        assert!(result.is_ok());

        let tracks = result.unwrap();
        assert_eq!(tracks.len(), 1);
        // Without INDEX 01, start position remains at 0:0:0
        assert_eq!(tracks[0].start_position.minute, 0);
        assert_eq!(tracks[0].start_position.second, 0);
        assert_eq!(tracks[0].start_position.sector, 0);
    }

    #[test]
    fn test_calculate_track_lengths_single_track() {
        let mut tracks = vec![Track {
            number: 1,
            track_type: TrackType::Mode2_2352,
            start_position: CDPosition::new(0, 0, 0),
            length_sectors: 0,
            file_offset: 0,
        }];

        let file_size = 100 * 2352; // 100 sectors
        DiscImage::calculate_track_lengths(&mut tracks, file_size);

        assert_eq!(tracks[0].length_sectors, 100);
    }

    #[test]
    fn test_calculate_track_lengths_multiple_tracks() {
        // Track 2 starts at 00:10:00 = 10*75 - 150 = 600 sectors
        let track2_sector = (10 * 75) - 150;
        let mut tracks = vec![
            Track {
                number: 1,
                track_type: TrackType::Mode2_2352,
                start_position: CDPosition::new(0, 0, 0),
                length_sectors: 0,
                file_offset: 0,
            },
            Track {
                number: 2,
                track_type: TrackType::Audio,
                start_position: CDPosition::new(0, 10, 0),
                length_sectors: 0,
                file_offset: track2_sector as u64 * 2352,
            },
        ];

        let file_size = 1000 * 2352; // Total 1000 sectors (enough for both tracks)
        DiscImage::calculate_track_lengths(&mut tracks, file_size);

        // First track length
        assert_eq!(tracks[0].length_sectors, track2_sector as u32);

        // Second track length
        let expected_second_length = 1000 - track2_sector;
        assert_eq!(tracks[1].length_sectors, expected_second_length as u32);
    }

    #[test]
    fn test_msf_to_sector() {
        // Test pregap handling (first 150 sectors)
        let pos = CDPosition::new(0, 0, 0);
        assert_eq!(DiscImage::msf_to_sector(&pos), 0);

        let pos = CDPosition::new(0, 2, 0);
        assert_eq!(DiscImage::msf_to_sector(&pos), 0); // 2 seconds = 150 frames = pregap

        // Test normal sectors
        let pos = CDPosition::new(0, 2, 1);
        assert_eq!(DiscImage::msf_to_sector(&pos), 1);

        let pos = CDPosition::new(0, 3, 0);
        assert_eq!(DiscImage::msf_to_sector(&pos), 75); // 1 second after pregap

        // Test larger values
        let pos = CDPosition::new(1, 0, 0);
        assert_eq!(DiscImage::msf_to_sector(&pos), 60 * 75 - 150); // 1 minute
    }

    #[test]
    fn test_msf_to_sector_saturating() {
        // Test that values below pregap saturate to 0
        let pos = CDPosition::new(0, 0, 50);
        assert_eq!(DiscImage::msf_to_sector(&pos), 0); // Below pregap, saturates to 0
    }

    #[test]
    fn test_read_sector_valid() {
        let disc = DiscImage::new_dummy();
        let pos = CDPosition::new(0, 2, 0); // Sector 0

        let data = disc.read_sector(&pos);
        assert!(data.is_some());
        assert_eq!(data.unwrap().len(), 2352);
    }

    #[test]
    fn test_read_sector_out_of_bounds() {
        let disc = DiscImage::new_dummy(); // 100 sectors
        let pos = CDPosition::new(10, 0, 0); // Way beyond available data

        let data = disc.read_sector(&pos);
        assert!(data.is_none());
    }

    #[test]
    fn test_read_sector_boundary() {
        let disc = DiscImage::new_dummy(); // 100 sectors
        let pos = CDPosition::new(0, 2, 99); // Last sector (sector 99)

        let data = disc.read_sector(&pos);
        assert!(data.is_some());

        // One past the end
        let pos = CDPosition::new(0, 2, 100); // Sector 100
        let data = disc.read_sector(&pos);
        assert!(data.is_none());
    }

    #[test]
    fn test_track_count() {
        let disc = DiscImage::new_dummy();
        assert_eq!(disc.track_count(), 1);
    }

    #[test]
    fn test_get_track_valid() {
        let disc = DiscImage::new_dummy();
        let track = disc.get_track(1);

        assert!(track.is_some());
        assert_eq!(track.unwrap().number, 1);
    }

    #[test]
    fn test_get_track_invalid() {
        let disc = DiscImage::new_dummy();
        let track = disc.get_track(99);

        assert!(track.is_none());
    }

    #[test]
    fn test_load_disc_invalid_cue_path() {
        let result = DiscImage::load("nonexistent.cue");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_disc_missing_file_directive() {
        let cue_content = r#"TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
        let (cue_file, _dir) = create_test_cue_file(cue_content);

        let result = DiscImage::load(cue_file.path().to_str().unwrap());
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                CdRomError::DiscLoadError(msg) => {
                    assert!(msg.contains("No FILE directive"));
                }
                _ => panic!("Expected DiscLoadError"),
            }
        }
    }

    #[test]
    fn test_load_disc_missing_bin_file() {
        let cue_content = r#"FILE "missing.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
        let (cue_file, _dir) = create_test_cue_file(cue_content);

        let result = DiscImage::load(cue_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_disc_valid() {
        let dir = TempDir::new().unwrap();

        // Create .bin file
        let bin_path = create_test_bin_file(&dir, "test.bin", 50);

        // Create .cue file
        let cue_content = format!(
            r#"FILE "{}" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#,
            bin_path.file_name().unwrap().to_str().unwrap()
        );

        let cue_file = NamedTempFile::new_in(dir.path()).unwrap();
        std::fs::write(cue_file.path(), cue_content).unwrap();

        // Load disc
        let result = DiscImage::load(cue_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let disc = result.unwrap();
        assert_eq!(disc.track_count(), 1);
        assert_eq!(disc.data.len(), 50 * 2352);
    }

    #[test]
    fn test_load_disc_multiple_tracks() {
        let dir = TempDir::new().unwrap();

        // Create .bin file with multiple tracks (need enough sectors)
        // 00:00:00 to 00:20:00 = 20*75 - 150 = 1350 sectors minimum
        let bin_path = create_test_bin_file(&dir, "multi.bin", 1500);

        // Create .cue file with multiple tracks
        let cue_content = format!(
            r#"FILE "{}" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 01 00:10:00
  TRACK 03 AUDIO
    INDEX 01 00:20:00
"#,
            bin_path.file_name().unwrap().to_str().unwrap()
        );

        let cue_file = NamedTempFile::new_in(dir.path()).unwrap();
        std::fs::write(cue_file.path(), cue_content).unwrap();

        // Load disc
        let result = DiscImage::load(cue_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let disc = result.unwrap();
        assert_eq!(disc.track_count(), 3);

        // Check track types
        assert_eq!(disc.get_track(1).unwrap().track_type, TrackType::Mode2_2352);
        assert_eq!(disc.get_track(2).unwrap().track_type, TrackType::Audio);
        assert_eq!(disc.get_track(3).unwrap().track_type, TrackType::Audio);
    }

    #[test]
    fn test_track_file_offset_calculation() {
        let cue_content = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 01 00:10:00
"#;

        let tracks = DiscImage::parse_cue(cue_content).unwrap();

        // Track 1 starts at 00:00:00
        assert_eq!(tracks[0].file_offset, 0);

        // Track 2 starts at 00:10:00 (10 seconds = 750 frames, minus 150 pregap = 600 sectors)
        let expected_offset = (10 * 75 - 150) as u64 * 2352;
        assert_eq!(tracks[1].file_offset, expected_offset);
    }

    #[test]
    fn test_read_sector_contains_expected_data() {
        let dir = TempDir::new().unwrap();
        let bin_path = dir.path().join("data.bin");

        // Create .bin with identifiable pattern
        let mut data = Vec::new();
        for sector in 0..10 {
            let mut sector_data = vec![sector as u8; 2352];
            data.append(&mut sector_data);
        }
        std::fs::write(&bin_path, data).unwrap();

        // Create .cue
        let cue_content = format!(
            r#"FILE "{}" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#,
            bin_path.file_name().unwrap().to_str().unwrap()
        );

        let cue_file = NamedTempFile::new_in(dir.path()).unwrap();
        std::fs::write(cue_file.path(), cue_content).unwrap();

        // Load and read
        let disc = DiscImage::load(cue_file.path().to_str().unwrap()).unwrap();

        // Read sector 0
        let pos = CDPosition::new(0, 2, 0);
        let sector_data = disc.read_sector(&pos).unwrap();
        assert_eq!(sector_data[0], 0);
        assert_eq!(sector_data[2351], 0);

        // Read sector 5
        let pos = CDPosition::new(0, 2, 5);
        let sector_data = disc.read_sector(&pos).unwrap();
        assert_eq!(sector_data[0], 5);
        assert_eq!(sector_data[2351], 5);
    }

    #[test]
    fn test_new_dummy() {
        let disc = DiscImage::new_dummy();

        assert_eq!(disc.track_count(), 1);
        assert_eq!(disc.data.len(), 100 * 2352);

        let track = disc.get_track(1).unwrap();
        assert_eq!(track.track_type, TrackType::Mode2_2352);
        assert_eq!(track.length_sectors, 100);
    }

    #[test]
    fn test_edge_case_empty_cue_file() {
        let result = DiscImage::parse_cue("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_edge_case_whitespace_only_cue() {
        let result = DiscImage::parse_cue("   \n\n   \t\t   ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_track_type_case_sensitivity() {
        assert_eq!(DiscImage::parse_track_type("audio"), TrackType::Mode2_2352); // Lowercase not recognized
        assert_eq!(DiscImage::parse_track_type("AUDIO"), TrackType::Audio); // Uppercase works
    }
}
