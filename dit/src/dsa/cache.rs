use std::{collections::HashMap, hash::Hash};

pub struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}
//For my use case, this thing must return the item it evicted in place of the new item it placed as
//this is required to update the texture atlas. Along with that the capacity of the LruCache may
//not match the capacity of the Atlas due to the fact that when removing entries from a dynamic
//texture atlas, fragmentation can easily happen if not maanged properly and it can still happen
//regardless.

pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> index into nodes
    nodes: Vec<Node<K, V>>, // arena
    head: Option<usize>,    // MRU
    tail: Option<usize>,    // LRU
    capacity: usize,
}
impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }
    fn evict(&mut self) -> Option<V> {
        let removed_item = None;
        if let Some(tail) = self.tail {
            let key = &self.nodes[tail].key;
            self.map.remove(key);

            self.remove_from_list(tail);
        }
        removed_item
    }
    fn remove_from_list(&mut self, i: usize) {
        let (prev, next) = {
            let node = &self.nodes[i];
            (node.prev, node.next)
        };

        if let Some(p) = prev {
            self.nodes[p].next = next;
        } else {
            self.head = next;
        }
        if let Some(n) = next {
            self.nodes[n].prev = prev;
        } else {
            self.tail = prev;
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut evicted = None;
        if self.map.len() == self.capacity {
            self.evict();
        }
        let index = self.nodes.len();
        self.nodes.push(Node {
            key: key.clone(),
            value,
            prev: None,
            next: self.head,
        });

        if let Some(h) = self.head {
            self.nodes[h].prev = Some(index);
        }

        self.head = Some(index);
        if self.tail.is_none() {
            self.tail = Some(index);
        }

        self.map.insert(key, index);
        evicted
    }
    ///Shrinks the cache to the desired size by calling evict the required amount of times to reach
    ///the target size. Will not return the nodes that evict returns. Call shrink_and_take()
    ///instead.
    pub fn shrink(&mut self, target: usize) {}
    ///Same as shrink but it returns the nodes that get evicted.
    pub fn shrink_and_take(&mut self, target: usize) -> Vec<Node<K, V>> {
        vec![]
    }
    ///The entry is cached
    pub fn get(&mut self, key: &K) {
        if let Some(index) = self.map.get(&key) {}
    }
}
trait Cache<K, V> {
    fn get() {}
    fn set() {}
    fn contains() {}
    fn clear() {}
}
