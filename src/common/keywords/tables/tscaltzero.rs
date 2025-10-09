//! Defines the `TSCALn` and `TZEROn` keywords for `ASCIITABLE` and `BINTABLE` extensions.
//! In `BINTABLE`, it can be used with all TFORM values, except `A`, `L` and `X`.
//! `TSCALn` default value is 1.0 and `TZEROn` default value is 0.0.
//! They are used in equation:
//! > `field_value = TZEROn + TSCALn * stored_value`
//!

use std::str::FromStr;

use crate::common::read::bytes2str;
use crate::{
  common::{
    DynValueKwr, FixedFormat, KwrFormatRead,
    write::{FixedFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_invalid_free_fmt_float_val_err, new_invalid_free_fmt_int_val_err},
};

/// We have to use a enum here instead of always a f64 because of the possible usage of i64 offset
/// possible larger than +-2^52 (i.e. requiring more bits than the number of bits in a f64 mantissa).
///
/// In 4.2.3, "The integer representation shall always be interpreted as a signed, decimal number."
/// But, if we do so, e are not able to represent the OFFSET value used with the unsigned long
/// datatype (value 9_223_372_036_854_775_808 in Tab. 19).
#[derive(Clone, Copy, Debug)]
pub enum UIF64 {
  U64(u64),
  I64(i64),
  F64(f64),
}
impl UIF64 {
  pub fn is_0(&self) -> bool {
    match self {
      Self::U64(0) | Self::I64(-0) | Self::F64(0.0) => true,
      _ => false,
    }
  }
  pub fn is_i8_offset(&self) -> bool {
    matches!(self, UIF64::I64(i) if *i == -128_i64)
  }
  pub fn is_u16_offset(&self) -> bool {
    matches!(self, UIF64::U64(i) if *i == 32768_u64)
  }
  pub fn is_u32_offset(&self) -> bool {
    matches!(self, UIF64::U64(i) if *i == 2147483648_u64)
  }
  pub fn is_u64_offset(&self) -> bool {
    matches!(self, UIF64::U64(i) if *i == 9223372036854775808_u64)
  }
  pub fn as_f32(&self) -> f32 {
    match &self {
      Self::U64(u) => *u as f32,
      Self::I64(i) => *i as f32,
      Self::F64(f) => *f as f32,
    }
  }
  pub fn as_f64(&self) -> f64 {
    match &self {
      Self::U64(u) => *u as f64,
      Self::I64(i) => *i as f64,
      Self::F64(f) => *f,
    }
  }
}
impl FromStr for UIF64 {
  type Err = Error;

  // We assume the string as already been trimmed.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains(|c| c == '.' || c == 'e' || c == 'E') {
      s.parse::<f64>()
        .map(UIF64::F64)
        .map_err(|e| new_invalid_free_fmt_float_val_err(e, s.as_bytes()))
    } else if s.starts_with('-') {
      s.parse::<i64>()
        .map(UIF64::I64)
        .map_err(|e| new_invalid_free_fmt_int_val_err(e, s.as_bytes()))
    } else {
      s.parse::<u64>()
        .map(UIF64::U64)
        .map_err(|e| new_invalid_free_fmt_int_val_err(e, s.as_bytes()))
    }
  }
}

// unsigned values of 0, 32768, and 65535, for example, are physically stored in the FITS image as -32768, 0, and 32767,
// Tab. 19:
// TFORMnNative
// data typePhysical
// data type
// ’B’  unsigned  byte                           -128
// ’I’  signed    unsigned 16-bit               32768
// ’J’  signed    unsigned 32-bit          2147483648
// ’K’  signed    unsigned 64-bit 9223372036854775808

/* In rust playground!
fn main() {
    let o = 32768_u16;
    println!("o: {}", o);

    let s = -32768_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s as u16));

    let s = -1_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s as u16));

    let s = 0_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s as u16));

    let s = 1_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s as u16));

    let s = 32767_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s as u16));

    println!("-----------------------");

    let o = (32768_u16 as i16);
    println!("o: {}", o);

    let s = -32768_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s) as u16);

    let s = -1_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s) as u16);

    let s = 0_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s) as u16);

    let s = 1_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s) as u16);

    let s = 32767_i16;
    println!("s: {}", s);
    println!("t: {}", o.wrapping_add(s) as u16);
}
*/

/// The `TSCAL` keyword.
/// To be used for TFORM `B`, `I`, `J`, `K`, `P`, `Q` only.
#[derive(Debug)]
pub struct TScal {
  n: u16,
  value: f64,
}

impl TScal {
  /// # Params
  /// * `n` the `TSCALn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TSCALn` keyword, i.e. value for column number `n`
  pub fn new(n: u16, value: f64) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn scale(&self) -> f64 {
    self.value
  }
}

impl DynValueKwr for TScal {
  const KW_PREFIX: &'static [u8] = b"TSCAL";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_real_value(kwr_value_comment).map(|(val, _comment)| Self::new(n, val))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Scale value of column #{}", self.n);
    FixedFormatWrite::write_real_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value,
      None,
      Some(comment.as_str()),
    )
  }
}

/// The `TZERO` keyword.
/// To be used for TFORM `B`, `I`, `J`, `K`, `P`, `Q` only.
#[derive(Debug)]
pub struct TZero {
  n: u16,
  value: UIF64,
}

impl TZero {
  /// # Params
  /// * `n` the `TEROn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TZEROn` keyword, i.e. value for column number `n`
  pub fn new(n: u16, value: UIF64) -> Self {
    Self { n, value }
  }

  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn zero(&self) -> UIF64 {
    self.value
  }

  pub fn is_i8_offset(&self) -> bool {
    self.value.is_i8_offset()
  }
  pub fn is_u16_offset(&self) -> bool {
    self.value.is_u16_offset()
  }
  pub fn is_u32_offset(&self) -> bool {
    self.value.is_u32_offset()
  }
  pub fn is_u64_offset(&self) -> bool {
    self.value.is_u64_offset()
  }
  pub fn zero_as_f32(&self) -> f32 {
    self.value.as_f32()
  }
  pub fn zero_as_f64(&self) -> f64 {
    self.value.as_f64()
  }
}

impl DynValueKwr for TZero {
  const KW_PREFIX: &'static [u8] = b"TZERO";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_real_str_value(kwr_value_comment)
      .and_then(|(val, _comment)| bytes2str(val).parse::<UIF64>())
      .map(|val| Self::new(n, val))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Zero value (offset) of column #{}", self.n);
    match self.value {
      UIF64::U64(val) => FixedFormatWrite::write_uint_value_kw_record(
        dest_kwr_it,
        &Self::keyword(self.n),
        val,
        Some(comment.as_str()),
      ),
      UIF64::I64(val) => FixedFormatWrite::write_int_value_kw_record(
        dest_kwr_it,
        &Self::keyword(self.n),
        val,
        Some(comment.as_str()),
      ),
      UIF64::F64(val) => FixedFormatWrite::write_real_value_kw_record(
        dest_kwr_it,
        &Self::keyword(self.n),
        val,
        None,
        Some(comment.as_str()),
      ),
    }
  }
}
