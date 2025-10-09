use std::{
  borrow::Cow,
  iter::Peekable,
  num::{ParseFloatError, ParseIntError},
};

use crate::error::{
  new_empty_hierarch_kw_err, new_empty_val_bool_err, new_empty_val_float_err,
  new_empty_val_int_err, new_hierarch_kwval_sep_not_found_err, new_invalid_fixed_fmt_bool_val_err,
  new_invalid_fixed_fmt_float_val_err, new_invalid_fixed_fmt_int_val_err,
  new_invalid_free_fmt_bool_val_err, new_invalid_free_fmt_float_val_err,
  new_invalid_free_fmt_int_val_err, new_string_value_closing_not_found_err,
  new_string_value_opening_not_found_err, Error,
};

use super::{VALUE_INDICATOR /* KW_RANGE, VC_RANGE, VI_RANGE*/};

/// Keyword Record Format.
/// Defines the methods to read and write the value and possibly the comment associated to
/// a keyword of known type in a keyword record.
/// # Important
/// * input slice **must** start after the the value indicator, i.e. at byte 10 out of 80 (possibly
/// a higher byte if used with the `HIERARCH` convention.
pub trait KwrFormatRead {
  /// Split a keyword value into 3 parts:
  /// * the keyword, bytes 1 o 8;
  /// * the value indicator, bytes 9 and 10;
  /// * the value plus possibly the comment, bytes 11 to 80.
  fn split_kw_indicator_value(kw_record: &[u8; 80]) -> (&[u8; 8], &[u8; 2], &[u8; 70]) {
    let (k, r) = kw_record.split_at(8);
    let (v, r) = r.split_at(2);
    // SAFETY: we know exactly where we split and the input size. So no problem.
    (
      unsafe { &*(k.as_ptr().cast::<[u8; 8]>()) },
      unsafe { &*(v.as_ptr().cast::<[u8; 2]>()) },
      unsafe { &*(r.as_ptr().cast::<[u8; 70]>()) },
    )
  }

  /// Parse the remaining comment (if any) on the value has already been parsed and removed from
  /// the given slice.
  fn parse_value_comment(part_of_kw_record: &[u8]) -> Option<&str> {
    if let [b'/', tail @ ..] = part_of_kw_record.trim_ascii_start() {
      Some(bytes2str(tail.trim_ascii()))
    } else {
      None
    }
  }

  /// In a keyword record starting by `HIERARCH  `, parse the keyword and return it together with
  /// the bytes located after the `=` value separator.
  /// The 10 first bytes (`HIERARCH  `) must have been remove from the given `part_of_kw_record` slice.
  fn parse_keyword_in_hierarch(
    part_of_kw_record: &[u8; 71],
  ) -> Result<(Cow<'_, str>, &[u8]), Error> {
    match part_of_kw_record.iter().position(|&b| b == b'=') {
      Some(i) => {
        let kw = &part_of_kw_record[..i];
        let tail = &part_of_kw_record[i + 1..];
        match kw
          .split(|b| b.is_ascii_whitespace())
          .filter(|s| !s.is_empty())
          .map(String::from_utf8_lossy)
          .reduce(|mut acc, e| {
            acc += ".";
            acc += e;
            acc
          }) {
          Some(kw) => Ok((kw, tail)),
          None => Err(new_empty_hierarch_kw_err(part_of_kw_record)),
        }
      }
      None => Err(new_hierarch_kwval_sep_not_found_err(part_of_kw_record)),
    }
  }

  /// Parse a logical value, either a 'T' or a 'F':
  /// * *fixed-format*:  at byte 30 (i.e. 20 from the end of the value indicator).
  /// * *free-format*: anywhere from byte 11 through 80 (i.e. from 1 to 70 from the end of the value indicator).
  ///
  /// # return
  /// * the parsed value
  /// * the remaining, unparsed, part of the keyword record.
  fn parse_logical_value(part_of_kw_record: &[u8]) -> Result<(bool, &[u8]), Error>;

  /// Parse an integer value:
  /// * *fixed-format*: right justified from bytes 11 through 30 (i.e. from 1 to 20 from the end of the value)
  /// * *free-format*: rom byte 11 through 80 (i.e. from 1 to 70 from the end of the value indicator)
  ///   leading '+' and '0's are possible.
  ///
  /// # return
  /// * the parsed value
  /// * the remaining, unparsed, part of the keyword record.
  fn parse_integer_value(part_of_kw_record: &[u8]) -> Result<(i64, &[u8]), Error> {
    Self::parse_integer_str_value(part_of_kw_record).and_then(|(v, c)| {
      bytes2str(v)
        .parse::<i64>()
        .map_err(|err| Self::new_invalid_int_val_err(err, part_of_kw_record))
        .map(|v| (v, c))
    })
  }

  /// Returns the string containing the integer value to be parsed.
  /// * *fixed-format*: right justified from bytes 11 through 30 (i.e. from 1 to 20 from the end of the value)
  /// * *free-format*: rom byte 11 through 80 (i.e. from 1 to 70 from the end of the value indicator)
  ///   leading '+' and '0's are possible.
  ///
  /// # return
  /// * the string before trying to parse it into an integer.
  /// * the remaining, unparsed, part of the keyword record.
  fn parse_integer_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error>;

  /// Returns a specific error in case of invalid integer error.
  fn new_invalid_int_val_err(err: ParseIntError, part_of_kw_record: &[u8]) -> Error;

  /// Parse an integer value:
  /// * *fixed-format*: right justified from bytes 11 through 30 (i.e. from 1 to 20 from the end of the value)
  /// * *free-format*: from byte 11 through 80 (i.e. from 1 to 70 from the end of the value indicator).
  ///
  /// # return
  /// * the parsed value
  /// * the remaining, unparsed, part of the keyword record.
  fn parse_real_value(part_of_kw_record: &[u8]) -> Result<(f64, &[u8]), Error> {
    Self::parse_real_str_value(part_of_kw_record).and_then(|(v, c)| {
      bytes2str(v)
        .parse::<f64>()
        .map_err(|err| Self::new_invalid_real_val_err(err, part_of_kw_record))
        .map(|v| (v, c))
    })
  }

  /// Returns the string containing the real value to be parsed.
  /// * *fixed-format*: right justified from bytes 11 through 30 (i.e. from 1 to 20 from the end of the value)
  /// * *free-format*: rom byte 11 through 80 (i.e. from 1 to 70 from the end of the value indicator)
  ///   leading '+' and '0's are possible.
  ///
  /// # return
  /// * the string before trying to parse it into an real.
  /// * the remaining, unparsed, part of the keyword record.
  fn parse_real_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error>;

  /// Returns a specific error in case of invalid real error.
  fn new_invalid_real_val_err(err: ParseFloatError, part_of_kw_record: &[u8]) -> Error;

  /// Parse a string value enclosed in `'`, when **we know for sure that the string does not
  /// contains a single quote**.
  /// Leading spaces are significant; trailing spaces are not.
  /// * *fixed-format*: the starting `'` must be at position 11/80, i.e. 0/70.
  /// * *free-format*:  the starting `'` can be anywhere from byte 11/80 (inclusive).
  fn parse_string_value_no_quote(part_of_kw_record: &[u8]) -> Result<(&str, &[u8]), Error>;

  /// Parse a string value enclosed between two simple quotes `'`.
  /// Inside, a single quote `'` is encoded by two successive  single quotes `''`.
  /// Leading spaces are significant; trailing spaces are not.
  /// * *fixed-format*: the starting `'` must be at position 11/80, i.e. 0/70.
  /// * *free-format*:  the starting `'` can be anywhere from byte 11/80 (inclusive).
  ///
  /// # Remark
  /// We have to return a `Cow<'_, str>` (instead of a `&str`) because we possibly have to replace
  /// two consecutive single quotes by a single quote.
  fn parse_string_value(part_of_kw_record: &[u8]) -> Result<(Cow<'_, str>, &[u8]), Error>;

  /// Parse a string value possibly split on several lines (using `CONTINUE`). if so,
  /// the trim content of each keyword record is concatenated.
  ///
  /// # Remark
  /// We pass the keyword record iterator because of the `CONTINUE` convention in which the
  /// value must be splitted on several keyord records.
  fn parse_possibly_long_string_value<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<Cow<'a, str>, Error>;

  /// Same as `parse_possibly_long_string_value`, also possibly concatenating the comments.
  fn parse_possibly_long_string_value_and_comment<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<(Cow<'a, str>, Option<Cow<'a, str>>), Error>;

  // add write methods!
  // => again, need specific action for multi-line string values!!
}

// Utility methods
pub(crate) const fn bytes2str(bytes: &[u8]) -> &str {
  unsafe { str::from_utf8_unchecked(bytes) }
}

/*
pub(crate) fn get_keyword(keyword_record: &[u8; 80]) -> &[u8; 8] {
  let slice = &keyword_record[KW_RANGE];
  unsafe { &*(slice.as_ptr().cast::<[u8; 8]>()) }
}
pub(crate) fn get_value_indicator(keyword_record: &[u8; 80]) -> &[u8; 2] {
  let slice = &keyword_record[VI_RANGE];
  unsafe { &*(slice.as_ptr().cast::<[u8; 2]>()) }
}
pub(crate) fn get_value_comment(keyword_record: &[u8; 80]) -> &[u8; 70] {
  let slice = &keyword_record[VC_RANGE];
  unsafe { &*(slice.as_ptr().cast::<[u8; 70]>()) }
}
pub(crate) fn get_left_trimmed_value_comment(keyword_record: &[u8; 80]) -> &[u8] {
  get_value_comment(keyword_record).trim_ascii_start()
}
*/

pub(crate) fn is_value_indicator(bytes: &[u8; 2]) -> bool {
  bytes == VALUE_INDICATOR
}

/// The main interest of *fixed-format* is to allow for faster parsing of mandatory keywords.
pub enum FixedFormatRead {}
impl FixedFormatRead {
  /// # Warning
  /// The input slice **must** be of size 70.
  fn split_value_comment(part_of_kw_record: &[u8]) -> (&[u8; 20], &[u8; 50]) {
    assert_eq!(part_of_kw_record.len(), 70);
    let (v, c) = part_of_kw_record.split_at(20);
    // SAFETY: we know exactly where we split and the input size. So no problem.
    // AU pire, on ne cast pas les deuxime et on retourne un simple &[u8]
    (unsafe { &*(v.as_ptr().cast::<[u8; 20]>()) }, unsafe {
      &*(c.as_ptr().cast::<[u8; 50]>())
    })
  }
}

impl KwrFormatRead for FixedFormatRead {
  /// Value 'T' of 'F' at index 29/80 or 19/70.
  /// # Warning
  /// The input
  fn parse_logical_value(part_of_kw_record: &[u8]) -> Result<(bool, &[u8]), Error> {
    let (v, c) = Self::split_value_comment(part_of_kw_record);
    // Should we check that all bytes before index 19 are b' '? (So far, we do not)
    match v[19] {
      b'T' => Ok((true, c)),
      b'F' => Ok((false, c)),
      c => Err(new_invalid_fixed_fmt_bool_val_err(c, part_of_kw_record)),
    }
  }

  fn parse_integer_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let (v, c) = Self::split_value_comment(part_of_kw_record);
    let v = v.trim_ascii_start();
    // also check that c does not starts with a digit?
    if v.len() > 0 {
      Ok((v, c))
    } else {
      Err(new_empty_val_int_err())
    }
  }
  fn new_invalid_int_val_err(err: ParseIntError, part_of_kw_record: &[u8]) -> Error {
    new_invalid_fixed_fmt_int_val_err(err, part_of_kw_record)
  }

  fn parse_real_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let (v, c) = Self::split_value_comment(part_of_kw_record);
    let v = v.trim_ascii_start();
    // also check that c does not starts with a digit?
    if v.len() > 0 {
      Ok((v, c))
    } else {
      Err(new_empty_val_float_err())
    }
  }
  fn new_invalid_real_val_err(err: ParseFloatError, part_of_kw_record: &[u8]) -> Error {
    new_invalid_fixed_fmt_float_val_err(err, part_of_kw_record)
  }

  /// The first byte is the opening single quote.
  fn parse_string_value_no_quote(part_of_kw_record: &[u8]) -> Result<(&str, &[u8]), Error> {
    if let [b'\'', tail @ ..] = part_of_kw_record {
      // look for closing '
      match tail.iter().position(|&b| b == b'\'') {
        Some(index) => Ok((bytes2str(&tail[..index]), &tail[index + 1..])),
        None => Err(new_string_value_closing_not_found_err(part_of_kw_record)),
      }
    } else {
      Err(new_string_value_opening_not_found_err(part_of_kw_record))
    }
  }

  /// The first byte is the opening single quote.
  fn parse_string_value(part_of_kw_record: &[u8]) -> Result<(Cow<'_, str>, &[u8]), Error> {
    let mut res = Cow::default();
    let mut sub = part_of_kw_record;
    loop {
      if let [b'\'', tail @ ..] = sub {
        match tail.iter().position(|&b| b == b'\'') {
          Some(i) => {
            if tail.get(i + 1).map(|b| *b == b'\'').unwrap_or(false) {
              res += Cow::from(bytes2str(&tail[..=i])); // includes the first single quote
              sub = &tail[i + 1..]; // includes the second single quote
            } else {
              res += Cow::from(bytes2str(&tail[..i]).trim_end()); // excludes the single quote
              return Ok((res, &tail[i + 1..]));
            }
          }
          None => return Err(new_string_value_closing_not_found_err(part_of_kw_record)),
        }
      } else {
        // Can be raised only at the first loop iteration
        return Err(new_string_value_opening_not_found_err(part_of_kw_record));
      }
    }
  }

  fn parse_possibly_long_string_value<'a, I: Iterator<Item = &'a [u8; 80]>>(
    _part_of_kw_record: &'a [u8],
    _wk_record_it: &mut Peekable<I>,
  ) -> Result<Cow<'a, str>, Error> {
    // Since no long string value possible in 'fixed-format'
    unreachable!()
  }

  fn parse_possibly_long_string_value_and_comment<'a, I: Iterator<Item = &'a [u8; 80]>>(
    _part_of_kw_record: &'a [u8],
    _wk_record_it: &mut Peekable<I>,
  ) -> Result<(Cow<'a, str>, Option<Cow<'a, str>>), Error> {
    // Since no long string value possible in 'fixed-format'
    unreachable!()
  }
}

pub enum FreeFormatRead {}

impl KwrFormatRead for FreeFormatRead {
  fn parse_logical_value(part_of_kw_record: &[u8]) -> Result<(bool, &[u8]), Error> {
    match part_of_kw_record.trim_ascii_start() {
      [b'T', tail @ ..] => Ok((true, tail)),
      [b'F', tail @ ..] => Ok((false, tail)),
      [c, ..] => Err(new_invalid_free_fmt_bool_val_err(*c, part_of_kw_record)),
      [] => Err(new_empty_val_bool_err()),
    }
  }

  fn parse_integer_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let tail = part_of_kw_record.trim_ascii_start();
    let index_first_non_digit_char = tail
      .iter()
      .position(|&c| !(c.is_ascii_digit() || c == b'+' || c == b'-'))
      .unwrap_or(tail.len());
    if index_first_non_digit_char > 0 {
      Ok(tail.split_at(index_first_non_digit_char))
    } else {
      Err(new_empty_val_int_err())
    }
  }
  fn new_invalid_int_val_err(err: ParseIntError, part_of_kw_record: &[u8]) -> Error {
    new_invalid_free_fmt_int_val_err(err, part_of_kw_record)
  }

  fn parse_real_str_value(part_of_kw_record: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let tail = part_of_kw_record.trim_ascii_start();
    let index_first_non_digit_char = tail
      .iter()
      .position(|&c| {
        !(c.is_ascii_digit()
          || c == b'+'
          || c == b'-'
          || c == b'.'
          || c == b'e'
          || c == b'E'
          || c == b'd'
          || c == b'D')
      })
      .unwrap_or(tail.len());
    if index_first_non_digit_char > 0 {
      Ok(tail.split_at(index_first_non_digit_char))
    } else {
      Err(new_empty_val_float_err())
    }
  }
  fn new_invalid_real_val_err(err: ParseFloatError, part_of_kw_record: &[u8]) -> Error {
    new_invalid_free_fmt_float_val_err(err, part_of_kw_record)
  }

  fn parse_string_value_no_quote(part_of_kw_record: &[u8]) -> Result<(&str, &[u8]), Error> {
    FixedFormatRead::parse_string_value_no_quote(part_of_kw_record.trim_ascii_start())
  }

  fn parse_string_value(part_of_kw_record: &[u8]) -> Result<(Cow<'_, str>, &[u8]), Error> {
    FixedFormatRead::parse_string_value(part_of_kw_record.trim_ascii_start())
  }

  fn parse_possibly_long_string_value<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<Cow<'a, str>, Error> {
    FreeFormatRead::parse_string_value(part_of_kw_record).and_then(|(mut v, _)| {
      while v.ends_with('&')
        && wk_record_it
          .peek()
          .map(|&kwr| kwr.starts_with(b"CONTINUE  "))
          .unwrap_or(false)
      {
        // Unwrap ok here since we tested with peek
        let kw_record = wk_record_it.next().unwrap();
        let (new_v, _) = FreeFormatRead::parse_string_value(&kw_record[10..])?;

        // Deal with the additional value
        let value_string = v.to_mut();
        value_string.pop().unwrap(); // remove the ending '&', tested before so unwrap is ok.
        if !new_v.is_empty() {
          // because CONTINUE can be use only for a long comment
          value_string.push(' ');
          value_string.push_str(&new_v);
        }
      }
      Ok(v)
    })
  }

  fn parse_possibly_long_string_value_and_comment<'a, I: Iterator<Item = &'a [u8; 80]>>(
    part_of_kw_record: &'a [u8],
    wk_record_it: &mut Peekable<I>,
  ) -> Result<(Cow<'a, str>, Option<Cow<'a, str>>), Error> {
    FreeFormatRead::parse_string_value(part_of_kw_record).and_then(|(mut v, c)| {
      let mut c = FreeFormatRead::parse_value_comment(c).map(Cow::from);
      while v.ends_with('&')
        && wk_record_it
          .peek()
          .map(|&kwr| kwr.starts_with(b"CONTINUE  "))
          .unwrap_or(false)
      {
        // Unwrap ok here since we tested with peek
        let kw_record = wk_record_it.next().unwrap();
        let (new_v, new_c) = FreeFormatRead::parse_string_value(&kw_record[10..])?;

        // Deal with the additional value
        let value_string = v.to_mut();
        // - remove the ending '&', unwrap is ok.
        value_string.pop().unwrap();
        if !new_v.is_empty() {
          // because CONTINUE can be use only to write a long comment.
          value_string.push(' ');
          value_string.push_str(&new_v);
        }

        // Deal with the additional comment
        let new_c = FreeFormatRead::parse_value_comment(new_c).map(Cow::from);
        c = match (c, new_c) {
          (None, None) => None,
          (Some(c), None) => Some(c),
          (None, Some(new_c)) => Some(new_c),
          (Some(mut c), Some(new_c)) => {
            let refmut_c = c.to_mut();
            refmut_c.push(' ');
            refmut_c.push_str(new_c.as_ref());
            Some(c)
          }
        };
      }
      Ok((v, c))
    })
  }
}
