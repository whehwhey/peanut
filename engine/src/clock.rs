//! A monotonic clock the engine can call on every target.
//!
//! Native builds use `std::time::Instant` unchanged. `wasm32-unknown-unknown`
//! has no clock (`std::time::Instant::now()` panics "time not implemented"), and
//! the browser playground needs no wall-clock timing anyway, so there it is a
//! zero-cost stub: every `ms=`/seconds figure the engine prints comes out 0.
//! (Real timing in the browser would need a JS `performance.now()` import; it is
//! a local-server nicety, not something a proof depends on.)

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use stub::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
mod stub {
    #[derive(Clone, Copy)]
    pub struct Instant;
    #[derive(Clone, Copy)]
    pub struct Duration;
    impl Instant {
        #[inline]
        pub fn now() -> Instant { Instant }
        #[inline]
        pub fn elapsed(&self) -> Duration { Duration }
    }
    impl Duration {
        #[inline]
        pub fn as_millis(&self) -> u128 { 0 }
        #[inline]
        pub fn as_micros(&self) -> u128 { 0 }
        #[inline]
        pub fn as_secs_f64(&self) -> f64 { 0.0 }
    }
}
