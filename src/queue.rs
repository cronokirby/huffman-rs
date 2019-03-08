/// Represents a PriorityQueue,
/// allowing us to insert items into it while maintaining an order.
/// `K` represents the key on which the list is sorted, in descending order.
/// This is the most useful order for Huffman coding.
/// `V` is the type of things this queue can store
/// Note that the order of insertion matters if K is the same.
pub struct PriorityQueue<K, V> {
    data: Vec<(K, V)>
}

impl <K : std::cmp::Ord, V> PriorityQueue<K, V> {
    /// Construct an empty priority queue
    pub fn new() -> Self {
        PriorityQueue { data: Vec::new() }
    }

    /// Construct a priority queue with a pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        PriorityQueue { data: Vec::with_capacity(capacity) }
    }

    /// Remove the lowest priority item from the queue, if it exists
    pub fn remove(&mut self) -> Option<(K, V)> {
        self.data.pop().map(|x| x)
    }

    /// Insert a value with a given priority key into the queue
    pub fn insert(&mut self, key: K, value: V) {
        // reverse binary search
        let index = match self.data.binary_search_by(|(probe_k, _)| key.cmp(probe_k)) {
            Ok(i) => i,
            Err(i) => i
        };
        self.data.insert(index, (key, value));
    }

    /// Try and take two elements from this queue
    pub fn remove_two(&mut self) -> Option<((K, V), (K, V))> {
        if self.data.len() < 2 {
            return None
        }
        let a = self.remove().unwrap();
        let b = self.remove().unwrap();
        Some((a, b))
    }
}

mod test {
    use super::PriorityQueue;

    #[test]
    fn inserting_preserves_order() {
        let mut q: PriorityQueue<i32, i32> = PriorityQueue::new();
        q.insert(100, 69);
        q.insert(1, 80);
        assert_eq!(q.remove(), Some((1, 80)));
    }
}