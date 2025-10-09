//! Build the minimal header allowing to explore the structure of a FITS file,
//! plus parse and store information needed to read a BINTABLE data
//! (column information to read each field).

use crate::{
  error::Error,
  hdu::{
    header::{builder::HeaderBuilder, Header},
    primary::header::PrimaryHeader,
    xtension::{
      asciitable::header::AsciiTableHeader,
      bintable::header::{BinTableHeader, BinTableHeaderWithColInfo},
      image::header::ImageHeader,
      unknown::UnknownXtensionHeader,
    },
  },
};

#[cfg(feature = "vot")]
use crate::hdu::primary::header::PrimaryHeaderWithVOTable;

/// Minimalist header builder: simply returns the already known header, without parsing additional keywords.
pub enum Bintable {}

impl HeaderBuilder for Bintable {
  #[cfg(not(feature = "vot"))]
  type PriH = PrimaryHeader;
  #[cfg(feature = "vot")]
  type PriH = PrimaryHeaderWithVOTable;
  type AscH = AsciiTableHeader;
  type BinH = BinTableHeaderWithColInfo;
  type ImgH = ImageHeader;
  type UnkH = UnknownXtensionHeader;

  #[cfg(not(feature = "vot"))]
  fn build_primary<'a, I>(
    header: PrimaryHeader,
    _kw_records_it: &mut I,
  ) -> Result<Self::PriH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(header)
  }

  #[cfg(feature = "vot")]

  fn build_primary<'a, I>(header: PrimaryHeader, kw_records_it: &mut I) -> Result<Self::PriH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    let mut header: PrimaryHeaderWithVOTable = header.into();
    header
      .consume_remaining_kw_records(kw_records_it)
      .map(|()| header)
  }

  fn build_asciitable<'a, I>(
    header: AsciiTableHeader,
    _kw_records_it: &mut I,
  ) -> Result<Self::AscH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(header)
  }

  fn build_bintable<'a, I>(
    header: BinTableHeader,
    kw_records_it: &mut I,
  ) -> Result<Self::BinH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    let mut header: BinTableHeaderWithColInfo = header.into();
    header
      .consume_remaining_kw_records(kw_records_it)
      .map(|()| header)
  }

  fn build_image<'a, I>(header: ImageHeader, _kw_records_it: &mut I) -> Result<Self::ImgH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(header)
  }

  fn build_unknown<'a, I>(
    header: UnknownXtensionHeader,
    _kw_records_it: &mut I,
  ) -> Result<Self::UnkH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(header)
  }
}
