//! Defines the `TTYPEn` (i.e. column names) keyword for `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    DynValueKwr,
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, KwrFormatRead,
  },
  error::{Error},
};

/// The `TTYPEn` keyword.
pub struct TType {
  n: u16,
  value: String,
}

impl TType {
  /// # Params
  /// * `n` the `TTYPEn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TTYPEn` keyword, i.e. name of the column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn col_name(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TType {
  const KW_PREFIX: &'static [u8] = b"TTYPE";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_string_value(kwr_value_comment).map(|(val, _comment)| {
      Self::new(n, val.into_owned())
    })
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Name of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}
