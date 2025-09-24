#[cfg(feature = "vot")]
use crate::common::keywords::tables::bintable::vot::{ntable::NTable, votmeta::VOTMeta};
use crate::{
  common::{
    DynValueKwr, ValueKwr,
    keywords::{
      bitpix::BitPix,
      naxis::{NAxis, NAxisn},
      simple::Simple,
    },
    read::{FixedFormatRead, KwrFormatRead, is_value_indicator},
  },
  error::Error,
  hdu::{HDUType, header::Header},
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

///
#[cfg(feature = "vot")]
pub struct PrimaryHeaderWithVOTable {
  /// Minimal Required Header to be able to skip data
  mrh: PrimaryHeader,
  /// Marker for FITS Plus file
  votmeta: Option<VOTMeta>,
  /// Number iof following Binary HDUs
  ntable: Option<NTable>,
}
#[cfg(feature = "vot")]
impl PrimaryHeaderWithVOTable {
  pub fn new(mrh: PrimaryHeader) -> Self {
    Self {
      mrh,
      votmeta: None,
      ntable: None,
    }
  }
  pub fn is_fits_plus(&self) -> bool {
    self
      .votmeta
      .as_ref()
      .map(|kw| kw.is_true())
      .unwrap_or(false)
  }
  pub fn n_bintable_hdu(&self) -> u16 {
    self.ntable.as_ref().map(|kw| kw.get()).unwrap_or(0)
  }
}

#[cfg(feature = "vot")]
impl Header for PrimaryHeaderWithVOTable {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    PrimaryHeader::from_starting_mandatory_kw_records(hdu_type, kw_records_it).map(|v| v.into())
  }

  fn data_byte_size(&self) -> u64 {
    self.mrh.data_byte_size()
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    self.mrh.write_starting_mandatory_kw_records(dest)
  }

  fn consume_remaining_kw_records<'a, I>(&mut self, kw_records_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    for (_, kwr) in kw_records_it {
      let (kw, ind, kw_value_comment) = FixedFormatRead::split_kw_indicator_value(kwr);
      // Skip keyword if it does not contain a value indicator
      if !is_value_indicator(ind) {
        continue;
      }
      // Analyse keywords
      match kw {
        VOTMeta::KEYWORD => {
          VOTMeta::from_value_comment(kw_value_comment).map(|kw| self.votmeta.replace(kw))?;
        }
        NTable::KEYWORD => {
          NTable::from_value_comment(kw_value_comment).map(|kw| self.ntable.replace(kw))?;
        }
        _ => {}
      }
    }
    Ok(())
  }
}
#[cfg(feature = "vot")]
impl From<PrimaryHeader> for PrimaryHeaderWithVOTable {
  fn from(mrh: PrimaryHeader) -> Self {
    Self::new(mrh)
  }
}
