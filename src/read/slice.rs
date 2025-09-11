//! Read a FITS file from in-memory (or memory mapped) data, allowing seek access.
//! This mode e.g. supports BINTABLE columns having data stored in the HEAP.

use std::marker::PhantomData;

use crate::{
  error::Error,
  hdu::header::{HDUHeader, builder::HeaderBuilder, raw::RawHeader},
};

/// The full content, i.e. all bytes, of a fits file.
/// # Lifetime
/// `'b`: "b" for "bytes", the lifetime of the full set of the fits file bytes
pub struct FitsBytes<'b> {
  bytes: &'b [u8],
}
impl<'b> FitsBytes<'b> {
  /// # Params
  /// * `bytes`: the full content, all bytes, of a FITS file
  pub fn from_slice(bytes: &'b [u8]) -> Self {
    Self { bytes }
  }

  pub fn new_iterator<B: HeaderBuilder>(&'b self) -> HDUIterator<'b, B> {
    HDUIterator::from_slice(self.bytes)
  }
}

/// All bytes of a HDU.
/// # Lifetime
/// `'u`: "u" for "HDU", the lifetime of the HDU bytes
pub struct HDU<'u, B: HeaderBuilder> {
  pub starting_byte: usize,
  pub raw_header: RawHeader<&'u [u8; 2880]>,
  pub parsed_header: HDUHeader<B>,
  pub data: &'u [u8],
}
impl<'u, B: HeaderBuilder> HDU<'u, B> {
  pub fn starting_byte(&self) -> usize {
    self.starting_byte
  }

  pub fn raw_header(&self) -> &RawHeader<&'u [u8; 2880]> {
    &self.raw_header
  }

  pub fn parsed_header(&self) -> &HDUHeader<B> {
    &self.parsed_header
  }

  pub fn data(&self) -> &'u [u8] {
    self.data
  }
}

pub struct HDUIterator<'a, B: HeaderBuilder> {
  bytes: &'a [u8],
  ptr: usize,
  _header_builder_type: PhantomData<B>,
}

impl<'a, B: HeaderBuilder> HDUIterator<'a, B> {
  /// # Params
  /// * `bytes`: the full content, all bytes, of a FITS file
  pub fn from_slice(bytes: &'a [u8]) -> Self {
    Self {
      bytes,
      ptr: 0,
      _header_builder_type: PhantomData,
    }
  }
}

// RawHeader<&[u8; 2880]>, &[u8]>
// get hdu_type
// parse basic hdu_type
// => provide a visitor to the HDU Iterator, its role is to parse the remaining of the keword iterator!!

impl<'a, B: HeaderBuilder> Iterator for HDUIterator<'a, B> {
  type Item = Result<HDU<'a, B>, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.ptr >= self.bytes.len() {
      None
    } else {
      Some(
        RawHeader::<&'a [u8]>::from_slice(&self.bytes[self.ptr..]).and_then(
          |(raw_header, remaining_bytes)| {
            let starting_byte = self.ptr;
            self.ptr += raw_header.byte_size();
            assert_eq!(self.bytes.len() - self.ptr, remaining_bytes.len());
            raw_header.build(starting_byte == 0).map(|parsed_header| {
              let data_byte_size = parsed_header.data_byte_size() as usize;
              self.ptr += data_byte_size as usize;
              let data = &remaining_bytes[..data_byte_size as usize];
              if self.ptr % 2880 != 0 {
                self.ptr = (1 + self.ptr / 2880) * 2880;
              }
              HDU {
                starting_byte,
                raw_header,
                parsed_header,
                data,
              }
            })
          },
        ),
      )
    }
  }
}
