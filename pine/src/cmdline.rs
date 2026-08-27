// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: Parses the arguments passed into cmdline
//

use shared::core::cmdline::BootConfiguration;
use spin::Once;

pub static BOOT_OPTIONS: Once<BootConfiguration> = Once::new();

#[derive(Default, Debug, Clone, Copy)]
pub struct CmdLine<'a> {
    data: &'a str,
}

impl<'a> CmdLine<'a> 
{
    ///
    /// This routine constructs a new CmdLine structure,
    /// of course given the data you pass in.
    ///
    /// # Arguments
    ///
    /// * data - A string slice of command line arguments
    ///
    pub const fn new(data: &'a str) -> Self 
    {
        Self { data }
    }

    ///
    /// This routine returns an iterator over the pairs parsed from the 
    /// command line string.
    ///
    pub fn iter(&self) -> impl Iterator<Item = (&'a str, Option<&'a str>)> 
    {
        let mut idx = 0;

        core::iter::from_fn(move || {
            idx = self.data[idx..]
                .find(|c: char| c != ' ' && c != '\t' && c != '\r' && c != '\n')
                .map(|i| idx + i)
                .unwrap_or(self.data.len());

            if idx >= self.data.len() {
                return None;
            }

            let start = idx;
            let end = self.data[start..]
                .find(['=', ' ', '\t'])
                .map(|i| start + i)
                .unwrap_or(self.data.len());

            idx = end;
            let name = &self.data[start..end];

            if self.data[idx..].is_empty() || !self.data[idx..].starts_with('=') {
                Some((name, None))
            } else {
                idx += 1;

                let quote = if self.data[idx..].starts_with('"') {
                    Some('"')
                } else if self.data[idx..].starts_with('\'') {
                    Some('\'')
                } else {
                    None
                };

                if quote.is_some() {
                    idx += 1;
                }

                let val_start = idx;
                let val_end = match quote {
                    Some(q) => self.data[val_start..]
                        .find(q)
                        .map(|i| val_start + i)
                        .unwrap_or(self.data.len()),
                    None => self.data[val_start..]
                        .find([' ', '\t'])
                        .map(|i| val_start + i)
                        .unwrap_or(self.data.len()),
                };

                idx = val_end;

                if quote.is_some() && val_end < self.data.len() {
                    idx += 1;
                }

                Some((name, Some(&self.data[val_start..val_end])))
            }
        })
    }

    ///
    /// This routine gets the string value associated with a given arg name.
    ///
    /// # Arguments
    ///
    /// * name - The key name of hte command line arg to lookup.
    ///
    pub fn get_string(&self, name: &str) -> Option<&'a str> 
    {
        self.iter()
            .find(|(key, _)| *key == name)
            .and_then(|(_, value)| value)
    }

    ///
    /// This routine parses a boolean argument from the command line.
    /// 
    /// Flags present without a value evaluate to 'true'.
    /// 
    /// Recognized true string values include `"true"`,
    ///                                       `"yes"`,
    ///                                       `"on"`,
    ///                                       `"1"`,
    /// 
    /// whilst false string values include
    ///                                       `"false"`,
    ///                                       `"no"`,
    ///                                       `"off"`,
    ///                                       `"0"`,
    /// # Arguments
    ///
    /// * name - The name of the boolean argument.
    ///
    pub fn get_bool(&self, name: &str) -> Option<bool> 
    {
        let entry = self.iter().find(|(key, _)| *key == name)?;
        match entry.1 {
            None => Some(true),
            Some("true") | Some("yes") | Some("on") | Some("1") => Some(true),
            Some("false") | Some("no") | Some("off") | Some("0") => Some(false),
            _ => None,
        }
    }
    
    ///
    /// This routine parses a usize argument value for the argument name.
    ///
    /// # Arguments
    ///
    /// * name - The name of hte usize argument to parse.
    ///
    pub fn get_usize(&self, name: &str) -> Option<usize> 
    {
        let value = self.get_string(name)?;
        value.parse::<usize>().ok()
    }
}

///
/// This routine parses the kernel command line string,
/// fills the boot configuration options,
/// and initializes the BOOT_OPTIONS once-cell storage.
///
/// # Arguments
///
/// * raw_cmdline - A string slice containing the raw command line args passed.
///
pub fn parse_and_store(raw_cmdline: &str) 
{
    let parser = CmdLine::new(raw_cmdline);
    let mut config = BootConfiguration::default();

    if let Some(smp) = parser.get_bool("disable_smp") {
        config.disable_smp = smp;
    }

    if let Some(mem_limit) = parser.get_usize("mem") {
        config.max_memory = Some(mem_limit);
    }

    if let Some(serial_baud) = parser.get_usize("serial_baud") {
        config.serial_baud = serial_baud as u32;
    }

    BOOT_OPTIONS.call_once(|| config);
}
