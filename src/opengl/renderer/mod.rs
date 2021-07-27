pub trait Renderer<T> {
    fn vertex_count(&self) -> usize;
    fn index_count(&self) -> usize;
    fn vertex_data(&self) -> *const T;
    fn index_data(&self) -> *const u32;
    fn set_needs_update(&mut self);
    fn needs_memory(&self) -> bool;

    fn upload_cpu_data(&mut self) {
        unsafe {
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vertex_count() * std::mem::size_of::<T>()) as _, self.vertex_data() as _);
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, (self.index_count() * std::mem::size_of::<u32>()) as _, self.index_data() as _);
        }
        self.set_needs_update();
    }
    /*
    fn reserve_gpu_memory_if_needed(&mut self) {
        if self.needs_memory() {}

        if self.reserved_vertex_count <= self.vtx_data.len() as _ {
            self.reserved_vertex_count = self.vtx_data.capacity() as _;
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<T>() * self.vtx_data.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
            }
        }

        if self.reserved_index_count <= self.indices.len() as _ {
            self.reserved_index_count = self.indices.capacity() as _;
            unsafe {
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<u32>() * self.indices.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
            }
        }
    }
    */
}
