#![no_std]
#![feature(allocator_api)]
#![feature(likely_unlikely)]
extern crate flanterm;
extern crate alloc;

pub mod arch;
pub mod cmdline;
pub mod fbcon;
pub mod mm;