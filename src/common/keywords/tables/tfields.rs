//! Defines the `TFIELD` (i.e. numbver of columns) keyword for `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    FixedFormat, KwrFormatRead, ValueKwr,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_unexpected_value},
};

/// The `TFIELD` keyword.
/// No need to use u32 or u64 since there is a maximum of 999 columns.
pub struct TFields(u16);

impl TFields {
  pub fn new(n_cols: u16) -> Self {
    Self(n_cols)
  }
  pub fn get(&self) -> u16 {
    self.0
  }
}

impl ValueKwr for TFields {
  const KEYWORD: &'static [u8; 8] = b"TFIELDS ";

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
      Some("Number of columns"),
    )
  }
}
