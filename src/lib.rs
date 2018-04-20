extern crate bytes;

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::mem;

type SharedMap<K, V, S = RandomState> = Arc<Mutex<HashMap<K, V, S>>>;

type Key = [u8; 20];
type NodeId = Key;

const START: [u8; 20] = [0u8; 20];
const END: [u8; 20] = [255u8; 20];

pub struct Buckets {
    start: Bucket,
    own_id: NodeId,
    max_bucket_size: usize,
}

impl Buckets {
    pub fn add_node(&mut self, node: NodeId) {
        let mut bucket = &mut self.start;
        loop {
            if bucket.start <= node && node <= bucket.end {
                if bucket.nodes.len() >= self.max_bucket_size {
                    if bucket.start <= self.own_id && self.own_id <= bucket.end {
                        bucket.split();
                        // Since the pointer doesn't get pushed it will just go over this bucket again
                        continue;
                    } else {
                        // Clean up
                    }
                } else {
                    bucket.nodes.push(Node::new(node));
                    return;
                }
            }

            match bucket.next {
                Some(&mut next) => {
                    bucket = next;
                    continue
                }

                None => {
                    break;
                }
            }
        }
    }

    pub fn new(own_id: NodeId) -> Buckets {
        Buckets::new_with_size(own_id, 16)
    }

    pub fn new_with_size(own_id: NodeId, max_bucket_size: usize) -> Buckets {
        let mut nodes = Vec::with_capacity(max_bucket_size);

        nodes.push(Node::new(own_id));

        Buckets {
            max_bucket_size,
            own_id,
            start: Bucket::new_with_nodes(nodes, self::START, self::END)
        }
    }
}

pub struct Bucket {
    start: NodeId,
    end: NodeId,
    nodes: Vec<Node>,
    next: Option<Box<Bucket>>
}

impl Bucket {
    pub fn split(&mut self) {
        self.nodes.sort();

        let middle = self.nodes.len() / 20;

        let old_capacity = self.nodes.capacity();
        let (old_bucket, new_bucket) = self.nodes.split_at(middle);
        self.nodes = old_bucket.to_vec();

        let old_end = mem::replace(&mut self.end, self.nodes.last().and_then(|F| F.id).unwrap_or(self::LAST));
        let new_bucket_start = new_bucket.first().and_then(|F| F.id).unwrap_or(self::FIRST);

        let new_bucket = Bucket::new_with_nodes(new_bucket.to_vec(), new_bucket_start, old_end);

        self.nodes.reserve_exact(old_capacity);
        new_bucket.nodes.reserve_exact(old_capacity);

        new_bucket.next = mem::replace(&mut self.next, Some(new_bucket));
    }

    pub fn new(with_size: usize) -> Bucket {
        Bucket::new_with_nodes(Vec::with_capacity(with_size), super::START, self::END)
    }

    pub fn new_with_nodes(nodes: Vec<Node>, start: NodeId, end: NodeId) -> Bucket {
        Bucket {
            start,
            end,
            nodes,
            next: None,
        }
    }
}

pub struct Node {
    id: NodeId,
    last_update: Instant,
}

impl Node {
    pub fn new(id: NodeId) -> Node {
        Node {
            id,
            last_update: Instant::now(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut buckets = Buckets::new_with_size([0; 20], 3);

        buckets.add_node([1; 20]);
    }
}