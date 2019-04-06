use std::sync::Arc;
use std::sync::RwLock;

const MAX_LEVEL: usize = 4;

pub struct Node<T> {
    level: usize,
    value: T,
    nexts: Vec<Option<Arc<RwLock<Node<T>>>>>,
}

pub struct SkipList<T: PartialOrd> {
    head: RwLock<Option<Arc<RwLock<Node<T>>>>>,
}

fn random_height() -> usize {
    rand::random::<usize>() % MAX_LEVEL
}

impl<T: PartialOrd> SkipList<T> {
    fn new() -> Self {
        return Self { head: RwLock::new(None) };
    }

    pub fn insert(&self, val: T) {
        let (_now, prev) = self.find_greater_or_equal(&val);

        let height = random_height();
        let nexts = Vec::new();
        let new_node = Arc::new(RwLock::new(Node {
            level: height,
            value: val,
            nexts,
        }));

        for i in 0..height {
            match &prev[i] {
                Some(prev) => {
                    match &prev.read().unwrap().nexts[i] {
                        Some(next) => {
                            new_node.write().unwrap().nexts[i] = Some(next.clone());
                            prev.write().unwrap().nexts[i] = Some(new_node.clone());
                        }
                        None => {
                            // TODO: is this situation special?
                            unreachable!();
                        }
                    }
                }
                None => {
                    // TODO: is this situation special?
                }
            }
        }

        match &*self.head.write().unwrap() {
            None => {
                self.head.write().unwrap().replace(new_node).unwrap();
            }
            Some(_) => {}
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
        match &*self.head.read().unwrap() {
            Some(head) => {
                let mut x = head.clone();

                let mut level = MAX_LEVEL - 1;
                loop {
                    let x_guard = x.read().unwrap();
                    match &x_guard.nexts[level] {
                        Some(next) => {
                            if &next.read().unwrap().value < val {
                                let new_x = next.clone();
                                drop(next);
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
                            if level == 0 {
                                return (None, prev);
                            } else {
                                level -= 1;
                            }
                        }
                    }
                }
            }
            None => (None, prev),
        }
    }
}
