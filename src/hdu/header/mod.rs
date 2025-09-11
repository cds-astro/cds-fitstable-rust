use crate::{
  error::Error,
  hdu::{header::builder::HeaderBuilder, HDUType},
};

pub mod builder;
pub mod raw;

pub trait Header: Sized {
  /// Consume from the iterator the starting mandatory keywords.
  ///
  /// The particularity of "starting mandatory" keywords is that their order, and possibly their values,
  /// are fixed by the FITS standard.
  ///
  /// Those keyword are necessary to build a new header with the minimal information needed to know
  /// the size of the data part of the HDU.
  /// For Primary HDU, those keywords are `SIMPLE`, `BITPIX`, `NAXIS` and `NAXISn`.
  /// For Xtension HDUs, the `XTENSION` keyword must have been already consumed from the iterator,
  /// allowing to choose the right HDU header to be built.
  /// The remaining starting mandatory keywords are then `BITPIX`, `NAXIS`, `NAXISn`, `PCOUNT` and `GCOUNT`.
  ///
  /// # Params
  /// * `hdu_type`: useful for generic header, or to check coherency, or to store the value of unknown extensions.
  /// * `kw_records_it`: iterator on `(kw_record_index, kw_record_bytes)` tuples.
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>;

  /// Return the size, in bytes, of the data associated to this header.
  fn data_byte_size(&self) -> u64;

  // parse_knwon_keywords(include_comment, include_history, include_blank)
  // * Store the position (from enumerate iterator) ?
  // * NO! Instead, store in a vec and make a Map<keyword, index in vec> ?

  /// Write the "starting mandatory" keywords, including `SIMPLE` (primary HDU) or `XTENSION` (other HDUs).
  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>;

  /// Consume the keywords records which once the "starting mandatory keywords"
  /// have already been consumed.
  /// # Default behavior
  /// Do nothing, does not even consume the iterator elements.
  fn consume_remaining_kw_records<'a, I>(&mut self, _kw_records_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    Ok(())
  }
}

pub enum HDUHeader<B: HeaderBuilder> {
  Primary(B::PriH),
  AsciiTable(B::AscH),
  BinTable(B::BinH),
  Image(B::ImgH),
  Unknown(B::UnkH),
}
impl<B: HeaderBuilder> HDUHeader<B> {
  pub fn data_byte_size(&self) -> u64 {
    match self {
      Self::Primary(h) => h.data_byte_size(),
      Self::AsciiTable(h) => h.data_byte_size(),
      Self::BinTable(h) => h.data_byte_size(),
      Self::Image(h) => h.data_byte_size(),
      Self::Unknown(h) => h.data_byte_size(),
    }
  }
}
