// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module provides common utilites needed
//
use num_traits::PrimInt;

///
/// This routine aligns a value to teh next higher multiple of 'alignment'
///
/// # Arguments
///
/// * value - The value to align
/// * alignment - The alignment to align up to
///
#[inline]
pub fn align_up<T: PrimInt>(value: T, alignment: T) -> T
{
    let mask = alignment - T::one();
    (value + mask) & !mask
}

///
/// This routine Aligns a value to the next lower multiple of 'alignment'
///
/// # Arguments
///
/// * value - The value to align
/// * alignment - The alignment to align down to
///
#[inline]
pub fn align_down<T: PrimInt>(value: T, alignment: T) -> T
{
    let mask = alignment - T::one();
    (value) & !mask
}

///
/// This routine divides a value after rounding up to higher multiple of alignemnt
///
/// # Arguments
///
/// * value - The total value to divide
/// * `alignment` - The alignment to divide by
#[inline]
pub fn divide_up<T: PrimInt>(value: T, alignment: T) -> T {
    align_up(value, alignment) / alignment
}