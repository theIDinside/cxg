#[macro_use]
pub mod macros;

/// Copies slice to memory pointed at by dst. 
#[inline(always)]
pub unsafe fn copy_slice_to<T>(dst: *mut T, slice: &[T]) {
    std::ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice.len());   
}