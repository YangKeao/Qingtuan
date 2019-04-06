use std::sync::Arc;
use std::sync::RwLock;

const MAX_LEVEL: usize = 4;

pub struct Node<T> {
    pub value: T,
    nexts: Vec<Option<Arc<RwLock<Node<T>>>>>,
}

pub struct SkipList<T: PartialOrd> {
    head: Arc<RwLock<Node<T>>>,
}

fn random_height() -> usize {
    rand::random::<usize>() % MAX_LEVEL + 1
}

pub struct SkipListIter<T: PartialOrd> {
    now: Arc<RwLock<Node<T>>>,
}

impl<T: PartialOrd> Iterator for SkipListIter<T> {
    type Item = Arc<RwLock<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let now = self.now.clone();
        let now_guard = now.read().unwrap();
        match &now_guard.nexts[0] {
            Some(next) => {
                self.now = next.clone();
                return Some(self.now.clone());
            }
            None => None,
        }
    }
}

impl<T: PartialOrd + Default> SkipList<T> {
    pub fn new() -> Self {
        let mut nexts = Vec::new();
        for _ in 0..MAX_LEVEL {
            nexts.push(None);
        }
        return Self {
            head: Arc::new(RwLock::new(Node {
                value: T::default(),
                nexts,
            })),
        };
    }

    pub fn iter(&self) -> SkipListIter<T> {
        SkipListIter::<T> {
            now: self.head.clone(),
        }
    }

    pub fn insert(&self, val: T) {
        //        println!();
        //        unsafe {
        //            let val = &val as *const T as *const u32;
        //            println!("INSERTING {}", *val);
        //        }

        let (_now, prev) = self.find_greater_or_equal(&val);

        let height = random_height();
        let mut nexts = Vec::new();
        for _ in 0..MAX_LEVEL {
            nexts.push(None);
        }
        let new_node = Arc::new(RwLock::new(Node { value: val, nexts }));

        for i in 0..height {
            match &prev[i] {
                Some(prev) => {
                    match &prev.read().unwrap().nexts[i] {
                        Some(next) => {
                            new_node.write().unwrap().nexts[i] = Some(next.clone());
                        }
                        None => {}
                    }
                    //                    unsafe {
                    //                        let val = &prev.read().unwrap().value as *const T as *const u32;
                    //                        if i == 0 {
                    //                            println!("{} {}", *val, i);
                    //                        }
                    //                    }
                    prev.write().unwrap().nexts[i] = Some(new_node.clone());
                }
                None => {
                    unreachable!();
                }
            }
        }
    }

    fn find_greater_or_equal(
        &self,
        val: &T,
    ) -> (
        Option<Arc<RwLock<Node<T>>>>,
        Vec<Option<Arc<RwLock<Node<T>>>>>,
    ) {
        let mut prev: Vec<Option<Arc<RwLock<Node<T>>>>> = vec![None; MAX_LEVEL];
        let mut x = self.head.clone();

        let mut level = MAX_LEVEL - 1;
        loop {
            let x_guard = x.read().unwrap();
            match &x_guard.nexts[level] {
                Some(next) => {
                    if &next.read().unwrap().value < val {
                        let new_x = next.clone();
                        drop(x_guard);
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
    use std::thread;

    #[test]
    fn insert_test() {
        let list = SkipList::new();
        for i in 0..1000 {
            list.insert(i);
        }
        for (index, num) in list.iter().enumerate() {
            assert_eq!(num.read().unwrap().value, index);
        }
    }

    #[test]
    fn random_insert() {
        let list = SkipList::new();
        let mut nums: Vec<u32> = Vec::new();
        for _ in 0..1000 {
            let num = rand::random();
            nums.push(num);
            list.insert(num);
        }
        nums.sort();
        for (index, num) in list.iter().enumerate() {
            assert_eq!(num.read().unwrap().value, nums[index]);
        }
    }

    #[test]
    fn multi_thread_random_insert() {
        let list = Arc::new(SkipList::new());
        let mut num_list = Vec::new();
        let mut nums: Vec<Arc<Vec<u32>>> = Vec::new();

        const THREAD_NUM: usize = 8;
        const NUMS_PER_THREAD: usize = 5;
        for _ in 0..THREAD_NUM {
            let mut local_nums = Vec::new();
            for _ in 0..NUMS_PER_THREAD {
                let num = rand::random();
                num_list.push(num);
                local_nums.push(num);
            }
            nums.push(Arc::new(local_nums));
        }
        num_list.sort();

        let mut threads = Vec::new();
        for i in 0..THREAD_NUM {
            let nums = nums[i].clone();
            let list = list.clone();
            threads.push(thread::spawn(move || {
                for i in 0..NUMS_PER_THREAD {
                    list.insert(nums[i]);
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }

        for (index, num) in list.iter().enumerate() {
            assert_eq!(num.read().unwrap().value, num_list[index]);
        }
    }
}
