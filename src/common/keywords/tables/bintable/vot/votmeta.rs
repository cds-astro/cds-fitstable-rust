//! Defines the `VOTMETA` keyword in the Primary HDU, marker of a `FITS Plus` file.
use crate::{
  common::{
    FixedFormat, KwrFormatRead, ValueKwr,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::Error,
};

/// The `VOTMeta` keyword.
/// No need to use u32 or u64 since there is a maximum of 999 columns.
pub struct VOTMeta(bool);

impl VOTMeta {
  pub fn new(is_fits_plus: bool) -> Self {
    Self(is_fits_plus)
  }
  pub fn is_true(&self) -> bool {
    self.0
  }
}

impl ValueKwr for VOTMeta {
  const KEYWORD: &'static [u8; 8] = b"VOTMETA ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_logical_value(kwr_value_comment).map(|_| ())
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_logical_value(kwr_value_comment).and_then(|(val, _comment)| Ok(Self(val)))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    FixedFormatWrite::write_boolean_value_kw_record(
      dest_kwr_it,
      Self::KEYWORD,
      self.0,
      Some("Table metadata in VOTable format"),
    )
  }
}
