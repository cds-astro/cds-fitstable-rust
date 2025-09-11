use crate::{
  common::{
    keywords::{
      bitpix::BitPix,
      naxis::{NAxis, NAxisn},
      simple::Simple,
    },
    DynValueKwr, ValueKwr,
  },
  error::Error,
  hdu::{header::Header, HDUType},
};

pub struct PrimaryHeader {
  pub simple: Simple,
  pub bitpix: BitPix,
  pub naxis: NAxis,
  pub naxisn: Vec<NAxisn>,
}

impl PrimaryHeader {
  pub fn new(simple: bool, bitpix: BitPix, mut naxisn: Vec<usize>) -> Self {
    Self {
      simple: Simple::new(simple),
      bitpix,
      naxis: NAxis::new(naxisn.len() as u16),
      naxisn: naxisn
        .drain(..)
        .enumerate()
        .map(|(i, l)| NAxisn::new(i as u16 + 1, l as u32))
        .collect(),
    }
  }

  /// Number of leading mandatory keyword records.
  pub fn n_kw_records(&self) -> usize {
    3 + self.naxisn.len()
  }
}

impl Header for PrimaryHeader {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    assert_eq!(hdu_type, HDUType::Primary);
    let simple = Simple::from_keyword_record_it(kw_records_it)?;
    let bitpix = BitPix::from_keyword_record_it(kw_records_it)?;
    let naxis = NAxis::from_keyword_record_it(kw_records_it)?;
    let naxisn = (1_u16..=naxis.get())
      .into_iter()
      .map(|n| NAxisn::from_keyword_record_it(n, kw_records_it))
      .collect::<Result<Vec<_>, Error>>()?;
    Ok(Self {
      simple,
      bitpix,
      naxis,
      naxisn,
    })
  }

  fn data_byte_size(&self) -> u64 {
    self.bitpix.byte_size()
      * self
        .naxisn
        .iter()
        .map(|na| na.axis_len() as u64)
        .reduce(|prod, e| prod * e)
        .unwrap_or(0)
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    self
      .simple
      .write_kw_record(dest)
      .and_then(|()| self.bitpix.write_kw_record(dest))
      .and_then(|()| self.naxis.write_kw_record(dest))
      .and_then(|()| {
        for e in &self.naxisn {
          e.write_kw_record(dest)?
        }
        Ok(())
      })
  }
}

// pb struct RandomGroup {
//   GROUPS
//   PCOUNT
//   GCOUNT
//   PTYPEn
//   PSCALn
//   PZEROn
// }
