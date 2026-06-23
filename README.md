# carousel

Time-driven page cycler for small displays.

A `Carousel` answers one question: given the current millisecond clock and
whether an external trigger fired, should the display show its idle screen or
one of the cycle pages, and which page?

It is pure scheduling. No rendering, no I/O, no atomics, no timer ownership.
The caller owns the framebuffer, the clock source, the trigger, and the
`CarouselState`; this crate only maps `(now, triggered)` to a `Frame`. That
keeps the timing logic `no_std`, allocation-free, and unit-testable without
hardware.

## Example

```rust
use carousel::{Carousel, CarouselState, Frame};

const SCREENS: Carousel = Carousel {
    page_count: 4,
    page_duration_ms: 5_000,
    cycle_window_ms: 60_000,
    cycle_period_ms: 5 * 60_000, // 0 => trigger-only, never auto-start
};

// Offset the first auto-cycle so the idle screen shows right after boot.
let mut state = CarouselState::new(SCREENS.cycle_period_ms);

// Each render tick, feed the clock and any pending trigger:
match SCREENS.frame_at(&mut state, now_ms, button_pressed) {
    Frame::Idle => { /* draw the idle screen */ }
    Frame::Page(index) => { /* draw page `index` */ }
}
```

## Behaviour

- An automatic burst opens every `cycle_period_ms` and runs for
  `cycle_window_ms`, advancing one page every `page_duration_ms` and wrapping
  through `0..page_count`. Outside a burst the carousel reports `Frame::Idle`.
- Passing `triggered = true` opens a burst immediately without disturbing the
  automatic schedule, so a manual burst never delays or skips the next
  periodic one.
- `cycle_period_ms = 0` disables automatic bursts: the carousel then cycles
  only when triggered.
- `page_count = 0` reports `Frame::Idle` on every tick.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
