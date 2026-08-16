//! Hard memory budget for the engine process.
//!
//! A counting global allocator: every live byte is tracked; when the total exceeds
//! `AM_MEM_MB` megabytes (default 2048) the process prints an `ERR memory budget`
//! line and exits with status 3.  This is the innermost of three guards (engine /
//! Python runner / system watchdog) that stop a runaway subset construction from
//! swapping the machine to death.  It fires long before the kernel is in trouble.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The process's `#[global_allocator]`: a thin wrapper over the system
/// allocator that charges/uncharges every (de)allocation against the
/// `AM_MEM_MB` budget (see module docs) before delegating to [`System`].
pub struct Budgeted;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static LIMIT: AtomicUsize = AtomicUsize::new(0); // 0 = not yet read from env
static TRIPPED: AtomicBool = AtomicBool::new(false);

/// Call once at the top of main (env lookup allocates, so it cannot happen inside alloc).
pub fn init() {
    let mb: usize = std::env::var("AM_MEM_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(2048);
    LIMIT.store(mb.saturating_mul(1 << 20).max(1), Ordering::Relaxed);
}

#[inline]
fn limit() -> usize {
    let l = LIMIT.load(Ordering::Relaxed);
    if l != 0 { l } else { 2048 << 20 }
}

fn charge(n: usize) {
    let now = LIVE.fetch_add(n, Ordering::Relaxed) + n;
    let mut p = PEAK.load(Ordering::Relaxed);
    while now > p {
        match PEAK.compare_exchange_weak(p, now, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break, Err(x) => p = x,
        }
    }
    if now > limit() && !TRIPPED.swap(true, Ordering::SeqCst) {
        // TRIPPED is set, so allocations made while reporting pass straight through.
        eprintln!("ERR memory budget exceeded: {} MB live > AM_MEM_MB={} MB", now >> 20, limit() >> 20);
        println!("ERR memory budget exceeded ({} MB)", now >> 20);
        std::process::exit(3);
    }
}

unsafe impl GlobalAlloc for Budgeted {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        charge(l.size());
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        charge(l.size());
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        if new > l.size() { charge(new - l.size()); } else { LIVE.fetch_sub(l.size() - new, Ordering::Relaxed); }
        unsafe { System.realloc(p, l, new) }
    }
}

/// High-water mark of tracked live bytes since process start, in MB.
pub fn peak_mb() -> usize { PEAK.load(Ordering::Relaxed) >> 20 }
/// Currently-live tracked bytes right now, in MB.
pub fn live_mb() -> usize { LIVE.load(Ordering::Relaxed) >> 20 }
