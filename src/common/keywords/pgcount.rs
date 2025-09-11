///! Defines the `PCOUNT` and `GCOUNT` keywords.
use crate::{
  common::{
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, KwrFormatRead, ValueKwr,
  },
  error::{new_unexpected_value, Error},
};

/// The `PCOUNT` keyword, i.e. number of bytes in the `heap`.
pub struct PCount(usize);

impl PCount {
  pub const fn new(pcount: usize) -> Self {
    Self(pcount)
  }
  pub fn get(&self) -> usize {
    self.0
  }
  pub fn byte_size(&self) -> usize {
    self.get()
  }
}

impl ValueKwr for PCount {
  const KEYWORD: &'static [u8; 8] = b"PCOUNT  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as usize != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self(val as usize))
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
      Some("Heap byte size"),
    )
  }
}

/// The `GCOUNT` keyword, number of random-group structures
/// (used in old FITS file if I understand correctly)
pub struct GCount(u32);

impl GCount {
  pub const fn new(gcount: u32) -> Self {
    Self(gcount)
  }
  pub fn get(&self) -> u32 {
    self.0
  }
}

impl ValueKwr for GCount {
  const KEYWORD: &'static [u8; 8] = b"GCOUNT  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as u32 != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self(val as u32))
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
      Some("Number of random groups"),
    )
  }
}
