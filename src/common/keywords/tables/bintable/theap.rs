//! Defines the `THEAP` keyword for BINTABLE` extensions.
//! The value is an offset, in bytes, between the start of the main table and te heap data area.
//! No such keyword if PCOUNT = 0
use crate::error::new_unexpected_value;
use crate::{
  common::{
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, KwrFormatRead, ValueKwr,
  },
  error::Error,
};

/// The `THEAP` keyword.
pub struct THeap(usize);

impl THeap {
  /// # Params
  /// * `byte_offset` offset, in bytes, between the start of the main table and te heap data area.
  pub fn new(byte_offset: usize) -> Self {
    Self(byte_offset)
  }
  /// Offset, in bytes, between the start of the main table and te heap data area.
  pub fn byte_offset(&self) -> usize {
    self.0
  }
}

impl ValueKwr for THeap {
  const KEYWORD: &'static [u8; 8] = b"THEAP   ";

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
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
      Some("Heap offset size, in bytes"),
    )
  }
}
