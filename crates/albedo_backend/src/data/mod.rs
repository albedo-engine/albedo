mod slice;
pub use slice::*;

use bytemuck::Pod;

pub fn reinterpret_vec<T: Pod>(mut v: Vec<T>) -> Vec<u8> {
    unsafe {
        let p = v.as_mut_ptr();
        let count_bytes = v.len() * std::mem::size_of::<T>();
        let capacity_bytes = v.capacity() * std::mem::size_of::<T>();
        std::mem::forget(v);
        Vec::from_raw_parts(p as *mut u8, count_bytes, capacity_bytes)
    }
}
