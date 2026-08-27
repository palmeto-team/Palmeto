// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module implements the internal functions for the fbcon (framebuffer console),
//          fbcon.rs is the public api,
//          this is the engine.. Simple enough?
//

#![allow(clippy::missing_safety_doc)]

use spin::Mutex;

use shared::core::requests::FRAMEBUFFER_REQUEST;

pub static DRAW_LOCK: Mutex<()> = Mutex::new(());

///
/// This routine plots a single pixel on the framebuffer.
///
#[inline]
pub unsafe fn plot_pixel(
    fb_ptr: *mut u8,
    pitch: usize,
    bpp: u16,
    fb_width: u64,
    fb_height: u64,
    color: u32,
    x: u32,
    y: u32,
) {
    if (x as u64) >= fb_width || (y as u64) >= fb_height {
        return;
    }

    let bytes_per_pixel = (bpp / 8) as usize;
    let offset = pitch * (y as usize) + (x as usize) * bytes_per_pixel;

    unsafe {
        let place = fb_ptr.add(offset);
        match bytes_per_pixel {
            1 => *place = (color | (color >> 8) | (color >> 16)) as u8,
            2 => *(place as *mut u16) = (color | (color >> 16)) as u16,
            3 => {
                *place = color as u8;
                *place.add(1) = (color >> 8) as u8;
                *place.add(2) = (color >> 16) as u8;
            }
            4 => *(place as *mut u32) = color,
            _ => {}
        }
    }
}

///
/// This routine fills a rectangular region of the display with a color.
///
pub fn fill_display(left: u32, top: u32, right: u32, bottom: u32, color: u32) {
    let response = match FRAMEBUFFER_REQUEST.response() {
        Some(resp) => resp,
        None => return,
    };

    if response.framebuffers().is_empty() {
        return;
    }

    let fb = response.framebuffers()[0];
    let fb_ptr = fb.address() as *mut u8;
    let pitch = fb.pitch as usize;
    let bpp = fb.bpp;
    let width = fb.width;
    let height = fb.height;

    let _guard = DRAW_LOCK.lock();

    for y in top..bottom {
        for x in left..right {
            unsafe {
                plot_pixel(fb_ptr, pitch, bpp, width, height, color, x, y);
            }
        }
    }
}

///
/// This routine queries the framebuffer information from the firmware.
///
pub fn query_framebuffer_information() -> Option<(u32, u32, u64, u32)> {
    let response = FRAMEBUFFER_REQUEST.response()?;
    if response.framebuffers().is_empty() {
        return None;
    }

    let fb = response.framebuffers()[0];
    Some((
        fb.width as u32,
        fb.height as u32,
        fb.address() as u64,
        fb.bpp as u32,
    ))
}