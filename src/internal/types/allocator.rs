pub use std::alloc::{ GlobalAlloc, Layout, handle_alloc_error };
pub use std::ptr::{ NonNull, read, write };
pub use std::mem;
pub use std::alloc::System;

macro_rules! allocate {
    ($type:ty) => ({
        let size = mem::size_of::<$type>();
        let align = mem::align_of::<$type>();
        let layout = Layout::from_size_align(size, align).unwrap();
        let raw_pointer = unsafe { System.alloc(layout) as *mut $type };
        let pointer: NonNull<$type> = NonNull::new(raw_pointer).unwrap();
        pointer
    });
}

macro_rules! deallocate {
    ($pointer:expr, $type:ty) => ({
        let size = mem::size_of::<$type>();
        let align = mem::align_of::<$type>();
        let layout = Layout::from_size_align(size, align).unwrap();
        System.dealloc($pointer.as_ptr() as *mut u8, layout);
    });
}
