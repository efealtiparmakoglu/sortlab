//! Data shapes that stress sorters differently.

/// Deterministic xorshift* generator: the benchmark must be reproducible.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    pub fn draw(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform value in [0, bound).
    pub fn below(&mut self, bound: u64) -> u64 {
        self.draw() % bound.max(1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pattern {
    Random,
    Sorted,
    Reversed,
    NearlySorted,
    FewUnique,
    Sawtooth,
}

pub const PATTERNS: &[(&str, Pattern)] = &[
    ("random", Pattern::Random),
    ("sorted", Pattern::Sorted),
    ("reversed", Pattern::Reversed),
    ("nearly", Pattern::NearlySorted),
    ("few-unique", Pattern::FewUnique),
    ("sawtooth", Pattern::Sawtooth),
];

/// Build `n` values in [0, max_value) for the given pattern.
pub fn generate(pattern: Pattern, n: usize, max_value: u64, seed: u64) -> Vec<u64> {
    let mut rng = Rng::new(seed);
    let mut data: Vec<u64> = match pattern {
        Pattern::Random => (0..n).map(|_| rng.below(max_value)).collect(),
        Pattern::Sorted => (0..n as u64).map(|i| i * max_value / n.max(1) as u64).collect(),
        Pattern::Reversed => {
            let mut v: Vec<u64> = (0..n as u64).map(|i| i * max_value / n.max(1) as u64).collect();
            v.reverse();
            v
        }
        Pattern::NearlySorted => {
            let mut v: Vec<u64> = (0..n as u64).map(|i| i * max_value / n.max(1) as u64).collect();
            let swaps = (n / 50).max(1);
            for _ in 0..swaps {
                let a = rng.below(n as u64) as usize;
                let b = rng.below(n as u64) as usize;
                v.swap(a, b);
            }
            v
        }
        Pattern::FewUnique => (0..n).map(|_| rng.below(8)).collect(),
        Pattern::Sawtooth => {
            let teeth = 5u64;
            (0..n as u64).map(|i| ((i % (n as u64 / teeth.max(1))) * max_value) / (n as u64 / teeth.max(1))).collect()
        }
    };
    data.resize(n, 0);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_patterns_have_right_length() {
        for (name, pattern) in PATTERNS {
            let data = generate(*pattern, 100, 10_000, 7);
            assert_eq!(data.len(), 100, "{name}");
        }
    }

    #[test]
    fn sorted_pattern_is_sorted_reversed_is_reversed() {
        let sorted = generate(Pattern::Sorted, 50, 1000, 1);
        let mut check = sorted.clone();
        check.sort_unstable();
        assert_eq!(sorted, check);

        let reversed = generate(Pattern::Reversed, 50, 1000, 1);
        let mut check = reversed.clone();
        check.sort_unstable();
        check.reverse();
        assert_eq!(reversed, check);
    }

    #[test]
    fn nearly_sorted_is_mostly_sorted() {
        let data = generate(Pattern::NearlySorted, 500, 10_000, 3);
        let mut sorted = data.clone();
        sorted.sort_unstable();
        let in_place = data.iter().zip(sorted.iter()).filter(|(a, b)| a == b).count();
        // 10 swapped pairs displace at most 20 positions
        assert!(in_place >= 475, "at least 95% in place, got {}", in_place);
    }

    #[test]
    fn few_unique_has_few_unique_values() {
        let data = generate(Pattern::FewUnique, 1000, 10_000, 5);
        let mut unique = data.clone();
        unique.sort_unstable();
        unique.dedup();
        assert!(unique.len() <= 8);
    }

    #[test]
    fn generator_is_deterministic() {
        let a = generate(Pattern::Random, 200, 1 << 20, 9);
        let b = generate(Pattern::Random, 200, 1 << 20, 9);
        assert_eq!(a, b);
    }
}
