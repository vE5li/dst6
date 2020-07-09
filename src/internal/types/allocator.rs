pub use std::alloc::{ AllocRef, Layout, Global, AllocInit, handle_alloc_error };
pub use std::ptr::{ NonNull, read, write };
pub use std::mem;

macro_rules! allocate {
    ($type:ty) => ({
        let size = mem::size_of::<$type>();
        let align = mem::align_of::<$type>();
        let layout = Layout::from_size_align(size, align).unwrap();
        let pointer: NonNull<$type> = match Global.alloc(layout, AllocInit::Uninitialized) {
            Ok(block) => block.ptr.cast::<$type>(),
            Err(_) => handle_alloc_error(layout),
        };
        pointer
    });
}

macro_rules! deallocate {
    ($pointer:expr, $type:ty) => ({
        let size = mem::size_of::<$type>();
        let align = mem::align_of::<$type>();
        let layout = Layout::from_size_align(size, align).unwrap();
        Global.dealloc($pointer.cast(), layout);
    });
}
