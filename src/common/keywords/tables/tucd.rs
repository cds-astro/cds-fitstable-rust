//! Defines the `TUCDn` (i.e. column ucd) keyword for `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    DynValueKwr, FixedFormat, KwrFormatRead,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::Error,
};

/// The `TUCDn` keyword.
#[derive(Debug)]
pub struct TUCD {
  n: u16,
  // could be replaced by a UCD type
  value: String,
}

impl TUCD {
  /// # Params
  /// * `n` the `TUCDn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TUCDn` keyword, i.e. the UCD of the column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn col_ucd(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TUCD {
  const KW_PREFIX: &'static [u8] = b"TUCD";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    // In a longer term we could add here code checking the validity of the UCD
    FixedFormat::parse_string_value(kwr_value_comment)
      .map(|(val, _comment)| Self::new(n, val.into_owned()))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("UCD of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}
