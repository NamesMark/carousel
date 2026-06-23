use crate::{Carousel, CarouselState, Frame};

const SCREENS: Carousel = Carousel {
    page_count: 4,
    page_duration_ms: 5_000,
    cycle_window_ms: 60_000,
    cycle_period_ms: 300_000,
};

#[test]
fn idle_before_first_auto() {
    let mut state = CarouselState::new(SCREENS.cycle_period_ms);
    assert_eq!(SCREENS.frame_at(&mut state, 0, false), Frame::Idle);
    assert_eq!(SCREENS.frame_at(&mut state, 299_999, false), Frame::Idle);
}

#[test]
fn auto_burst_walks_pages_then_reverts() {
    let mut state = CarouselState::new(SCREENS.cycle_period_ms);
    // Burst opens at the period boundary on page 0.
    assert_eq!(SCREENS.frame_at(&mut state, 300_000, false), Frame::Page(0));
    // Each page holds for page_duration_ms.
    assert_eq!(SCREENS.frame_at(&mut state, 304_999, false), Frame::Page(0));
    assert_eq!(SCREENS.frame_at(&mut state, 305_000, false), Frame::Page(1));
    assert_eq!(SCREENS.frame_at(&mut state, 310_000, false), Frame::Page(2));
    assert_eq!(SCREENS.frame_at(&mut state, 315_000, false), Frame::Page(3));
    // Window is 60s, pages are 5s: 12 views, so it wraps back through 0.
    assert_eq!(SCREENS.frame_at(&mut state, 320_000, false), Frame::Page(0));
    // Just before the window closes.
    assert_eq!(SCREENS.frame_at(&mut state, 359_999, false), Frame::Page(3));
    // Window closed: back to idle.
    assert_eq!(SCREENS.frame_at(&mut state, 360_000, false), Frame::Idle);
}

#[test]
fn trigger_opens_burst_mid_period() {
    let mut state = CarouselState::new(SCREENS.cycle_period_ms);
    assert_eq!(SCREENS.frame_at(&mut state, 10_000, false), Frame::Idle);
    assert_eq!(SCREENS.frame_at(&mut state, 10_000, true), Frame::Page(0));
    assert_eq!(SCREENS.frame_at(&mut state, 12_500, false), Frame::Page(0));
    assert_eq!(SCREENS.frame_at(&mut state, 15_000, false), Frame::Page(1));
}

#[test]
fn trigger_does_not_disturb_auto_schedule() {
    let mut state = CarouselState::new(SCREENS.cycle_period_ms);
    // Manual burst early on, then let it close.
    assert_eq!(SCREENS.frame_at(&mut state, 10_000, true), Frame::Page(0));
    assert_eq!(SCREENS.frame_at(&mut state, 70_001, false), Frame::Idle);
    // The automatic burst still fires at the original period boundary.
    assert_eq!(SCREENS.frame_at(&mut state, 299_999, false), Frame::Idle);
    assert_eq!(SCREENS.frame_at(&mut state, 300_000, false), Frame::Page(0));
}

#[test]
fn auto_reschedules_for_next_period() {
    let mut state = CarouselState::new(SCREENS.cycle_period_ms);
    assert_eq!(SCREENS.frame_at(&mut state, 300_000, false), Frame::Page(0));
    assert_eq!(SCREENS.frame_at(&mut state, 360_000, false), Frame::Idle);
    // Second automatic burst one full period after the first.
    assert_eq!(SCREENS.frame_at(&mut state, 599_999, false), Frame::Idle);
    assert_eq!(SCREENS.frame_at(&mut state, 600_000, false), Frame::Page(0));
}

#[test]
fn zero_period_is_trigger_only() {
    const MANUAL: Carousel = Carousel {
        page_count: 3,
        page_duration_ms: 5_000,
        cycle_window_ms: 60_000,
        cycle_period_ms: 0,
    };
    let mut state = CarouselState::new(0);
    // Never auto-starts even long past any plausible period.
    assert_eq!(MANUAL.frame_at(&mut state, 10_000_000, false), Frame::Idle);
    // Still responds to triggers.
    assert_eq!(MANUAL.frame_at(&mut state, 10_000_000, true), Frame::Page(0));
}

#[test]
fn zero_pages_is_always_idle() {
    const EMPTY: Carousel = Carousel {
        page_count: 0,
        page_duration_ms: 5_000,
        cycle_window_ms: 60_000,
        cycle_period_ms: 300_000,
    };
    let mut state = CarouselState::new(0);
    assert_eq!(EMPTY.frame_at(&mut state, 0, true), Frame::Idle);
    assert_eq!(EMPTY.frame_at(&mut state, 300_000, false), Frame::Idle);
}

#[test]
fn page_index_wraps_within_count() {
    let mut state = CarouselState::new(0);
    // Open a burst at t=0 via trigger, then sample deep into the window.
    SCREENS.frame_at(&mut state, 0, true);
    // step = 55_000 / 5_000 = 11; 11 % 4 = 3.
    assert_eq!(SCREENS.frame_at(&mut state, 55_000, false), Frame::Page(3));
}

#[test]
fn zero_page_duration_does_not_panic() {
    const FAST: Carousel = Carousel {
        page_count: 2,
        page_duration_ms: 0,
        cycle_window_ms: 60_000,
        cycle_period_ms: 300_000,
    };
    let mut state = CarouselState::new(0);
    // Clamped to 1ms per page; just assert it returns a valid page.
    assert_eq!(FAST.frame_at(&mut state, 0, true), Frame::Page(0));
    assert_eq!(FAST.frame_at(&mut state, 1, false), Frame::Page(1));
}
