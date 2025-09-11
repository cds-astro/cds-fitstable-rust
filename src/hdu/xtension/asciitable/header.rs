use crate::{
  common::{
    ValueKwr,
    keywords::{
      bitpix::BitPix,
      naxis::{NAxis, NAxis1, NAxis2},
      pgcount::{GCount, PCount},
      tables::tfields::TFields,
      xtension::Xtension,
    },
  },
  error::Error,
  hdu::{HDUType, header::Header},
};

pub const XTENSION: Xtension = Xtension::AsciiTable;
pub const BITPIX: BitPix = BitPix::U8;
pub const NAXIS: NAxis = NAxis::new(2);
pub const PCOUNT: PCount = PCount::new(0);
pub const GCOUNT: GCount = GCount::new(1);

pub struct AsciiTableHeader {
  naxis1: NAxis1,
  /// Number of rows in the table
  naxis2: NAxis2,
  /// Number of columns in the table
  tfield: TFields,
  /*
  Then, in any order:
  TBCOLn (mandatory)
  TFORMn (mandatory)
  TTYPEn (recommanded)
  TUNITn (optional)
  TSCALn (optional)
  TZEROn (optional)
  TNULLn (optional)
  TDISPn (optional)
  */
}

impl AsciiTableHeader {
  /// # Params
  /// * `naxis1`: byte size of a row
  /// * `naxis2`: number of rows in the table
  /// * `tfield`: number of columns in the table
  pub fn new(naxis1: u32, naxis2: u64, tfield: u16) -> Self {
    Self {
      naxis1: NAxis1::new(naxis1),
      naxis2: NAxis2::new(naxis2),
      tfield: TFields::new(tfield),
    }
  }

  /// Number of leading mandatory keyword records (from `XTENSION`, inclusive, to `TFIELD`, inclusive).
  pub fn n_kw_records(&self) -> usize {
    8
  }

  pub fn n_cols(&self) -> usize {
    self.tfield.get() as usize
  }

  pub fn n_rows(&self) -> usize {
    self.naxis2.get() as usize
  }

  pub fn row_byte_size(&self) -> usize {
    self.naxis1.get() as usize
  }
}

impl Header for AsciiTableHeader {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    // We assume XTENSION has already been parsed (needed to decide to enter here!).
    assert_eq!(hdu_type, HDUType::Extension(Xtension::AsciiTable));
    BITPIX.check_keyword_record_it(kw_records_it)?;
    NAXIS.check_keyword_record_it(kw_records_it)?;
    let naxis1 = NAxis1::from_keyword_record_it(kw_records_it)?;
    let naxis2 = NAxis2::from_keyword_record_it(kw_records_it)?;
    PCOUNT.check_keyword_record_it(kw_records_it)?;
    GCOUNT.check_keyword_record_it(kw_records_it)?;
    let tfield = TFields::from_keyword_record_it(kw_records_it)?;
    Ok(Self {
      naxis1,
      naxis2,
      tfield,
    })
  }

  fn data_byte_size(&self) -> u64 {
    BITPIX.byte_size() * ((self.naxis1.get() as u64) * self.naxis2.get())
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    XTENSION
      .write_kw_record(dest)
      .and_then(|()| BITPIX.write_kw_record(dest))
      .and_then(|()| NAXIS.write_kw_record(dest))
      .and_then(|()| self.naxis1.write_kw_record(dest))
      .and_then(|()| self.naxis2.write_kw_record(dest))
      .and_then(|()| PCOUNT.write_kw_record(dest))
      .and_then(|()| GCOUNT.write_kw_record(dest))
      .and_then(|()| self.tfield.write_kw_record(dest))
  }
}
