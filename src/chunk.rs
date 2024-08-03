use std::collections::HashMap;

use crate::interval_tree::{CellWiseIter, ContiguousIntervalTree};

pub type IndexPart = u64;
pub type Index = [IndexPart; 3];

const CHUNK_SIZE: [usize; 3] = [2 << 4, 2 << 4, 2 << 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkIndex {
    value: Index,
}
impl ChunkIndex {
    pub fn new(value: Index) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            .map(|(x, n)| x / IndexPart::try_from(n).unwrap())
            .collect::<Vec<IndexPart>>()
            .try_into()
            .unwrap();
        ChunkIndex::new(index)
    }
    pub fn interval_tree_index(&self) -> usize {
        let mut index = 0;
        let mut mag = 1;
        for (x, n) in self.value.iter().copied().zip(CHUNK_SIZE) {
            let i = x % IndexPart::try_from(n).unwrap();
            index += usize::try_from(i).unwrap() * mag;
            mag *= n;
        }
        index
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSet<T> {
    chunks: HashMap<ChunkIndex, Chunk<T>>,
}
impl<T> ChunkSet<T> {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    pub fn chunk(&self, index: ChunkIndex) -> Option<&Chunk<T>> {
        self.chunks.get(&index)
    }
    pub fn set_chunk(&mut self, index: ChunkIndex, chunk: Chunk<T>) {
        self.chunks.insert(index, chunk);
    }
}
impl<T> Default for ChunkSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ValueIter<'a, T> {
    chunk_set: &'a ChunkSet<T>,
    range: core::ops::RangeInclusive<VoxelIndex>,
    index_iter: IndexIter,
    cell_iter: Option<(ChunkIndex, CellWiseIter<'a, T>)>,
}
impl<'a, T> ValueIter<'a, T> {
    pub fn new(chunk_set: &'a ChunkSet<T>, range: core::ops::RangeInclusive<VoxelIndex>) -> Self {
        let index_iter = IndexIter::new(range.start().value()..=range.end().value());
        Self {
            chunk_set,
            range,
            index_iter,
            cell_iter: None,
        }
    }

    fn set_cell_iter(&mut self, index: VoxelIndex) {
        let cell_iter = self
            .chunk_set
            .chunk(index.chunk_index())
            .unwrap()
            .data()
            .cell_wise_iter(index.interval_tree_index());
        self.cell_iter = Some((index.chunk_index(), cell_iter));
    }
}
impl<'a, T> Iterator for ValueIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.index_iter.next()?;
        let i = VoxelIndex::new(next);
        if next[0] == self.range.start().value()[0] {
            self.set_cell_iter(i);
        }
        if i.chunk_index() != self.cell_iter.as_ref().unwrap().0 {
            self.set_cell_iter(i);
        }
        Some(self.cell_iter.as_mut().unwrap().1.next().unwrap())
    }
}
#[cfg(test)]
#[test]
fn test_value_iter() {
    let mut counter = 0;
    let mut chunk_set = ChunkSet::new();
    let chunk_size = CHUNK_SIZE.iter().product();
    for y in 0..=1 {
        for x in 0..=1 {
            let mut nodes = vec![];
            for i in 0..chunk_size {
                nodes.push(crate::interval_tree::IntervalNode {
                    cell_i_start: i,
                    value: counter,
                });
                counter += 1;
            }
            let data = ContiguousIntervalTree::new(nodes, chunk_size);
            let chunk = Chunk::new(data);
            chunk_set.set_chunk(ChunkIndex::new([x, y, 0]), chunk);
        }
    }
    let start = [CHUNK_SIZE[0] - 1, CHUNK_SIZE[1] - 1, 0];
    let end = [CHUNK_SIZE[0], CHUNK_SIZE[1], 0];
    let start = start
        .iter()
        .copied()
        .map(|x| IndexPart::try_from(x).unwrap())
        .collect::<Vec<IndexPart>>()
        .try_into()
        .unwrap();
    let end = end
        .iter()
        .copied()
        .map(|x| IndexPart::try_from(x).unwrap())
        .collect::<Vec<IndexPart>>()
        .try_into()
        .unwrap();
    let start = VoxelIndex::new(start);
    let end = VoxelIndex::new(end);
    let mut iter = ValueIter::new(&chunk_set, start..=end);
    assert_eq!(
        iter.next().copied(),
        Some((CHUNK_SIZE[0] - 1) + (CHUNK_SIZE[0] * (CHUNK_SIZE[1] - 1)))
    );
    assert_eq!(
        iter.next().copied(),
        Some((CHUNK_SIZE[0] * (CHUNK_SIZE[1] - 1)) + CHUNK_SIZE.iter().product::<usize>())
    );
    assert_eq!(
        iter.next().copied(),
        Some((CHUNK_SIZE[0] - 1) + CHUNK_SIZE.iter().product::<usize>() * 2)
    );
    assert_eq!(
        iter.next().copied(),
        Some(CHUNK_SIZE.iter().product::<usize>() * 3)
    );
    assert_eq!(iter.next().copied(), None);
}

#[derive(Debug, Clone)]
pub struct IndexIter {
    range: core::ops::RangeInclusive<Index>,
    next: Option<Index>,
}
impl IndexIter {
    pub fn new(range: core::ops::RangeInclusive<Index>) -> Self {
        for (s, e) in range
            .start()
            .iter()
            .copied()
            .zip(range.end().iter().copied())
        {
            assert!(s <= e);
        }
        let next = *range.start();
        Self {
            range,
            next: Some(next),
        }
    }
}
impl Iterator for IndexIter {
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        let mut mag = 0;
        for (x, (s, e)) in self.next.as_mut().unwrap().iter_mut().zip(
            self.range
                .start()
                .iter()
                .copied()
                .zip(self.range.end().iter().copied()),
        ) {
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
#[cfg(test)]
#[test]
fn test_index_iter() {
    let start = [0, 1, 2];
    let inclusive_end = [1, 3, 2];
    let mut iter = IndexIter::new(start..=inclusive_end);
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
        assert_eq!(data.capacity(), CHUNK_SIZE.iter().product());
        Self { data }
    }

    pub fn data(&self) -> &ContiguousIntervalTree<T> {
        &self.data
    }
}
