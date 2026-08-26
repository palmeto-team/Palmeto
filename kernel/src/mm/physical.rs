// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module is the physical memory manager.
//          A physical memory manager handles the raw RAM of a computer.
//
//          Memory will be treated as a fixed-size array of 'frames',
//          each frame will usually be 4KB (or up to 64KB on ARM64).
//
//          It will track which frames are free,
//          and which ones are used.
//
//
#![allow(unused_imports)]

use super::PhysAddr;

//
// CRATE
//
use crate::{
    arch,
    mm::mmdat
};

//
// SHARED
//
use shared::core::utils::divide_up;

//
// RUST
//
use alloc::alloc::AllocError;
use core::{hint::unlikely, panic::Location};

//
// LIBRARIES
//
use bitflags::bitflags;