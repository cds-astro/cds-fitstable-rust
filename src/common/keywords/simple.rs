//! Defines the `SIMPLE` keyword.

use crate::{
  common::{
    read::KwrFormatRead,
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, ValueKwr,
  },
  error::{new_unexpected_value, Error},
};

/// The `SIMPLE` keyword, i.e. conform to the standard or not.
/// First keyword of the primary haader only.
pub struct Simple(bool);

impl Simple {
  pub fn new(is_simple: bool) -> Self {
    Self(is_simple)
  }
  pub fn get(&self) -> bool {
    self.0
  }
}

impl ValueKwr for Simple {
  const KEYWORD: &'static [u8; 8] = b"SIMPLE  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_logical_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_logical_value(kwr_value_comment).map(|(val, _comment)| Self(val))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    FixedFormatWrite::write_boolean_value_kw_record(
      dest_kwr_it,
      Self::KEYWORD,
      self.0,
      Some("File conforms to FITS standard"),
    )
  }
}
