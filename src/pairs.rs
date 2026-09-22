//! Pair-aware variants used by the empirical stability probe. A u64-only
//! sort cannot exhibit stability: equal elements are indistinguishable, so
//! these four representative sorts operate on (key, payload) pairs and the
//! probe measures whether equal keys keep their input order.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub key: u64,
    pub payload: u64,
}

/// Stable: adjacent swaps only, equal keys never jump over each other.
pub fn insertion_pairs(data: &mut [Pair]) {
    for i in 1..data.len() {
        let key = data[i];
        let mut j = i;
        while j > 0 && data[j - 1].key > key.key {
            data[j] = data[j - 1];
            j -= 1;
        }
        data[j] = key;
    }
}

/// Stable: left side wins ties.
pub fn merge_pairs(data: &mut [Pair]) {
    let n = data.len();
    if n <= 1 {
        return;
    }
    let mid = n / 2;
    merge_pairs(&mut data[..mid]);
    merge_pairs(&mut data[mid..]);
    let mut buffer = Vec::with_capacity(n);
    let (mut left, mut right) = (0usize, mid);
    while left < mid && right < n {
        if data[right].key < data[left].key {
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

/// Unstable: partitioning moves equal keys across each other.
pub fn quick_pairs(data: &mut [Pair]) {
    if data.len() <= 1 {
        return;
    }
    let n = data.len();
    // middle-element pivot moved out of the way, guaranteed to land in place
    data.swap(n / 2, n - 1);
    let pivot = data[n - 1].key;
    let mut store = 0usize;
    for i in 0..n - 1 {
        if data[i].key < pivot {
            data.swap(i, store);
            store += 1;
        }
    }
    data.swap(store, n - 1);
    let (left, right) = data.split_at_mut(store);
    quick_pairs(left);
    quick_pairs(&mut right[1..]);
}

/// Unstable: heapify reorders equal keys.
pub fn heap_pairs(data: &mut [Pair]) {
    let n = data.len();
    let sift_down = |data: &mut [Pair], mut root: usize, end: usize| {
        loop {
            let child = 2 * root + 1;
            if child >= end {
                break;
            }
            let child = if child + 1 < end && data[child + 1].key > data[child].key {
                child + 1
            } else {
                child
            };
            if data[root].key < data[child].key {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Shuffled (key, payload) pairs plus the input rank of each payload:
    /// a stable sort keeps payload order *as it appeared in the input*,
    /// which is not the same as numerically increasing payloads.
    fn sample(n: usize) -> (Vec<Pair>, Vec<usize>) {
        let mut data: Vec<Pair> = (0..n)
            .map(|i| Pair { key: (i / 4) as u64, payload: i as u64 })
            .collect();
        let mut rng = crate::patterns::Rng::new(42);
        for i in (1..n).rev() {
            let j = rng.below((i + 1) as u64) as usize;
            data.swap(i, j);
        }
        let mut rank = vec![0usize; n];
        for (input_index, pair) in data.iter().enumerate() {
            rank[pair.payload as usize] = input_index;
        }
        (data, rank)
    }

    fn preserves_input_order(data: &[Pair], rank: &[usize]) -> bool {
        data.chunks(4).all(|chunk| {
            chunk
                .windows(2)
                .all(|w| rank[w[0].payload as usize] < rank[w[1].payload as usize])
        })
    }

    #[test]
    fn insertion_and_merge_are_stable() {
        for f in [insertion_pairs as fn(&mut [Pair]), merge_pairs] {
            let (mut data, rank) = sample(256);
            f(&mut data);
            assert!(
                preserves_input_order(&data, &rank),
                "expected stable: {}",
                std::any::type_name::<fn(&mut [Pair])>()
            );
        }
    }

    #[test]
    fn quick_and_heap_lose_order_on_this_input() {
        let mut unstable_seen = 0;
        for f in [quick_pairs as fn(&mut [Pair]), heap_pairs] {
            let (mut data, rank) = sample(256);
            f(&mut data);
            if !preserves_input_order(&data, &rank) {
                unstable_seen += 1;
            }
        }
        assert_eq!(unstable_seen, 2, "both unstable sorts must reorder equal keys");
    }
}
