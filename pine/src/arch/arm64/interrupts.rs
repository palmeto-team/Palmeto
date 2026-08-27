// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: 'API' for interrupt handling,
//          We use a Dispatch table approach,
//          An interrupt will register itself (like the timer)
//
use spin::Mutex;

use crate::arch::arm64::assembly::interrupt::{save_and_disable_interrupts,
                                              restore_interrupts};

use shared::{debug};

const MAX_IRQS: usize = 1024;

type IrqHandler = fn();

static IRQ_TABLE: Mutex<[Option<IrqHandler>; MAX_IRQS]> = Mutex::new(
    [None; MAX_IRQS]
);

///
/// This routine registers an interrupt handler.
///
pub fn register_handler(irq: usize, handler: IrqHandler)
{
    if irq >= MAX_IRQS {
        debug!("FAILED TO REGISTER HANDLER: {}", irq);
        return;
    }

    let state = unsafe {
        save_and_disable_interrupts()
    };

    {
        let mut table = IRQ_TABLE.lock();
        table[irq]    = Some(handler);
    }

    unsafe {
        restore_interrupts(state)
    };
}

///
/// This routine dispatches an interrupt to its registered handler.
///
pub fn dispatch(irq: usize)
{
    if irq >= MAX_IRQS
    {
        debug!("INVALID INTERRUPT REQUEST: {}", irq);
        return;
    }

    let state = unsafe {
        save_and_disable_interrupts()
    };

    let handler = {
        let table = IRQ_TABLE.lock();
        table[irq]
    };

    unsafe {
        restore_interrupts(state)
    };
    
    if let Some(func) = handler
    {
        func();
    } else {
        shared::debug!("SPURIOUS OR UNHANDLED IRQ: {}", irq);
    }
}