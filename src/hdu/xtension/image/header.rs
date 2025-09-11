use crate::hdu::HDUType;
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
  hdu::header::Header,
};

pub const XTENSION: Xtension = Xtension::Image;
pub const PCOUNT: PCount = PCount::new(0);
pub const GCOUNT: GCount = GCount::new(1);

pub struct ImageHeader {
  pub bitpix: BitPix,
  pub naxis: NAxis,
  pub naxisn: Vec<NAxisn>,
}

impl Header for ImageHeader {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    assert_eq!(hdu_type, HDUType::Extension(Xtension::Image));
    let bitpix = BitPix::from_keyword_record_it(kw_records_it)?;
    let naxis = NAxis::from_keyword_record_it(kw_records_it)?;
    let naxisn = (1_u16..=naxis.get())
      .into_iter()
      .map(|n| NAxisn::from_keyword_record_it(n, kw_records_it))
      .collect::<Result<Vec<_>, Error>>()?;
    PCOUNT.check_keyword_record_it(kw_records_it)?;
    GCOUNT.check_keyword_record_it(kw_records_it)?;
    Ok(Self {
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
    XTENSION
      .write_kw_record(dest)
      .and_then(|()| self.bitpix.write_kw_record(dest))
      .and_then(|()| self.naxis.write_kw_record(dest))
      .and_then(|()| {
        for e in &self.naxisn {
          e.write_kw_record(dest)?
        }
        Ok(())
      })
      .and_then(|()| PCOUNT.write_kw_record(dest))
      .and_then(|()| GCOUNT.write_kw_record(dest))
  }
}
