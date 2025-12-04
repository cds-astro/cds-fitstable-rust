//! Read a FITS file from in-memory (or memory mapped) data, allowing seek access.
//! This mode e.g. supports BINTABLE columns having data stored in the HEAP.
use std::{io::Write, marker::PhantomData};

use crate::{
  error::{Error, new_io_err},
  hdu::header::{
    HDUHeader,
    builder::{HeaderBuilder, r#impl::bintable::Bintable},
    raw::RawHeader,
  },
};

#[cfg(feature = "vot")]
use votable::{VOTable, VOTableError, impls::mem::VoidTableDataContent, votable::VOTableWrapper};

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
  pub fn is_primary_hdu(&self) -> bool {
    matches!(&self.parsed_header, HDUHeader::Primary(_))
  }
  pub fn is_bintable_hdu(&self) -> bool {
    matches!(&self.parsed_header, HDUHeader::BinTable(_))
  }

  pub fn starting_byte(&self) -> usize {
    self.starting_byte
  }

  pub fn data_starting_byte(&self) -> usize {
    self.starting_byte + self.raw_header.byte_size()
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

  pub fn copy_hdu<W: Write>(&self, w: &mut W) -> Result<(), Error> {
    self
      .copy_header(w)
      .and_then(|()| self.copy_data(w))
      .and_then(|()| self.copy_blanks(w))
  }

  pub fn copy_header<W: Write>(&self, w: &mut W) -> Result<(), Error> {
    self.raw_header.copy(w)
  }

  pub fn copy_data<W: Write>(&self, w: &mut W) -> Result<(), Error> {
    w.write_all(self.data).map_err(new_io_err)
  }

  pub fn copy_blanks<W: Write>(&self, w: &mut W) -> Result<(), Error> {
    let rem2880 = self.data.len() % 2880;
    if rem2880 != 0 {
      w.write_all(vec![0_u8; 2880 - rem2880].as_slice())
        .map_err(new_io_err)
    } else {
      Ok(())
    }
  }
}

impl<'u> HDU<'u, Bintable> {
  #[cfg(feature = "vot")]
  pub fn is_fits_plus_primary_hdu(&self) -> bool {
    match &self.parsed_header {
      HDUHeader::Primary(h) if h.is_fits_plus() => true,
      _ => false,
    }
  }

  #[cfg(feature = "vot")]
  pub fn n_bintable_hdu(&self) -> u16 {
    match &self.parsed_header {
      HDUHeader::Primary(h) if h.is_fits_plus() => h.n_bintable_hdu(),
      _ => 0,
    }
  }

  #[cfg(feature = "vot")]
  pub fn parse_votable_if_any(&self) -> Option<Result<VOTable<VoidTableDataContent>, VOTableError>> {
    match &self.parsed_header {
      HDUHeader::Primary(h) if h.is_fits_plus() => Some(
        VOTableWrapper::<VoidTableDataContent>::from_ivoa_xml_bytes(self.data).map(|w| w.votable),
      ),
      _ => None,
    }
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
      let is_primary = self.ptr == 0;
      Some(
        RawHeader::<&'a [u8]>::from_slice(is_primary, &self.bytes[self.ptr..]).and_then(
          |(raw_header, remaining_bytes)| {
            let starting_byte = self.ptr;
            self.ptr += raw_header.byte_size();
            assert_eq!(self.bytes.len() - self.ptr, remaining_bytes.len());

            raw_header.build(is_primary).map(|parsed_header| {
              let data_byte_size = parsed_header.data_byte_size() as usize;
              self.ptr += data_byte_size;
              // get data part
              let data = &remaining_bytes[..data_byte_size];
              // ensure the pointer points to the first byte of a 2880 byte block
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

/*
/// Decorator for `HDUIterator` for FITS Plus files, automatically updating columns information.
#[cfg(feature = "vot")]
pub struct HDUIteratorWithVOT<'a> {
  hdu_it: HDUIterator<'a, Bintable>,
}
*/
