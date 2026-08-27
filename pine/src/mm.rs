pub mod memmap;
pub mod freelist;

use spin::Mutex;
use freelist::PhysicalMemoryManager;

pub const PAGE_SIZE: u64 = 4096;

pub static PMM: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl PhysAddr
{
    pub const fn as_u64(self) -> u64 { self.0 }

    pub fn to_virt(self, hhdm_offset: u64) -> VirtAddr
    {
        VirtAddr(self.0 + hhdm_offset)
    }
}

impl VirtAddr
{
    pub const fn as_u64(self) -> u64 { self.0 }

    pub fn to_phys(self, hhdm_offset: u64) -> PhysAddr
    {
        PhysAddr(self.0 - hhdm_offset)
    }

    pub fn as_ptr<T>(self) -> *mut T
    {
        self.0 as *mut T
    }
}

pub fn init()
{
    //
    // Memory Map
    //
    let mut mem_info = memmap::MemoryMapInfo::new();
    mem_info.parse();

    //
    // Physical Memory Manager
    //
    let hhdm_offset = shared::core::requests::HHDM_REQUEST
        .response()
        .map(|r| r.offset)
        .expect("Failed to get hhdm_offset");

    PMM.lock()
       .init(mem_info,hhdm_offset)
       .expect("Failed to initialize Physical Memory Manager");
    
}