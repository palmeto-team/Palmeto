// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This file has assembly instruction wrappers,
//          for interrupt handling...
//          instructions.rs already has assembly wrappers,
//          but this is another layer for clean 'interrupts'
//

#![allow(unused)]
#![allow(clippy::missing_safety_doc)]

use super::instructions;

///
/// This routine enables interrupts.
///
pub unsafe fn enable_interrupts()
{
    unsafe
    {
        instructions::sti();
    }
}

///
/// This routine disables interrupts.
///
pub unsafe fn disable_interrupts()
{
    unsafe
    {
        instructions::cli();
    }
}

///
/// This routine returns the current interrupt state.
///
pub unsafe fn get_interrupt_state() -> bool
{
    unsafe
    {
        instructions::interrupt_state()
    }
}

///
/// This routine toggles interrupts and returns the previous state.
///
pub unsafe fn toggle_interrupts(state: bool) -> bool
{
    unsafe
    {
        instructions::toggle_interrupts(state)
    }
}

///
/// This routine saves the current state and disables interrupts.
///
pub unsafe fn save_and_disable_interrupts() -> bool
{
    unsafe
    {
        instructions::toggle_interrupts(false)
    }
}

///
/// This routine restores the previous interrupt state.
///
pub unsafe fn restore_interrupts(previous_state: bool)
{
    unsafe {
        instructions::toggle_interrupts(previous_state);
    }
}