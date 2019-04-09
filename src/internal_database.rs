use crate::memtable::*;
use crate::slice::Slice;

pub struct InternalDatabase {
    memtable: MemTable 
}

impl InternalDatabase {
    pub fn new() -> InternalDatabase {
        InternalDatabase {
            memtable: MemTable::new()
        }
    }

    pub fn get(&self, key: Slice) -> Slice {
        self.memtable.find(key).unwrap()
    }

    pub fn put(&mut self, key: Slice, val: Slice) {
        self.memtable.insert(key, val)
    }
}
