use std::fmt::Debug;

/// Module that queries Linux about resource usage via the status fd /proc/self/status
pub mod process_info;

/// Debug Info
/// Custom written resource usage tool. Currently only checks the amount of allocated heap space, that has been given to the process
/// since the main function (start). Therefore, we don't know how much heap was allocated to us *prior* to the main function begin running
/// But since that point, we will have an exact measurement of the current heap space.
#[derive(Debug)]
pub struct DebugInfo {
    heap_address_at_main: usize,
    current_heap_address: Option<usize>,
}

impl DebugInfo {
    /// Call this function, at any specific time, to begin measuring *from* that point in real time and execution time how much Heap memory we've acquried by the OS.
    pub fn begin() -> DebugInfo {
        let initial_heap_address = unsafe { libc::sbrk(0) as usize };
        let current_heap_address = Some(initial_heap_address);
        DebugInfo { heap_address_at_main: initial_heap_address, current_heap_address }
    }

    pub fn new(heap_address_at_main: usize) -> DebugInfo {
        DebugInfo { heap_address_at_main, current_heap_address: None }
    }

    pub fn heap_increase_since_start(&mut self) -> usize {
        let current_heap_address = unsafe { libc::sbrk(0) as usize };
        self.current_heap_address = Some(current_heap_address);
        self.current_heap_address.unwrap_or(self.heap_address_at_main) - self.heap_address_at_main
    }
}
