// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: This module handles early time initialization,
//          we can use the limine Time request,
//          which tracks what time it is during boot.
//
//          Atleast we get to skip all that complex timing stuff..
//
use shared::{core::requests::DATE_AT_BOOT_REQUEST, debug, fatal};

use pine::arch::arm64::exception::timer;

///
/// This routine handles setting up DateTime related stuff,
/// we will get boot time from limine,
/// then set our current timer to the boot time.
///
pub fn init_time()
{
    if let Some(response) = DATE_AT_BOOT_REQUEST.response()
    {
        //
        // Current time in seconds
        //
        let boot_time_seconds = response.timestamp;
        
        //
        // Manually set the timer to the current time
        //
        match timer::set_manual_unix_time_ms((boot_time_seconds as u64) * 1_000)
        {
            //
            // It worked
            //
            Ok(()) =>
            {

                //
                // This empty structure will be filled
                //
                let mut dt = timer::DateTime
                {
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                };

                //
                // Fill our date time structure
                //
                if timer::now_datetime(&mut dt, true)
                {
                    //
                    // Store in a string buffer...
                    //
                    let mut buf = [0u8; 64];

                    timer::datetime_to_string(&dt, &mut buf, 20);
                
                    if let Ok(time_str) = core::str::from_utf8(&buf)
                    {
                        debug!("SYSTEM TIME: {}", time_str.trim_end_matches('\0'));
                    }
                }
            }
            Err(status) => {
                debug!("FAILED TO SET BOOT TIME, STATUS: {:?}", status);
            }
        }
    } else {
        fatal!("DATE AT BOOT TIME INVALID..");
    }
}