//! Benchmark for `normalize::normalize_str`.
//!
//! Measures the real production function (no copy) over deterministic
//! clean and dirty inputs. Compare a candidate against a saved baseline
//! with:
//!
//! ```text
//! cargo bench --bench normalize -- --save-baseline pre   # while baseline
//! cargo bench --bench normalize -- --baseline pre        # candidate
//! ```

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use normalize_punctuation::normalize::normalize_str;

/// Fixed ~100-byte ASCII sentence; inputs are built by repeating it.
const BASE: &str =
    "The quick brown fox jumps over the lazy dog near the quiet riverbank as autumn leaves fall.\n";

/// The exotic-punctuation patterns `normalize_str` rewrites. Kept in
/// step with `REPLACEMENTS` in `src/normalize.rs`; drift here only
/// affects the shape of the benchmark inputs, never correctness.
const DIRTY: &[&str] = &[
    "‘",
    "’",
    "“",
    "”",
    "ˋ",
    "‚",
    "„",
    "…",
    "« ",
    "«\u{a0}",
    "«\u{202f}",
    "«\u{2009}",
    " »",
    "\u{a0}»",
    "\u{202f}»",
    "\u{2009}»",
    "\u{a0}",
    "\u{202f}",
    "\u{2009};",
    "\u{2009}?",
    "\u{2009}!",
    "\u{2009}:",
    "\u{2009}",
    "«",
    "»",
    "‐",
    "﹘",
    "−",
    "–",
];

/// Clean input of exactly `size` bytes (`BASE` is ASCII, so byte == char).
fn clean(size: usize) -> String {
    let mut s = String::with_capacity(size);
    while s.len() < size {
        s.push_str(BASE);
    }
    s.truncate(size);
    s
}

/// Dirty input of ~`size` bytes: pre-seed one occurrence of each of the
/// patterns (full arm coverage even at 1 KB), then inject a pattern roughly
/// every `stride` bytes, round-robin. Fully deterministic (no RNG).
fn dirty(size: usize, stride: usize) -> String {
    let mut s = String::with_capacity(size + 128);
    for pattern in DIRTY {
        s.push_str(pattern);
        s.push(' ');
    }
    let mut since = 0usize;
    let mut next = 0usize;
    for ch in BASE.chars().cycle() {
        if s.len() >= size {
            break;
        }
        s.push(ch);
        since += 1;
        if since >= stride {
            s.push_str(DIRTY[next % DIRTY.len()]);
            next += 1;
            since = 0;
        }
    }
    s
}

fn bench(c: &mut Criterion) {
    let sizes = [
        ("1KB", 1024usize),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    let mut group = c.benchmark_group("normalize");
    for (label, size) in sizes {
        let inputs = [
            ("clean", clean(size)),
            ("dirty_typical", dirty(size, 100)),
            ("dirty_heavy", dirty(size, 10)),
        ];
        for (kind, input) in &inputs {
            group.throughput(Throughput::Bytes(input.len() as u64));
            group.bench_with_input(BenchmarkId::new(*kind, label), input, |b, input| {
                b.iter(|| normalize_str(black_box(input)));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
