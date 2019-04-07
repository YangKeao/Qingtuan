use crate::skiplist::SkipList;
use std::cmp::Ordering;
use libc::memcmp;

pub(crate) struct Slice {
    pub(crate) data: *mut u8,
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

struct Key {
    version_number: u32,
    data: Slice,
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

pub struct Record {
    key: Key,
    value: Slice
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp( other) == Some(Ordering::Equal)
    }
}

pub struct MemTable {
    data: SkipList<Record>
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
}