use std::{
  io,
  num::{ParseFloatError, ParseIntError},
};

use thiserror::Error;

/// An error that can be produced during FITS parsing or read/writing.
pub type Error = Box<FitsError>;

fn bytes2str(bytes: &[u8]) -> String {
  String::from_utf8_lossy(bytes).into()
}

pub(crate) fn new_empty_val_bool_err() -> Error {
  FitsError::EmptyBooleanValueError.into()
}

pub(crate) fn new_empty_val_int_err() -> Error {
  FitsError::EmptyIntegerValueError.into()
}

pub(crate) fn new_empty_val_float_err() -> Error {
  FitsError::EmptyFloatValueError.into()
}

pub(crate) fn new_io_err(err: io::Error) -> Error {
  FitsError::Io(err).into()
}

pub(crate) fn new_parse_u16_err(err: ParseIntError) -> Error {
  FitsError::ParseU16Error(err).into()
}

// Fixed-format
pub(crate) fn new_invalid_fixed_fmt_bool_val_err(found: u8, part_of_kw_record: &[u8]) -> Error {
  let found = found as char;
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFixedFmtBoolValue {
    found,
    part_of_kw_record,
  }
  .into()
}

pub(crate) fn new_invalid_fixed_fmt_int_val_err(
  err: ParseIntError,
  part_of_kw_record: &[u8],
) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFixedFmtIntegerValue {
    err,
    part_of_kw_record,
  }
  .into()
}

pub(crate) fn new_invalid_fixed_fmt_float_val_err(
  err: ParseFloatError,
  part_of_kw_record: &[u8],
) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFixedFmtRealValue {
    err,
    part_of_kw_record,
  }
  .into()
}

// Free-format

pub(crate) fn new_invalid_free_fmt_bool_val_err(found: u8, part_of_kw_record: &[u8]) -> Error {
  let found = found as char;
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFreeFmtBoolValue {
    found,
    part_of_kw_record,
  }
  .into()
}

pub(crate) fn new_invalid_free_fmt_int_val_err(
  err: ParseIntError,
  part_of_kw_record: &[u8],
) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFreeFmtIntegerValue {
    err,
    part_of_kw_record,
  }
  .into()
}

pub(crate) fn new_invalid_free_fmt_float_val_err(
  err: ParseFloatError,
  part_of_kw_record: &[u8],
) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::InvalidFreeFmtRealValue {
    err,
    part_of_kw_record,
  }
  .into()
}

// Others

pub(crate) fn new_string_value_opening_not_found_err(part_of_kw_record: &[u8]) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::StringValueOpeningNotFound { part_of_kw_record }.into()
}

pub(crate) fn new_string_value_closing_not_found_err(part_of_kw_record: &[u8]) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::StringValueClosingNotFound { part_of_kw_record }.into()
}

pub(crate) fn new_empty_hierarch_kw_err(part_of_kw_record: &[u8]) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::EmptyHierarchKeyword { part_of_kw_record }.into()
}

pub(crate) fn new_hierarch_kwval_sep_not_found_err(part_of_kw_record: &[u8]) -> Error {
  let part_of_kw_record = bytes2str(part_of_kw_record);
  FitsError::HierarchEqualSeparatorNotFound { part_of_kw_record }.into()
}

pub(crate) fn new_unexpected_kw(expected: &[u8; 8], found: &[u8; 8]) -> Error {
  let expected = bytes2str(expected);
  let found = bytes2str(found);
  FitsError::UnexpectedKeyword { expected, found }.into()
}

pub(crate) fn new_unexpected_value<T: ToString, S: ToString>(expected: T, found: S) -> Error {
  let expected = expected.to_string();
  let found = found.to_string();
  FitsError::UnexpectedValue { expected, found }.into()
}

pub(crate) fn new_0_or_1_repeatcount(found: u16) -> Error {
  FitsError::UnexpectedRepeatCount { found }.into()
}

pub(crate) fn new_unexpected_value_list<T: ToString>(expected: &[T], found: &[u8]) -> Error {
  let mut expected_str = String::from("One of [");
  let mut first: bool = true;
  for s in expected {
    if first {
      first = false;
    } else {
      expected_str.push_str(", ");
    }
    expected_str.push_str(s.to_string().as_str());
  }
  expected_str.push(']');
  let found = bytes2str(found);
  FitsError::UnexpectedValue {
    expected: expected_str,
    found,
  }
  .into()
}

pub(crate) fn new_unsupported_by_visitor(expected_dt: &str, found_dt: &str) -> Error {
  FitsError::NotSupportedByVisitor {
    expected_dt: expected_dt.into(),
    found_dt: found_dt.into(),
  }
  .into()
}

pub(crate) fn new_value_indicator_not_found(expected: &[u8; 2], found: &[u8; 2]) -> Error {
  let expected = bytes2str(expected);
  let found = bytes2str(found);
  FitsError::ValueIndicatorNotFound { expected, found }.into()
}

pub(crate) fn new_depleted_write_it() -> Error {
  FitsError::DepletedWriteIterator.into()
}

pub(crate) fn new_depleted_reader_it() -> Error {
  FitsError::DepletedWriteIterator.into()
}

pub(crate) fn new_custom<I: Into<String>>(msg: I) -> Error {
  FitsError::Custom { msg: msg.into() }.into()
}

#[derive(Error, Debug)]
pub enum FitsError {
  // IO related
  #[error("I/O error: {0}.")]
  Io(#[from] io::Error),
  #[error(
    "Writable keyword record iterator depleted (should not happen, logical error in the code)!"
  )]
  DepletedWriteIterator,
  #[error("Keyword record iterator depleted!")]
  DepletedReaderIterator,

  // Parse error
  #[error("Parse u16 error: {0}.")]
  ParseU16Error(#[from] ParseIntError),

  // Not valid
  #[error("Invalid fixed-format boolean value. Expected 'F' or 'T' at position 19; Actual: '{found}'. In '{part_of_kw_record}'.")]
  InvalidFixedFmtBoolValue {
    found: char,
    part_of_kw_record: String,
  },
  #[error("Invalid fixed-format integer value. Error: '{err:?}'. In '{part_of_kw_record}'.")]
  InvalidFixedFmtIntegerValue {
    err: ParseIntError,
    part_of_kw_record: String,
  },
  #[error("Invalid fixed-format real value. Error: '{err:?}'. In '{part_of_kw_record}'.")]
  InvalidFixedFmtRealValue {
    err: ParseFloatError,
    part_of_kw_record: String,
  },
  #[error("Invalid free-format boolean value. Expected first non blank character 'F' or 'T'; Actual: '{found}'. In '{part_of_kw_record}'.")]
  InvalidFreeFmtBoolValue {
    found: char,
    part_of_kw_record: String,
  },
  #[error("Invalid free-format integer value. Error: '{err:?}'. In '{part_of_kw_record}'.")]
  InvalidFreeFmtIntegerValue {
    err: ParseIntError,
    part_of_kw_record: String,
  },
  #[error("Invalid free-format real value. Error: '{err:?}'. In '{part_of_kw_record}'.")]
  InvalidFreeFmtRealValue {
    err: ParseFloatError,
    part_of_kw_record: String,
  },

  // Empty
  #[error("Wrong keyword value. Expected; boolean. Actual: empty.")]
  EmptyBooleanValueError,
  #[error("Wrong keyword value. Expected; integer. Actual: empty.")]
  EmptyIntegerValueError,
  #[error("Wrong keyword value. Expected; float. Actual: empty.")]
  EmptyFloatValueError,
  #[error("No keyword found after HIERARCH in \"{part_of_kw_record}\".")]
  EmptyHierarchKeyword { part_of_kw_record: String },

  // Not found
  #[error("Value indicator not found. Expected: \"{expected}\". Actual: \"{found}\").")]
  ValueIndicatorNotFound { expected: String, found: String },
  #[error("Keyword/Value '=' separator not found after HIERARCH in \"{part_of_kw_record}\".")]
  HierarchEqualSeparatorNotFound { part_of_kw_record: String },
  #[error("String value no found, empty keyword record value.")]
  StringValueNotFound,
  #[error("Invalid fixed-format string value: first character must be a single quote in \"{part_of_kw_record}\".")]
  StringValueOpeningNotFound { part_of_kw_record: String },
  #[error(
    "Invalid fixed-format string value: closing single quote not found in \"{part_of_kw_record}\"."
  )]
  StringValueClosingNotFound { part_of_kw_record: String },

  // Unexpected
  #[error("Wrong FITS keyword. Expected: {expected}. Actual: {found}).")]
  UnexpectedKeyword { expected: String, found: String },
  #[error("Wrong value. Expected: '{expected}'. Actual: '{found}'.")]
  UnexpectedValue { expected: String, found: String },
  #[error("Wrong TFORM repeat count. Expected: 0 or 1. Actual: '{found}'.")]
  UnexpectedRepeatCount { found: u16 },

  #[error("Visitor unsupported datatype. Expected: {expected_dt}. Found: {found_dt}.")]
  NotSupportedByVisitor {
    expected_dt: String,
    found_dt: String,
  },

  // Context
  #[error("Error: {source}\nKeyword context: {keyword}.")]
  WithKeywordContext { keyword: String, source: Error },
  #[error("Error: {source}\nKeyword context: {keyword}. Value: {value}")]
  WithKeywordValueContext {
    keyword: String,
    value: String,
    source: Error,
  },
  #[error("Error: {source}\nKeyword record context: {keyword_record}.")]
  WithKwRecordContext {
    keyword_record: String,
    source: Error,
  },

  #[error("Custom error: '{msg}'.")]
  Custom { msg: String },
}

impl FitsError {
  /// Add to the error the keyword on which the error occurs.
  pub(crate) fn kw_context(self, keyword: &[u8; 8]) -> Error {
    Self::WithKeywordContext {
      keyword: bytes2str(keyword),
      source: self.into(),
    }
    .into()
  }
  /// Add to the error the keyword on which the error occurs.
  pub(crate) fn kw_val_context(self, keyword: &[u8; 8], value: &str) -> Error {
    Self::WithKeywordValueContext {
      keyword: bytes2str(keyword),
      value: value.into(),
      source: self.into(),
    }
    .into()
  }
  /// Add to the error the full keyword record on which the error occurs.
  pub(crate) fn kwr_context(self, kw_record: &[u8; 80]) -> Error {
    Self::WithKwRecordContext {
      keyword_record: bytes2str(kw_record),
      source: self.into(),
    }
    .into()
  }
}
