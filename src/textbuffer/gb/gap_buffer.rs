use std::ops::Range;
use std::ptr::copy as copyrange;
use std::ops::Index;

use crate::textbuffer::{metadata::MetaData};



use super::super::{CharBuffer, SubstringClone};

#[derive(Clone, Copy)]
pub enum Cursor {
    Absolute(usize),
    Buffer
}

// TODO: implement into iterator for gap buffer
// TODO: see above, then implement extend, so that we can do
//      let mut s = String::new();
//      s.extend(self), where self = GapBuffer<char>. The IntoIterator trait, automatically turns self, into an iterator
//      Example of how its normally used:
//      let mut s = String::from("hello ");
//      s.extend(['w','o','r','l','d'].into_iter()); // s now -> "hello world"


#[allow(unused)]
pub struct GapBuffer<T> where T: Clone + Copy {
    data: Vec<T>,
    gap: Range<usize>,
    metadata: MetaData
}

impl <T> GapBuffer<T> where T: Clone + Copy {
    pub fn new() -> GapBuffer<T> {
        GapBuffer {
            data: Vec::new(),
            gap: 0..0,
            metadata: MetaData::new(None)
        }
    }

        /// Returns pointer to begin, and element count up until gap.start, and pointer to where the gap ends in the buffer, and element count until end of buffer
    fn data_pointers_mut(&mut self) -> ((*mut T, usize), (*mut T, usize)) {
        let res = unsafe {
        ((self.data.as_mut_ptr(), self.gap.start),
        (self.data.as_mut_ptr().offset(self.gap.end as isize), self.capacity() - self.gap.end))
        };
        res
    }

    fn data_pointers(&self)  -> ((*const T, usize), (*const T, usize)) {
        let res = unsafe {
        ((self.data.as_ptr(), self.gap.start),
        (self.data.as_ptr().offset(self.gap.end as isize), self.capacity() - self.gap.end))
        };
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


    /// Requires for us to *only* have one gap (which is currently the only feature of this gap buffer. Implementing a multi gap buffer, should probably 
    /// be implemented as a buffer of multiple gap buffers instead, for simplicity, allocated in some form of arena for "cache" locality or at least memory locality in RAM 
    /// (as to prevent page faults)
    pub fn free_space_size(&self) -> usize { self.gap.len() }

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
            unsafe {
                Some(&*self.space(raw))
            }
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
        use std::ptr::copy_nonoverlapping as memcpy;
        if self.gap.len() <= slice.len() {
            self.enlarge_gap_sized(slice.len() * 3);
        }
        unsafe {
            memcpy(slice.as_ptr(), self.data.as_mut_ptr().offset(self.gap.start as isize), slice.len());
        }
        self.gap.start += slice.len();
    }

    pub fn insert_container_data(&mut self, data: &Vec<T>) {
        use std::ptr::copy_nonoverlapping as memcpy;
        if self.gap.len() <= data.len() {
            self.enlarge_gap_sized(data.len());
        }
        unsafe {
            memcpy(data.as_ptr(), self.data.as_mut_ptr().offset(self.gap.start as isize), data.len());
        }
        self.gap.start += data.len();
    }

    pub fn insert_slice_data(&mut self, data: &[T]) {
        use std::ptr::copy_nonoverlapping as memcpy;
        if self.gap.len() <= data.len() {
            self.enlarge_gap_sized(data.len());
        }
        unsafe {
            memcpy(data.as_ptr(), self.data.as_mut_ptr().offset(self.gap.start as isize), data.len());
        }
        self.gap.start += data.len();
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

    pub fn map_into<Iter>(&mut self, iterable: Iter) where Iter: IntoIterator<Item=T> {
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
        let e = unsafe {
            std::ptr::read(self.space(self.gap.end))
        };
        self.gap.end += 1;
        Some(e)
    }

    /**
    Works like the "backspace" key when you edit text. It deletes what is BEFORE the cursor. (gap.start - 1)
    */
    pub fn remove(&mut self) -> Option<T> {
        if self.gap.start == 0 {
            return None;
        }
        let e = unsafe {
            std::ptr::read(self.space(self.gap.start - 1))
        };
        self.gap.start -= 1;
        Some(e)
    }

    fn enlarge_gap(&mut self) {
        use std::ptr::copy_nonoverlapping as copyNoOverlap;
        let mut newcap = self.capacity() * 2;
        if newcap == 0 {
            // existing vector data is empty.. choosing 16 bytes = 128 bit gap. Perhaps this will optimize string copies using AVX? We'll see.
            newcap = 16;
        }

        let mut newbuf = Vec::with_capacity(newcap);
        let aftergap = self.capacity() - self.gap.end;
        let newgap = self.gap.start .. newbuf.capacity() - aftergap;
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
        use std::ptr::copy_nonoverlapping as memcpy;
        let mut newbuf: Vec<T> = Vec::with_capacity(self.capacity() + added_gap_size);
        let ((src_a, elem_count_a), (src_b, elem_count_b)) = self.data_pointers_mut();
        let newgap = self.gap.start .. newbuf.capacity() - elem_count_b;
        unsafe {            
            memcpy(src_a, newbuf.as_mut_ptr(), elem_count_a);
            let newgap_end = newbuf.as_mut_ptr().offset(newgap.end as isize);
            memcpy(src_b, newgap_end, elem_count_b);
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
        GapBufferIterator {
            pos: 0,
            end: pos,
            buffer: self
        }
    }

    pub fn iter_cursor_to_end(&self, cursor: Cursor) -> GapBufferIterator<T> {
        let pos = match cursor {
            Cursor::Absolute(pos) => pos,
            Cursor::Buffer => self.get_pos(),
        };
        GapBufferIterator {
            pos,
            end: self.len(),
            buffer: self
        }
    }

    pub fn iter(&self) -> GapBufferIterator<T> {
        GapBufferIterator {
            pos: 0,
            end: self.len(),
            buffer: self
        }
    }
}

impl GapBuffer<char> {

    pub fn debug(&self) {
        let (a, b) = self.data_slices();
        println!("{}", a.iter().map(|v| *v).chain(self.gap.clone().map(|_| '-')).chain(b.iter().map(|v| *v)).collect::<String>());
    }
}

pub struct GapBufferIterator<'a, T> where T: Clone + Copy {
    pos: usize,
    end: usize,
    buffer: &'a GapBuffer<T>
}

impl<'a, T> Iterator for GapBufferIterator<'a, T> where T: Clone + Copy {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.buffer.len() {
            self.pos += 1;
            self.buffer.get(self.pos - 1)
        } else {
            None
        }
    }

    fn position<P>(&mut self, mut predicate: P) -> Option<usize> where
        Self: Sized,
        P: FnMut(Self::Item) -> bool, {
        while let Some(ch) = self.next() {
         if predicate(ch) {
             return Some(self.pos);
         }
        }
        None
    }

    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize> where P: FnMut(Self::Item) -> bool, Self: ExactSizeIterator + DoubleEndedIterator, {
        while let Some(ch) = self.next_back() {
            if predicate(ch) {
                return Some(self.end)
            }
        }
        None
    }
}

impl<'a, T> DoubleEndedIterator for GapBufferIterator<'a, T> where T: Clone + Copy {
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

impl<'a> CharBuffer<'a> for GapBuffer<char> {
    type ItemIterator = GapBufferIterator<'a, char>;

    fn insert(&mut self, data: char) {
        self.insert_item(data);
    }

    fn insert_slice_fast(&mut self, slice: &[char]) {
        if self.available_space() < slice.len() {
            // todo(verify, optimize): I remember reading somewhere that increasing with 1.3 is actually the optimized factor, due to "crawl" in memory when reallocating...
            // this is however just "from memory" (pun intended). I have no data to suggest this is true, but for now, let's just keep it like this
            let new_cap = (self.capacity() as f32 * 1.5 + slice.len() as f32).ceil() as usize;
            let new_gap_len = new_cap - self.len();
            let gap = self.gap.start .. self.gap.start + new_gap_len;
            let mut new_storage = Vec::<char>::with_capacity(new_cap);
            let (sub_slice_a, sub_slice_b) = self.data_slices();
            unsafe {
                std::ptr::copy_nonoverlapping(sub_slice_a.as_ptr(), new_storage.as_mut_ptr(), sub_slice_a.len());
                std::ptr::copy_nonoverlapping(slice.as_ptr(), new_storage.as_mut_ptr().offset(sub_slice_a.len() as _), slice.len());
                std::ptr::copy_nonoverlapping(sub_slice_b.as_ptr(), new_storage.as_mut_ptr().offset(gap.end as _), sub_slice_b.len());
                new_storage.set_len(42);
            }
            self.gap = gap;
            self.gap.start += slice.len();
            self.data = new_storage;
        } else {
            for item in slice {
                self.insert_item(*item);
            }
        };        
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }

    fn rebuild_metadata(&mut self) {
        todo!("no metadata functionality exists yet for GapBuffer");
    }

    fn delete(&mut self, dir: crate::textbuffer::Movement) {
        let _ = dir;
        todo!()
    }

    fn meta_data(&self) -> &MetaData {
        todo!("no metadata functionality exists yet for GapBuffer");
    }

    fn iter(&'a self) -> Self::ItemIterator {
        GapBufferIterator {
            pos: 0,
            end: self.len(),
            buffer: self
        }
    }

}

impl<T> Drop for GapBuffer<T> where T: Clone + Copy {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.gap.start {
                std::ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end .. self.capacity() {
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