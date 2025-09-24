//! Defines the `TDIMn` (i.e. column multiarray info) keyword for `BINTABLE` extensions.

use std::{
  fmt::{Display, Formatter},
  str::FromStr,
};

use crate::{
  common::{
    DynValueKwr, FreeFormat, KwrFormatRead,
    write::{FreeFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_parse_u16_err},
};

///First dimension is the one varying most rapidly.
pub struct TDimValue(Vec<u16>);
impl From<Vec<u16>> for TDimValue {
  fn from(value: Vec<u16>) -> Self {
    Self(value)
  }
}
impl FromStr for TDimValue {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.trim()
      .trim_matches(&['(', ')'])
      .split(',')
      .map(|s| s.parse::<u16>())
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| new_parse_u16_err(e))
      .map(Self)
  }
}
impl Display for TDimValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "(")?;
    let mut it = self.0.iter();
    if let Some(n) = it.next() {
      write!(f, "{}", n)?;
      for n in it {
        write!(f, ",{}", n)?;
      }
    }
    write!(f, ")")
  }
}

/// The `TDIMn` keyword.
pub struct TDim {
  n: u16,
  value: TDimValue,
}

impl TDim {
  /// # Params
  /// * `n` the `TDIMn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TDMIn` keyword, i.e. multi-array format of the column number `n`
  pub fn new(n: u16, value: TDimValue) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn dimensions(&self) -> &[u16] {
    self.value.0.as_slice()
  }
}

impl DynValueKwr for TDim {
  const KW_PREFIX: &'static [u8] = b"TDIM";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FreeFormat::parse_string_value_no_quote(kwr_value_comment)
      .and_then(|(val, _comment)| val.parse::<TDimValue>().map(|v| Self::new(n, v)))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Dimensions of column #{}", self.n);
    FreeFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.to_string().as_str(),
      Some(comment.as_str()),
    )
  }
}
