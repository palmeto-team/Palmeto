// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This is the entry point file for the O/S,
//          It contains "_start"
//          It will handle VERY early initialization

#![no_std]
#![no_main]

//
// !!! MODULES
//
mod panic;
mod relocate;
mod dtbinit;
mod timinit;
mod cmdinit;

//
// !!! KERNEL IMPORTS
//
use pine::fbcon;
use pine::arch;
use pine::mm;

//
// !!! SHARED IMPORTS
//
use shared::core::requests::BASE_REVISION;

///
/// This routine is the very first function called by limine,
/// it will handle very early setup and initializing systems.
///
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    //
    // CHECK BASE REVISION SUPPORT
    //
    if !BASE_REVISION.is_supported() {
        panic!("Limine does not support the requested base revision");
    }

    //
    // RELOCATE INITIALIZATION
    //
    relocate::init();

    //
    // ARCHITECTURE INITALIZATION
    //
    arch::init();

    //
    // FRAMEBUFFER CONSOLE INITIALIZATION
    //
    fbcon::initialize();
    fbcon::reset_display();

    //
    // DEVICE TREE BLOB INITIALIZATION
    //
    dtbinit::init().expect("Failed to initialize DTB...");

    //
    // TIMER INITIALIZATION
    //
    timinit::init_time();
    
    //
    // COMMAND LINE INITIALIZATION
    //
    cmdinit::init();

    //
    // MEMORY MAP
    //
    let mut mem_info = mm::memmap::MemoryMapInfo::new();
    mem_info.parse();
    mem_info.debug_print();
    
    loop {
        core::hint::spin_loop();
    }
}