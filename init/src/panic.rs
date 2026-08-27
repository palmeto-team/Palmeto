// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module handles kernel panics
//
use core::panic::PanicInfo;

use pine::{arch::arm64::assembly::instructions, fbcon};

use shared::{core::color, println};

///
/// This routine handles kernel panics,
/// if some bad rust code is called,
/// or the kernel runs into an exception,
/// this routine is called.
/// 
/// It will print information about the panic,
/// and infinitely halt.
///
/// # Arguments
///
/// * info - Information provided by Rust core::
///
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    fbcon::change_screen_color(color::FBCON_COLOR_BLUE);

    let esr = unsafe { instructions::read_esr_el1() };
    let elr = unsafe { instructions::read_elr_el1() };
    let far = unsafe { instructions::read_far_el1() };
    let ec  = (esr >> 26) & 0x3F;

    println!("          KERNEL PANIC            ");
    println!("REASON: {info}");
    println!("EXCEPTION:");
    println!("  ESR_EL1: {esr:#018X} (Class: {ec:#04X})");
    println!("  ELR_EL1: {elr:#018X}");
    println!("  FAR_EL1: {far:#018X}");

    println!("STACK TRACE:");

    let mut fp: u64;

    unsafe {
        core::arch::asm!("mov {}, x29", out(reg) fp);
    }

    for _ in 0..10 {
        if fp == 0 {
            break;
        }
        let prev_fp = unsafe { *(fp as *const u64) };
        let lr = unsafe { *((fp + 8) as *const u64) };

        if lr == 0 {
            break;
        }

        println!("  at {lr:#018X}");
        fp = prev_fp;
    }

    loop {
        core::hint::spin_loop();
    }
}