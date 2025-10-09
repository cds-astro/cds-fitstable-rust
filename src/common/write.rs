use std::{io::Write, ptr::copy_nonoverlapping};

use log::warn;

use crate::{
  common::{VALUE_COMMENT_SEPARATOR, VALUE_INDICATOR},
  error::{new_depleted_write_it, Error},
};

/// To write long keyword (or long comment) keuword card, use the `HierarchFormatWrite` implementation.
pub trait KwrFormatWrite {
  // REGULAR VALUE KEYWORD

  /// Split the keyword record in two parts:
  /// * first the keyword part
  /// * then the remaining
  ///
  /// # Input
  /// An emtpy keyword record from which the keyword has been removed.
  fn split_kw_mut(kw_record: &mut [u8; 80]) -> (&mut [u8; 8], &mut [u8; 72]) {
    let (k, r) = kw_record.split_at_mut(8);
    // SAFETY: we know exactly where we split and the input size. So no problem.
    (
      unsafe { &mut *(k.as_mut_ptr().cast::<[u8; 8]>()) },
      unsafe { &mut *(r.as_mut_ptr().cast::<[u8; 72]>()) },
    )
  }

  /// Split the keyword record, from which the keyword has already been removed, in two:
  /// * the two byte of the value indicator
  /// * the remaining value + comment part
  /// # Input
  /// An emtpy keyword record from which the keyword has been removed.
  fn split_value_indicator_mut(kw_record_wo_kew: &mut [u8; 72]) -> (&mut [u8; 2], &mut [u8; 70]) {
    let (v, r) = kw_record_wo_kew.split_at_mut(2);
    (
      unsafe { &mut *(v.as_mut_ptr().cast::<[u8; 2]>()) },
      unsafe { &mut *(r.as_mut_ptr().cast::<[u8; 70]>()) },
    )
  }

  /// # Warning
  /// * `dest` and `kw` must be non-overlapping
  fn write_keyword(dest: &mut [u8; 8], kw: &[u8; 8]) {
    unsafe { copy_nonoverlapping(kw.as_ptr(), dest.as_mut_ptr(), 8) }
  }

  /// # Warning
  /// * `dest` and `kw` must be non-overlapping
  fn write_value_indicator(dest: &mut [u8; 2]) {
    unsafe { copy_nonoverlapping(VALUE_INDICATOR.as_ptr(), dest.as_mut_ptr(), 2) }
  }

  fn write_kw_and_value_indicator<'a>(
    kw_record: &'a mut [u8; 80],
    kw: &[u8; 8],
  ) -> &'a mut [u8; 70] {
    let (k, r) = Self::split_kw_mut(kw_record);
    Self::write_keyword(k, kw);
    let (vi, vc) = Self::split_value_indicator_mut(r);
    Self::write_value_indicator(vi);
    vc
  }

  // Faire un header writer qui écrit des block de 2880 par 2880 ayant un iter_mut sur des chunks de 80

  // * boolean

  /// **Must** write exactly 80 bytes, too long comments are truncated
  fn write_boolean_value_kw_record<'a, I>(
    dest: &mut I,
    kw: &[u8; 8],
    value: bool,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    dest
      .next()
      .unwrap_or_else(|| Err(new_depleted_write_it()))
      .and_then(|kwr| {
        Self::write_boolean_value_comment(
          Self::write_kw_and_value_indicator(kwr, kw),
          value,
          comment,
        )
      })
  }

  fn write_boolean_value_comment(
    dest: &mut [u8; 70],
    value: bool,
    comment: Option<&str>,
  ) -> Result<(), Error>;

  // * integer

  /// **Must** write exactly 80 bytes, too long comments are truncated
  fn write_int_value_kw_record<'a, I>(
    dest: &mut I,
    kw: &[u8; 8],
    value: i64,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    dest
      .next()
      .unwrap_or_else(|| Err(new_depleted_write_it()))
      .and_then(|kwr| {
        Self::write_int_value_comment(Self::write_kw_and_value_indicator(kwr, kw), value, comment)
      })
  }

  fn write_int_value_comment(
    dest: &mut [u8; 70],
    value: i64,
    comment: Option<&str>,
  ) -> Result<(), Error>;

  /// **Must** write exactly 80 bytes, too long comments are truncated
  // only because of TZERO for u64 type
  fn write_uint_value_kw_record<'a, I>(
    dest: &mut I,
    kw: &[u8; 8],
    value: u64,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    dest
      .next()
      .unwrap_or_else(|| Err(new_depleted_write_it()))
      .and_then(|kwr| {
        Self::write_uint_value_comment(Self::write_kw_and_value_indicator(kwr, kw), value, comment)
      })
  }

  // only because of TZERO for u64 type
  fn write_uint_value_comment(
    dest: &mut [u8; 70],
    value: u64,
    comment: Option<&str>,
  ) -> Result<(), Error>;

  // * real

  /// **Must** write exactly 80 bytes, too long comments are truncated
  fn write_real_value_kw_record<'a, I>(
    dest: &mut I,
    kw: &[u8; 8],
    value: f64,
    n_sig_digits: Option<usize>,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    dest
      .next()
      .unwrap_or_else(|| Err(new_depleted_write_it()))
      .and_then(|kwr| {
        Self::write_real_value_comment(
          Self::write_kw_and_value_indicator(kwr, kw),
          value,
          n_sig_digits,
          comment,
        )
      })
  }

  fn write_real_value_comment(
    dest: &mut [u8; 70],
    value: f64,
    n_sig_digits: Option<usize>,
    comment: Option<&str>,
  ) -> Result<(), Error>;

  // * String

  /// **Must** write exactly 80 bytes, too long comments are truncated
  fn write_string_value_kw_record<'a, I>(
    dest: &mut I,
    kw: &[u8; 8],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    dest
      .next()
      .unwrap_or_else(|| Err(new_depleted_write_it()))
      .and_then(|kwr| {
        Self::write_string_value_comment(
          Self::write_kw_and_value_indicator(kwr, kw),
          value,
          comment,
        )
      })
  }

  fn write_string_value_comment(
    dest: &mut [u8; 70],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error>;

  /// We assume the comment fit into `dest`, i.e. is made of at most as many bytes as bytes available
  /// in `dest`. Else, the comment is truncated.
  fn write_comment(dest: &mut [u8], comment: &str) {
    if dest.len() > 3 {
      let (sep, tail) = dest.split_at_mut(3);
      unsafe { copy_nonoverlapping(VALUE_COMMENT_SEPARATOR.as_ptr(), sep.as_mut_ptr(), 3) };
      if comment.len() <= tail.len() {
        unsafe { copy_nonoverlapping(comment.as_ptr(), tail.as_mut_ptr(), comment.len()) };
      } else {
        // Need to truncate the comment.
        let mut s = String::from(comment);
        // It is not supposed to contain non-ASCII characters but...
        while s.len() > tail.len() {
          s.pop();
        }
        unsafe { copy_nonoverlapping(s.as_ptr(), tail.as_mut_ptr(), comment.len()) };
      }
    }
  }

  // LONG STRING VALUE or COMMENT KEYWORD (CONTINUE)

  /// Possibly use continue
  fn write_possibly_long_string_value_and_long_comment<'a, I>(
    dest: &mut [u8],
    dest_it: &mut I,
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>;

  // HIERARCH

  // Use hierarch, for keyword larger than 8 bytes.
  fn write_long_keyword(dest: &mut [u8; 80], kw: &str) -> Result<(), Error>;

  /// # output
  /// * The remaining bytes that can be written.
  fn write_longkw_boolean_value(dest: &mut [u8], value: bool) -> &mut [u8];

  /// # output
  /// * The remaining bytes that can be written.
  fn write_longkw_int_value(dest: &mut [u8], value: i64) -> &mut [u8];

  /// # output
  /// * The remaining bytes that can be written.
  fn write_longkw_real_value(dest: &mut [u8], value: f64) -> &mut [u8];

  /// Possibly use continue
  fn write_longkw_possibly_long_string_value_and_long_comment<'a, I>(
    dest: &mut [u8],
    dest_it: &mut I,
    value: &str,
    comment: Option<&str>,
  ) where
    I: Iterator<Item = &'a mut [u8; 80]>;
}

pub enum FixedFormatWrite {}
impl FixedFormatWrite {
  /// Reserved for values containing less than 20 characters, including both the starting and the ending
  /// single quotes for string value.
  /// # Remark
  /// For mandatory keywords, the constraint (<20 characters value) seems to be fulfilled.
  fn split_value_comment_mut(kw_record_wo_kew: &mut [u8; 70]) -> (&mut [u8; 20], &mut [u8; 50]) {
    let (v, c) = kw_record_wo_kew.split_at_mut(20);
    (
      unsafe { &mut *(v.as_mut_ptr().cast::<[u8; 20]>()) },
      unsafe { &mut *(c.as_mut_ptr().cast::<[u8; 50]>()) },
    )
  }

  fn write_comment_if_any(dest: &mut [u8], comment: Option<&str>) -> Result<(), Error> {
    if let Some(comment) = comment {
      Self::write_comment(dest, comment);
    }
    Ok(())
  }
}
impl KwrFormatWrite for FixedFormatWrite {
  fn write_boolean_value_comment(
    dest: &mut [u8; 70],
    value: bool,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let (v, c) = Self::split_value_comment_mut(dest);
    let b = if value { b'T' } else { b'F' };
    v[19] = b;
    Self::write_comment_if_any(c, comment)
  }

  fn write_int_value_comment(
    dest: &mut [u8; 70],
    value: i64,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let (v, c) = Self::split_value_comment_mut(dest);
    write!(v.as_mut_slice(), "{:>20}", value).unwrap(); // Min i64 value has 20 characters (including the '-' sign)
    Self::write_comment_if_any(c, comment)
  }

  fn write_uint_value_comment(
    dest: &mut [u8; 70],
    value: u64,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let (v, c) = Self::split_value_comment_mut(dest);
    write!(v.as_mut_slice(), "{:>20}", value).unwrap(); // Min i64 value has 20 characters (including the '-' sign)
    Self::write_comment_if_any(c, comment)
  }

  fn write_real_value_comment(
    dest: &mut [u8; 70],
    value: f64,
    n_sig_digits: Option<usize>,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let (v, c) = Self::split_value_comment_mut(dest);
    match n_sig_digits {
      Some(mut n_sig_digits) => {
        let mut res = write_engineering(&mut v.as_mut_slice(), value, n_sig_digits);
        while let Err(_) = res {
          n_sig_digits -= 1;
          res = write_engineering(&mut v.as_mut_slice(), value, n_sig_digits);
        }
      }
      None => {
        let s = format!("{}", value);
        if s.len() < v.len() {
          unsafe { copy_nonoverlapping(s.as_ptr(), v.as_mut_ptr(), s.len()) };
        } else {
          let s = format!("{:E}", value);
          if s.len() < v.len() {
            unsafe { copy_nonoverlapping(s.as_ptr(), v.as_mut_ptr(), s.len()) };
          } else {
            // Convert into a float...
            let s = format!("{:E}", value as f32);
            assert!(
              s.len() < v.len(),
              "Too long float representation for Fixed-format keyword."
            );
            unsafe { copy_nonoverlapping(s.as_ptr(), v.as_mut_ptr(), s.len()) };
          }
        }
      }
    }
    Self::write_comment_if_any(c, comment)
  }

  /// WARNING: so far we consider that fixed-format String values (i.e. mandatory keywords values)
  /// are well-defined and do not contain single quotes "'". If it is not the case, see FreeFormat
  /// handling of singe quotes.
  fn write_string_value_comment(
    dest: &mut [u8; 70],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    if value.len() <= 18 {
      let (v, c) = Self::split_value_comment_mut(dest);
      v[0] = b'\'';
      v[19] = b'\'';
      let v = &mut v[1..19];
      unsafe { copy_nonoverlapping(value.as_ptr(), v.as_mut_ptr(), value.len()) };
      Self::write_comment_if_any(c, comment)
    } else {
      FreeFormatWrite::write_string_value_comment(dest, value, comment)
    }
  }

  fn write_possibly_long_string_value_and_long_comment<'a, I>(
    _dest: &mut [u8],
    _dest_it: &mut I,
    _value: &str,
    _comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    // Since no long string value possible in 'fixed-format'
    unreachable!()
  }

  fn write_long_keyword(_dest: &mut [u8; 80], _kw: &str) -> Result<(), Error> {
    // Since no long string value possible in 'fixed-format'
    unreachable!()
  }

  fn write_longkw_boolean_value(_dest: &mut [u8], _value: bool) -> &mut [u8] {
    // Since HIERARCH value possible in 'fixed-format'
    unreachable!()
  }

  fn write_longkw_int_value(_dest: &mut [u8], _value: i64) -> &mut [u8] {
    // Since HIERARCH value possible in 'fixed-format'
    unreachable!()
  }

  fn write_longkw_real_value(_dest: &mut [u8], _value: f64) -> &mut [u8] {
    // Since HIERARCH value possible in 'fixed-format'
    unreachable!()
  }

  fn write_longkw_possibly_long_string_value_and_long_comment<'a, I>(
    _dest: &mut [u8],
    _dest_it: &mut I,
    _value: &str,
    _comment: Option<&str>,
  ) where
    I: Iterator<Item = &'a mut [u8; 80]>,
  {
    // Since HIERARCH value possible in 'fixed-format'
    unreachable!()
  }
}

pub enum FreeFormatWrite {}

impl FreeFormatWrite {
  fn write_comment_if_any(dest: &mut [u8], comment: Option<&str>) -> Result<(), Error> {
    if let Some(comment) = comment {
      let dest_len = dest.len();
      if dest_len >= 50 && comment.len() <= 47 {
        // Align with FixedFormat
        let dest = &mut dest[dest_len - 50..];
        FixedFormatWrite::write_comment(dest, comment);
      } else {
        // Does not align to try to avoid comment truncation
        Self::write_comment(dest, comment);
      }
    }
    Ok(())
  }

  fn write_string_value_comment_gen_noreplace(
    dest: &mut [u8],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let len_max = dest.len() - 2;
    if value.len() <= len_max {
      let len_p1 = value.len() + 1;
      let len_p2 = len_p1 + 1;
      let (v, c) = dest.split_at_mut(len_p2);
      v[0] = b'\'';
      v[len_p1] = b'\'';
      let v = &mut v[1..len_p1];
      unsafe { copy_nonoverlapping(value.as_ptr(), v.as_mut_ptr(), value.len()) };
      Self::write_comment_if_any(c, comment)
    } else {
      // truncate!
      let mut s = String::from(value);
      while s.len() > len_max {
        s.pop();
      }
      warn!("Value '{}' truncated to '{}'", value, s);
      Self::write_string_value_comment_gen(dest, s.as_str(), comment)
    }
  }

  fn write_string_value_comment_gen(
    dest: &mut [u8],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    if value.contains('\'') {
      let value = value.replace('\'', "''");
      Self::write_string_value_comment_gen_noreplace(dest, value.as_str(), comment)
    } else {
      Self::write_string_value_comment_gen_noreplace(dest, value, comment)
    }
  }
}

impl KwrFormatWrite for FreeFormatWrite {
  fn write_boolean_value_comment(
    dest: &mut [u8; 70],
    value: bool,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    dest[0] = if value { b'T' } else { b'F' };
    Self::write_comment_if_any(&mut dest[1..], comment)
  }

  fn write_int_value_comment(
    dest: &mut [u8; 70],
    value: i64,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let mut vbuff = Vec::with_capacity(20);
    write!(&mut vbuff, "{}", value).unwrap();
    unsafe { copy_nonoverlapping(vbuff.as_ptr(), dest.as_mut_ptr(), vbuff.len()) };
    Self::write_comment_if_any(&mut dest[vbuff.len()..], comment)
  }

  // only because of TZERO for u64 type
  fn write_uint_value_comment(
    dest: &mut [u8; 70],
    value: u64,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let mut vbuff = Vec::with_capacity(20);
    write!(&mut vbuff, "{}", value).unwrap();
    unsafe { copy_nonoverlapping(vbuff.as_ptr(), dest.as_mut_ptr(), vbuff.len()) };
    Self::write_comment_if_any(&mut dest[vbuff.len()..], comment)
  }

  fn write_real_value_comment(
    dest: &mut [u8; 70],
    value: f64,
    n_sig_digits: Option<usize>,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    let mut vbuff = Vec::with_capacity(20);
    match n_sig_digits {
      Some(n_sig_digits) => write_engineering(&mut vbuff, value, n_sig_digits),
      None => write!(&mut vbuff, "{}", value),
    }
    .unwrap();
    unsafe { copy_nonoverlapping(vbuff.as_ptr(), dest.as_mut_ptr(), vbuff.len()) };
    Self::write_comment_if_any(&mut dest[vbuff.len()..], comment)
  }

  fn write_string_value_comment(
    dest: &mut [u8; 70],
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error> {
    Self::write_string_value_comment_gen(dest, value, comment)
  }

  fn write_possibly_long_string_value_and_long_comment<'a, I>(
    dest: &mut [u8],
    _dest_it: &mut I,
    value: &str,
    comment: Option<&str>,
  ) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    // Size of the value + comment (if any)
    // * +2 for opening and closing single quotes
    // * +3 for the value/comment separator " / "
    let len = value.len() + 2 + comment.map(|c| c.len() + 3).unwrap_or(0);
    if len <= dest.len() {
      Self::write_string_value_comment_gen(dest, value, comment)
    } else {
      match comment {
        Some(_comment) => {
          // First try to write the value in a  minimum of card, then write comment.
          // Or devide each line by the ratio of the value size over the comment size ?
          todo!()
        }
        None => {
          // NO COMMENT, WRITE LONG VALUE
          todo!()
        }
      }
    }
  }

  fn write_long_keyword(_dest: &mut [u8; 80], _kw: &str) -> Result<(), Error> {
    todo!()
  }

  fn write_longkw_boolean_value(_dest: &mut [u8], _value: bool) -> &mut [u8] {
    todo!()
  }

  fn write_longkw_int_value(_dest: &mut [u8], _value: i64) -> &mut [u8] {
    todo!()
  }

  fn write_longkw_real_value(_dest: &mut [u8], _value: f64) -> &mut [u8] {
    todo!()
  }

  fn write_longkw_possibly_long_string_value_and_long_comment<'a, I>(
    _dest: &mut [u8],
    _dest_it: &mut I,
    _value: &str,
    _comment: Option<&str>,
  ) where
    I: Iterator<Item = &'a mut [u8; 80]>,
  {
    // First try to write the value ina  minimum of card, then write comment.
    todo!()
  }
}

pub enum HierarchFormatWrite {}

/// By Hadrien Grasland.
/// # params
/// * sig_digits: number of significant digits required.
pub fn write_engineering(
  writer: &mut impl Write,
  x: f64,
  sig_digits: usize,
) -> Result<(), std::io::Error> {
  let mut precision = sig_digits - 1;
  let log_x = x.abs().log10();
  if (log_x >= -3. && log_x <= (sig_digits as f64)) || x == 0. {
    // Print using naive notation
    if x != 0. {
      // Since Rust's precision controls number of digits after the
      // decimal point, we must adjust it depending on magnitude in order
      // to operate at a constant number of significant digits.
      precision = (precision as isize - log_x.trunc() as isize) as usize;

      // Numbers smaller than 1 must get one extra digit since the leading
      // zero does not count as a significant digit.
      if log_x < 0. {
        precision += 1
      }
    }
    write!(writer, "{:.1$}", x, precision)
  } else {
    // Print using scientific notation
    write!(writer, "{:.1$E}", x, precision)
  }
}
