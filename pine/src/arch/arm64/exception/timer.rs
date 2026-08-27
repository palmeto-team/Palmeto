// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: ARM64 Generic Timer
//

#![allow(clippy::new_without_default)]

use core::arch::asm;
use spin::Mutex;

use shared::{debug};
use shared::core::status::{KResult, Status};

use crate::arch::arm64::exception::intrcntrl;
use crate::arch::arm64::interrupts;

const US_PER_SEC: u64 = 1_000_000;
const MS_PER_SEC: u64 = 1_000;
const MS_PER_MIN: u64 = 60 * MS_PER_SEC;
const SECS_PER_DAY: u64 = 86400;
const US_PER_MAX_SLEW_CAP: i64 = 60 * 1_000_000;

const TIMER_SLEW_MAX_PPM: i32 = 500;
const TIMER_FREQ_MAX_PPM: i32 = 500;

pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

pub struct TimeState {
    pub sync: bool,
    pub wall_base_mono_us: u64,
    pub wall_base_unix_us: i64,
    pub freq_ppm: i32,
    pub slew_rem_us: i64,
    pub tz_offset_min: i32,
}

impl TimeState {
    pub const fn new() -> Self {
        Self {
            sync: false,
            wall_base_mono_us: 0,
            wall_base_unix_us: 0,
            freq_ppm: 0,
            slew_rem_us: 0,
            tz_offset_min: 0,
        }
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}

pub const COMPATIBLE_STRINGS: &[&str] = &[
    "arm,armv8-timer",
    "arm,armv7-timer",
];

pub static TIME_STATE: Mutex<TimeState> = Mutex::new(TimeState::new());

///
/// This routine handles timer interrupts.
///
pub fn timer_interrupt_handler()
{
    //
    // 1-SECOND
    //
    reset(1000);
}

///
/// This routine initializes the timer from a device tree node.
///
pub fn try_init_node(node: &fdt::node::FdtNode) -> KResult<()> {
    debug!("FOUND TIMER...");

    if let Ok(irq) = intrcntrl::parse_interrupt(node, 1)
    {
        debug!("TIMER GIC IRQ: {}", irq);
        interrupts::register_handler(irq,timer_interrupt_handler);
        intrcntrl::enable_irq(irq);
    }

    init(1000);
    Ok(())
}

///
/// This routine reads the timer frequency from the hardware.
///
pub fn rd_cntfrq_el0() -> u64 {
    let v: u64;
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg) v, options(nostack, preserves_flags));
    }
    v
}

///
/// This routine resets the timer with the given millisecond interval.
///
pub fn reset(time: u64) {
    let freq: u64 = rd_cntfrq_el0();
    let interval: u64 = (freq * time) / MS_PER_SEC;

    unsafe {
        asm!(
            "msr cntp_tval_el0, {}",
            in(reg) interval,
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine enables the physical timer.
///
pub fn enable() {
    let val: u64 = 1;
    unsafe {
        asm!(
            "msr cntp_ctl_el0, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
        asm!(
            "msr cntkctl_el1, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine disables the physical timer.
///
pub fn disable() {
    let ctl: u64 = 0;
    unsafe {
        asm!(
            "msr cntp_ctl_el0, {}",
            in(reg) ctl,
            options(nostack, preserves_flags)
        );
        asm!(
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine permanently disables the timer.
///
pub fn permanent_disable_timer() {
    let ctl: u64 = 0;
    unsafe {
        asm!(
            "msr cntp_ctl_el0, {}",
            in(reg) ctl,
            options(nostack, preserves_flags)
        );
        asm!(
            "msr cntv_ctl_el0, {}",
            in(reg) ctl,
            options(nostack, preserves_flags)
        );
        asm!(
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine returns the current timer count.
///
pub fn now() -> u64 {
    let val: u64;
    unsafe {
        asm!(
            "mrs {}, cntvct_el0",
            out(reg) val,
            options(nostack, preserves_flags)
        );
    }
    val
}

///
/// This routine returns the current time in milliseconds.
///
pub fn now_msec() -> u64 {
    let ticks = now();
    let freq = rd_cntfrq_el0();
    if freq == 0 {
        return 0;
    }
    (ticks * MS_PER_SEC) / freq
}

///
/// This routine returns the current time in microseconds.
///
pub fn now_usec() -> u64 {
    let ticks = now();
    let freq = rd_cntfrq_el0();
    if freq == 0 {
        return 0;
    }
    let q = ticks / freq;
    let r = ticks % freq;
    q * US_PER_SEC + (r * US_PER_SEC) / freq
}

///
/// This routine initializes the timer subsystem.
///
pub fn init(msecs: u64) {
    reset(msecs);
    enable();

    let mut state = TIME_STATE.lock();
    state.wall_base_mono_us = now_usec();
    state.wall_base_unix_us = 0;
    state.freq_ppm = 0;
    state.slew_rem_us = 0;
    state.sync = false;
}

///
/// This routine resets the virtual timer.
///
pub fn virtual_reset(smsecs: u64) {
    let freq = rd_cntfrq_el0();
    let interval = (freq * smsecs) / MS_PER_SEC;

    unsafe {
        asm!(
            "msr cntv_tval_el0, {}",
            in(reg) interval,
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine enables the virtual timer.
///
pub fn virtual_enable() {
    let val: u64 = 1;
    unsafe {
        asm!(
            "msr cntv_ctl_el0, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine disables the virtual timer.
///
pub fn virtual_disable() {
    let val: u64 = 0;
    unsafe {
        asm!(
            "msr cntv_ctl_el0, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

///
/// This routine returns the remaining time on the virtual timer in milliseconds.
///
pub fn virtual_remaining_msec() -> u64 {
    let ticks: u64;
    let freq = rd_cntfrq_el0();
    if freq == 0 {
        return 0;
    }

    unsafe {
        asm!(
            "mrs {}, cntv_tval_el0",
            out(reg) ticks,
            options(nostack, preserves_flags)
        );
    }

    (ticks * MS_PER_SEC) / freq
}

///
/// This routine advances the wall time to the current monotonic time.
///
fn wall_advance_to(mono_now_us: u64) -> i64 {
    let mut state = TIME_STATE.lock();

    if state.wall_base_mono_us == 0 {
        state.wall_base_mono_us = mono_now_us;
    }

    let dt_u = mono_now_us - state.wall_base_mono_us;
    if dt_u == 0 {
        return state.wall_base_unix_us;
    }

    let dt = dt_u as i64;
    let mut base = state.wall_base_unix_us;
    let adj = dt + (dt * (state.freq_ppm as i64)) / US_PER_SEC as i64;
    base += adj;

    let mut max_slew = (dt * (TIMER_SLEW_MAX_PPM as i64)) / US_PER_SEC as i64;
    if max_slew < 1 {
        max_slew = 1;
    }

    if state.slew_rem_us != 0 {
        let apply = state.slew_rem_us.clamp(-max_slew, max_slew);
        state.slew_rem_us -= apply;
        base += apply;
    }

    state.wall_base_mono_us = mono_now_us;
    state.wall_base_unix_us = base;
    base
}

///
/// This routine returns the current wall time in microseconds.
///
pub fn wall_time_us() -> u64 {
    wall_advance_to(now_usec()) as u64
}

///
/// This routine returns the current UNIX time in microseconds.
///
pub fn unix_time_us() -> u64 {
    let synced = TIME_STATE.lock().sync;
    if !synced {
        return 0;
    }

    let u = wall_advance_to(now_usec());
    if u < 0 {
        return 0;
    }

    u as u64
}

///
/// This routine synchronizes the UNIX time in microseconds.
///
pub fn sync_set_unix_us(unix_us: u64) {
    let now_us = now_usec();
    let mut state = TIME_STATE.lock();

    state.wall_base_mono_us = now_us;
    state.wall_base_unix_us = unix_us as i64;
    state.slew_rem_us = 0;
    state.sync = true;
}

///
/// This routine applies a time slew adjustment in microseconds.
///
pub fn sync_slew_us(delta_us: i64) {
    let mut state = TIME_STATE.lock();
    let v = state.slew_rem_us + delta_us;
    state.slew_rem_us = v.clamp(-US_PER_MAX_SLEW_CAP, US_PER_MAX_SLEW_CAP);
}

///
/// This routine sets the frequency adjustment in PPM.
///
pub fn sync_set_freq_ppm(ppm: i32) {
    let mut state = TIME_STATE.lock();
    state.freq_ppm = ppm.clamp(-TIMER_FREQ_MAX_PPM, TIMER_FREQ_MAX_PPM);
}

///
/// This routine gets the frequency adjustment in PPM.
///
pub fn sync_get_freq_ppm() -> i32 {
    TIME_STATE.lock().freq_ppm
}

///
/// This routine applies an SNTP time sample.
///
pub fn apply_sntp_sample_us(server_unix_us: u64) {
    sync_set_unix_us(server_unix_us);
}

///
/// This routine checks if the system time is synchronized.
///
pub fn is_synchronised() -> bool {
    TIME_STATE.lock().sync
}

///
/// This routine returns the current UNIX time in milliseconds.
///
pub fn unix_time_ms() -> u64 {
    let us = unix_time_us();
    if us == 0 {
        return 0;
    }
    us / MS_PER_SEC
}

///
/// This routine sets the timezone offset in minutes.
///
pub fn set_timezone_minutes(minutes: i32) {
    let mut state = TIME_STATE.lock();
    state.tz_offset_min = minutes;
}

///
/// This routine gets the timezone offset in minutes.
///
pub fn get_timezone_minutes() -> i32 {
    TIME_STATE.lock().tz_offset_min
}

///
/// This routine returns the current local time in milliseconds.
///
pub fn local_time_ms() -> u64 {
    let utc_ms = unix_time_ms();
    if utc_ms == 0 {
        return 0;
    }

    let tz_offset = TIME_STATE.lock().tz_offset_min as i64;
    let adj = (utc_ms as i64) + tz_offset * MS_PER_MIN as i64;
    if adj < 0 {
        return 0;
    }

    adj as u64
}

///
/// This routine sets the UNIX time manually in milliseconds.
///
pub fn set_manual_unix_time_ms(unix_ms: u64) -> KResult<()> {
    let mut state = TIME_STATE.lock();
    if state.sync {
        return Err(Status::INVALID_DEVICE_REQUEST);
    }

    state.wall_base_mono_us = now_usec();
    state.wall_base_unix_us = (unix_ms * MS_PER_SEC) as i64;
    state.sync              = true;
    Ok(())
}

///
/// This routine calculates the number of days from a civil date.
///
fn days_from_civil(mut y: i64, m: u32, d: u32) -> i64 {
    y -= if m <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u32;
    let m_adj = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * m_adj + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + yoe / 400 + doy;
    era * 146097 + (doe as i64) - 719468
}

///
/// This routine converts days to a civil date.
///
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y_full = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100 + yoe / 400);
    let mp = (5 * doy + 2) / 153;
    let dd = doy - (153 * mp + 2) / 5 + 1;
    let mm = if mp < 10 { mp + 3 } else { mp - 9 };
    y_full += if mm <= 2 { 1 } else { 0 };
    (y_full, mm, dd)
}

///
/// This routine converts UNIX milliseconds to a DateTime structure.
///
pub fn unix_ms_to_datetime(unix_ms: u64, use_local: bool, out: &mut DateTime) {
    let tz_offset = TIME_STATE.lock().tz_offset_min as i64;
    let mut ms = unix_ms as i64;

    if use_local {
        ms += tz_offset * MS_PER_MIN as i64;
    }
    if ms < 0 {
        ms = 0;
    }

    let sec = (ms as u64) / MS_PER_SEC;
    let sod = sec % SECS_PER_DAY;
    let days = sec / SECS_PER_DAY;

    let (y, m, d) = civil_from_days(days as i64);

    out.year = y as u16;
    out.month = m as u8;
    out.day = d as u8;
    out.hour = (sod / 3600) as u8;
    out.minute = ((sod % 3600) / 60) as u8;
    out.second = (sod % 60) as u8;
}

///
/// This routine converts a DateTime structure to UNIX milliseconds.
///
pub fn datetime_to_unix_ms(dt: &DateTime, is_local: bool) -> u64 {
    let mut y = dt.year as i64;
    if y < 1970 {
        y = 1970;
    }

    let m = if (1..=12).contains(&dt.month) { dt.month as u32 } else { 1 };
    let d = if (1..=31).contains(&dt.day) { dt.day as u32 } else { 1 };
    let h = if dt.hour <= 23 { dt.hour as u64 } else { 0 };
    let minute = if dt.minute <= 59 { dt.minute as u64 } else { 0 };
    let s = if dt.second <= 59 { dt.second as u64 } else { 0 };

    let days = days_from_civil(y, m, d);
    let sec = (if days >= 0 { days as u64 } else { 0 }) * SECS_PER_DAY + h * 3600 + minute * 60 + s;

    let tz_offset = TIME_STATE.lock().tz_offset_min as i64;
    let mut ms = (sec as i64) * MS_PER_SEC as i64;

    if is_local {
        ms -= tz_offset * MS_PER_MIN as i64;
    }
    if ms < 0 {
        ms = 0;
    }

    ms as u64
}

///
/// This routine gets the current time as a DateTime structure.
///
pub fn now_datetime(out: &mut DateTime, use_local: bool) -> bool {
    let ms = if use_local { local_time_ms() } else { unix_time_ms() };
    if ms == 0 {
        return false;
    }
    unix_ms_to_datetime(ms, false, out);
    true
}

///
/// This routine converts a DateTime to a string.
///
pub fn datetime_to_string(dt: &DateTime, buf: &mut [u8], buflen: u32) {
    if buflen < 20 || buf.len() < 20 {
        return;
    }

    let y = dt.year;
    let write_two = |val: u8, dest: &mut [u8], start: usize| {
        dest[start] = b'0' + (val / 10);
        dest[start + 1] = b'0' + (val % 10);
    };

    buf[0] = b'0' + ((y / 1000) % 10) as u8;
    buf[1] = b'0' + ((y / 100) % 10) as u8;
    buf[2] = b'0' + ((y / 10) % 10) as u8;
    buf[3] = b'0' + (y % 10) as u8;
    buf[4] = b'-';

    write_two(dt.month, buf, 5);
    buf[7] = b'-';
    write_two(dt.day, buf, 8);
    buf[10] = b' ';
    write_two(dt.hour, buf, 11);
    buf[13] = b':';
    write_two(dt.minute, buf, 14);
    buf[16] = b':';
    write_two(dt.second, buf, 17);
    buf[19] = 0;
}