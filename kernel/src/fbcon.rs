// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module acts as the public API for fbcon (framebuffer console),
//          It provides the write methods, the initialization method, and more,
//          this file is very messy to be honest..
//

//
// MODULES THAT WE NEED..
//
pub mod alloc;
pub mod framebuffer;
pub mod fbfont;

//
// STD AND EXTERNAL
//
use core::fmt::{self, Write};
use spin::Mutex;
use flanterm::fb::{FlantermFb, Font, Rotation};

//
// THIS PROJECT
//
use shared::{core::requests::FRAMEBUFFER_REQUEST, print};
pub use framebuffer::{fill_display, query_framebuffer_information};

//
// CURSOR CONSTANTS
//
pub const HIDE_CURSOR: &str = "\x1b[?25l";
pub const SHOW_CURSOR: &str = "\x1b[?25h";
pub const CLEAR_SCREEN_HOME_CURSOR: &str = "\x1b[H\x1b[2J";

pub struct TermWrapper(pub FlantermFb<'static>);
unsafe impl Send for TermWrapper {}

pub struct FbConsole;

impl fmt::Write for FbConsole 
{
    ///
    /// This routine writes a string slice to the fbcon by
    /// forwarding it to the string writing (internal functions)
    ///
    /// # Arguments
    ///
    /// * s - The string slice to write.
    ///
    fn write_str(&mut self, s: &str) -> fmt::Result 
    {
        write_string(s);
        Ok(())
    }
}

impl shared::library::ulogger::sink::LogSink for FbConsole 
{
    ///
    /// This routine writes a slice of data to the fbcon
    /// converting the UTF-8 byte stream into chars
    ///
    /// # Arguments
    ///
    /// * data - A byte slice containing the log.
    ///
    fn write(&self, data: &[u8]) 
    {
        if let Ok(s) = core::str::from_utf8(data) {
            write_string(s);
        }
    }
}

pub static FBCON_SINK: FbConsole                  = FbConsole;
static FBCON_TERM:     Mutex<Option<TermWrapper>> = Mutex::new(None);

///
/// This routine initializes the fbcon using
/// Limine's framebuffer request,
/// applies the display font,
/// registers as a global log sink,
/// and hides the cursor.
///
pub fn initialize() 
{

    if let Some(resp) = FRAMEBUFFER_REQUEST.response()
        && let Some(fb) = resp.framebuffers().first()
    {
        //
        // DEFAULT FLANTERM FONT IS KIND OF UGLY...
        // SWAP TO THE ONE DEFINED IN (fbfont.rs)
        //
        let custom_font = Font {
            font: &fbfont::FBCON_DISPLAY_FONT.0,
            width: 8,
            height: 16,
            spacing: 0,
        };
        
        let framebuffer: *mut() = fb.address();

        let term = FlantermFb::new(
            unsafe
            {&mut *core::ptr::slice_from_raw_parts_mut(framebuffer as *mut u32, fb.pitch as usize * fb.height as usize)},
            fb.width as usize,
            fb.height as usize,
            fb.pitch as usize,
            fb.red_mask_size,
            fb.red_mask_shift,
            fb.green_mask_size,
            fb.green_mask_shift,
            fb.blue_mask_size,
            fb.blue_mask_shift,
            Some(custom_font),
            1,
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            Rotation::Rot0,
        )
        .expect("FLANTERM_FAILED_TO_INITIALIZE");

        *FBCON_TERM.lock() = Some(TermWrapper(term));

        let _ = shared::library::ulogger::register_sink(&FBCON_SINK);

        write_string(HIDE_CURSOR);
    }
}

///
/// This routine resets the console display by
/// clearing the screen and moving the cursor home.
///
pub fn reset_display() {
    write_string(CLEAR_SCREEN_HOME_CURSOR);
}

///
/// This routine writes a single character to the fbcon
///
/// # Arguments
///
/// * character - The character to write.
///
pub fn write_char(character: char) {
    let mut buf = [0u8; 4];
    let encoded = character.encode_utf8(&mut buf);
    write_string(encoded);
}

///
/// This routine writes a string slice to the fbcon,
/// automatically converting new-line characters ('\n') into
/// ('\n\r')
///
/// # Arguments
///
/// * s - The stirng to write
///
pub fn write_string(s: &str) {
    if let Some(wrapper) = FBCON_TERM.lock().as_mut() {
        let mut buf = [0u8; 4];
        for c in s.chars() {
            if c == '\n' {
                let _ = wrapper.0.write_str("\r\n");
            } else {
                let encoded = c.encode_utf8(&mut buf);
                let _ = wrapper.0.write_str(encoded);
            }
        }
    }
}

///
/// This routine will change the color of the framebuffer,
/// automatically handling text contrast.
/// 
/// This implementation is NOT good,
/// it's just a temp solution until i rework this driver,
/// only really used for kernel panic so far..
///
pub fn change_screen_color(color: u32) {
    
    //
    // Make sure we have a framebuffer first
    //
    if let Some(resp) = FRAMEBUFFER_REQUEST.response()
        && let Some(fb) = resp.framebuffers().first()
    {
        //
        // Get the width and height
        //
        let width: u32 = fb.width.try_into().unwrap();
        let height: u32 = fb.height.try_into().unwrap();

        //
        // Fill the screen with the color
        //
        fill_display(0, 0, width, height, color);
    }

    //
    // Extract the RGB values
    //
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF)  as u8;
    let b = (color & 0xFF) as u8;

    //
    // What is the readable text contrast?
    // Could be dark text for ligth backgrounds,
    // or light text for dark backgrounds
    //
    let (tr, tg, tb) = if (r as u16 + g as u16 + b as u16) / 3 > 128 {
        (0, 0, 0)
    } else {
        (255, 255, 255)
    };

    //
    // Some stuff here...
    // I don't know what's going on??
    //
    // Perhaps we shouldn't send to serial
    //
    print!(
        "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m\x1b[2J\x1b[H",
        r, g, b, tr, tg, tb
    );
}