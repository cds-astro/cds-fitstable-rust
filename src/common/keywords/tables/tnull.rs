//! Defines the `TNULLn` (i.e. value meaning NULL) keyword for `ASCIITABLE` and `BINTABLE` extensions.
use crate::{
  common::{
    DynValueKwr,
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, KwrFormatRead,
  },
  error::{Error},
};

/// The `TNull` keyword.
/// To be used for TFORM `B`, `I`, `J`, `K`, `P`, `Q` only.
pub struct TNull {
  n: u16,
  value: String,
}

impl TNull {
  /// # Params
  /// * `n` the `TNULLn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TNULLn` keyword, i.e. value coding NULL for column number `n`
  pub fn new(n: u16, value: String) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn col_null_value(&self) -> &str {
    self.value.as_str()
  }
}

impl DynValueKwr for TNull {
  const KW_PREFIX: &'static [u8] = b"TNULL";

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
    let comment = format!("Null value of column #{}", self.n);
    FixedFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.as_str(),
      Some(comment.as_str()),
    )
  }
}
