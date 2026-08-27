//
// This module provides a freelist allocator for the physical memory manager
// I'm too lazy to write my own,
// so im using the one from evalynOS.
// Licensed under MIT,
// copyright below.
//

/*
    MIT License

    Copyright (c) 2026 Evalyn Goemer & EvalynOS Contributors

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

*/

use core::ptr::null_mut;

use shared::core::
{
    status::{
        KResult, Status
    }, 
    
    utils::{
        align_down, 
        align_up
    }
};

use crate::mm::
{
    PAGE_SIZE, 
    PhysAddr, 
    VirtAddr,
 
    memmap::{
        MemoryMapInfo, 
        RegionType
    }
};

//
// NOTE:
//  I don't think i need #[repr(C)] here?
//  I only did this because i ported the code from C,
//  so just making sure it's aligned.
//
#[repr(C)]
pub struct FreeListNode
{
    next: *mut FreeListNode,
}

pub struct PhysicalMemoryManager
{
    head: *mut FreeListNode,
    fill_entry: usize,
    fill_offset: u64,
    hhdm_offset: u64,
    mem_info: MemoryMapInfo,
}

//
// SAFETY: PhysicalMemoryManager is accessed through a Mutex
// (see mm::PMM).
//
unsafe impl Send for PhysicalMemoryManager{}
unsafe impl Sync for PhysicalMemoryManager{}

impl Default for PhysicalMemoryManager
{
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalMemoryManager
{
    ///
    /// This routine constructs a new PhysicalMemoryManager structure,
    /// all variables will be zeroed / blank,
    /// it's up to init and other functions to fill them.
    ///
    pub const fn new() -> Self
    {
        Self
        {
            head: null_mut(),
            fill_entry: 0,
            fill_offset: 0,
            hhdm_offset: 0,
            mem_info: MemoryMapInfo::new()
        }
    }

    ///
    /// This routine initializes the structure,
    /// including setting it's variables.
    /// 
    /// This routine has a safe-guard on double-calling,
    /// so that's why it returns KResult<>
    /// 
    /// The caller should check the return value,
    /// on Err() you shall probably panic?
    /// As resetting the variables actively would likely break the system.
    ///
    /// # Arguments
    ///
    /// * mem_info    - Memory Map from limine,
    /// * hhdm_offset - Higher Half Direct Map offset
    ///
    pub fn init(&mut self, mem_info: MemoryMapInfo, hhdm_offset: u64) -> KResult<()>
    {
        //
        // init() should not be called twice,
        // otherwise it would reset our tracking..
        // I think this is a decent stop?
        // Just return an error (ALREADY_COMPLETE).
        //
        if self.hhdm_offset != 0
        {
            return Err(Status::ALREADY_COMPLETE);
        }

        //
        // Fresh initialization.
        //
        self.mem_info = mem_info;
        self.hhdm_offset = hhdm_offset;
        self.fill(512)?;

        //
        // No need to return anything,
        // only return Err() for KResult<>
        //
        Ok(())
    }

    ///
    /// This routine refills the free list,
    /// it will walk the ememory map and push physical pages onto head.
    /// 
    /// It will do so one at a time.
    ///
    /// # Arguments
    ///
    /// * pages - Number of pages to add
    ///
    pub fn fill(&mut self, 
                pages: u64) -> KResult<u64>
    {
        //
        // Counter for how many pages were added.a
        //
        let mut pages_added = 0;

        while self.fill_entry < self.mem_info.region_count &&
              pages_added < pages
        {
            let region = match self.mem_info.regions[self.fill_entry]
            {
                Some(r) => r,
                None => {
                    self.fill_entry += 1;
                    self.fill_offset = 0;
                    continue;
                }
            };

            if region.region_type != RegionType::Usable
            {
                self.fill_entry += 1;
                self.fill_offset = 0;
                continue;
            }

            let mut aligned_start = align_up(region.base, PAGE_SIZE);
            let aligned_end       = align_down(region.base + region.length, PAGE_SIZE);

            if aligned_start == 0
            {
                aligned_start = PAGE_SIZE;
            }

            if aligned_end <= aligned_start 
            {
                self.fill_entry += 1;
                self.fill_offset = 0;
                continue;
            }

            if self.fill_offset == 0 
            {
                self.fill_offset = aligned_start - region.base;
            }

            while region.base + self.fill_offset + PAGE_SIZE <= aligned_end && pages_added < pages 
            {
                let page_addr = region.base + self.fill_offset;
                let node_ptr  = (page_addr + self.hhdm_offset) as *mut FreeListNode;
                
                unsafe 
                {
                    (*node_ptr).next = self.head;
                    self.head = node_ptr;
                }

                pages_added += 1;
                self.fill_offset += PAGE_SIZE;
            }

            if region.base + self.fill_offset + PAGE_SIZE > aligned_end 
            {
                self.fill_entry += 1;
                self.fill_offset = 0;
            }
        }

        //
        // If we failed to add a single page,
        // that means we literally have no memory.
        //
        if pages_added == 0
        {
            return Err(Status::NO_MEMORY);
        }

        Ok(pages_added)
    }

    ///
    /// This routine allocates a page in the free-list.
    /// 
    /// The KResult<> return type is used if NO_MEMORY is available,
    /// (otherwise known as self.head.is_null())
    /// 
    /// It Is the callers job to make sure they check the return type.
    /// 
    /// Otherwise, it will return phys (Ok(phys))
    ///
    pub fn allocate_page(&mut self) -> KResult<PhysAddr>
    {
        //
        // 512x4096 = 2MB
        //
        if self.head.is_null()
        {
            self.fill(512)?;
        }

        //
        // If it's still null,
        // we are out of memory.
        //
        if self.head.is_null()
        {
            return Err(Status::NO_MEMORY);
        }

        let node = self.head;
        unsafe
        {
            self.head = (*node).next;
        }

        let phys = VirtAddr(node as u64).to_phys(
            self.hhdm_offset
        );

        unsafe
        {
            core::ptr::write_bytes(node as *mut u8, 0, PAGE_SIZE as usize);
        }

        Ok(phys)
    }
    
    ///
    /// This routine will free a page in the free-list.
    ///
    /// # Arguments
    ///
    /// * phys - Physical address to free.
    ///
    pub fn free_page(&mut self, phys: PhysAddr) -> KResult<()>
    {
        //
        // If the physical address is 0,
        // we have a bad argument.
        //
        if phys.as_u64() == 0
        {
            return Err(Status::INVALID_PARAMETER);
        }

        //
        // If the physical address is not aligned with PAGE_SIZE (4090),
        // we have a bad argument.
        //
        if !phys.as_u64().is_multiple_of(PAGE_SIZE)
        {
            return Err(Status::INVALID_PARAMETER);
        }

        let node = phys.to_virt(self.hhdm_offset).as_ptr::<FreeListNode>();
        unsafe
        {
            (*node).next = self.head;
            self.head = node;
        }

        Ok(())
    }

    pub fn debug_print(&self)
    {
        self.mem_info.debug_print();
    }
}