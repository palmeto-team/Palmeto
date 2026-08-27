// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module initializes the limine command line parser
//
use shared::{core::requests::CMDLINE_REQUEST};

/// This routine initializes the command line parser. 
/// Users are allowed to pass commands to Limine, such as `video=1080x720`.
/// 
/// We must manually parse the command line, if a recognized 
/// command is found, we can try and do something with it
/// 
pub fn init()
{
    if let Some(cmd_response) = CMDLINE_REQUEST.response() 
    {
        let raw_ptr: *const u8 = cmd_response.cmdline().as_ptr();

        if !raw_ptr.is_null() 
        {
            let mut len = 0;

            unsafe 
            {
                while *raw_ptr.offset(len) != 0 
                {
                    len += 1;
                }
                
                let byte_slice = core::slice::from_raw_parts(raw_ptr, len as usize);
                
                if let Ok(cmd_str) = core::str::from_utf8(byte_slice) 
                {
                    pine::cmdline::parse_and_store(cmd_str);
                }
            }
        }
    }
}