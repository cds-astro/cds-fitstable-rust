///! Defines both `NAXIS`, `NAXIS1`, `NAXIS2` and `NAXISn` keywords.
use crate::{
  common::{
    write::{FixedFormatWrite, KwrFormatWrite},
    DynValueKwr, FixedFormat, KwrFormatRead, ValueKwr,
  },
  error::{new_unexpected_value, Error},
};

/// The `NAXIS` keyword.
/// No need to use u32 or u64 since there is one as many `NAXISn` keywords as the `NAXIS` value,
/// and I do not expect a FITS header with more than 65000 `NAXISn` keywords...
pub struct NAxis(u16);

impl NAxis {
  pub const fn new(n_axis: u16) -> Self {
    Self(n_axis)
  }
  pub fn get(&self) -> u16 {
    self.0
  }
}

impl ValueKwr for NAxis {
  const KEYWORD: &'static [u8; 8] = b"NAXIS   ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as u16 != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self(val as u16))
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
      Some("Number of data axis"),
    )
  }
}

/// The `NAXIS1` keyword (see also `NAXISn` if the number of `NAXISn` is not known in advance).
pub struct NAxis1(u32);

impl NAxis1 {
  pub fn new(size: u32) -> Self {
    Self(size)
  }
  pub fn get(&self) -> u32 {
    self.0
  }
}

impl ValueKwr for NAxis1 {
  const KEYWORD: &'static [u8; 8] = b"NAXIS1  ";

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
      Some("Length of data axis 1"),
    )
  }
}

/// The `NAXIS2` keyword (see also `NAXISn` if the number of `NAXISn` is not known in advance).
pub struct NAxis2(u64);

impl NAxis2 {
  pub fn new(size: u64) -> Self {
    Self(size)
  }
  pub fn get(&self) -> u64 {
    self.0
  }
}

impl ValueKwr for NAxis2 {
  const KEYWORD: &'static [u8; 8] = b"NAXIS2  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as u64 != self.0 {
        Err(new_unexpected_value(self.0, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self(val as u64))
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
      Some("Length of data axis 1"),
    )
  }
}

/// The `NAXISn` keyword.
pub struct NAxisn {
  n: u16,
  value: u32,
}
impl NAxisn {
  /// # Params
  /// * `n` the `NAXISn` number in `[1, NAXIS]`.
  /// * `value` value associated to this `NAXISn` keyword, i.e. the axis len, or number of elements
  pub fn new(n: u16, value: u32) -> Self {
    Self { n, value }
  }

  pub fn axis_nbr(&self) -> u16 {
    self.n
  }
  pub fn axis_len(&self) -> u32 {
    self.value
  }
}

impl DynValueKwr for NAxisn {
  const KW_PREFIX: &'static [u8] = b"NAXIS";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as u32 != self.value {
        Err(new_unexpected_value(self.value, val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val >= 0 {
        Ok(Self::new(n, val as u32))
      } else {
        Err(new_unexpected_value("positive integer", val))
      }
    })
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Length of data axis {}", self.n);
    FixedFormatWrite::write_int_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value as i64,
      Some(comment.as_str()),
    )
  }
}
