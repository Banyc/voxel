use std::collections::HashMap;

use crate::interval_tree::ContiguousIntervalTree;

pub type Index = [usize; 3];

const CHUNK_SIZE: [usize; 3] = [32, 32, 32];

#[derive(Debug, Clone)]
pub struct ChunkIndex {
    value: Index,
}
impl ChunkIndex {
    pub fn new(value: Index) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct VoxelIndex {
    value: Index,
}
impl VoxelIndex {
    pub fn new(value: Index) -> Self {
        Self { value }
    }
    pub fn value(&self) -> Index {
        self.value
    }

    pub fn chunk_index(&self) -> ChunkIndex {
        let index: Index = self
            .value
            .iter()
            .copied()
            .zip(CHUNK_SIZE)
            .map(|(x, n)| x / n)
            .collect::<Vec<usize>>()
            .try_into()
            .unwrap();
        ChunkIndex::new(index)
    }
    pub fn interval_tree_index(&self) -> usize {
        let mut index = 0;
        let mut mag = 1;
        for (x, n) in self.value.iter().copied().zip(CHUNK_SIZE) {
            let i = x % n;
            index += i * mag;
            mag += 1;
        }
        index
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSet<T> {
    chunks: HashMap<Index, Chunk<T>>,
}
impl<T> ChunkSet<T> {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}
impl<T> Default for ChunkSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IndexIter {
    start: Index,
    inclusive_end: Index,
    next: Option<Index>,
}
impl IndexIter {
    pub fn new(start: Index, inclusive_end: Index) -> Self {
        for (s, e) in start.iter().copied().zip(inclusive_end) {
            assert!(s <= e);
        }
        Self {
            start,
            inclusive_end,
            next: Some(start),
        }
    }
}
impl Iterator for IndexIter {
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        let mut mag = 0;
        for (x, (s, e)) in self
            .next
            .as_mut()
            .unwrap()
            .iter_mut()
            .zip(self.start.iter().copied().zip(self.inclusive_end))
        {
            if *x != e {
                *x += 1;
                break;
            }
            *x = s;
            mag += 1;
        }
        if next.len() == mag {
            self.next = None;
        }
        Some(next)
    }
}
#[test]
fn test_index_iter() {
    let start = [0, 1, 2];
    let inclusive_end = [1, 3, 2];
    let mut iter = IndexIter::new(start, inclusive_end);
    assert_eq!(iter.next(), Some([0, 1, 2]));
    assert_eq!(iter.next(), Some([1, 1, 2]));
    assert_eq!(iter.next(), Some([0, 2, 2]));
    assert_eq!(iter.next(), Some([1, 2, 2]));
    assert_eq!(iter.next(), Some([0, 3, 2]));
    assert_eq!(iter.next(), Some([1, 3, 2]));
    assert_eq!(iter.next(), None);
}

#[derive(Debug, Clone)]
pub struct Chunk<T> {
    data: ContiguousIntervalTree<T>,
}
impl<T> Chunk<T> {
    pub fn new(data: ContiguousIntervalTree<T>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &ContiguousIntervalTree<T> {
        &self.data
    }
}
