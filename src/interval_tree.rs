#[derive(Debug, Clone)]
pub struct ContiguousIntervalTree<T> {
    nodes: Vec<IntervalNode<T>>,
    capacity: usize,
}
impl<T> ContiguousIntervalTree<T> {
    fn check_rep(&self) {
        assert_eq!(self.nodes.first().unwrap().index_start, 0);
        let mut prev_index = None;
        for node in &self.nodes {
            if let Some(prev_index) = prev_index {
                assert!(prev_index < node.index_start);
            }
            prev_index = Some(node.index_start);
        }
        assert!(prev_index.unwrap() < self.capacity);
    }

    pub fn new(nodes: Vec<IntervalNode<T>>, capacity: usize) -> Self {
        let this = Self { nodes, capacity };
        this.check_rep();
        this
    }

    fn interval_end(&self, arr_index: usize) -> usize {
        self.nodes
            .get(arr_index + 1)
            .map(|x| x.index_start)
            .unwrap_or(self.capacity)
    }
    fn arr_index(&self, index: usize) -> usize {
        let mut start = 0;
        let mut end = self.nodes.len();
        loop {
            assert!(start < end);
            let mid = (start + end) / 2;
            let interval = &self.nodes[mid];
            let interval_end = self.interval_end(mid);
            match interval.index_start.cmp(&index) {
                std::cmp::Ordering::Equal | std::cmp::Ordering::Less => {
                    if (interval.index_start..interval_end).contains(&index) {
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
        &self.nodes[self.arr_index(index)].value
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
        let new_node = IntervalNode {
            index_start: index,
            value,
        };
        let arr_index = self.arr_index(index);
        let interval = &self.nodes[arr_index];
        let interval_end = self.interval_end(arr_index);
        if interval.value == new_node.value {
            return;
        }
        if interval.index_start == index {
            let mut should_merge_with_prev_node = false;
            if let Some(prev) = arr_index.checked_sub(1).and_then(|i| self.nodes.get(i)) {
                if prev.value == new_node.value {
                    should_merge_with_prev_node = true;
                }
            }

            let is_only_one = interval_end - interval.index_start == 1;
            if !is_only_one {
                // Shrink the current node to the right
                self.nodes[arr_index].index_start += 1;
            }

            if should_merge_with_prev_node {
                if is_only_one {
                    // Remove the current node
                    self.nodes.remove(arr_index);
                }
                return;
            }

            let range_end = if is_only_one {
                // Remove the current node
                arr_index + 1
            } else {
                // Prepend to the current node
                arr_index
            };
            self.nodes.splice(arr_index..range_end, [new_node]);
            return;
        }
        let is_at_last = index == interval_end - 1;
        match is_at_last {
            false => {
                // Split the current node
                let orig_value = interval.value.clone();
                self.nodes.splice(
                    arr_index + 1..arr_index + 1,
                    [
                        new_node,
                        IntervalNode {
                            index_start: index + 1,
                            value: orig_value,
                        },
                    ],
                );
            }
            true => {
                if let Some(next) = self.nodes.get_mut(arr_index + 1) {
                    if next.value == new_node.value {
                        // Merge with the next node
                        next.index_start += 1;
                        return;
                    }
                }

                // Append to the current node
                self.nodes.splice(arr_index + 1..arr_index + 1, [new_node]);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntervalNode<T> {
    pub index_start: usize,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct CellWiseIter<'a, T> {
    tree: &'a ContiguousIntervalTree<T>,
    arr_index: usize,
    interval_i: usize,
}
impl<'a, T> CellWiseIter<'a, T> {
    pub fn new(tree: &'a ContiguousIntervalTree<T>) -> Self {
        Self {
            tree,
            arr_index: 0,
            interval_i: 0,
        }
    }
}
impl<'a, T> Iterator for CellWiseIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.arr_index == self.tree.nodes.len() {
            return None;
        }
        let interval = &self.tree.nodes[self.arr_index];
        self.interval_i += 1;
        let interval_end = self.tree.interval_end(self.arr_index);
        let interval_len = interval_end - interval.index_start;
        if self.interval_i == interval_len {
            self.arr_index += 1;
            self.interval_i = 0;
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
                    index_start: 0,
                    value: 0,
                },
                IntervalNode {
                    index_start: 3,
                    value: 1,
                },
                IntervalNode {
                    index_start: 4,
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
                    index_start: 0,
                    value: 0,
                },
                IntervalNode {
                    index_start: 3,
                    value: 1,
                },
                IntervalNode {
                    index_start: 4,
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
