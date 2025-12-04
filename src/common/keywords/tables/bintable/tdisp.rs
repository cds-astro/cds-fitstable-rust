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
  error::{Error, new_parse_u16_err, new_unexpected_value},
};

/// Remainder: display '*****' (`w` times) if width of string representation is larger than `w`.
#[derive(Debug)]
pub enum TDispValue {
  // Char format
  A { w: u16 },
  // Logical format
  L { w: u16 },
  // Integer format
  // - regular representation
  I { w: u16, m: Option<u16> },
  // - binary representation
  B { w: u16, m: Option<u16> },
  // - octal representation
  O { w: u16, m: Option<u16> },
  // - hexadecimal representation
  Z { w: u16, m: Option<u16> },
  // Float format
  // - decimal representation
  F { w: u16, d: u16 },
  // - scientific representation. Fraction in 0.1 <= frac < 1.0
  E { w: u16, d: u16, e: Option<u16> },
  // - scientific with exponent multiple of 3. Fraction in 1.0 <= frac < 1000
  EN { w: u16, d: u16 },
  // - scientific, like EN, but non 0 leading digits if non 0. Fraction in 1.0 <= frac < 10
  ES { w: u16, d: u16 },
  // - generic: either F or E
  G { w: u16, d: u16, e: Option<u16> },
  // - same a E
  D { w: u16, d: u16, e: Option<u16> },
}
impl TDispValue {
  pub fn get_width_and_prec(&self) -> (u16, Option<u16>) {
    match self {
      Self::A { w } |
      Self::L { w } |
      Self::I { w, m: _ } |
      Self::B { w, m: _ } |
      Self::O { w, m: _ } |
      Self::Z { w, m: _ } => (*w, None),
      Self::F { w, d } |
      Self::E { w, d, e: _ } |
      Self::EN { w, d } |
      Self::ES { w, d } |
      Self::G { w, d, e: _ } |
      Self::D { w, d, e: _ } => (*w, Some(*d))
    }
  }
}
impl FromStr for TDispValue {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (letter, tail) = s.split_at(1);
    match letter {
      "A" => parse_w(tail).map(|w| TDispValue::A { w }),
      "L" => parse_w(tail).map(|w| TDispValue::L { w }),
      "I" => parse_wm(tail).map(|(w, m)| TDispValue::I { w, m }),
      "B" => parse_wm(tail).map(|(w, m)| TDispValue::B { w, m }),
      "O" => parse_wm(tail).map(|(w, m)| TDispValue::O { w, m }),
      "Z" => parse_wm(tail).map(|(w, m)| TDispValue::Z { w, m }),
      "F" => parse_wd(tail).map(|(w, d)| TDispValue::F { w, d }),
      "E" => match tail.chars().next() {
        Some('N') => parse_wd(tail.split_at(1).1).map(|(w, d)| TDispValue::EN { w, d }),
        Some('S') => parse_wd(tail.split_at(1).1).map(|(w, d)| TDispValue::ES { w, d }),
        _ => parse_wde(tail).map(|(w, d, e)| TDispValue::E { w, d, e }),
      },
      "G" => parse_wde(tail).map(|(w, d, e)| TDispValue::G { w, d, e }),
      "D" => parse_wde(tail).map(|(w, d, e)| TDispValue::D { w, d, e }),
      _ => Err(new_unexpected_value("TDISP should starts [ALIBOZFEGD]", s)),
    }
    .map_err(|e| e.kw_val_context(b"TDISPn  ", s))
  }
}
/*impl TDispValue {
  pub fn display(&self, field: Field) -> String {
    match (&self, field) {

    }
  }
}*/
fn parse_w(s: &str) -> Result<u16, Error> {
  s.parse::<u16>().map_err(new_parse_u16_err)
}
fn parse_wm(s: &str) -> Result<(u16, Option<u16>), Error> {
  match s.split_once('.') {
    Some((w, m)) => w
      .parse::<u16>()
      .and_then(|w| m.parse::<u16>().map(|m| (w, Some(m)))),
    None => s.parse::<u16>().map(|w| (w, None)),
  }
  .map_err(new_parse_u16_err)
}
fn parse_wd(s: &str) -> Result<(u16, u16), Error> {
  match s.split_once('.') {
    Some((w, d)) => w
      .parse::<u16>()
      .and_then(|w| d.parse::<u16>().map(|d| (w, d)))
      .map_err(new_parse_u16_err),
    None => Err(new_unexpected_value("w.d", s)),
  }
}
fn parse_wde(s: &str) -> Result<(u16, u16, Option<u16>), Error> {
  match s.split_once('E') {
    Some((wd, e)) => e
      .parse::<u16>()
      .map(|e| (wd, Some(e)))
      .map_err(new_parse_u16_err),
    None => Ok((s, None)),
  }
  .and_then(|(wd, e)| parse_wd(wd).map(|(w, d)| (w, d, e)))
}

impl Display for TDispValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::A { w } => f.write_fmt(format_args!("A{}", w)),
      Self::L { w } => f.write_fmt(format_args!("L{}", w)),
      Self::I { w, m } => match m {
        Some(m) => f.write_fmt(format_args!("I{}.{}", w, m)),
        None => f.write_fmt(format_args!("I{}", w)),
      },
      Self::B { w, m } => match m {
        Some(m) => f.write_fmt(format_args!("B{}.{}", w, m)),
        None => f.write_fmt(format_args!("B{}", w)),
      },
      Self::O { w, m } => match m {
        Some(m) => f.write_fmt(format_args!("D{}.{}", w, m)),
        None => f.write_fmt(format_args!("D{}", w)),
      },
      Self::Z { w, m } => match m {
        Some(m) => f.write_fmt(format_args!("Z{}.{}", w, m)),
        None => f.write_fmt(format_args!("Z{}", w)),
      },
      Self::F { w, d } => f.write_fmt(format_args!("F{}.{}", w, d)),
      Self::E { w, d, e } => match e {
        Some(e) => f.write_fmt(format_args!("E{}.{}E{}", w, d, e)),
        None => f.write_fmt(format_args!("E{}.{}", w, d)),
      },
      Self::EN { w, d } => f.write_fmt(format_args!("EN{}.{}", w, d)),
      Self::ES { w, d } => f.write_fmt(format_args!("ES{}.{}", w, d)),
      Self::G { w, d, e } => match e {
        Some(e) => f.write_fmt(format_args!("G{}.{}E{}", w, d, e)),
        None => f.write_fmt(format_args!("G{}.{}", w, d)),
      },
      Self::D { w, d, e } => match e {
        Some(e) => f.write_fmt(format_args!("D{}.{}E{}", w, d, e)),
        None => f.write_fmt(format_args!("D{}.{}", w, d)),
      },
    }
  }
}

#[derive(Debug)]
pub struct TDispn {
  n: u16,
  value: TDispValue,
}
impl TDispn {
  /// # Params
  /// * `n` the `NAXISn` number in `[1, NAXIS]`.
  /// * `value` value associated to this `NAXISn` keyword, i.e. the axis len, or number of elements
  pub fn new(n: u16, value: TDispValue) -> Self {
    Self { n, value }
  }

  /// Column number starts at 1.
  pub fn col_nbr(&self) -> u16 {
    self.n
  }
  pub fn data_type(&self) -> &TDispValue {
    &self.value
  }
}

impl DynValueKwr for TDispn {
  const KW_PREFIX: &'static [u8] = b"TFORM";

  fn n(&self) -> u16 {
    self.n
  }

  fn check_value(&self, _kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    unreachable!() // not supposed to be called
  }

  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FreeFormat::parse_string_value_no_quote(kwr_value_comment)
      .and_then(|(val, _comment)| val.parse::<TDispValue>().map(|v| Self::new(n, v)))
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    let comment = format!("Column {} display info", self.n);
    FreeFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      &Self::keyword(self.n),
      self.value.to_string().as_str(),
      Some(comment.as_str()),
    )
  }
}
