//! To be able to skip/deal with unknown extensions.

use crate::{
  common::{
    keywords::{
      bitpix::BitPix,
      naxis::{NAxis, NAxisn},
      pgcount::{GCount, PCount},
      xtension::Xtension,
    },
    DynValueKwr, ValueKwr,
  },
  error::Error,
  hdu::{header::Header, HDUType},
};

pub struct UnknownXtensionHeader {
  pub xtension: Xtension,
  pub bitpix: BitPix,
  pub naxis: NAxis,
  pub naxisn: Vec<NAxisn>,
  pub pcount: PCount,
  pub gcount: GCount,
}

impl Header for UnknownXtensionHeader {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    let xtension = match hdu_type {
      HDUType::Extension(Xtension::Unknown(kw)) => Xtension::Unknown(kw),
      _ => panic!("Supposed to be an Unknown extension!"),
    };
    let bitpix = BitPix::from_keyword_record_it(kw_records_it)?;
    let naxis = NAxis::from_keyword_record_it(kw_records_it)?;
    let naxisn = (1_u16..=naxis.get())
      .into_iter()
      .map(|n| NAxisn::from_keyword_record_it(n, kw_records_it))
      .collect::<Result<Vec<_>, Error>>()?;
    let pcount = PCount::from_keyword_record_it(kw_records_it)?;
    let gcount = GCount::from_keyword_record_it(kw_records_it)?;
    Ok(Self {
      xtension,
      bitpix,
      naxis,
      naxisn,
      pcount,
      gcount,
    })
  }

  fn data_byte_size(&self) -> u64 {
    self.bitpix.byte_size()
      * (self.gcount.get() as u64)
      * (self.pcount.get() as u64
        + self
          .naxisn
          .iter()
          .map(|na| na.axis_len() as u64)
          .reduce(|prod, e| prod * e)
          .unwrap_or(0))
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    self
      .xtension
      .write_kw_record(dest)
      .and_then(|()| self.bitpix.write_kw_record(dest))
      .and_then(|()| self.naxis.write_kw_record(dest))
      .and_then(|()| {
        for e in &self.naxisn {
          e.write_kw_record(dest)?
        }
        Ok(())
      })
      .and_then(|()| self.pcount.write_kw_record(dest))
      .and_then(|()| self.gcount.write_kw_record(dest))
  }
}
