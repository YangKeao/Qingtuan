use super::extend_iter::ExtendIter;
use std::sync::Arc;
use std::sync::RwLock;

const MAX_LEVEL: usize = 256;

pub struct Node<T> {
    pub value: T,
    nexts: Vec<RwLock<Option<Arc<Node<T>>>>>,
}

pub struct SkipList<T: PartialOrd> {
    head: Arc<Node<T>>,
}

fn random_height() -> usize {
    rand::random::<usize>() % MAX_LEVEL + 1
}

pub struct SkipListIter<T: PartialOrd> {
    now: Arc<Node<T>>,
}

impl<T: PartialOrd> Iterator for SkipListIter<T> {
    type Item = Arc<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let now = self.now.clone();

        let read_guard = now.nexts[0].read().unwrap();
        match &*read_guard {
            Some(next) => {
                self.now = next.clone();
                return Some(self.now.clone());
            }
            None => None,
        }
    }
}

impl<T: PartialOrd + Default> ExtendIter<T> for SkipListIter<T> {
    fn seek(&mut self, val: &T) -> Option<Self::Item> {
        let quasi_skiplist = SkipList {
            head: self.now.clone(),
        };
        let (now, _) = quasi_skiplist.find_greater_or_equal(val);
        match now {
            Some(now) => {
                self.now = now.clone();
                Some(now.clone())
            }
            None => None,
        }
    }
}

impl<T: PartialOrd + Default> SkipList<T> {
    pub fn new() -> Self {
        let mut nexts = Vec::with_capacity(MAX_LEVEL);
        for _ in 0..MAX_LEVEL {
            nexts.push(RwLock::new(None));
        }
        return Self {
            head: Arc::new(Node {
                value: T::default(),
                nexts,
            }),
        };
    }

    pub fn iter(&self) -> SkipListIter<T> {
        SkipListIter::<T> {
            now: self.head.clone(),
        }
    }

    pub fn insert(&mut self, val: T) {
        let (_now, prev) = self.find_greater_or_equal(&val);

        let height = random_height();
        let mut nexts = Vec::with_capacity(MAX_LEVEL);
        for _ in 0..MAX_LEVEL {
            nexts.push(RwLock::new(None));
        }
        let new_node = Node { value: val, nexts };

        for i in 0..height {
            match &prev[i] {
                Some(prev) => match &*prev.nexts[i].read().unwrap() {
                    Some(next) => {
                        new_node.nexts[i].write().unwrap().replace(next.clone());
                    }
                    None => {}
                },
                None => {
                    unreachable!();
                }
            }
        }

        let new_node_ptr = Arc::new(new_node);
        for i in 0..height {
            match &prev[i] {
                Some(prev) => {
                    prev.nexts[i].write().unwrap().replace(new_node_ptr.clone());
                }
                None => {
                    unreachable!();
                }
            }
        }
    }

    pub fn find_greater_or_equal(
        &self,
        val: &T,
    ) -> (Option<Arc<Node<T>>>, Vec<Option<Arc<Node<T>>>>) {
        let mut prev: Vec<Option<Arc<Node<T>>>> = vec![None; MAX_LEVEL];
        let mut x = self.head.clone();

        let mut level = MAX_LEVEL - 1;
        loop {
            let read_guard = x.nexts[level].read().unwrap();
            match &*read_guard {
                Some(next) => {
                    if &next.value < val {
                        let new_x = next.clone();
                        drop(read_guard);
                        x = new_x;
                        continue;
                    } else {
                        prev[level] = Some(x.clone());
                        if level == 0 {
                            return (Some(next.clone()), prev);
                        } else {
                            level -= 1;
                        }
                    }
                }
                None => {
                    prev[level] = Some(x.clone());
                    if level == 0 {
                        return (None, prev);
                    } else {
                        level -= 1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_test() {
        let mut list = SkipList::new();
        for i in 0..1000 {
            list.insert(i);
        }
        for (index, num) in list.iter().enumerate() {
            assert_eq!(num.value, index);
        }
    }

    #[test]
    fn random_insert() {
        let mut list = SkipList::new();
        let mut nums: Vec<u32> = Vec::new();
        for _ in 0..1000 {
            let num = rand::random();
            nums.push(num);
            list.insert(num);
        }
        nums.sort();
        for (index, num) in list.iter().enumerate() {
            assert_eq!(num.value, nums[index]);
        }
    }

    #[test]
    fn seek_test() {
        let mut list = SkipList::new();
        for i in 0..1000 {
            list.insert(i);
        }

        let mut iter = list.iter();
        let val = iter.seek(&500).unwrap();
        assert_eq!(val.value, 500);

        let mut acc = 500;
        while let Some(val) = iter.next() {
            acc += 1;
            assert_eq!(val.value, acc);
        }
    }
}
