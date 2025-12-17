//! Defines the `BITPIX` keyword.

use crate::{
  common::{
    write::{FixedFormatWrite, KwrFormatWrite},
    FixedFormat, KwrFormatRead, ValueKwr,
  },
  error::{new_unexpected_value, new_unexpected_value_list, Error},
};

// TODO: change the implementation to support not yet define datatypes (to be able to skip HDUs
// TODO: of unknown BITPIX value): the absolute value is always the number of bits of the datatype.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BitPix {
  U8 = 8,
  I16 = 16,
  I32 = 32,
  I64 = 64,
  F32 = -32,
  F64 = -64,
  // To be able to skip HDU with not (yet) supported BTPIX
  // (see fitsbit discussions about supporting f16 or f128).
  // We use i16 because of possible +-128 bit types
  // Unknown(i16),
}
impl BitPix {
  /// Returns the BitPx value associated to this BitPix enum.
  pub const fn i16_value(&self) -> i16 {
    *self as i16
  }

  /// Return the size, in bytes, of the data value this BitPix is associated with.
  pub fn byte_size(&self) -> u64 {
    (self.i16_value().abs() >> 3) as u64
  }
}

impl ValueKwr for BitPix {
  const KEYWORD: &'static [u8; 8] = b"BITPIX  ";

  fn check_value(&self, kwr_value_comment: &[u8; 70]) -> Result<(), Error> {
    FixedFormat::parse_integer_value(kwr_value_comment).and_then(|(val, _comment)| {
      if val as i16 != self.i16_value() {
        Err(new_unexpected_value(self.i16_value(), val))
      } else {
        Ok(())
      }
    })
  }

  fn from_value_comment(kwr_value_comment: &[u8; 70]) -> Result<Self, Error> {
    FixedFormat::parse_integer_str_value(kwr_value_comment).and_then(|(val, _comment)| match val {
      b"8" => Ok(BitPix::U8),
      b"16" => Ok(BitPix::I16),
      b"32" => Ok(BitPix::I32),
      b"64" => Ok(BitPix::I64),
      b"-32" => Ok(BitPix::F32),
      b"-64" => Ok(BitPix::F64),
      _ => Err(new_unexpected_value_list(&[8, 16, 32, 64, -32, -64], val)),
    })
  }

  fn write_kw_record<'a, I>(&self, dest_kwr_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    FixedFormatWrite::write_int_value_kw_record(
      dest_kwr_it,
      Self::KEYWORD,
      self.i16_value() as i64,
      Some("Data element bit size"),
    )
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::*;

  #[test]
  fn test_write_kw() {
    let bitpix = [
      BitPix::U8,
      BitPix::I16,
      BitPix::I32,
      BitPix::I64,
      BitPix::F32,
      BitPix::F64,
    ];
    for bitpix in bitpix {
      let mut dest1: Vec<u8> = Default::default();
      write!(
        dest1,
        "BITPIX  = {:>20} / Data element bit size                          ",
        bitpix.i16_value()
      )
      .unwrap();
      assert_eq!(
        dest1.len(),
        80,
        "\"{}\"",
        String::from_utf8_lossy(dest1.as_slice())
      );

      let mut dest2 = vec![[32_u8; 80]; 1];
      bitpix
        .write_kw_record(&mut dest2.iter_mut().map(Ok))
        .unwrap();
      assert_eq!(
        dest2[0].len(),
        80,
        "\"{}\"",
        String::from_utf8_lossy(dest2[0].as_slice())
      );

      assert_eq!(
        dest1.as_slice(),
        dest2[0].as_slice(),
        "\n{}\n!=\n{}",
        String::from_utf8_lossy(dest1.as_slice()),
        String::from_utf8_lossy(dest2[0].as_slice())
      );
    }
  }
}
