//
// TODO:
//  THIS IS VERY TEMP,
//  UNTIL I GET MEMORY MANAGEMENT SETUP..
//

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

const BLOCK_SIZE: usize = 0x20_0000;
const ENTRY_COUNT: usize = 512;
const EXTRA_TABLE_COUNT: usize = 4;

const VALID: u64 = 1;
const TABLE: u64 = 1 << 1;
const ACCESS: u64 = 1 << 10;
const INNER_SHAREABLE: u64 = 3 << 8;
const DEVICE_MEMORY: u64 = 1 << 2;
const NON_EXECUTABLE: u64 = 1 << 54;
const ADDRESS_MASK: u64 = 0x0000_FFFF_FFFF_F000;

#[repr(C, align(4096))]
struct PageTable([u64; ENTRY_COUNT]);

static mut EXTRA_TABLES: [PageTable; EXTRA_TABLE_COUNT] = [
    PageTable([0; ENTRY_COUNT]),
    PageTable([0; ENTRY_COUNT]),
    PageTable([0; ENTRY_COUNT]),
    PageTable([0; ENTRY_COUNT]),
];

static mut NEXT_TABLE: usize = 0;

unsafe fn table_physical(table: *const PageTable, kernel_physical: u64, kernel_virtual: u64) -> u64 {
    (table as u64).wrapping_sub(kernel_virtual).wrapping_add(kernel_physical)
}

unsafe fn next_table(kernel_physical: u64, kernel_virtual: u64) -> Option<u64> {
    if NEXT_TABLE >= EXTRA_TABLE_COUNT {
        return None;
    }

    let table = addr_of_mut!(EXTRA_TABLES[NEXT_TABLE]);
    NEXT_TABLE += 1;
    for entry in (*table).0.iter_mut() {
        *entry = 0;
    }
    Some(table_physical(table, kernel_physical, kernel_virtual))
}

pub unsafe fn map_device_block(
    physical: usize,
    hhdm: u64,
    kernel_physical: u64,
    kernel_virtual: u64,
) {
    let physical = (physical as u64) & !(BLOCK_SIZE as u64 - 1);
    let virtual_address = hhdm + physical;
    let mut table_physical = read_ttbr1() & ADDRESS_MASK;

    for level in 0..2 {
        let shift = 39 - level * 9;
        let index = ((virtual_address >> shift) & 0x1ff) as usize;
        let table = (hhdm + table_physical) as *mut u64;
        let entry = read_volatile(table.add(index));

        if entry & VALID == 0 {
            let child = next_table(kernel_physical, kernel_virtual)
                .expect("not enough page-table storage for MMIO");
            write_volatile(table.add(index), child | VALID | TABLE);
            table_physical = child;
        } else {
            table_physical = entry & ADDRESS_MASK;
        }
    }

    let shift = 21;
    let index = ((virtual_address >> shift) & 0x1ff) as usize;
    let table = (hhdm + table_physical) as *mut u64;

    if read_volatile(table.add(index)) & VALID != 0 {
        return;
    }

    write_volatile(
        table.add(index),
        physical | VALID | ACCESS | INNER_SHAREABLE | DEVICE_MEMORY | NON_EXECUTABLE,
    );

    flush_translation(virtual_address);
}

unsafe fn read_ttbr1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {0}, ttbr1_el1", out(reg) value);
    value
}

unsafe fn flush_translation(address: u64) {
    core::arch::asm!(
        "dsb ish",
        "tlbi vae1is, {0}",
        "dsb ish",
        "isb",
        in(reg) address >> 12,
    );
}