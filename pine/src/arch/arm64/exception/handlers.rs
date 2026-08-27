// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This is where any exception handlers go,
//          if you want a certain function called instead of panic,
//          you register it here,
//          said function will be called upon the exception
//

use crate::arch::arm64::exception::exceptions::{ExceptionHandlers, RegisterStateRef};
use crate::arch::arm64::exception::{intrcntrl};
use crate::arch::arm64::interrupts;

use crate::exception_handlers;

use shared::{fatal};

pub struct KernelExceptionHandlers;

#[allow(unused)]
impl ExceptionHandlers for KernelExceptionHandlers {
    ///
    /// This routine handles synchronous CPU exceptions.
    ///
    extern "C" fn sync_current(register_state: RegisterStateRef) 
    {

        let esr: u64;
        let far: u64;

        unsafe {
            core::arch::asm!("mrs {}, esr_el1", out(reg) esr);
            core::arch::asm!("mrs {}, far_el1", out(reg) far);
        }

        fatal!("UNRECOVERABLE CPU EXCEPTION: ESR={:x}, FAR={:x}", esr, far);
    }

    ///
    /// This routine handles interrupt requests.
    ///
    extern "C" fn irq_current(register_state: RegisterStateRef)
    {
        let iar = intrcntrl::read_and_ack_interrupt();
        
        //
        // The lower 10 bits have the ID..
        //
        let irq_id = (iar & 0x3FF) as usize;

        if irq_id < 1020
        {
            interrupts::dispatch(irq_id);
            intrcntrl::end_of_interrupt(iar);
        }
    }
}

exception_handlers!(KernelExceptionHandlers);

///
/// This routine initializes the exception handlers.
///
pub fn init() {
    unsafe {
        core::arch::asm!(
            "adr x0, vector_table_el1",
            "msr vbar_el1, x0",
            options(nomem, nostack)
        );
    }
}