use log::warn;

use crate::hdu::HDUType;
use crate::{
  common::{
    read::{bytes2str, FreeFormatRead, KwrFormatRead},
    write::{FreeFormatWrite, KwrFormatWrite},
    ValueKwr,
  },
  error::{new_unexpected_value, Error},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Xtension {
  Image,            // IMAGE___
  AsciiTable,       // TABLE___
  BinTable,         // BINTABLE
  Unknown([u8; 8]), // Useful to skip it
}

impl Xtension {
  fn from_value(value: &[u8; 8]) -> Self {
    match value {
      b"IMAGE   " => Self::Image,
      b"TABLE   " => Self::AsciiTable,
      b"BINTABLE" => Self::BinTable,
      _ => {
        warn!("Xtension value '{}' not recognized.", bytes2str(value));
        Self::Unknown(value.clone())
      }
    }
  }

  fn from_trimmed_value(value: &[u8]) -> Self {
    match value {
      b"IMAGE" => Self::Image,
      b"TABLE" => Self::AsciiTable,
      b"BINTABLE" => Self::BinTable,
      _ => {
        warn!("Xtension value '{}' not recognized.", bytes2str(value));
        let mut val = [b' '; 8];
        (&mut val[0..value.len()]).copy_from_slice(value);
        Self::Unknown(val)
      }
    }
  }

  pub fn value(&self) -> &[u8; 8] {
    match self {
      Self::Image => b"IMAGE   ",
      Self::AsciiTable => b"TABLE   ",
      Self::BinTable => b"BINTABLE",
      Self::Unknown(value) => value,
    }
  }
}

impl ValueKwr for Xtension {
  const KEYWORD: &'static [u8; 8] = b"XTENSION";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    // Try a quick parsing...
    if kwr_value_comment[0] == b'\'' && kwr_value_comment[9] == b'\'' {
      let found = &kwr_value_comment[1..=8];
      if self.value().as_slice() == found {
        Ok(())
      } else {
        Err(new_unexpected_value(
          bytes2str(self.value()),
          bytes2str(found),
        ))
      }
    } else {
      // ... but if it fails, make an effort to parse the value
      FreeFormatRead::parse_string_value_no_quote(kwr_value_comment.as_slice()).and_then(
        |(found, _tail)| {
          if found.trim() == bytes2str(self.value()).trim() {
            Ok(())
          } else {
            Err(new_unexpected_value(bytes2str(self.value()), found.trim()))
          }
        },
      )
    }
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    // Try a quick parsing...
    if kwr_value_comment[0] == b'\'' && kwr_value_comment[9] == b'\'' {
      let found = &kwr_value_comment[1..=8];
      let found = unsafe { &*(found.as_ptr().cast::<[u8; 8]>()) };
      Ok(Self::from_value(found))
    } else {
      // ... but if it fails, make an effort to parse the value
      FreeFormatRead::parse_string_value_no_quote(kwr_value_comment.as_slice())
        .map(|(found, _tail)| Self::from_trimmed_value(found.trim().as_bytes()))
    }
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    FreeFormatWrite::write_string_value_kw_record(
      dest_kwr_it,
      Self::KEYWORD,
      bytes2str(self.value()),
      Some("Data element bit size"),
    )
  }
}
