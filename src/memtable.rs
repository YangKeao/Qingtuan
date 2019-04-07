use crate::skiplist::SkipList;
use std::cmp::Ordering;
use libc::memcmp;
use std::sync::{RwLock, Arc};
use crate::extend_iter::ExtendIter;
use std::ptr::null_mut;

pub struct Slice {
    pub data: *mut u8,
    size: usize,
}

impl Drop for Slice {
    fn drop(&mut self) {
        let data = unsafe { std::slice::from_raw_parts(self.data, self.size) };
        drop(data);
    }
}

impl PartialOrd for Slice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.size < other.size {
            return Some(Ordering::Less);
        } else if self.size == other.size {
            let res = unsafe {
                memcmp(self.data as *const core::ffi::c_void, other.data as *const core::ffi::c_void, self.size)
            };
            if res == 0 {
                return Some(Ordering::Equal);
            } else if res < 0 {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Greater);
            }
        } else {
            return Some(Ordering::Greater);
        }
    }
}

impl PartialEq for Slice {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Clone for Slice {
    fn clone(&self) -> Self {
        unsafe {
            let mut mem = Box::new(Vec::with_capacity(self.size));
            let ptr = mem.as_mut_ptr();
            Box::leak(mem);
            libc::memcpy(ptr as *mut core::ffi::c_void, self.data as *const core::ffi::c_void, self.size);
            return Slice {
                data: ptr,
                size: self.size
            }
        }
    }
}

impl Slice {
    fn empty() -> Slice {
        return Slice {
            data: null_mut(),
            size: 0,
        }
    }
}

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
                } else if self.version_number < other.version_number {
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
    value: Slice
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp( other) == Some(Ordering::Equal)
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
    data: Arc<RwLock<SkipList<Record>>>
}

impl MemTable {
    pub fn new() -> MemTable {
        Self {
            data: Arc::new(RwLock::new(SkipList::new()))
        }
    }
    pub fn insert(&mut self, key: Slice, val: Slice) {
        self.data.write().unwrap().insert(Record {
            key: Key {
                version_number: 0, // TODO: version management
                data: key
            },
            value: val
        });
    }
    pub fn find(&self, key: Slice) -> Option<Slice> {
        let mut list_iter = self.data.read().unwrap().iter();
        let res = list_iter.seek(&Record {
            key: Key {
                version_number: 0,
                data: key
            },
            value: Slice::empty()
        });
        match res {
            Some(val) => {
                Some(val.read().unwrap().value.value.clone())
            }
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::CString;

    fn slice_from_str(str: &str) -> Slice {
        let len = str.len();
        let mut str = CString::new(str).unwrap();
        return Slice {
            data: str.into_raw() as *mut u8,
            size: len,
        }
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
            table.insert(slice_from_str(&format!("{}", i)), slice_from_str(&format!("{}", i+1)));
        }

        for i in 0..1000 {
            let value = table.find(slice_from_str(&format!("{}", i))).unwrap();
            assert!(value == slice_from_str(&format!("{}", i+1)));
        }
    }
}