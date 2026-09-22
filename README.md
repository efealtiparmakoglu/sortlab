# sortlab

Benchmark pack for **12 sorting algorithms** over **6 data patterns** —
with stability probing, CSV export and ASCII charts. Pure Rust, zero
crates, deterministic generator.

```text
$ sortlab -n 3000 -r 3 --csv sortlab.csv
sorted (microseconds, median)
quick3       | 6.8
shell        | 24.4
heap         | 41.8
selection    |################################################ 6143.2
...
```

## The field

| Family | Algorithms |
|---|---|
| Quadratic | bubble, insertion, selection, cocktail, gnome |
| Efficient comparison | shell, merge, quick (median-of-three), quick3 (Dutch flag), heap |
| Non-comparison | radix (LSD base-65536), counting (auto-falls back to radix for huge ranges) |

Every algorithm is written from scratch, correctness-checked before
timing, and benchmarked over six patterns: **random, sorted, reversed,
nearly-sorted, few-unique, sawtooth**. Same seed, same data — every run
of the matrix is reproducible.

## The three report sections

1. **Charts** — one ASCII bar chart per pattern, median microseconds
   (`-r` reps per cell). Read the whole matrix and you see insertion
   quietly winning nearly-sorted inputs while selection grinds at O(n²)
   everywhere, and quick3 eating sorted data for dessert.
2. **CSV** — `--csv file` dumps the raw matrix
   (`algorithm,pattern,n,micros`) for spreadsheets or gnuplot.
3. **Stability** — stability is invisible to a `fn(&mut [u64])` sort
   (equal elements are indistinguishable), so four representative
   algorithms run for real on `(key, payload)` pairs with shuffled
   equal-key payloads: **insertion/merge measure stable,
   quick/heap measure unstable**. The remaining sorts carry the standard
   theoretical classification and are marked `classified` in the output.

## Usage

```
sortlab [-n size] [-r reps] [-s seed] [-a algo] [-p pattern] [--csv file]
```

Filters restrict the matrix to one algorithm or one pattern. Exit codes:
0 ok, 1 I/O failure, 2 usage error.

## Development

```console
$ cargo test     # 13 tests: every algorithm × every case, patterns, pairs, csv, chart
$ cargo clippy --all-targets -- -D warnings
```

## License

MIT — see [LICENSE](LICENSE).
