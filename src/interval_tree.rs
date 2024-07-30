use arrayvec::ArrayVec;

#[derive(Debug, Clone)]
pub struct ContiguousIntervalTree<T, const N: usize> {
    nodes: ArrayVec<IntervalNode<T>, N>,
}
impl<T, const N: usize> ContiguousIntervalTree<T, N> {
    fn check_rep(&self) {
        assert_eq!(self.nodes.first().unwrap().index_start, 0);
        let mut prev_index = None;
        for node in &self.nodes {
            if let Some(prev_index) = prev_index {
                assert!(prev_index < node.index_start);
            }
            prev_index = Some(node.index_start);
        }
    }

    pub fn new(nodes: ArrayVec<IntervalNode<T>, N>) -> Self {
        let this = Self { nodes };
        this.check_rep();
        this
    }

    pub fn get(&self, index: usize) -> &T {
        let mut start = 0;
        let mut end = self.nodes.len();
        while start != end {
            assert!(start < end);
            let mid = (start + end) / 2;
            let interval = &self.nodes[mid];
            let interval_end = self.nodes.get(mid + 1).map(|x| x.index_start).unwrap_or(N);
            match interval.index_start.cmp(&index) {
                std::cmp::Ordering::Equal | std::cmp::Ordering::Less => {
                    if (interval.index_start..interval_end).contains(&index) {
                        return &interval.value;
                    }
                    start = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    end = mid;
                }
            }
        }
        &self.nodes[start].value
    }
}

#[derive(Debug, Clone)]
pub struct IntervalNode<T> {
    pub index_start: usize,
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let it: ContiguousIntervalTree<usize, 16> =
            ContiguousIntervalTree::new(ArrayVec::from_iter([
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
            ]));
        assert_eq!(*it.get(0), 0);
        assert_eq!(*it.get(1), 0);
        assert_eq!(*it.get(2), 0);
        assert_eq!(*it.get(3), 1);
        assert_eq!(*it.get(4), 2);
        assert_eq!(*it.get(5), 2);
        assert_eq!(*it.get(15), 2);
    }
}
