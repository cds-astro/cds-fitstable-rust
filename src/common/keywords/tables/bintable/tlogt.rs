use std::{
  fmt::{Display, Formatter},
  str::FromStr,
};

use crate::{
  common::{
    DynValueKwr, FreeFormat,
    read::KwrFormatRead,
    write::{FreeFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_unexpected_value},
};

#[derive(Debug)]
pub enum LogicalType {
  UTF8,
}
impl FromStr for LogicalType {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with("UTF-8") {
      Ok(Self::UTF8)
    } else {
      Err(new_unexpected_value("TLOGT should starts with UTF-8", s))
    }
  }
}
impl Display for LogicalType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UTF8 => f.write_fmt(format_args!("UTF-8")),
    }
  }
}

/// Declare a Logical Type for column `n`.
#[derive(Debug)]
pub struct TLogTn {
  pub n: u16,
  pub value: LogicalType,
}
impl TLogTn {
  /// # Params
  /// * `n` the `TFORMn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TFORMn` keyword, i.e. column data type
  pub fn new(n: u16, value: LogicalType) -> Self {
        Self { n, value }
    }

  /// Column number starts at 1.
  pub fn col_nbr(&self) -> u16 {
        self.n
    }
  pub fn logical_type(&self) -> &LogicalType {
        &self.value
    }
}

impl DynValueKwr for TLogTn {
  const KW_PREFIX: &'static [u8] = b"TLOGT";

  fn n(&self) -> u16 {
        self.n
    }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
   FreeFormat::parse_string_value_no_quote(kwr_value_comment)
    .and_then(|(val, _comment)| val.parse::<LogicalType>().map(|v| Self::new(n, v)))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Column {} logical type", self.n);
    FreeFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.to_string().as_str(),
      Some(comment.as_str()),
    )
  }
}