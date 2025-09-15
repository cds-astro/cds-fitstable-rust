use std::{
  fmt::{Display, Formatter, Write},
  str::FromStr,
};

use crate::{
  common::{
    DynValueKwr, FreeFormat,
    read::KwrFormatRead,
    write::{FreeFormatWrite, KwrFormatWrite},
  },
  error::{Error, new_0_or_1_repeatcount, new_unexpected_value, new_unexpected_value_list},
};

/// Repeat count and extra character.
pub struct RepeatCountAndExtraChar {
  /// Repeat count
  r: Option<u16>,
  /// Extra char (?)
  a: Option<u8>,
}
impl RepeatCountAndExtraChar {
  fn new(r: Option<u16>, a: Option<u8>) -> Self {
    Self { r, a }
  }
  fn to_string_with(&self, letter: u8) -> String {
    match (self.r, self.a) {
      (None, None) => format!("{}", letter as char),
      (Some(r), None) => format!("{}{}", r, letter as char),
      (None, Some(a)) => format!("{}{}", letter as char, a as char),
      (Some(r), Some(a)) => format!("{}{}{}", r, letter as char, a as char),
    }
  }
  pub fn repeat_count(&self) -> u16 {
    self.r.unwrap_or(1)
  }
}
/*
impl From<ZeroOrOneRepeatCountAndExtraChar> for RepeatCountAndExtraChar {
  fn from(value: ZeroOrOneRepeatCountAndExtraChar) -> Self {
    let r = value.r_is_1.map(|b| b as u16);
    Self::new(r, value.a)
  }
}*/

/// Store `r`, `X`, `nbr`, `a` in `r[PQ]X(nbr)a`.
pub struct VariableLenghtArrayInfo {
  /// Value is 0 et Some(false), else it is 1.
  r_is_1: Option<bool>,
  /// Data type of the array elements
  data_type: VariableLenghtArrayDataType,
  /// Maximum number of element in the array
  max_len: u16,
  /// Extra char (?)
  a: Option<u8>,
}
impl VariableLenghtArrayInfo {
  fn new(
    r_is_1: Option<bool>,
    data_type: VariableLenghtArrayDataType,
    max_len: u16,
    a: Option<u8>,
  ) -> Self {
    Self {
      r_is_1,
      data_type,
      max_len,
      a,
    }
  }
  fn to_string_with(&self, letter: u8) -> String {
    match (self.r_is_1, self.a) {
      (None, None) => format!(
        "{}{}({})",
        letter as char,
        self.data_type.char(),
        self.max_len
      ),
      (Some(r), None) => format!(
        "{}{}{}({})",
        r as u16,
        letter as char,
        self.data_type.char(),
        self.max_len
      ),
      (None, Some(a)) => format!(
        "{}{}({}){}",
        letter as char,
        self.data_type.char(),
        self.max_len,
        a as char
      ),
      (Some(r), Some(a)) => format!(
        "{}{}{}({}){}",
        r as u16,
        letter as char,
        self.data_type.char(),
        self.max_len,
        a as char
      ),
    }
  }
  pub fn is_repeat_count_eq_1(&self) -> bool {
    self.r_is_1.unwrap_or(true)
  }
  pub fn data_type(&self) -> VariableLenghtArrayDataType {
    self.data_type
  }
  pub fn max_len(&self) -> u16 {
    self.max_len
  }
}

/*
impl TryFrom<RepeatCountAndExtraChar> for ZeroOrOneRepeatCountAndExtraChar {
  type Error = Error;

  fn try_from(value: RepeatCountAndExtraChar) -> Result<Self, Self::Error> {
    match value.r {
      Some(1) => Ok(Self::new(Some(true), value.a)),
      Some(0) => Ok(Self::new(Some(false), value.a)),
      None => Ok(Self::new(None, value.a)),
      Some(r) => Err(new_0_or_1_repeatcount(r)),
    }
  }
}*/

#[derive(Debug, Clone, Copy)]
pub enum VariableLenghtArrayDataType {
  /// Logical (bool)
  L,
  // Bit encoded on bytes
  // X,
  /// Unsigned Byte (u8)
  B,
  /// Short integer (i16)
  I,
  /// Integer (i32)
  J,
  /// Long integer (i64)
  K,
  /// Character ASCII (u8)
  A,
  /// Float (f32)
  E,
  /// Double (f64)
  D,
  /// Complex f32 (f32, f32)
  C,
  /// Complex f64 (f64, f64)
  M,
}
impl VariableLenghtArrayDataType {
  pub fn char(&self) -> char {
    match &self {
      Self::L => 'L',
      // Self::X => 'X',
      Self::B => 'B',
      Self::I => 'I',
      Self::J => 'J',
      Self::K => 'K',
      Self::A => 'A',
      Self::E => 'E',
      Self::D => 'D',
      Self::C => 'C',
      Self::M => 'M',
    }
  }
  pub fn from_char(c: u8) -> Result<Self, Error> {
    match c {
      b'L' => Ok(Self::L),
      // b'X' => Ok(Self::X),
      b'B' => Ok(Self::B),
      b'I' => Ok(Self::I),
      b'J' => Ok(Self::J),
      b'K' => Ok(Self::K),
      b'A' => Ok(Self::A),
      b'E' => Ok(Self::E),
      b'D' => Ok(Self::D),
      b'C' => Ok(Self::C),
      b'M' => Ok(Self::M),
      _ => Err(new_unexpected_value_list(
        &["L", "X", "B", "I", "J", "K", "A", "E", "D", "C", "M"],
        &[c],
      )),
    }
  }
}
impl FromStr for VariableLenghtArrayDataType {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.len() == 1 {
      Self::from_char(s.as_bytes()[0])
    } else {
      Err(new_unexpected_value_list(
        &["L", "X", "B", "I", "J", "K", "A", "E", "D", "C", "M"],
        s.as_ref(),
      ))
    }
  }
}

pub enum TFormValue {
  /// Logical (bool)
  L(RepeatCountAndExtraChar),
  /// Bit encoded on bytes
  X(RepeatCountAndExtraChar),
  /// Unsigned Byte (u8)
  B(RepeatCountAndExtraChar),
  /// Short integer (i16)
  I(RepeatCountAndExtraChar),
  /// Integer (i32)
  J(RepeatCountAndExtraChar),
  /// Long integer (i64)
  K(RepeatCountAndExtraChar),
  /// Character ASCII (u8)
  A(RepeatCountAndExtraChar),
  /// Float (f32)
  E(RepeatCountAndExtraChar),
  /// Double (f64)
  D(RepeatCountAndExtraChar),
  /// Complex f32 (f32, f32)
  C(RepeatCountAndExtraChar),
  /// Complex f64 (f64, f64)
  M(RepeatCountAndExtraChar),
  /// Array descriptor 32-bit (u32)
  P(VariableLenghtArrayInfo),
  /// Array descriptor 64-bit (u64)
  Q(VariableLenghtArrayInfo),
}

impl TFormValue {
  /*fn letter(&self) -> u8 {
    match self {
      Self::L(_) => b'L',
      Self::X(_) => b'X',
      Self::B(_) => b'B',
      Self::I(_) => b'I',
      Self::J(_) => b'J',
      Self::K(_) => b'K',
      Self::A(_) => b'A',
      Self::E(_) => b'E',
      Self::D(_) => b'D',
      Self::C(_) => b'C',
      Self::M(_) => b'M',
      Self::P(_) => b'P',
      Self::Q(_) => b'Q',
    }
  }*/
}

impl FromStr for TFormValue {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();
    let il = bytes.partition_point(|s| s.is_ascii_digit());
    let r = if il == 0 {
      None
    } else {
      // Unwrap is ok since we tested for digit.
      // The only possible problem comes from a value > u16::MAX_VALUE
      Some(
        unsafe { str::from_utf8_unchecked(&bytes[..il]) }
          .parse::<u16>()
          .unwrap(),
      )
    };
    let main_dt = bytes[il];
    if main_dt == b'P' || main_dt == b'Q' {
      let err = new_unexpected_value("(\\d+)?[PQ][LXBIJKAEDCM](len)\\D?", s);
      let is_r_1 = match r {
        Some(1) => Ok(Some(true)),
        Some(0) => Ok(Some(false)),
        None => Ok(None),
        Some(r) => Err(new_0_or_1_repeatcount(r)),
      }?;
      // +1 for datatype, +1 for '(', at least +1 for nbr, +1 for ')'
      if il + 4 < bytes.len() {
        // Parse data type
        let array_dt = VariableLenghtArrayDataType::from_char(bytes[il + 1])?;
        // Parse `(max_len)`
        if bytes[il + 2] != b'(' {
          return Err(err);
        }
        let bytes = &bytes[il + 3..];
        let il = bytes.partition_point(|s| s.is_ascii_digit());
        if il == 0 {
          return Err(err);
        }
        // Unwrap is ok since we tested for digit.
        // The only possible problem comes from a value > u16::MAX_VALUE
        let max_len = unsafe { str::from_utf8_unchecked(&bytes[..il]) }
          .parse::<u16>()
          .unwrap();
        if bytes[il] != b')' {
          return Err(err);
        }
        // Parse extra char (if any)
        let a = if il + 1 < bytes.len() {
          None
        } else {
          Some(bytes[il + 1])
        };
        match main_dt {
          b'P' => Ok(Self::P(VariableLenghtArrayInfo::new(
            is_r_1, array_dt, max_len, a,
          ))),
          b'Q' => Ok(Self::Q(VariableLenghtArrayInfo::new(
            is_r_1, array_dt, max_len, a,
          ))),
          _ => unreachable!(), // because we are inside "if main_dt == b'P' || main_dt == b'Q'"
        }
      } else {
        Err(err)
      }
    } else {
      let a = if il + 1 < bytes.len() {
        None
      } else {
        Some(bytes[il + 1])
      };
      let rcec = RepeatCountAndExtraChar::new(r, a);
      match bytes[il] {
        b'L' => Ok(Self::L(rcec)),
        b'X' => Ok(Self::X(rcec)),
        b'B' => Ok(Self::B(rcec)),
        b'I' => Ok(Self::I(rcec)),
        b'J' => Ok(Self::J(rcec)),
        b'K' => Ok(Self::K(rcec)),
        b'A' => Ok(Self::A(rcec)),
        b'E' => Ok(Self::E(rcec)),
        b'D' => Ok(Self::D(rcec)),
        b'C' => Ok(Self::C(rcec)),
        b'M' => Ok(Self::M(rcec)),
        _ => Err(new_unexpected_value("(\\d+)?[LXBIJKAEDCM]\\D?", s)),
      }
    }
  }
}

impl Display for TFormValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(
      match self {
        Self::L(e) => e.to_string_with(b'L'),
        Self::X(e) => e.to_string_with(b'X'),
        Self::B(e) => e.to_string_with(b'B'),
        Self::I(e) => e.to_string_with(b'I'),
        Self::J(e) => e.to_string_with(b'J'),
        Self::K(e) => e.to_string_with(b'K'),
        Self::A(e) => e.to_string_with(b'A'),
        Self::E(e) => e.to_string_with(b'E'),
        Self::D(e) => e.to_string_with(b'D'),
        Self::C(e) => e.to_string_with(b'C'),
        Self::M(e) => e.to_string_with(b'M'),
        Self::P(e) => e.to_string_with(b'P'),
        Self::Q(e) => e.to_string_with(b'Q'),
      }
      .as_str(),
    )
  }
}

pub struct TFormn {
  n: u16,
  value: TFormValue,
}
impl TFormn {
  /// # Params
  /// * `n` the `TFORMn` number in `[1, TFIELD]`.
  /// * `value` value associated to this `TFORMn` keyword, i.e. column data type
  pub fn new(n: u16, value: TFormValue) -> Self {
    Self { n, value }
  }

  /// Column number starts at 1.
  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn tform_type(&self) -> &TFormValue {
    &self.value
  }
}

impl DynValueKwr for TFormn {
  const KW_PREFIX: &'static [u8] = b"TFORM";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FreeFormat::parse_string_value_no_quote(kwr_value_comment)
      .and_then(|(val, _comment)| val.parse::<TFormValue>().map(|v| Self::new(n, v)))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Column {} data type", self.n);
    FreeFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.to_string().as_str(),
      Some(comment.as_str()),
    )
  }
}
