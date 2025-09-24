//! Defines the `NTABLE` keyword (in the Primary Header), i.e. the number of `BINTABLE` extensions
//! following the Primary HDU in a `FITS Plus` file.
use crate::{
  common::{
    FixedFormat, KwrFormatRead, ValueKwr,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_unexpected_value},
};

/// The `NTABLE` keyword.
/// No need to use u32 or u64 since there is a maximum of 999 columns.
pub struct NTable(u16);

impl NTable {
  pub fn new(n_cols: u16) -> Self {
    Self(n_cols)
  }
  pub fn get(&self) -> u16 {
    self.0
  }
}

impl ValueKwr for NTable {
  const KEYWORD: &'static [u8; 8] = b"NTABLE  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as u16 != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self(val as u16))
      } else {
        Err(new_unexpected_value("positive integer", val))
      }
    })
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    FixedFormatWrite::write_int_value_kw_record(
      dest_kwr_it,
      Self::KEYWORD,
      self.0 as i64,
      Some("Number of following BINTABLE HDUs"),
    )
  }
}
