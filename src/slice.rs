use libc::memcmp;
use std::cmp::Ordering;
use std::ptr::null_mut;

pub struct Slice {
    pub data: *mut u8,
    pub size: usize,
}

unsafe impl Send for Slice {}
unsafe impl Sync for Slice {}

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
                memcmp(
                    self.data as *const core::ffi::c_void,
                    other.data as *const core::ffi::c_void,
                    self.size,
                )
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
            libc::memcpy(
                ptr as *mut core::ffi::c_void,
                self.data as *const core::ffi::c_void,
                self.size,
            );
            return Slice {
                data: ptr,
                size: self.size,
            };
        }
    }
}

impl From<String> for Slice {
    fn from(str: String) -> Slice {
        Slice::from(str.into_bytes())
    }
}

impl From<Vec<u8>> for Slice {
    fn from(vec: Vec<u8>) -> Slice {
        let len = vec.len();
        Slice {
            data: (*Box::leak(vec.into_boxed_slice())).as_mut_ptr(),
            size: len,
        }
    }
}

impl Into<Vec<u8>> for Slice {
    fn into(self) -> Vec<u8> {
        unsafe { (*std::slice::from_raw_parts(self.data, self.size)).to_vec() }
    }
}

impl Slice {
    pub fn empty() -> Slice {
        return Slice {
            data: null_mut(),
            size: 0,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn slice_partial_ord() {
        let slice1 = Slice::from(String::from("aaaaaaa"));
        let slice2 = Slice::from(String::from("aaaaaab"));

        assert!(slice1 < slice2);
    }

    #[test]
    fn slice_from_vec() {
        let slice1 = {
            let vec = vec![1, 1, 1, 1, 100];
            Slice::from(vec)
        };

        let slice2 = {
            let vec = vec![1, 1, 1, 1, 101];
            Slice::from(vec)
        };

        assert!(slice1 < slice2)
    }
}
