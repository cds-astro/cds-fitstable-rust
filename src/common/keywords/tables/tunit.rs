//! Defines the `TUNITn` (i.e. column unit) keyword for `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    DynValueKwr, FixedFormat, KwrFormatRead,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::Error,
};

/// The `TUNITn` keyword.
pub struct TUnit {
  n: u16,
  value: String,
}

impl TUnit {
  /// # Params
  /// * `n` the `TUNITn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TUNITn` keyword, i.e. unit of the column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn col_unit(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TUnit {
  const KW_PREFIX: &'static [u8] = b"TUNIT";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_string_value(kwr_value_comment)
      .map(|(val, _comment)| Self::new(n, val.into_owned()))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Unit of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}
