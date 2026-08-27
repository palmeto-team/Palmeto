// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module handles mmu_state debugging.
//
#![allow(clippy::missing_safety_doc)]

use shared::{debug};

use crate::arch::arm64::assembly::instructions::{
    read_mair_el1,
    read_tcr_el1,
    read_ttbr1_el1
};

///
/// This routine dumps Limines MMU config,
/// this is used to make sure we are right about the configuration.
///
pub unsafe fn dump_mmu_state()
{
    let mair = unsafe { read_mair_el1() };
    let tcr  = unsafe { read_tcr_el1() };
    let ttbr1 = unsafe { read_ttbr1_el1() };

    debug!("MAIR_EL1    = {:#018x}", mair);
    debug!("TCR_EL1     = {:#018x}", tcr);
    debug!("TTBR1_EL1   = {:#018x}", ttbr1);

    let t1sz = (tcr >> 16) & 0x3F;
    let tg1  = (tcr >> 30) & 0x3;
    let ips  = (tcr >> 32) & 0x7;

    let va_bits = 64 - t1sz;
    let granule_kb = match tg1
    {
        0b10 => 4,
        0b01 => 16,
        0b11 => 64,
        _    => 0,
    };

    debug!("T1SZ        = {}", t1sz);
    debug!("VA_BITS     = {}", va_bits);
    debug!("TG1         = {:#04b}",tg1);
    debug!("GRANULE     = {}KB", granule_kb);
    debug!("IPS         = {:#05b}", ips);

    let attr0 = mair & 0xFF;
    let attr1 = (mair >> 8) & 0xFF;

    debug!("MAIR Attr0 (index 0)      = {:#04x}", attr0);
    debug!("MAIR Attr1 (index 1)      = {:#04x}", attr1);

    let table_phys = ttbr1 & 0x0000_FFFF_FFFF_F000;
    debug!("TTBR1 table physical addr = {:#018x}", table_phys);

}