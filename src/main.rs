use std::path::Path;
use std::process::exit;

use sortlab::algorithms::ALGORITHMS;
use sortlab::bench::{bench_cell, stability_report};
use sortlab::chart::bar_chart;
use sortlab::csv::write_csv;
use sortlab::patterns::{Pattern, PATTERNS};

const USAGE: &str = r#"sortlab v1.0.0 — benchmark pack for 12 sorting algorithms

USAGE
  sortlab [options]

OPTIONS
  -n, --size <N>        elements per benchmark cell (default: 2000)
  -r, --reps <N>        runs per cell, median reported (default: 5)
  -a, --algo <name>     restrict to one algorithm (bubble..gnome)
  -p, --pattern <name>  restrict to one pattern (random..sawtooth)
  -s, --seed <N>        data generator seed (default: 42)
      --csv <file>      also write the raw matrix as CSV
  -h, --help            show this help
  -V, --version         print version

PATTERNS: random, sorted, reversed, nearly, few-unique, sawtooth
ALGORITHMS: bubble, insertion, selection, shell, merge, quick, quick3,
heap, radix, counting, cocktail, gnome

Every cell is correctness-checked before timing. The report ends with a
stability section: insertion/merge/quick/heap are measured on (key,
payload) pairs, the rest carry the standard classification."#;

fn value(argv: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    argv.get(*i).cloned().ok_or_else(|| format!("{} requires a value", flag))
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut size = 2000usize;
    let mut reps = 5usize;
    let mut seed = 42u64;
    let mut algo_filter: Option<String> = None;
    let mut pattern_filter: Option<String> = None;
    let mut csv_path: Option<String> = None;

    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        match arg {
            "-n" | "--size" => match value(&argv, &mut i, arg).and_then(|v| v.parse::<usize>().map_err(|_| format!("--size expects a number, got \"{v}\""))) {
                Ok(v) => size = v,
                Err(m) => fail_usage(&m),
            },
            "-r" | "--reps" => match value(&argv, &mut i, arg).and_then(|v| v.parse::<usize>().map_err(|_| format!("--reps expects a number, got \"{v}\""))) {
                Ok(v) if v > 0 => reps = v,
                Ok(_) => fail_usage("--reps must be at least 1"),
                Err(m) => fail_usage(&m),
            },
            "-s" | "--seed" => match value(&argv, &mut i, arg).and_then(|v| v.parse::<u64>().map_err(|_| format!("--seed expects a number, got \"{v}\""))) {
                Ok(v) => seed = v,
                Err(m) => fail_usage(&m),
            },
            "-a" | "--algo" => match value(&argv, &mut i, arg) {
                Ok(v) => algo_filter = Some(v),
                Err(m) => fail_usage(&m),
            },
            "-p" | "--pattern" => match value(&argv, &mut i, arg) {
                Ok(v) => pattern_filter = Some(v),
                Err(m) => fail_usage(&m),
            },
            "--csv" => match value(&argv, &mut i, arg) {
                Ok(v) => csv_path = Some(v),
                Err(m) => fail_usage(&m),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            "-V" | "--version" => {
                println!("sortlab 1.0.0");
                return;
            }
            _ => fail_usage(&format!("unknown option \"{arg}\"")),
        }
        i += 1;
    }

    if let Some(name) = &algo_filter {
        if !ALGORITHMS.iter().any(|(label, _)| label == name) {
            fail_usage(&format!("unknown algorithm \"{name}\""));
        }
    }
    if let Some(name) = &pattern_filter {
        if Pattern::try_from(name).is_none() {
            fail_usage(&format!("unknown pattern \"{name}\""));
        }
    }

    let algos: Vec<(&str, sortlab::algorithms::SortFn)> = ALGORITHMS
        .iter()
        .copied()
        .filter(|(label, _)| algo_filter.as_deref().is_none_or(|f| f == *label))
        .collect();
    let patterns: Vec<(&str, sortlab::patterns::Pattern)> = PATTERNS
        .iter()
        .copied()
        .filter(|(label, _)| pattern_filter.as_deref().is_none_or(|f| f == *label))
        .collect();

    println!("sortlab: {size} elements, {reps} reps, seed {seed}");
    println!();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for (pattern_name, _pattern) in &patterns {
        let mut timings: Vec<(String, f64)> = Vec::new();
        for (algo_name, _sort) in &algos {
            let row = bench_cell(algo_name, pattern_name, size, reps, seed);
            assert!(row.ok, "{} produced unsorted output on {}", row.algorithm, row.pattern);
            timings.push((row.algorithm.clone(), row.micros));
            rows.push(vec![
                row.algorithm,
                row.pattern,
                row.n.to_string(),
                format!("{:.1}", row.micros),
            ]);
        }
        println!("{}", bar_chart(&format!("{pattern_name} (microseconds, median)"), &timings, 48));
    }

    if let Some(path) = csv_path {
        write_csv(Path::new(&path), &["algorithm", "pattern", "n", "micros"], &rows)
            .unwrap_or_else(|e| fail_runtime(e));
        println!("csv written to {path}");
    }

    println!("stability (measured on pairs where marked, else classified):");
    for row in stability_report() {
        let marker = match (row.stable, row.measured) {
            (true, true) => "stable   (measured)",
            (true, false) => "stable   (classified)",
            (false, true) => "unstable (measured)",
            (false, false) => "unstable (classified)",
        };
        println!("  {:<12} {}", row.algorithm, marker);
    }
}

fn fail_usage(message: &str) -> ! {
    eprintln!("sortlab: {message}");
    eprintln!("sortlab: run 'sortlab --help' for usage");
    exit(2);
}

fn fail_runtime(message: String) -> ! {
    eprintln!("sortlab: {message}");
    exit(1);
}
