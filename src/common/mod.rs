use std::{
  borrow::Cow,
  io::Write,
  iter::Peekable,
  num::{ParseFloatError, ParseIntError},
  ops::Range,
  ptr::copy_nonoverlapping,
};

use crate::{
  common::read::bytes2str,
  error::{new_depleted_reader_it, new_unexpected_kw, new_value_indicator_not_found, Error},
};

pub mod header;
pub mod keywords;
pub mod read;
pub mod write;

use self::{
  read::{FixedFormatRead, FreeFormatRead, KwrFormatRead},
  write::KwrFormatWrite,
};

/// Value of the value indicator, if present.
const VALUE_INDICATOR: &[u8; 2] = b"= ";
/// Value of the seprator between a value and the comment, if present.
const VALUE_COMMENT_SEPARATOR: &[u8; 3] = b" / ";

/// Keyword byte range in a raw keyword record.
pub(crate) const KW_RANGE: Range<usize> = 0..8;
/*
/// Value Indicator byte range in a raw keyword record.
pub(crate) const VI_RANGE: Range<usize> = 8..10;
/// Value plus comment byte range in a raw keyword record.
pub(crate) const VC_RANGE: Range<usize> = 10..80;
*/

/// A keyword record essentially containing a value.
/// The comment is not important: it is not parsed, always the same comment is written.
/// This trait is essentially dedicated to well known mandatory keywords.
pub trait ValueKwr: Sized {
  const KEYWORD: &'static [u8; 8];

  fn keyword_str() -> &'static str {
    unsafe { str::from_utf8_unchecked(Self::KEYWORD) }
  }
  fn keyword_string() -> String {
    String::from(Self::keyword_str())
  }

  fn check_keyword(found: &[u8; 8]) -> Result<(), Error> {
    if found != Self::KEYWORD {
      Err(new_unexpected_kw(Self::KEYWORD, found))
    } else {
      Ok(())
    }
  }

  fn check_value_indicator(found: &[u8; 2]) -> Result<(), Error> {
    if found != VALUE_INDICATOR {
      Err(new_value_indicator_not_found(VALUE_INDICATOR, found))
    } else {
      Ok(())
    }
  }

  /// Checks the keyword and the value indicator and returns the value + comment part of the
  /// given keyword record.
  /// # Info
  /// The returned error do contains a keyword record context.
  /// # TIP
  /// This method is called when a given keyword record is expected, thus for mandatory
  /// keyword record of pre-defined order.
  fn check_kw_and_value_indicator(kw_record: &[u8; 80]) -> Result<&[u8; 70], Error> {
    let (kw, ind, value_comment) = FixedFormat::split_kw_indicator_value(kw_record);
    Self::check_keyword(kw)
      .and_then(|()| Self::check_value_indicator(ind))
      .map_err(|e| e.kwr_context(kw_record))
      .map(|()| value_comment)
  }

  /// Here we expect a given keyword record (of given value) and want to verify we get the expected one.
  fn check_keyword_record(&self, kw_record: &[u8; 80]) -> Result<(), Error> {
    Self::check_kw_and_value_indicator(kw_record)
      .and_then(|bytes| self.check_value(bytes))
      .map_err(|e| e.kw_context(Self::KEYWORD))
  }

  fn check_keyword_record_it<'a, I>(&self, kw_record_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    kw_record_it
      .next()
      .ok_or_else(|| new_depleted_reader_it())
      .and_then(|(_, bytes)| self.check_keyword_record(bytes))
  }

  /// Check whether the value in the given part of keyword record matches with the value `self` contains.
  ///
  /// # Remarks
  /// * We assume that the keyword and the value indicator have already been checked and removed.
  /// * No need to add a context to the error, it is done at a higher level if needed.
  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error>;

  /// Create a new object, parsing the given value/comment part of a keyword record.
  /// # Remark
  /// * We assume that the keyword and the value indicator have already been checked and removed.
  /// * No need to add a context to the error, it is done at a higher level if needed.
  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error>;

  /// Here we expect a given keyword record and we need to extract the value.
  /// # TIP
  /// If the keyword has already been parsed/checked, call `from_value_comment` instead.
  fn from_keyword_record(kw_record: &[u8; 80]) -> Result<Self, Error> {
    Self::check_kw_and_value_indicator(kw_record)
      .and_then(Self::from_value_comment)
      .map_err(|e| e.kw_context(Self::KEYWORD))
  }

  fn from_keyword_record_it<'a, I>(kw_record_it: &mut I) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    kw_record_it
      .next()
      .ok_or_else(|| new_depleted_reader_it())
      .and_then(|(_, bytes)| Self::from_keyword_record(bytes))
  }

  /// # Param
  /// * `dest_kwr_it` iterator on empty keyword records to be written.
  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>;
}

/// Made for keywords having a name made of a prefix + a number part: `PREFIXn`,
/// like `TFORMn`, `NAXISn`, ...
pub trait DynValueKwr: Sized {
  ///# Warning: the prefix size must be lower than 8 bytes!!
  const KW_PREFIX: &'static [u8];

  fn n(&self) -> u16;

  fn kw_prefix_str() -> &'static str {
    bytes2str(Self::KW_PREFIX)
  }

  fn keyword(n: u16) -> [u8; 8] {
    let mut kw = [b' '; 8];
    let to = Self::KW_PREFIX.len();
    unsafe { copy_nonoverlapping(Self::KW_PREFIX.as_ptr(), kw.as_mut_ptr(), to) };
    write!(&mut kw[to..], "{}", n).expect(
      format!(
        "Too large value for keyword prefix {}",
        bytes2str(Self::KW_PREFIX)
      )
      .as_str(),
    );
    kw
  }

  fn keyword_string(n: u16) -> String {
    let kw = Self::keyword(n);
    String::from(unsafe { str::from_utf8_unchecked(kw.as_slice()) })
  }

  /// # Warning: internally, the keyword is built from `n`.
  fn check_keyword(n: u16, found: &[u8; 8]) -> Result<(), Error> {
    // We could have checked first the prefix, and then n.
    let kw = Self::keyword(n);
    if found != &kw {
      Err(new_unexpected_kw(&kw, found))
    } else {
      Ok(())
    }
  }

  fn check_value_indicator(found: &[u8; 2]) -> Result<(), Error> {
    if found != VALUE_INDICATOR {
      Err(new_value_indicator_not_found(VALUE_INDICATOR, found))
    } else {
      Ok(())
    }
  }

  /// Checks the keyword and the value indicator and returns the value + comment part of the
  /// given keyword record.
  /// # Info
  /// The returned error do contains a keyword record context.
  /// # TIP
  /// This method is called when a given keyword record is expected, thus for mandatory
  /// keyword record of pre-defined order.
  fn check_kw_and_value_indicator(n: u16, kw_record: &[u8; 80]) -> Result<&[u8; 70], Error> {
    let (kw, ind, value_comment) = FixedFormat::split_kw_indicator_value(kw_record);
    Self::check_keyword(n, kw)
      .and_then(|()| Self::check_value_indicator(ind))
      .map_err(|e| e.kwr_context(kw_record))
      .map(|()| value_comment)
  }

  /// Here we expect a given keyword record (of given value) and want to verify we get the expected one.
  fn check_keyword_record(&self, kw_record: &[u8; 80]) -> Result<(), Error> {
    let n = self.n();
    Self::check_kw_and_value_indicator(n, kw_record)
      .and_then(|bytes| self.check_value(bytes))
      .map_err(|e| e.kw_context(&Self::keyword(n)))
  }

  fn check_keyword_record_it<'a, I>(&self, kw_record_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    kw_record_it
      .next()
      .ok_or_else(|| new_depleted_reader_it())
      .and_then(|(_, bytes)| self.check_keyword_record(bytes))
  }

  /// Check whether the value in the given part of keyword record matches with the value `self` contains.
  ///
  /// # Remarks
  /// * We assume that the keyword and the value indicator have already been checked and removed.
  /// * No need to add a context to the error, it is done at a higher level if needed.
  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error>;

  /// Create a new object, parsing the given value/comment part of a keyword record.
  /// # Remark
  /// * We assume that the keyword and the value indicator have already been checked and removed.
  /// * No need to add a context to the error, it is done at a higher level if needed.
  fn from_value_comment(n: u16, kwr_value_comment: &[u8; 70]) -> Result<Self, Error>;

  /// Here we expect a given keyword record and we need to extract the value.
  /// # TIP
  /// If the keyword has already been parsed/checked, call `from_value_comment` instead.
  fn from_keyword_record(n: u16, kw_record: &[u8; 80]) -> Result<Self, Error> {
    Self::check_kw_and_value_indicator(n, kw_record)
      .and_then(|bytes| Self::from_value_comment(n, bytes))
      .map_err(|e| e.kw_context(&Self::keyword(n)))
  }

  fn from_keyword_record_it<'a, I>(n: u16, kw_record_it: &mut I) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    kw_record_it
      .next()
      .ok_or_else(|| new_depleted_reader_it())
      .and_then(|(_, bytes)| Self::from_keyword_record(n, bytes))
  }

  /// # Param
  /// * `dest_kwr_it` iterator on empty keyword records to be written.
  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>;
}

// ValueCommentKwr

/// Keyword Record Format.
/// Defines the methods to read and write the value and possibly the comment associated to
/// a keyword of known type in a keyword record.
trait KwrFormat: KwrFormatRead + KwrFormatWrite {}

// CONTINUE only for long string values!!

/// The main interest of *fixed-format* is to allow for faster parsing of mandatory keywords.
pub enum FixedFormat {}

impl KwrFormatRead for FixedFormat {
  fn parse_logical_value(part_of_kw_record: &[u8]) -> Result<(bool, &[u8]), Error> {
    FixedFormatRead::parse_logical_value(part_of_kw_record)
  }
  fn parse_integer_value(part_of_kw_record: &[u8]) -> Result<(i64, &[u8]), Error> {
    FixedFormatRead::parse_integer_value(part_of_kw_record)
  }

  fn parse_integer_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    FixedFormatRead::parse_integer_str_value(part_of_kw_record)
  }

  fn new_invalid_int_val_err(err: ParseIntError, part_of_kw_record: &[u8]) -> Error {
    FixedFormatRead::new_invalid_int_val_err(err, part_of_kw_record)
  }

  fn parse_real_value(part_of_kw_record: &[u8]) -> Result<(f64, &[u8]), Error> {
    FixedFormatRead::parse_real_value(part_of_kw_record)
  }

  fn parse_real_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    FixedFormatRead::parse_real_str_value(part_of_kw_record)
  }

  fn new_invalid_real_val_err(err: ParseFloatError, part_of_kw_record: &[u8]) -> Error {
    FixedFormatRead::new_invalid_real_val_err(err, part_of_kw_record)
  }

  fn parse_string_value_no_quote(part_of_kw_record: &[u8]) -> Result<(&str, &[u8]), Error> {
    FixedFormatRead::parse_string_value_no_quote(part_of_kw_record)
  }
  fn parse_string_value(part_of_kw_record: &[u8]) -> Result<(Cow<'_, str>, &[u8]), Error> {
    FixedFormatRead::parse_string_value(part_of_kw_record)
  }
  fn parse_possibly_long_string_value<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<Cow<'a, str>, Error> {
    FixedFormatRead::parse_possibly_long_string_value(part_of_kw_record, wk_record_it)
  }
  fn parse_possibly_long_string_value_and_comment<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<(Cow<'a, str>, Option<Cow<'a, str>>), Error> {
    FixedFormatRead::parse_possibly_long_string_value_and_comment(part_of_kw_record, wk_record_it)
  }
}

/*
impl KwrFormatWrite for FixedFormat {}

impl KwrFormat for FixedFormat {}
*/

pub enum FreeFormat {}

impl KwrFormatRead for FreeFormat {
  fn parse_logical_value(part_of_kw_record: &[u8]) -> Result<(bool, &[u8]), Error> {
    FreeFormatRead::parse_logical_value(part_of_kw_record)
  }
  fn parse_integer_value(part_of_kw_record: &[u8]) -> Result<(i64, &[u8]), Error> {
    FreeFormatRead::parse_integer_value(part_of_kw_record)
  }

  fn parse_integer_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    FreeFormatRead::parse_integer_str_value(part_of_kw_record)
  }

  fn new_invalid_int_val_err(err: ParseIntError, part_of_kw_record: &[u8]) -> Error {
    FreeFormatRead::new_invalid_int_val_err(err, part_of_kw_record)
  }

  fn parse_real_value(part_of_kw_record: &[u8]) -> Result<(f64, &[u8]), Error> {
    FreeFormatRead::parse_real_value(part_of_kw_record)
  }

  fn parse_real_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    FreeFormatRead::parse_real_str_value(part_of_kw_record)
  }

  fn new_invalid_real_val_err(err: ParseFloatError, part_of_kw_record: &[u8]) -> Error {
    FreeFormatRead::new_invalid_real_val_err(err, part_of_kw_record)
  }

  fn parse_string_value_no_quote(part_of_kw_record: &[u8]) -> Result<(&str, &[u8]), Error> {
    FreeFormatRead::parse_string_value_no_quote(part_of_kw_record)
  }
  fn parse_string_value(part_of_kw_record: &[u8]) -> Result<(Cow<'_, str>, &[u8]), Error> {
    FreeFormatRead::parse_string_value(part_of_kw_record)
  }
  fn parse_possibly_long_string_value<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<Cow<'a, str>, Error> {
    FreeFormatRead::parse_possibly_long_string_value(part_of_kw_record, wk_record_it)
  }
  fn parse_possibly_long_string_value_and_comment<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<(Cow<'a, str>, Option<Cow<'a, str>>), Error> {
    FreeFormatRead::parse_possibly_long_string_value_and_comment(part_of_kw_record, wk_record_it)
  }
}

/*
impl KwrFormatWrite for FreeFormat {}

impl KwrFormat for FreeFormat {}
*/
