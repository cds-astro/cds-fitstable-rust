use crate::{
  error::Error,
  hdu::{
    header::Header,
    primary::header::PrimaryHeader,
    xtension::{
      asciitable::header::AsciiTableHeader, bintable::header::BinTableHeader,
      image::header::ImageHeader, unknown::UnknownXtensionHeader,
    },
  },
};

pub mod r#impl;

pub trait HeaderBuilder {
  /// Primary header type
  type PriH: Header;
  /// Ascii table extension header type
  type AscH: Header;
  /// Binary table extension header type
  type BinH: Header;
  /// Image extension header type
  type ImgH: Header;
  /// Unknown extension header type
  type UnkH: Header;

  fn build_primary<'a, I>(
    header: PrimaryHeader,
    kw_records_it: &mut I,
  ) -> Result<Self::PriH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;

  fn build_asciitable<'a, I>(
    header: AsciiTableHeader,
    kw_records_it: &mut I,
  ) -> Result<Self::AscH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;

  fn build_bintable<'a, I>(
    header: BinTableHeader,
    kw_records_it: &mut I,
  ) -> Result<Self::BinH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;

  fn build_image<'a, I>(header: ImageHeader, kw_records_it: &mut I) -> Result<Self::ImgH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;

  fn build_unknown<'a, I>(
    header: UnknownXtensionHeader,
    kw_records_it: &mut I,
  ) -> Result<Self::UnkH, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;
}
