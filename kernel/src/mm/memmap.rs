// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module parses the limine memory map.
//
//          Limine will provide us with something called a 'memory map',
//          this is a list that tells us which memory address (physical) are safe,
//          
//          There is 8 memory types,
//
//          USABLE                  - Free Physical Ram
//          RESERVED                - Off-Limits (used for UEFI/BIOS)
//          ACPI_RECLAIMABLE        - ACPI Configuration Tables
//          ACPI_NVS                - ACPI Non-Volatile Sleeping Memory
//          BAD_MEMORY              - Broken/DeFectiuve RAM (built-in checks by the motherboard)
//          BOOTLOADER_RECLAIMABLE  - Memory Being Used by Limine
//          EXECUTABLES_AND_MODULES - Location Where Limine Loaded The Kernel
//          FRAMEBUFFER             - Memory Buffer That Holds Pixels Being Displayed On The Monitor
//
use shared::core::requests::MEMMAP_REQUEST;
use shared::println;

use limine::memmap;

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion
{
    pub base: u64,
    pub length: u64,
    pub region_type: RegionType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootloaderReclaimable,
    ExecutablesAndModules,
    Framebuffer,
    Other,
}

pub struct MemoryMapInfo {
    pub regions: [Option<MemoryRegion>; 64],
    pub region_count: usize,
}

impl MemoryMapInfo
{
    pub const fn new() -> Self
    {
        Self {
            regions: [None; 64],
            region_count: 0
        }
    }

    pub fn parse(&mut self)
    {
        if let Some(response) = MEMMAP_REQUEST.response() {
            let entries = response.entries();

            for entry in entries {
                if self.region_count >= self.regions.len() {
                    break;
                }

                let region_type = match entry.type_ {
                    memmap::MEMMAP_USABLE => RegionType::Usable,
                    memmap::MEMMAP_RESERVED => RegionType::Reserved,
                    memmap::MEMMAP_ACPI_RECLAIMABLE => RegionType::AcpiReclaimable,
                    memmap::MEMMAP_ACPI_NVS => RegionType::AcpiNvs,
                    memmap::MEMMAP_BAD_MEMORY => RegionType::BadMemory,
                    memmap::MEMMAP_BOOTLOADER_RECLAIMABLE => RegionType::BootloaderReclaimable,
                    memmap::MEMMAP_EXECUTABLE_AND_MODULES => RegionType::ExecutablesAndModules,
                    memmap::MEMMAP_FRAMEBUFFER => RegionType::Framebuffer,
                    _ => RegionType::Other,
                };

                self.regions[self.region_count] = Some(MemoryRegion {
                    base: entry.base,
                    length: entry.length,
                    region_type,
                });

                self.region_count += 1;
            }
        }
    }

    pub fn debug_print(&self)
    {
        for i in 0..self.region_count
        {
            if let Some(region) = self.regions[i] 
            {
                println!(
                    "Base: 0x{:016x} | Length: 0x{:08x} | Type: {:?}",
                    region.base,
                    region.length,
                    region.region_type
                )
            }
        }
    }
}

impl Default for MemoryMapInfo
{
    fn default() -> Self
    {
        Self::new()
    }
}