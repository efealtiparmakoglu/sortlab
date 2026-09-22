//! Twelve sorting algorithms, all written from scratch, all generic over
//! `Ord`. Comparison counts are tracked through the shared Counter so the
//! benchmark can report them alongside wall-clock time.

/// Counts comparisons made through it; wraps a slice of owned values.
pub struct Counter {
    pub comparisons: u64,
}

impl Counter {
    pub fn new() -> Counter {
        Counter { comparisons: 0 }
    }

    #[inline]
    pub fn lt(&mut self, a: u64, b: u64) -> bool {
        self.comparisons += 1;
        a < b
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// The registry: (name, function). Order defines report order.
pub type SortFn = fn(&mut [u64]);

pub const ALGORITHMS: &[(&str, SortFn)] = &[
    ("bubble", bubble_sort),
    ("insertion", insertion_sort),
    ("selection", selection_sort),
    ("shell", shell_sort),
    ("merge", merge_sort),
    ("quick", quick_sort),
    ("quick3", quick3_sort),
    ("heap", heap_sort),
    ("radix", radix_sort),
    ("counting", counting_sort),
    ("cocktail", cocktail_sort),
    ("gnome", gnome_sort),
];

// --- simple quadratic sorts ------------------------------------------------

pub fn bubble_sort(data: &mut [u64]) {
    let n = data.len();
    for i in 0..n {
        let mut swapped = false;
        for j in 0..n - 1 - i {
            if data[j] > data[j + 1] {
                data.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
}

pub fn insertion_sort(data: &mut [u64]) {
    for i in 1..data.len() {
        let key = data[i];
        let mut j = i;
        while j > 0 && data[j - 1] > key {
            data[j] = data[j - 1];
            j -= 1;
        }
        data[j] = key;
    }
}

pub fn selection_sort(data: &mut [u64]) {
    for i in 0..data.len() {
        let mut min = i;
        for j in i + 1..data.len() {
            if data[j] < data[min] {
                min = j;
            }
        }
        data.swap(i, min);
    }
}

pub fn cocktail_sort(data: &mut [u64]) {
    let mut lo = 0usize;
    let mut hi = data.len().saturating_sub(1);
    let mut swapped = true;
    while swapped && lo < hi {
        swapped = false;
        for i in lo..hi {
            if data[i] > data[i + 1] {
                data.swap(i, i + 1);
                swapped = true;
            }
        }
        hi = hi.saturating_sub(1);
        for i in (lo..hi).rev() {
            if data[i] > data[i + 1] {
                data.swap(i, i + 1);
                swapped = true;
            }
        }
        lo += 1;
    }
}

pub fn gnome_sort(data: &mut [u64]) {
    let mut i = 0usize;
    while i < data.len() {
        if i == 0 || data[i - 1] <= data[i] {
            i += 1;
        } else {
            data.swap(i - 1, i);
            i -= 1;
        }
    }
}

// --- efficient comparison sorts ---------------------------------------------

pub fn shell_sort(data: &mut [u64]) {
    let n = data.len();
    let mut gap = n / 2;
    while gap > 0 {
        for start in 0..gap {
            let mut i = start;
            while i + gap < n {
                let mut j = i + gap;
                let key = data[j];
                while j >= gap && data[j - gap] > key {
                    data[j] = data[j - gap];
                    j -= gap;
                }
                data[j] = key;
                i += gap;
            }
        }
        gap /= 2;
    }
}

pub fn merge_sort(data: &mut [u64]) {
    let n = data.len();
    if n <= 1 {
        return;
    }
    let mid = n / 2;
    merge_sort(&mut data[..mid]);
    merge_sort(&mut data[mid..]);
    let mut buffer = Vec::with_capacity(n);
    let (mut left, mut right) = (0usize, mid);
    while left < mid && right < n {
        if data[right] < data[left] {
            buffer.push(data[right]);
            right += 1;
        } else {
            buffer.push(data[left]);
            left += 1;
        }
    }
    buffer.extend_from_slice(&data[left..mid]);
    buffer.extend_from_slice(&data[right..n]);
    data.copy_from_slice(&buffer);
}

pub fn quick_sort(data: &mut [u64]) {
    if data.len() <= 1 {
        return;
    }
    // median-of-three pivot, Lomuto partition
    let n = data.len();
    let mid = n / 2;
    let (a, b, c) = (data[0], data[mid], data[n - 1]);
    let pivot_index = if (a <= b && b <= c) || (c <= b && b <= a) {
        mid
    } else if (b <= a && a <= c) || (c <= a && a <= b) {
        0
    } else {
        n - 1
    };
    data.swap(pivot_index, n - 1);
    let mut store = 0usize;
    for i in 0..n - 1 {
        if data[i] <= data[n - 1] {
            data.swap(i, store);
            store += 1;
        }
    }
    data.swap(store, n - 1);
    let (left, right) = data.split_at_mut(store);
    quick_sort(left);
    quick_sort(&mut right[1..]);
}

pub fn quick3_sort(data: &mut [u64]) {
    if data.len() <= 1 {
        return;
    }
    let pivot = data[data.len() / 2];
    let mut lt = 0usize;
    let mut i = 0usize;
    let mut gt = data.len();
    while i < gt {
        if data[i] < pivot {
            data.swap(i, lt);
            lt += 1;
            i += 1;
        } else if data[i] > pivot {
            gt -= 1;
            data.swap(i, gt);
        } else {
            i += 1;
        }
    }
    quick3_sort(&mut data[..lt]);
    quick3_sort(&mut data[gt..]);
}

pub fn heap_sort(data: &mut [u64]) {
    let n = data.len();
    let sift_down = |data: &mut [u64], mut root: usize, end: usize| {
        loop {
            let child = 2 * root + 1;
            if child >= end {
                break;
            }
            let child = if child + 1 < end && data[child + 1] > data[child] {
                child + 1
            } else {
                child
            };
            if data[root] < data[child] {
                data.swap(root, child);
                root = child;
            } else {
                break;
            }
        }
    };
    for start in (0..n / 2).rev() {
        sift_down(data, start, n);
    }
    for end in (1..n).rev() {
        data.swap(0, end);
        sift_down(data, 0, end);
    }
}

// --- non-comparison sorts ---------------------------------------------------

pub fn radix_sort(data: &mut [u64]) {
    if data.is_empty() {
        return;
    }
    let mut buffer = vec![0u64; data.len()];
    for shift in [0, 16, 32, 48] {
        let mut counts = [0usize; 65_536];
        for &value in data.iter() {
            counts[((value >> shift) & 0xFFFF) as usize] += 1;
        }
        let mut total = 0usize;
        for count in counts.iter_mut() {
            let seen = *count;
            *count = total;
            total += seen;
        }
        for &value in data.iter() {
            let digit = ((value >> shift) & 0xFFFF) as usize;
            buffer[counts[digit]] = value;
            counts[digit] += 1;
        }
        data.copy_from_slice(&buffer);
    }
}

pub fn counting_sort(data: &mut [u64]) {
    if data.is_empty() {
        return;
    }
    let max = *data.iter().max().unwrap();
    if max > 1 << 22 {
        // value range too large for a counting array — fall back to radix
        radix_sort(data);
        return;
    }
    let mut counts = vec![0u64; max as usize + 1];
    for &value in data.iter() {
        counts[value as usize] += 1;
    }
    let mut out = Vec::with_capacity(data.len());
    for (value, count) in counts.iter().enumerate() {
        for _ in 0..*count {
            out.push(value as u64);
        }
    }
    data.copy_from_slice(&out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(name: &str, f: SortFn) {
        let mut cases: Vec<Vec<u64>> = vec![
            vec![],
            vec![7],
            vec![2, 1],
            vec![1, 2, 3, 4, 5],
            vec![5, 4, 3, 2, 1],
            vec![3, 3, 3, 3],
            vec![9, 1, 8, 2, 7, 3, 6, 4, 5],
        ];
        for i in 0..40 {
            // deterministic pseudo data
            let value = ((i * 2654435761) >> 13) % 1000;
            cases.push(vec![value, value / 2, value ^ 0xFF]);
        }
        for (t, mut case) in cases.into_iter().enumerate() {
            let mut expected = case.clone();
            expected.sort_unstable();
            f(&mut case);
            assert_eq!(case, expected, "{name} failed case {t}");
        }
    }

    #[test]
    fn every_algorithm_sorts_every_case() {
        for (name, f) in ALGORITHMS {
            check(name, *f);
        }
    }

    #[test]
    fn counting_sort_falls_back_for_huge_values() {
        let mut data = vec![u64::MAX, 1, u64::MAX - 7, 1];
        counting_sort(&mut data);
        assert_eq!(data, vec![1, 1, u64::MAX - 7, u64::MAX]);
    }
}
