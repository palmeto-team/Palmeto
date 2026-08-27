// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This is really dumb..
//          We don't yet have a heap allocator,
//          So instead we use a static bump allocator..
//

//
// TODO:
//  - COMPLETELY DELETE THIS AFTER MEMORY MANAGEMENT PART
//
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

const STATIC_HEAP_SIZE: usize = 1024 * 1024 * 16;
static STATIC_HEAP: SyncUnsafeCell<[u8; STATIC_HEAP_SIZE]> =
    SyncUnsafeCell(UnsafeCell::new([0; STATIC_HEAP_SIZE]));
static STATIC_HEAP_POS: AtomicUsize = AtomicUsize::new(0);

pub struct StaticBumpAllocator;

unsafe impl GlobalAlloc for StaticBumpAllocator {
    ///
    /// This routine allocates memory from the static heap.
    ///
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        loop {
            let current_pos = STATIC_HEAP_POS.load(Ordering::Relaxed);
            let aligned_pos = (current_pos + align - 1) & !(align - 1);

            if aligned_pos + size > STATIC_HEAP_SIZE {
                return core::ptr::null_mut();
            }

            if STATIC_HEAP_POS
                .compare_exchange_weak(
                    current_pos,
                    aligned_pos + size,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                let heap_ptr = (STATIC_HEAP.0).get() as *mut u8;
                unsafe {
                    return heap_ptr.add(aligned_pos);
                }
            }
        }
    }

    ///
    /// This routine deallocates memory (no-op for bump allocator).
    ///
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }
}

#[global_allocator]
static ALLOCATOR: StaticBumpAllocator = StaticBumpAllocator;