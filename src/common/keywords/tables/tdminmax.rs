//! Defines the `TDMIN` and `TDMAX` (i.e. min and max of a columns) keywordq for both
//! `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    write::{FixedFormatWrite, KwrFormatWrite},
    DynValueKwr, FixedFormat, KwrFormatRead,
  },
  error::Error,
};

/// The `TDMINn` keyword.
pub struct TDMin {
  n: u16,
  value: String,
}

impl TDMin {
  /// # Params
  /// * `n` the `TDMINn` number in `[1, TFIELD]`.
  /// * `value` min value associated to this `TDMINn` keyword, i.e. for the column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn min_value(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TDMin {
  const KW_PREFIX: &'static [u8] = b"TDMIN";

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
    let comment = format!("Min value of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}

/// The `TDMAXn` keyword.
pub struct TDMax {
  n: u16,
  value: String,
}

impl TDMax {
  /// # Params
  /// * `n` the `TDMAXn` number in `[1, TFIELD]`.
  /// * `value` min value associated to this `TDMAXn` keyword, i.e. for the column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn max_value(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TDMax {
  const KW_PREFIX: &'static [u8] = b"TDMAX";

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
    let comment = format!("Max value of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}
