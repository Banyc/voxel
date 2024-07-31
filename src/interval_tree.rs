#[derive(Debug, Clone)]
pub struct ContiguousIntervalTree<T> {
    intervals: Vec<IntervalNode<T>>,
    capacity: usize,
}
impl<T> ContiguousIntervalTree<T> {
    fn check_rep(&self) {
        assert_eq!(self.intervals.first().unwrap().cell_i_start, 0);
        let mut prev_index = None;
        for node in &self.intervals {
            if let Some(prev_index) = prev_index {
                assert!(prev_index < node.cell_i_start);
            }
            prev_index = Some(node.cell_i_start);
        }
        assert!(prev_index.unwrap() < self.capacity);
    }

    pub fn new(nodes: Vec<IntervalNode<T>>, capacity: usize) -> Self {
        let this = Self {
            intervals: nodes,
            capacity,
        };
        this.check_rep();
        this
    }

    fn interval_cell_i_end(&self, interval_i: usize) -> usize {
        self.intervals
            .get(interval_i + 1)
            .map(|x| x.cell_i_start)
            .unwrap_or(self.capacity)
    }
    fn interval_i(&self, cell_i: usize) -> usize {
        let mut start = 0;
        let mut end = self.intervals.len();
        loop {
            assert!(start < end);
            let mid = (start + end) / 2;
            let interval = &self.intervals[mid];
            let interval_cell_i_end = self.interval_cell_i_end(mid);
            match interval.cell_i_start.cmp(&cell_i) {
                std::cmp::Ordering::Equal | std::cmp::Ordering::Less => {
                    if (interval.cell_i_start..interval_cell_i_end).contains(&cell_i) {
                        return mid;
                    }
                    start = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    end = mid;
                }
            }
        }
    }

    /// Time complexity: $O(\log N)$
    pub fn get(&self, index: usize) -> &T {
        &self.intervals[self.interval_i(index)].value
    }
    pub fn cell_wise_iter(&self) -> CellWiseIter<'_, T> {
        CellWiseIter::new(self)
    }
}
impl<T> ContiguousIntervalTree<T>
where
    T: Clone + Eq,
{
    /// Time complexity: $O(N)$
    pub fn set(&mut self, index: usize, value: T) {
        let new = IntervalNode {
            cell_i_start: index,
            value,
        };
        let interval_i = self.interval_i(index);
        let interval = &self.intervals[interval_i];
        let interval_cell_i_end = self.interval_cell_i_end(interval_i);
        if interval.value == new.value {
            return;
        }
        if interval.cell_i_start == index {
            let mut should_merge_with_prev_node = false;
            if let Some(prev) = interval_i
                .checked_sub(1)
                .and_then(|i| self.intervals.get(i))
            {
                if prev.value == new.value {
                    should_merge_with_prev_node = true;
                }
            }

            let is_only_one = interval_cell_i_end - interval.cell_i_start == 1;
            if !is_only_one {
                // Shrink the current node to the right
                self.intervals[interval_i].cell_i_start += 1;
            }

            if should_merge_with_prev_node {
                if is_only_one {
                    // Remove the current node
                    self.intervals.remove(interval_i);
                }
                return;
            }

            let range_end = if is_only_one {
                // Remove the current node
                interval_i + 1
            } else {
                // Prepend to the current node
                interval_i
            };
            self.intervals.splice(interval_i..range_end, [new]);
            return;
        }
        let is_at_last = index == interval_cell_i_end - 1;
        match is_at_last {
            false => {
                // Split the current node
                let orig_value = interval.value.clone();
                self.intervals.splice(
                    interval_i + 1..interval_i + 1,
                    [
                        new,
                        IntervalNode {
                            cell_i_start: index + 1,
                            value: orig_value,
                        },
                    ],
                );
            }
            true => {
                if let Some(next) = self.intervals.get_mut(interval_i + 1) {
                    if next.value == new.value {
                        // Merge with the next node
                        next.cell_i_start += 1;
                        return;
                    }
                }

                // Append to the current node
                self.intervals.insert(interval_i + 1, new);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntervalNode<T> {
    pub cell_i_start: usize,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct CellWiseIter<'a, T> {
    tree: &'a ContiguousIntervalTree<T>,
    interval_i: usize,
    cell_i: usize,
}
impl<'a, T> CellWiseIter<'a, T> {
    pub fn new(tree: &'a ContiguousIntervalTree<T>) -> Self {
        Self {
            tree,
            interval_i: 0,
            cell_i: 0,
        }
    }
}
impl<'a, T> Iterator for CellWiseIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.interval_i == self.tree.intervals.len() {
            return None;
        }
        let interval = &self.tree.intervals[self.interval_i];
        self.cell_i += 1;
        let interval_end = self.tree.interval_cell_i_end(self.interval_i);
        let interval_len = interval_end - interval.cell_i_start;
        if self.cell_i == interval_len {
            self.interval_i += 1;
            self.cell_i = 0;
        }
        Some(&interval.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let it = ContiguousIntervalTree::new(
            Vec::from_iter([
                IntervalNode {
                    cell_i_start: 0,
                    value: 0,
                },
                IntervalNode {
                    cell_i_start: 3,
                    value: 1,
                },
                IntervalNode {
                    cell_i_start: 4,
                    value: 2,
                },
            ]),
            16,
        );
        assert_eq!(*it.get(0), 0);
        assert_eq!(*it.get(1), 0);
        assert_eq!(*it.get(2), 0);
        assert_eq!(*it.get(3), 1);
        assert_eq!(*it.get(4), 2);
        assert_eq!(*it.get(5), 2);
        assert_eq!(*it.get(15), 2);

        let cells = it.cell_wise_iter().copied().collect::<Vec<usize>>();
        assert_eq!(cells, [0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
    }

    #[test]
    fn test_set() {
        let mut it = ContiguousIntervalTree::new(
            Vec::from_iter([
                IntervalNode {
                    cell_i_start: 0,
                    value: 0,
                },
                IntervalNode {
                    cell_i_start: 3,
                    value: 1,
                },
                IntervalNode {
                    cell_i_start: 4,
                    value: 2,
                },
            ]),
            16,
        );
        it.set(0, 3);
        it.check_rep();
        assert_eq!(*it.get(0), 3);
        assert_eq!(*it.get(1), 0);
        assert_eq!(*it.get(2), 0);
        it.set(2, 4);
        it.check_rep();
        assert_eq!(*it.get(1), 0);
        assert_eq!(*it.get(2), 4);
        assert_eq!(*it.get(3), 1);
        it.set(5, 5);
        it.check_rep();
        assert_eq!(*it.get(4), 2);
        assert_eq!(*it.get(5), 5);
        assert_eq!(*it.get(6), 2);
    }

    #[test]
    fn test_splice() {
        let mut vec = vec![1, 2, 3];
        vec.splice(1..1, [4, 5]);
        assert_eq!(vec, [1, 4, 5, 2, 3]);
    }
}
