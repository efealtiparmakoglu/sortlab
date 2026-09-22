//! Benchmark harness: correctness precheck, timed runs, stability report.

use crate::algorithms::ALGORITHMS;
use crate::pairs;
use crate::patterns::PATTERNS;
use crate::patterns::{generate, Pattern, Rng};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct BenchRow {
    pub algorithm: String,
    pub pattern: String,
    pub n: usize,
    /// median of `reps` runs, in microseconds
    pub micros: f64,
    /// output verified sorted
    pub ok: bool,
}

fn is_sorted(data: &[u64]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

/// Median duration over `reps` runs on fresh copies of the data.
/// Each algorithm gets identical input bytes; wall-clock, in microseconds.
pub fn time_sort(sort: crate::algorithms::SortFn, data: &[u64], reps: usize) -> f64 {
    let mut durations: Vec<f64> = Vec::with_capacity(reps);
    for _ in 0..reps {
        let mut copy = data.to_vec();
        let start = Instant::now();
        sort(&mut copy);
        durations.push(start.elapsed().as_secs_f64() * 1e6);
        assert!(is_sorted(&copy), "sort produced unsorted output");
    }
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    durations[durations.len() / 2]
}

/// Run one (algorithm, pattern, n) cell of the benchmark matrix.
pub fn bench_cell(algorithm: &str, pattern: &str, n: usize, reps: usize, seed: u64) -> BenchRow {
    let (name, sort) = ALGORITHMS
        .iter()
        .find(|(label, _)| *label == algorithm)
        .unwrap_or_else(|| panic!("unknown algorithm {algorithm}"));
    let pattern_enum = Pattern::try_from(pattern)
        .unwrap_or_else(|| panic!("unknown pattern {pattern}"));
    let data = generate(pattern_enum, n, u64::MAX / 4, seed);
    let mut copy = data.clone();
    sort(&mut copy);
    let ok = is_sorted(&copy);
    let micros = if ok { time_sort(*sort, &data, reps) } else { f64::NAN };
    BenchRow {
        algorithm: (*name).to_string(),
        pattern: pattern.to_string(),
        n,
        micros,
        ok,
    }
}

impl Pattern {
    pub fn try_from(text: &str) -> Option<Pattern> {
        PATTERNS
            .iter()
            .find(|(name, _)| *name == text)
            .map(|(_, pattern)| *pattern)
    }
}

#[derive(Clone, Debug)]
pub struct StabilityRow {
    pub algorithm: String,
    /// measured on (key, payload) pairs, or classified for u64-only sorts
    pub stable: bool,
    pub measured: bool,
}

/// Empirical stability probe. A u64-only sort cannot exhibit stability
/// (equal elements are indistinguishable), so four representative
/// algorithms run on (key, payload) pairs with shuffled equal-key payloads.
/// The remaining algorithms carry the standard theoretical classification.
pub fn stability_report() -> Vec<StabilityRow> {
    // keys repeat in runs of 4; payloads are shuffled before the probe.
    // rank[payload] = input position, so a stable sort leaves each
    // equal-key chunk in increasing rank order.
    let n = 256usize;
    let mut sample: Vec<pairs::Pair> = (0..n)
        .map(|i| pairs::Pair { key: (i / 4) as u64, payload: i as u64 })
        .collect();
    let mut rng = Rng::new(42);
    for i in (1..n).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        sample.swap(i, j);
    }
    let mut rank = vec![0usize; n];
    for (input_index, pair) in sample.iter().enumerate() {
        rank[pair.payload as usize] = input_index;
    }
    let preserves = |data: &[pairs::Pair]| {
        data.chunks(4)
            .all(|chunk| chunk.windows(2).all(|w| rank[w[0].payload as usize] < rank[w[1].payload as usize]))
    };

    let mut rows = Vec::new();
    for (name, _sort) in ALGORITHMS {
        match *name {
            "insertion" | "merge" => {
                let mut data = sample.clone();
                if *name == "insertion" {
                    pairs::insertion_pairs(&mut data);
                } else {
                    pairs::merge_pairs(&mut data);
                }
                rows.push(StabilityRow {
                    algorithm: (*name).to_string(),
                    stable: preserves(&data),
                    measured: true,
                });
            }
            "quick" | "heap" => {
                let mut data = sample.clone();
                if *name == "quick" {
                    pairs::quick_pairs(&mut data);
                } else {
                    pairs::heap_pairs(&mut data);
                }
                rows.push(StabilityRow {
                    algorithm: (*name).to_string(),
                    stable: preserves(&data),
                    measured: true,
                });
            }
            // standard classification: stable
            "bubble" | "cocktail" | "gnome" | "counting" | "radix" => {
                rows.push(StabilityRow { algorithm: (*name).to_string(), stable: true, measured: false });
            }
            // standard classification: unstable
            "selection" | "shell" | "quick3" => {
                rows.push(StabilityRow { algorithm: (*name).to_string(), stable: false, measured: false });
            }
            other => unreachable!("unregistered algorithm {other}"),
        }
    }
    rows
}
