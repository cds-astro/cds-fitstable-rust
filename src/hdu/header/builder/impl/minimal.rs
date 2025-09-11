//! Build the minimal header allowing to explore the structure of a FITS file.

use crate::{
  error::Error,
  hdu::{
    header::builder::HeaderBuilder,
    primary::header::PrimaryHeader,
    xtension::{
      asciitable::header::AsciiTableHeader, bintable::header::BinTableHeader,
      image::header::ImageHeader, unknown::UnknownXtensionHeader,
    },
  },
};

/// Minimalist heaer builder: simply returns the already known header, without parsing additional keywords.
pub enum Minimalist {}

impl HeaderBuilder for Minimalist {
  type PriH = PrimaryHeader;
  type AscH = AsciiTableHeader;
  type BinH = BinTableHeader;
  type ImgH = ImageHeader;
  type UnkH = UnknownXtensionHeader;

  fn build_primary<'a, I>(
    header: PrimaryHeader,
    _kw_records_it: &mut I,
  ) -> Result<Self::PriH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(header)
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
    Ok(header)
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
