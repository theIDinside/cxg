use crate::utils::copy_slice_to;
use std::ops::Index;
use std::ops::Range;
use std::ptr::copy as copyrange;

use super::super::SubstringClone;

#[derive(Clone, Copy)]
pub enum Cursor {
    Absolute(usize),
    Buffer,
}

// TODO: implement into iterator for gap buffer
// TODO: see above, then implement extend, so that we can do
//      let mut s = String::new();
//      s.extend(self), where self = GapBuffer<char>. The IntoIterator trait, automatically turns self, into an iterator
//      Example of how its normally used:
//      let mut s = String::from("hello ");
//      s.extend(['w','o','r','l','d'].into_iter()); // s now -> "hello world"

#[allow(unused)]
pub struct GapBuffer<T>
where
    T: Clone + Copy,
{
    data: Vec<T>,
    gap: Range<usize>,
}

impl<T> GapBuffer<T>
where
    T: Clone + Copy,
{
    pub fn new() -> GapBuffer<T> {
        GapBuffer { data: Vec::new(), gap: 0..0 }
    }
    /// Returns pointer to begin, and element count up until gap.start, and pointer to where the gap ends in the buffer, and element count until end of buffer
    fn data_pointers_mut(&mut self) -> ((*mut T, usize), (*mut T, usize)) {
        let res = unsafe { ((self.data.as_mut_ptr(), self.gap.start), (self.data.as_mut_ptr().offset(self.gap.end as isize), self.capacity() - self.gap.end)) };
        res
    }

    fn data_pointers(&self) -> ((*const T, usize), (*const T, usize)) {
        let res = unsafe { ((self.data.as_ptr(), self.gap.start), (self.data.as_ptr().offset(self.gap.end as isize), self.capacity() - self.gap.end)) };
        res
    }

    /// Slices is much easier for us to work with, from the "customer" perspective (i.e my program that uses this buffer)
    pub fn data_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        let ((a, alen), (b, blen)) = self.data_pointers_mut();
        let a = unsafe { std::slice::from_raw_parts_mut(a, alen) };
        let b = unsafe { std::slice::from_raw_parts_mut(b, blen) };
        (a, b)
    }

    pub fn data_slices(&self) -> (&[T], &[T]) {
        let ((a, alen), (b, blen)) = self.data_pointers();
        let a = unsafe { std::slice::from_raw_parts(a, alen) };
        let b = unsafe { std::slice::from_raw_parts(b, blen) };
        (a, b)
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }

    pub fn gap(&self) -> std::ops::Range<usize> {
        self.gap.clone()
    }

    /// Requires for us to *only* have one gap (which is currently the only feature of this gap buffer. Implementing a multi gap buffer, should probably
    /// be implemented as a buffer of multiple gap buffers instead, for simplicity, allocated in some form of arena for "cache" locality or at least memory locality in RAM
    /// (as to prevent page faults)
    pub fn free_space_size(&self) -> usize {
        self.gap.len()
    }

    pub fn get_pos(&self) -> usize {
        self.gap.start
    }

    // unsafe fn. Should only be called after index has been determined to be within valid range.
    unsafe fn space(&self, index: usize) -> *const T {
        self.data.as_ptr().offset(index as isize)
    }

    unsafe fn space_mut(&mut self, index: usize) -> *mut T {
        self.data.as_mut_ptr().offset(index as isize)
    }

    fn index_to_raw(&self, index: usize) -> usize {
        if index < self.gap.start {
            index
        } else {
            index + self.gap.len()
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let raw = self.index_to_raw(index);
        if raw < self.capacity() {
            unsafe { Some(&*self.space(raw)) }
        } else {
            None
        }
    }

    pub fn set_gap_position(&mut self, pos: usize) {
        if pos != self.gap.start {
            if pos > self.len() {
                panic!("GapBuffer index {} out of bounds", pos);
            }
            unsafe {
                let gap = self.gap.clone();
                if pos > gap.start {
                    let distance = pos - gap.start;
                    copyrange(self.space(gap.end), self.space_mut(gap.start), distance);
                } else if pos < gap.start {
                    let distance = gap.start - pos;
                    copyrange(self.space(pos), self.space_mut(gap.end - distance), distance);
                }
                self.gap = pos..pos + gap.len();
            }
        }
    }

    pub fn insert_slice(&mut self, slice: &[T]) {
        if self.gap.len() <= slice.len() {
            self.enlarge_gap_sized(slice.len() * 3);
        }
        unsafe {
            let destination = self.data.as_mut_ptr().offset(self.gap.start as isize);
            copy_slice_to(destination, slice);
        }
        self.gap.start += slice.len();
    }

    pub fn insert_item(&mut self, elem: T) {
        if self.gap.len() == 0 {
            self.enlarge_gap();
        }

        unsafe {
            let index = self.gap.start;
            std::ptr::write(self.space_mut(index), elem);
        }
        self.gap.start += 1;
    }

    pub fn map_into<Iter>(&mut self, iterable: Iter)
    where
        Iter: IntoIterator<Item = T>,
    {
        for item in iterable {
            self.insert_item(item);
        }
    }

    /**
    Works like the "delete" key when you edit text. It deletes what is AFTER the cursor. (gap.end + 1)
    */
    pub fn delete(&mut self) -> Option<T> {
        if self.gap.end == self.capacity() {
            return None;
        }
        let e = unsafe { std::ptr::read(self.space(self.gap.end)) };
        self.gap.end += 1;
        Some(e)
    }

    /// Erases data in the range text_range.start .. end, in text representational terms
    ///  
    pub fn erase(&mut self, text_range: std::ops::Range<usize>) {
        debug_assert!(text_range.end <= self.len(), "you can't erase data not contained by this buffer");
        let len = text_range.len();
        self.set_gap_position(text_range.start);
        self.gap.end += len;
    }

    /**
    Works like the "backspace" key when you edit text. It deletes what is BEFORE the cursor. (gap.start - 1)
    */
    pub fn remove(&mut self) -> Option<T> {
        if self.gap.start == 0 {
            return None;
        }
        let e = unsafe { std::ptr::read(self.space(self.gap.start - 1)) };
        self.gap.start -= 1;
        Some(e)
    }

    fn enlarge_gap(&mut self) {
        use std::ptr::copy_nonoverlapping as copyNoOverlap;
        // a growth factor of < 1.5 is preferable to prevent "memory crawl" and being able to re-use previously freed space
        let mut newcap = (self.capacity() as f32 * 1.40).round() as usize;
        if newcap == 0 {
            // existing vector data is empty.. choosing 16 bytes = 128 bit gap. Perhaps this will optimize string copies using AVX? We'll see.
            newcap = 512;
        }

        let mut newbuf = Vec::with_capacity(newcap);
        let aftergap = self.capacity() - self.gap.end;
        let newgap = self.gap.start..newbuf.capacity() - aftergap;
        unsafe {
            copyNoOverlap(self.space(0), newbuf.as_mut_ptr(), self.gap.start);
            let newgap_end = newbuf.as_mut_ptr().offset(newgap.end as isize);
            copyNoOverlap(self.space(self.gap.end), newgap_end, aftergap);
        }
        self.data = newbuf;
        self.gap = newgap;
    }

    /// Enlarge gap with added_gap_size elements
    /// Problem domain:
    ///     First, calculate new gap length, which is current len + added_size
    ///     Create a new buffer, with capacity of self.capacity() + added_size
    ///     copy everying from 0 .. gap.start to new buffer [0 .. gap.start]
    ///     copy everything from self.data[gap.end .. capacity()] to new_buffer[gap.end + added_size .. new_buffer.capacity()]
    ///     we have now a new buffer, with a gap len of gap.len() + added_size and all the contents of the old buffer copied
    pub fn enlarge_gap_sized(&mut self, added_gap_size: usize) {
        let mut newbuf: Vec<T> = Vec::with_capacity(self.capacity() + added_gap_size);
        // let ((src_a, elem_count_a), (src_b, elem_count_b)) = self.data_pointers_mut();
        let tmp = self.gap.clone();
        let (ssrc_a, ssrc_b) = self.data_slices_mut();
        let newgap = tmp.start..(tmp.end + added_gap_size);
        // self.gap.start..newbuf.capacity() - ssrc_b.len() was how we previously calculated new gap. it should be identical
        assert_eq!(tmp.start..newbuf.capacity() - ssrc_b.len(), newgap, "calculation of new gap error");

        unsafe {
            // memcpy(src_a, newbuf.as_mut_ptr(), elem_count_a);
            copy_slice_to(newbuf.as_mut_ptr(), ssrc_a);
            let newgap_end = newbuf.as_mut_ptr().offset(newgap.end as isize);
            // memcpy(src_b, newgap_end, elem_count_b);
            copy_slice_to(newgap_end, ssrc_b);
        }
        self.data = newbuf;
        self.gap = newgap;
    }

    pub fn new_with_capacity(cap: usize) -> GapBuffer<T> {
        let mut gb = GapBuffer::new();
        let buf = Vec::with_capacity(cap);
        gb.gap = 0..cap;
        gb.data = buf;
        gb
    }

    pub fn iter_begin_to_cursor(&self, cursor: Cursor) -> GapBufferIterator<T> {
        let pos = match cursor {
            Cursor::Absolute(pos) => pos,
            Cursor::Buffer => self.get_pos(),
        };
        GapBufferIterator { pos: 0, end: pos, buffer: self }
    }

    pub fn iter_cursor_to_end(&self, cursor: Cursor) -> GapBufferIterator<T> {
        let pos = match cursor {
            Cursor::Absolute(pos) => pos,
            Cursor::Buffer => self.get_pos(),
        };
        GapBufferIterator { pos, end: self.len(), buffer: self }
    }

    pub fn iter(&self) -> GapBufferIterator<T> {
        GapBufferIterator { pos: 0, end: self.len(), buffer: self }
    }
}

impl GapBuffer<char> {
    pub fn debug(&self) {
        let (a, b) = self.data_slices();
        println!(
            "{}",
            a.iter()
                .map(|v| *v)
                .chain(self.gap.clone().map(|_| '-'))
                .chain(b.iter().map(|v| *v))
                .collect::<String>()
        );
    }
}

pub struct GapBufferIterator<'a, T>
where
    T: Clone + Copy,
{
    pos: usize,
    end: usize,
    buffer: &'a GapBuffer<T>,
}

impl<'a, T> ExactSizeIterator for GapBufferIterator<'a, T>
where
    T: Clone + Copy,
{
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        // Note: This assertion is overly defensive, but it checks the invariant
        // guaranteed by the trait. If this trait were rust-internal,
        // we could use debug_assert!; assert_eq! will check all Rust user
        // implementations too.
        assert_eq!(upper, Some(lower));
        lower
    }
}

impl<'a, T> Iterator for GapBufferIterator<'a, T>
where
    T: Clone + Copy,
{
    type Item = &'a T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.pos, Some(self.end - self.pos))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.buffer.len() {
            self.pos += 1;
            self.buffer.get(self.pos - 1)
        } else {
            None
        }
    }

    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        while let Some(ch) = self.next() {
            if predicate(ch) {
                return Some(self.pos);
            }
        }
        None
    }

    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
        Self: ExactSizeIterator + DoubleEndedIterator,
    {
        while let Some(ch) = self.next_back() {
            if predicate(ch) {
                return Some(self.end);
            }
        }
        None
    }
}

impl<'a, T> DoubleEndedIterator for GapBufferIterator<'a, T>
where
    T: Clone + Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end >= self.pos {
            if let Some(c) = self.buffer.get(self.end) {
                self.end -= 1;
                Some(c)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Index<usize> for GapBuffer<char> {
    type Output = char;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> Drop for GapBuffer<T>
where
    T: Clone + Copy,
{
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end..self.capacity() {
                std::ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}

impl SubstringClone for GapBuffer<char> {
    fn read_string(&self, range: std::ops::Range<usize>) -> String {
        self.iter().skip(range.start).take(range.len()).collect()
    }
}
