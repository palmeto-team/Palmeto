// SPDX-License-Identifier: GPL-3.0-or-later
//
// Purpose: This module handles early initialization,
//          such as converting the memory map,
//          storing BootInformation,
//          and more.
//
//          Requests are stored in requests.rs,
//          but this file will interact with them.
//
#![allow(dead_code)]
#![allow(unused)]


use limine::{framebuffer::Framebuffer, 
             memmap, 
             paging::PagingMode
            };

use spin::{Mutex, Once};

use kernel::mm::mmdat::{PhysMemory, 
                   PhysMemoryUsage};

use shared::{core::requests::{CMDLINE_REQUEST, 
                              DTB_REQUEST, 
                              HHDM_REQUEST, 
                              KERNEL_ADDR_REQUEST, 
                              MEMMAP_REQUEST, 
                              MODULE_REQUEST, 
                              PAGING_REQUEST, 
                              RSDP_REQUEST}};

use kernel::{cmdline::CmdLine, mm::{PhysAddr, VirtAddr}};

#[derive(Debug)]
pub struct BootInfo {

    pub command_line: CmdLine<'static>,

    pub files: &'static [BootFile],

    pub hhdm_address: Option<VirtAddr>,

    pub paging_level: Option<usize>,

    pub memory_map: Mutex<&'static mut [PhysMemory]>,

    pub highest_phys: Option<PhysAddr>,

    pub kernel_phys: Option<PhysAddr>,

    pub kernel_virt: Option<VirtAddr>,

    pub rsdp_addr: Option<PhysAddr>,

    pub fdt_addr: Option<PhysAddr>,
}

static BOOT_INFO: Once<BootInfo> = Once::new();

impl BootInfo {
    ///
    /// This routine constructs a new BootInfo struct,
    /// the structure will be completely empty,
    /// so you do not need to worry about passing args.
    ///
    pub const fn new() -> Self {
        Self {
            command_line: CmdLine::new(""),
            files: &[],
            hhdm_address: None,
            paging_level: None,
            memory_map: Mutex::new(&mut []),
            highest_phys: None,
            kernel_phys: None,
            kernel_virt: None,
            rsdp_addr: None,
            fdt_addr: None
        }
    }

    ///
    /// This routine will register BOOT_INFO
    ///
    pub fn register(self) {
        BOOT_INFO.call_once(|| self);
    }

    ///
    /// This routine will attempt to get the BOOT_INFO,
    /// If it fails it will throw an error,
    /// If the operation is successful,
    /// It will give you the BOOT_INFO structure
    ///
    pub fn get() -> &'static Self {
        BOOT_INFO.get().expect("boot info not initialized")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BootFile {
    pub data: PhysAddr,
    pub length: usize,
    pub name: &'static str,
}

impl BootFile {
    ///
    /// This routine constructs a new BootFile structure,
    /// The structure will contain empty values,
    /// you must fill it later
    ///
    pub const fn new() -> Self {
        Self {
            data: PhysAddr::null(),
            length: 0,
            name: "",
        }
    }
}

static mut MEMMAP_BUF: [PhysMemory; 128] = [PhysMemory::empty(); _];
static mut FILE_BUF:   [BootFile; 32]    = [BootFile::new(); _];

const      STRING_BUF_LEN: usize = 2048;
static mut CMDLINE_BUF:    [u8; STRING_BUF_LEN] = [0; _];
static mut FILE_NAME_BUF:  [u8; STRING_BUF_LEN] = [0; _];

///
/// This routine constructs a new BootInfo structure,
/// fills it's parameters,
/// then registers it as the global (BOOT_INFO) structure.
/// 
/// This routine must be called,
/// otherwise BOOT_INFO will not exist
///
pub fn fill_info()
{
    let mut info = BootInfo::new();
    {
        //
        // Convert the memory map,
        // 128 entries
        //
        let mut entries = MEMMAP_REQUEST.response().unwrap().entries();
        let mut total_entries = 0;

        entries.iter().enumerate().for_each(|(i, entry)| unsafe 
        {

            MEMMAP_BUF[i] = PhysMemory 
            {
                length: entry.length as usize,
                address: entry.base.into(),
                usage: match entry.type_ 
                {
                    memmap::MEMMAP_USABLE => PhysMemoryUsage::Usable,
                    memmap::MEMMAP_BOOTLOADER_RECLAIMABLE | memmap::MEMMAP_EXECUTABLE_AND_MODULES => {
                        PhysMemoryUsage::Reclaimable
                    }
                    _ => PhysMemoryUsage::Reserved,
                },
            };
            
            total_entries += 1;
        });

        info.highest_phys = Some({
            let last = entries.iter().last().unwrap();
            (last.base + last.length).into()
        });

        let kernel_addr = KERNEL_ADDR_REQUEST.response().unwrap();

        info.hhdm_address = Some(HHDM_REQUEST.response().unwrap().offset.into());

        let paging = PAGING_REQUEST.response().unwrap().mode;

        {
            match paging {
                PagingMode::AARCH64_4LVL => info.paging_level = Some(4),
                PagingMode::AARCH64_5LVL => info.paging_level = Some(5),
                _ => {}
            }
        }

        unsafe {
            info.memory_map = Mutex::new(&mut MEMMAP_BUF[0..total_entries]);
        }

        info.kernel_phys = Some(kernel_addr.physical_base.into());
        info.kernel_virt = Some(kernel_addr.virtual_base.into());
    }

    info.command_line = unsafe {
        let line = CMDLINE_REQUEST.response().unwrap().cmdline();
        let len = line.len().min(STRING_BUF_LEN);
        let buf = &raw mut CMDLINE_BUF as *mut u8;
        core::ptr::copy_nonoverlapping(line.as_ptr().cast(), buf, len);
        CmdLine::new(str::from_utf8(core::slice::from_raw_parts(buf, len)).unwrap_or_default())
    };

    info.rsdp_addr = RSDP_REQUEST.response().and_then(|x| {
        let hhdm = info.hhdm_address.unwrap().value();
        let addr = x.address as usize;
        addr.checked_sub(hhdm).map(PhysAddr::from)
    });

    info.fdt_addr = DTB_REQUEST.response().and_then(|x| {
        let hhdm = HHDM_REQUEST.response().unwrap().offset as usize;
        let addr = x.dtb_ptr as usize;
        addr.checked_sub(hhdm).map(PhysAddr::from)
    });

    if let Some(response) = MODULE_REQUEST.response() 
    {
        let mut name_offset = 0;

        for (i, entry) in response.modules().iter().enumerate() 
        {
            unsafe 
            {
                let name = entry.path().rsplit_once('/').unwrap().1;
                assert!(name_offset + name.len() <= STRING_BUF_LEN);

                let copied = (&raw mut FILE_NAME_BUF as *mut u8).add(name_offset);
                core::ptr::copy_nonoverlapping(name.as_ptr(), copied, name.len());
                
                name_offset += name.len();

                let file_data = entry.data();
                let file_ptr = file_data.as_ptr() as usize;
                let hhdm = info.hhdm_address.unwrap().value();
                FILE_BUF[i] = BootFile 
                {
                    data: file_ptr
                        .checked_sub(hhdm)
                        .unwrap_or(file_ptr)
                        .into(),

                    length: file_data.len(),
                    name: str::from_utf8_unchecked(core::slice::from_raw_parts(copied, name.len())),
                }
            };
        }
        unsafe {
            info.files = &FILE_BUF[0..response.modules().len()];
        }
    }

    info.register();

}