//! Shared settings module.
//!
//! This module defines data structures to access the settings defined in the meta language.
//!
//! Each settings group is translated to a `Settings` struct either in this module or in its
//! ISA-specific `settings` module. The struct provides individual getter methods for all of the
//! settings. It also implements the `Stringwise` trait which allows settings to be manipulated by
//! name.

use std::fmt;
use std::result;

/// A setting descriptor holds the information needed to generically set and print a setting.
///
/// Each settings group will be represented as a constant DESCRIPTORS array.
pub struct Descriptor {
    /// Lower snake-case name of setting as defined in meta.
    pub name: &'static str,

    /// Offset of byte containing this setting.
    pub offset: u32,

    /// Additional details, depending on the kind of setting.
    pub detail: Detail,
}

/// The different kind of settings along with descriptor bits that depend on the kind.
#[derive(Clone, Copy)]
pub enum Detail {
    /// A boolean setting only uses one bit, numbered from LSB.
    Bool {
        bit: u8,
    },

    /// A numerical setting uses the whole byte.
    Num,

    /// An Enum setting uses a range of enumerators.
    Enum {
        /// Numerical value of last enumerator, allowing for 1-256 enumerators.
        last: u8,

        /// First enumerator in the ENUMERATORS table.
        enumerators: u16,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// No setting by this name exists.
    BadName,

    /// Type mismatch for setting (e.g., setting an enum setting as a bool).
    BadType,

    /// This is not a valid value for this setting.
    BadValue,
}

pub type Result<T> = result::Result<T, Error>;

/// Interface for working with a group of settings as strings.
pub trait Stringwise {
    /// Look up a setting by name, return the details of the setting along with a reference to the
    /// byte holding the value of the setting.
    fn lookup_mut(&mut self, name: &str) -> Result<(Detail, &mut u8)>;

    /// Get an enumerator string from the `Detail::enumerators` value and an offset.
    fn enumerator(&self, enums: u16, value: u8) -> &'static str;

    /// Format a setting value as a TOML string. This is mostly for use by the generateed `Display`
    /// implementation.
    fn format_toml_value(&self, detail: Detail, byte: u8, f: &mut fmt::Formatter) -> fmt::Result {
        match detail {
            Detail::Bool { bit } => write!(f, "{}", (byte & (1 << bit)) != 0),
            Detail::Num => write!(f, "{}", byte),
            Detail::Enum { last, enumerators } => {
                if byte <= last {
                    write!(f, "\"{}\"", self.enumerator(enumerators, byte))
                } else {
                    write!(f, "{}", byte)
                }
            }
        }
    }

    /// Set a boolean setting by name.
    fn set_bool(&mut self, name: &str, value: bool) -> Result<()> {
        let (detail, byte) = try!(self.lookup_mut(name));
        if let Detail::Bool { bit } = detail {
            let mask = 1 << bit;
            if value {
                *byte |= mask;
            } else {
                *byte &= !mask;
            }
            Ok(())
        } else {
            Err(Error::BadType)
        }
    }
}

// Include code generated by `meta/gen_settings.py`. This file contains a public `Settings` struct
// with an impl for all of the settings defined in `meta/cretonne/settings.py`.
include!(concat!(env!("OUT_DIR"), "/settings.rs"));

#[cfg(test)]
mod tests {
    use super::Settings;
    use super::Error::*;
    use super::Stringwise;

    #[test]
    fn display_default() {
        let s = Settings::default();
        assert_eq!(s.to_string(),
                   "[shared]\n\
                    enable_simd = true\n");
    }

    #[test]
    fn modify_bool() {
        let mut s = Settings::default();
        assert_eq!(s.enable_simd(), true);
        assert_eq!(s.set_bool("not_there", true), Err(BadName));

        assert_eq!(s.set_bool("enable_simd", true), Ok(()));
        assert_eq!(s.enable_simd(), true);

        assert_eq!(s.set_bool("enable_simd", false), Ok(()));
        assert_eq!(s.enable_simd(), false);
    }
}