// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This file uses raw assembly instructions,
//          wrapping them in functions,
//          AARCH64 instructions are a little weird..
//          I'm used to X86_64...
//

#![allow(unused)]
#![allow(clippy::missing_safety_doc)]

use core::arch::{asm, global_asm};

pub struct ArmCpuIdResult {
    pub midr: u64,
    pub mpidr: u64,
    pub id_aa64pfr0: u64,
}

///
/// This routine reads CPU feature registers.
///
#[inline]
pub unsafe fn read_cpu_features() -> ArmCpuIdResult {
    let midr: u64;
    let mpidr: u64;
    let id_aa64pfr0: u64;
    unsafe {
        asm!(
            "mrs {0}, midr_el1",
            "mrs {1}, mpidr_el1",
            "mrs {2}, id_aa64pfr0_el1",
            out(reg) midr,
            out(reg) mpidr,
            out(reg) id_aa64pfr0,
        );
    }
    ArmCpuIdResult { midr, mpidr, id_aa64pfr0 }
}

///
/// This routine reads the system control register EL1.
///
#[inline]
pub unsafe fn read_sctlr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {0}, sctlr_el1", out(reg) value);
    }
    value
}

///
/// This routine writes to the system control register EL1.
///
#[inline]
pub unsafe fn write_sctlr_el1(value: u64) {
    unsafe {
        asm!("msr sctlr_el1, {0}", in(reg) value);
    }
}

///
/// This routine reads the current timestamp counter.
///
#[inline]
pub fn rdtsc() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {0}, cntvct_el0", out(reg) value);
    }
    value
}

///
/// This routine saves the floating point context.
///
#[inline]
pub unsafe fn save_fp_context(memory: *mut u128) {
    unsafe {
        asm!(
            "stp q0, q1, [{0}]",
            "stp q2, q3, [{0}, #32]",
            in(reg) memory
        );
    }
}

///
/// This routine reads an 8-bit value from memory-mapped I/O.
///
#[inline]
pub unsafe fn mmio_read8(address: usize) -> u8 {
    unsafe {
        core::ptr::read_volatile(address as *const u8)
    }
}

///
/// This routine writes an 8-bit value to memory-mapped I/O.
///
#[inline]
pub unsafe fn mmio_write8(address: usize, value: u8) {
    unsafe {
        core::ptr::write_volatile(address as *mut u8, value);
    }
}

///
/// This routine reads the TLS pointer register.
///
#[inline]
pub unsafe fn read_tls_pointer() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {0}, tpidr_el0", out(reg) value);
    }
    value
}

///
/// This routine reads the exception syndrome register EL1.
///
#[inline]
pub unsafe fn read_esr_el1() -> u64 {
    let val: u64;
    unsafe { asm!("mrs {0}, esr_el1", out(reg) val); }
    val
}

///
/// This routine reads the exception link register EL1.
///
#[inline]
pub unsafe fn read_elr_el1() -> u64 {
    let val: u64;
    unsafe { asm!("mrs {0}, elr_el1", out(reg) val); }
    val
}

///
/// This routine reads the fault address register EL1.
///
#[inline]
pub unsafe fn read_far_el1() -> u64 {
    let val: u64;
    unsafe { asm!("mrs {0}, far_el1", out(reg) val); }
    val
}

///
/// This routine writes to the TLS pointer register.
///
#[inline]
pub unsafe fn write_tls_pointer(value: u64) {
    unsafe {
        asm!("msr tpidr_el0, {0}", in(reg) value);
    }
}

///
/// This routine disables interrupts.
///
#[inline]
pub unsafe fn cli() {
    unsafe {
        asm!("msr daifset, #2");
    }
}

///
/// This routine enables interrupts.
///
#[inline]
pub unsafe fn sti() {
    unsafe {
        asm!("msr daifclr, #2", options(nostack, preserves_flags));
    }
}

///
/// This routine halts the CPU.
///
#[inline]
pub unsafe fn halt() {
    unsafe {
        asm!("wfi");
    }
}

///
/// This routine halts the CPU forever in a loop.
///
pub fn halt_forever() {
    loop {
        unsafe {
            cli();
            halt();
        }
    }
}

///
/// This routine returns the current interrupt state.
///
#[inline]
pub unsafe fn interrupt_state() -> bool {
    let value: u64;
    unsafe {
        asm!("mrs {0}, daif", out(reg) value);
    }
    (value & (1 << 7)) == 0
}

///
/// This routine toggles interrupts and returns the previous state.
///
#[inline]
pub unsafe fn toggle_interrupts(state: bool) -> bool {
    let current_state = unsafe { interrupt_state() };
    if state {
        unsafe { sti(); }
    } else {
        unsafe { cli(); }
    }
    current_state
}