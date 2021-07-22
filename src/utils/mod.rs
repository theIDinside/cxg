#[macro_use]
pub mod macros;

/// Copies slice to memory pointed at by dst.
#[inline(always)]
pub unsafe fn copy_slice_to<T>(dst: *mut T, slice: &[T]) {
    std::ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice.len());
}

pub trait AsUsize {
    fn as_usize(&self) -> usize;
}

impl AsUsize for usize {
    #[inline(always)]
    fn as_usize(&self) -> usize {
        *self
    }
}

/// Calculates difference; |a - b|
pub fn difference<T>(a: T, b: T) -> usize
where
    T: std::ops::Sub<Output = T> + std::ops::SubAssign + std::cmp::PartialEq + std::cmp::PartialOrd + AsUsize,
{
    if a < b {
        (b - a).as_usize()
    } else {
        (a - b).as_usize()
    }
}

pub trait CountDigits {
    fn digits(&self) -> u8;
}

impl CountDigits for usize {
    fn digits(&self) -> u8 {
        let mut value = *self;
        let mut digits = 1;
        if value < 10 {
            digits
        } else {
            while value >= 10 {
                value /= 10;
                digits += 1;
            }
            digits
        }
    }
}

/// Converts a vec of u32 to Vec<char>, unsafely. If you fuck up the code points, it's on you.
pub fn convert_vec_of_u32_utf(data: &[u32]) -> Vec<char> {
    unsafe { data.iter().map(|&c| std::char::from_u32_unchecked(c)).collect() }
}
