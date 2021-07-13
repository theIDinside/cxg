#[macro_use]
pub mod macros;


pub fn copy_slice_to<T>(dst: *mut T, slice: &[T]) {
    unsafe {
        std::ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice.len());   
    }
}