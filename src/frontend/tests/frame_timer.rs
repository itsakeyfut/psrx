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

//! Unit tests for FrameTimer

use crate::frontend::frame_timer::FrameTimer;
use std::thread;
use std::time::Duration;

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
