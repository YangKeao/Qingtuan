use crate::extend_iter::ExtendIter;
use crate::skiplist::SkipList;
use crate::slice::Slice;
use std::cmp::Ordering;

#[derive(Clone)]
struct Key {
    version_number: u32,
    data: Slice,
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.data.partial_cmp(&other.data).unwrap() {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => {
                if self.version_number == other.version_number {
                    Some(Ordering::Equal)
                } else if self.version_number > other.version_number {
                    // Note: Order of Version Number is unusual because we want the Key with biggest version_number smaller than we provide.
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

#[derive(Clone)]
pub struct Record {
    key: Key,
    value: Slice,
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Default for Record {
    fn default() -> Self {
        Record {
            key: Key {
                data: Slice::empty(),
                version_number: 0,
            },
            value: Slice::empty(),
        }
    }
}

pub struct MemTable {
    data: SkipList<Record>,
}

impl MemTable {
    pub fn new() -> MemTable {
        Self {
            data: SkipList::new(),
        }
    }
    pub fn insert(&mut self, key: Slice, val: Slice) {
        self.data.insert(Record {
            key: Key {
                version_number: 0, // TODO: version management
                data: key,
            },
            value: val,
        });
    }
    pub fn find(&self, key: Slice) -> Option<Slice> {
        let mut list_iter = self.data.iter();
        let res = list_iter.seek(&Record {
            key: Key {
                version_number: 0,
                data: key,
            },
            value: Slice::empty(),
        });
        match res {
            Some(val) => Some(val.value.value.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    extern crate test;
    use super::*;
    use std::ffi::CString;
    use test::Bencher;

    fn slice_from_str(str: &str) -> Slice {
        let len = str.len();
        let str = CString::new(str).unwrap();
        return Slice {
            data: str.into_raw() as *mut u8,
            size: len,
        };
    }

    #[test]
    fn slice_partial_ord() {
        let slice1 = slice_from_str("aaaaaaa");
        let slice2 = slice_from_str("aaaaaab");

        assert!(slice1 < slice2);
    }

    #[test]
    fn table_insert_test() {
        let mut table = MemTable::new();
        for i in 0..1000 {
            table.insert(
                slice_from_str(&format!("{}", i)),
                slice_from_str(&format!("{}", i + 1)),
            );
        }

        for i in 0..1000 {
            let value = table.find(slice_from_str(&format!("{}", i))).unwrap();
            assert!(value == slice_from_str(&format!("{}", i + 1)));
        }
    }

    #[bench]
    fn insert(b: &mut Bencher) {
        let mut table = MemTable::new();
        let mut key: u32 = 0;
        b.iter(move || {
            key += 1;
            table.insert(
                slice_from_str(&format!("{}", key)),
                slice_from_str(&format!("{}", key + 1)),
            );
        });
    }
}
