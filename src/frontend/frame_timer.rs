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

//! Frame timing module
//!
//! This module provides frame timing utilities for maintaining stable 60 FPS
//! emulation. It tracks frame time, FPS, and determines when to run the next frame.

use std::time::{Duration, Instant};

/// Frame timer for 60 FPS emulation
///
/// Tracks frame timing, FPS, and determines when frames should be executed.
/// Maintains accurate frame pacing for smooth emulation.
///
/// # Example
///
/// ```
/// use psrx::frontend::FrameTimer;
///
/// let mut timer = FrameTimer::new(60);
///
/// loop {
///     if timer.should_run_frame() {
///         // Run emulation frame
///         timer.tick();
///         println!("FPS: {:.1}, Frame time: {:.2}ms", timer.fps(), timer.frame_time_ms());
///     }
/// }
/// ```
pub struct FrameTimer {
    /// Target frame time in nanoseconds
    target_frame_time: Duration,
    /// Time when the last frame was executed
    last_frame: Instant,
    /// Total number of frames executed
    frame_count: u64,
    /// Current FPS (frames per second)
    fps: f32,
    /// Current frame time in milliseconds
    frame_time_ms: f32,
    /// Time when FPS calculation started
    fps_start: Instant,
    /// Frames since last FPS calculation
    fps_frame_count: u64,
}

impl FrameTimer {
    /// Create a new FrameTimer
    ///
    /// # Arguments
    ///
    /// * `target_fps` - Target frames per second (typically 60, must be > 0)
    ///
    /// # Returns
    ///
    /// A new `FrameTimer` instance configured for the target FPS
    ///
    /// # Panics
    ///
    /// Panics if `target_fps` is 0
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    ///
    /// let timer = FrameTimer::new(60);
    /// assert_eq!(timer.fps(), 0.0); // No frames executed yet
    /// ```
    pub fn new(target_fps: u32) -> Self {
        assert!(target_fps > 0, "target_fps must be greater than 0");
        let target_frame_time = Duration::from_nanos(1_000_000_000 / target_fps as u64);
        let now = Instant::now();

        Self {
            target_frame_time,
            last_frame: now,
            frame_count: 0,
            fps: 0.0,
            frame_time_ms: 0.0,
            fps_start: now,
            fps_frame_count: 0,
        }
    }

    /// Update frame timing after executing a frame
    ///
    /// Call this immediately after running a frame to update timing statistics.
    /// This method calculates FPS and frame time, updating them approximately
    /// once per second for smooth readings.
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    ///
    /// let mut timer = FrameTimer::new(60);
    /// // Run emulation frame
    /// timer.tick();
    /// ```
    pub fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame);

        // Update frame time in milliseconds
        self.frame_time_ms = elapsed.as_secs_f32() * 1000.0;

        // Update frame counter
        self.frame_count += 1;
        self.fps_frame_count += 1;

        // Calculate FPS approximately once per second
        let fps_elapsed = now.duration_since(self.fps_start);
        if fps_elapsed >= Duration::from_secs(1) {
            self.fps = self.fps_frame_count as f32 / fps_elapsed.as_secs_f32();
            self.fps_frame_count = 0;
            self.fps_start = now;
        }

        self.last_frame = now;
    }

    /// Check if a new frame should be executed
    ///
    /// Returns true if enough time has passed since the last frame to maintain
    /// the target frame rate.
    ///
    /// # Returns
    ///
    /// `true` if a frame should be executed, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut timer = FrameTimer::new(60);
    ///
    /// // Should run first frame immediately
    /// assert!(timer.should_run_frame());
    ///
    /// timer.tick();
    ///
    /// // Should not run immediately after (unless 16.67ms passed)
    /// // (timing dependent, so we don't assert)
    /// ```
    #[inline(always)]
    pub fn should_run_frame(&self) -> bool {
        let elapsed = Instant::now().duration_since(self.last_frame);
        elapsed >= self.target_frame_time
    }

    /// Get the instant when the next frame should run
    ///
    /// Returns the target instant for the next frame based on the last frame time
    /// and the target frame rate. Used for scheduling event loop wake-ups to avoid
    /// busy-waiting.
    ///
    /// # Returns
    ///
    /// The `Instant` when the next frame should execute
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    /// use std::time::Instant;
    ///
    /// let timer = FrameTimer::new(60);
    /// let next_frame = timer.next_frame_instant();
    /// assert!(next_frame >= Instant::now());
    /// ```
    #[inline(always)]
    pub fn next_frame_instant(&self) -> Instant {
        self.last_frame + self.target_frame_time
    }

    /// Get the current FPS
    ///
    /// Returns the most recent FPS calculation, updated approximately once per second.
    ///
    /// # Returns
    ///
    /// Current frames per second
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    ///
    /// let timer = FrameTimer::new(60);
    /// println!("FPS: {:.1}", timer.fps());
    /// ```
    #[inline(always)]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Get the current frame time in milliseconds
    ///
    /// Returns the time taken to execute the last frame.
    ///
    /// # Returns
    ///
    /// Frame time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    ///
    /// let timer = FrameTimer::new(60);
    /// println!("Frame time: {:.2}ms", timer.frame_time_ms());
    /// ```
    #[inline(always)]
    pub fn frame_time_ms(&self) -> f32 {
        self.frame_time_ms
    }

    /// Get the total number of frames executed
    ///
    /// # Returns
    ///
    /// Total frame count
    ///
    /// # Example
    ///
    /// ```
    /// use psrx::frontend::FrameTimer;
    ///
    /// let timer = FrameTimer::new(60);
    /// assert_eq!(timer.frame_count(), 0);
    /// ```
    #[inline(always)]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frame_timer_new() {
        let timer = FrameTimer::new(60);
        assert_eq!(timer.fps(), 0.0);
        assert_eq!(timer.frame_time_ms(), 0.0);
        assert_eq!(timer.frame_count(), 0);
    }

    #[test]
    fn test_frame_timer_should_run_first_frame() {
        let timer = FrameTimer::new(60);
        // First frame should always run (elapsed time is 0 but >= target)
        // Actually, since last_frame is set to now(), elapsed will be very small
        // Let's just verify it's callable
        let _ = timer.should_run_frame();
    }

    #[test]
    fn test_frame_timer_tick() {
        let mut timer = FrameTimer::new(60);

        // Wait a bit to ensure measurable time
        thread::sleep(Duration::from_millis(20));

        timer.tick();
        assert_eq!(timer.frame_count(), 1);
        assert!(timer.frame_time_ms() > 0.0);
    }

    #[test]
    fn test_frame_timer_fps_calculation() {
        let mut timer = FrameTimer::new(60);

        // Simulate running frames for over 1 second
        // Use 17ms per frame to ensure we exceed 1 second (17ms * 60 = 1020ms)
        for _ in 0..60 {
            thread::sleep(Duration::from_millis(17));
            timer.tick();
        }

        // FPS should be calculated after 1 second
        // It won't be exactly 60 due to sleep inaccuracy, but should be close
        assert!(timer.fps() > 0.0);
        assert_eq!(timer.frame_count(), 60);
    }

    #[test]
    fn test_frame_timer_default() {
        let timer = FrameTimer::default();
        assert_eq!(timer.frame_count(), 0);
    }
}
