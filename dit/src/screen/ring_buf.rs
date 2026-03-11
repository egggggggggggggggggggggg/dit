use std::ops::{Index, Range};

struct RingBuffer<T: Copy + Default> {
    data: Vec<T>,
    //start is the write index.
    start: usize,
    capacity: usize,
    //end is the read index.
    end: usize,
}
impl<T: Copy + Default> Index<usize> for RingBuffer<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let abs_index = (self.end + index) % self.capacity;
        &self.data[abs_index]
    }
}
impl<T: Copy + Default> RingBuffer<T> {
    ///Do not use this. If you do this, will allocate a capacity of 1 to ensure operations acting
    ///on this can work without panicking.
    fn new() -> Self {
        Self {
            data: vec![T::default()],
            start: 0,
            capacity: 1,
            end: 0,
        }
    }
    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            start: 0,
            capacity,
            end: 0,
        }
    }
    fn expand(&mut self, exact: usize) {
        if self.capacity >= exact {
            return;
        }
        self.make_contiguous();
        self.data.resize(exact, T::default());
        self.capacity = exact;
    }
    ///Shrinks the Vec by removing values that are not within the specified range.
    ///The range provided is not absolute into the Vec but rather relative.
    fn shrink(&mut self, range: Range<usize>) {
        if range.end > self.capacity {
            panic!(
                "Range has exceeded the possible Vec capacity. If you want it to wrap around, implement it yourself."
            )
        }
        //check if we can just get a slice from the start and end slice. if yes we can just easily
        //use that for the new vec and return without an expensive make_contiguous call.
        if self.end + range.start < self.capacity && self.end + range.end <= self.capacity {
            self.data = self.data[self.end + range.start..self.end + range.end].to_vec();
            return;
        }
        self.make_contiguous();
        self.data = self.data[range].to_vec();
    }
    fn make_contiguous(&mut self) {
        if self.start >= self.end || self.data.is_empty() {
            return;
        }
        let mut new_data = Vec::with_capacity(self.capacity);
        new_data.extend_from_slice(&self.data[self.end..self.start]);
        if self.start < self.end {
            new_data.extend_from_slice(&self.data[0..self.start]);
        }
        self.data = new_data;
        self.end = 0;
        self.start = self.data.len(); // next write goes after last element
    }
    pub fn batch_push(&mut self, input: &[T]) {
        let cap = self.capacity;
        let n = input.len();
        let write = self.start;
        let first = (cap - write).min(n);
        self.data[write..write + first].copy_from_slice(&input[..first]);
        if n > first {
            let second = n - first;
            self.data[0..second].copy_from_slice(&input[first..]);
        }
        self.start = (self.start + n) % cap;
        let used = (self.start + cap - self.end) % cap;
        if used >= cap {
            self.end = (self.start + 1) % cap;
        }
    }
    pub fn push(&mut self, new_value: T) {
        self.data[self.start] = new_value;
        self.start = (self.start + 1) % self.capacity;
        if self.start == self.end {
            self.end = (self.end + 1) % self.capacity; // overwrite oldest
        }
    }
    pub fn batch_pop() {}
}
