//! Time-driven page cycler for small displays.
//!
//! A [`Carousel`] answers one question: given the current millisecond
//! clock and whether an external trigger fired, should the display show
//! its idle screen or one of the cycle pages, and which page?
//!
//! It is pure scheduling. There is no rendering, no I/O, no atomics, and
//! no timer ownership here. The caller owns the framebuffer, the clock
//! source, the trigger, and the [`CarouselState`]; this crate only maps
//! `(now, triggered)` to a [`Frame`]. That keeps the timing logic
//! `no_std`, allocation-free, and unit-testable without hardware.
//!
//! ```
//! use carousel::{Carousel, CarouselState, Frame};
//!
//! const SCREENS: Carousel = Carousel {
//!     page_count: 4,
//!     page_duration_ms: 5_000,
//!     cycle_window_ms: 60_000,
//!     cycle_period_ms: 5 * 60_000,
//! };
//!
//! // Offset the first auto-cycle so the idle screen shows right after boot.
//! let mut state = CarouselState::new(SCREENS.cycle_period_ms);
//!
//! // Each render tick, feed the clock and any pending trigger:
//! match SCREENS.frame_at(&mut state, now_ms(), button_pressed()) {
//!     Frame::Idle => { /* draw the idle screen */ }
//!     Frame::Page(index) => { /* draw page `index` */ }
//! }
//! # fn now_ms() -> u64 { 0 }
//! # fn button_pressed() -> bool { false }
//! ```
#![no_std]

/// Declarative cycle configuration.
///
/// All fields are public so a configuration is a plain `const` literal.
/// Times are milliseconds on whatever monotonic clock the caller feeds
/// to [`Carousel::frame_at`].
#[derive(Clone, Copy, Debug)]
pub struct Carousel {
    /// Number of distinct cycle pages. Page indices run `0..page_count`.
    /// Zero disables cycling entirely; every frame is [`Frame::Idle`].
    pub page_count: u8,
    /// How long each page stays on screen before advancing to the next.
    /// A value of zero is clamped to one millisecond to stay panic-free.
    pub page_duration_ms: u64,
    /// Total length of one cycle burst. After this many milliseconds the
    /// carousel reverts to [`Frame::Idle`]. With `page_duration_ms` this
    /// sets how many page views a burst contains.
    pub cycle_window_ms: u64,
    /// Idle gap between the start of one automatic burst and the next.
    /// Zero means never auto-start: bursts then happen only when the
    /// caller passes `triggered = true`.
    pub cycle_period_ms: u64,
}

/// What the caller should draw this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    /// No burst is active. Draw the idle screen.
    Idle,
    /// A burst is active. Draw the page with this index, always in
    /// `0..page_count`.
    Page(u8),
}

/// Mutable carousel state, owned by the caller's render loop.
///
/// Kept separate from [`Carousel`] so the configuration can stay an
/// immutable `const` while the per-loop bookkeeping lives on the stack.
#[derive(Clone, Copy, Debug)]
pub struct CarouselState {
    /// Clock value, in milliseconds, at which the active burst began.
    start: u64,
    /// Clock value, in milliseconds, at which the active burst ends. Zero
    /// means no burst is active, so any `now >= 0` reverts to idle.
    until: u64,
    /// Clock value, in milliseconds, at which the next automatic burst is
    /// due.
    next_auto: u64,
}

impl CarouselState {
    /// Start idle, with the first automatic burst due at
    /// `first_auto_at_ms` on the caller's clock. Pass
    /// `now + cycle_period_ms` to show the idle screen immediately after
    /// boot, or `0` to let a burst start on the very first tick.
    #[must_use]
    pub const fn new(first_auto_at_ms: u64) -> Self {
        Self {
            start: 0,
            until: 0,
            next_auto: first_auto_at_ms,
        }
    }
}

impl Carousel {
    /// Advance the carousel to `now_ms` and return what to draw.
    ///
    /// `triggered` reports whether an external event (a button, a remote
    /// command) asked for a burst since the last call. A trigger starts a
    /// fresh burst without disturbing the automatic schedule, so a manual
    /// burst will not delay or skip the next periodic one.
    ///
    /// Call this once per render tick. It is the only method that mutates
    /// `state`.
    pub fn frame_at(&self, state: &mut CarouselState, now_ms: u64, triggered: bool) -> Frame {
        let auto_due = self.cycle_period_ms != 0 && now_ms >= state.next_auto;
        if auto_due {
            state.next_auto = now_ms + self.cycle_period_ms;
        }
        if triggered || auto_due {
            state.start = now_ms;
            state.until = now_ms + self.cycle_window_ms;
        }

        if self.page_count == 0 || now_ms >= state.until {
            return Frame::Idle;
        }

        let elapsed = now_ms - state.start;
        let step = elapsed / self.page_duration_ms.max(1);
        let index = step % u64::from(self.page_count);
        Frame::Page(u8::try_from(index).unwrap_or(0))
    }
}

#[cfg(test)]
#[path = "tests/carousel.rs"]
mod tests;
