use std::{convert::TryInto, io::Read};

use crate::{
  common::{
    ValueKwr,
    keywords::{simple::Simple, xtension::Xtension},
    read::{FixedFormatRead, KwrFormatRead},
  },
  error::{Error, new_io_err},
  hdu::{
    HDUType,
    header::{HDUHeader, Header, builder::HeaderBuilder},
    primary::header::PrimaryHeader,
    xtension::{
      asciitable::header::AsciiTableHeader, bintable::header::BinTableHeader,
      image::header::ImageHeader, unknown::UnknownXtensionHeader,
    },
  },
};
// Read RawHeader
// Analyse RawHeader to get at least HDU and DATA size
// * return HDU type
// * return Kw_iterator

// FAIRE UN MODUCLE READ
// READ RAW

/// # Warning
/// In practise, the type `T` must be either `[u8; 2880]` or `&[u8; 2880]`.
/// Any other type may lead to panics.
/// To build such a structure, use only the provided `from_xxx` methods.
pub struct RawHeader<T: AsRef<[u8]>> {
  /// Header blocks, i.e. chunks of 2880 bytes (36 keyword record of 80 bytes each).
  blocks: Vec<T>,
  /// Index of the keyword record starting by 'END     ', in the blocks concatenation.
  end_position: usize,
}

impl<T: AsRef<[u8]>> RawHeader<T> {
  /// Read the Header from the given reader.
  /// * The reader position **must be** the first byte of the first header keyword record.
  /// * The total number of bytes read is exactly the value returned by the`byte_size` method.
  /// # TIP
  /// * This method is dedicated to streaming mode reading.
  pub fn from_reader<R: Read>(reader: &mut R) -> Result<RawHeader<[u8; 2880]>, Error> {
    // Read the header by chunks of 2880
    let mut blocks = Vec::with_capacity(6);
    loop {
      let mut chunk2880 = [0_u8; 2880];
      reader.read_exact(&mut chunk2880).map_err(new_io_err)?;
      let end_position = Self::end_position(&chunk2880);
      blocks.push(chunk2880);
      if let Some(mut end_position) = end_position {
        end_position += 36 * (blocks.len() - 1);
        return Ok(RawHeader::<[u8; 2880]> {
          blocks,
          end_position,
        });
      }
    }
  }

  /// Returns the bytes that have not been consumed.
  /// # TIP
  /// * This method is to be used with MMap or when the full file is i memory.
  pub fn from_slice(bytes: &[u8]) -> Result<(RawHeader<&[u8; 2880]>, &[u8]), Error> {
    // Read the header by chunks of 2880
    let mut blocks = Vec::with_capacity(6);
    loop {
      let (chunk2880, remainder) = bytes.split_at(2880);
      let chunk2880: &[u8; 2880] = chunk2880.try_into().unwrap();
      let end_position = Self::end_position(&chunk2880);
      blocks.push(chunk2880);
      if let Some(mut end_position) = end_position {
        end_position += 36 * (blocks.len() - 1);
        return Ok((
          RawHeader::<&[u8; 2880]> {
            blocks,
            end_position,
          },
          remainder,
        ));
      }
    }
  }

  /// Returns the position of the "END" keyword, if any, in a chunk of 36 keyword records.
  /// The returned value is in `[0, 36[`.
  fn end_position(chunk2880: &[u8; 2880]) -> Option<usize> {
    debug_assert_eq!(chunk2880.len(), 2880);
    for (i, chunk) in chunk2880.chunks(80).enumerate() {
      if chunk.starts_with(b"END     ") {
        return Some(i);
      }
    }
    None
  }

  pub fn hdu_type(&self) -> Result<HDUType, Error> {
    let first_kw = self.kw_records_iter().next().unwrap();
    let (kw, vi, val) = FixedFormatRead::split_kw_indicator_value(first_kw);
    match kw {
      Simple::KEYWORD => Ok(HDUType::Primary),
      Xtension::KEYWORD => Xtension::from_value_comment(val).map(HDUType::Extension),
      _ => todo!(),
    }
  }

  pub fn build<B: HeaderBuilder>(&self, is_primary: bool) -> Result<HDUHeader<B>, Error> {
    let mut kw_it = self.kw_records_iter().enumerate();
    if is_primary {
      PrimaryHeader::from_starting_mandatory_kw_records(HDUType::Primary, &mut kw_it)
        .and_then(|h| B::build_primary(h, &mut kw_it).map(HDUHeader::Primary))
    } else {
      let first_kw = kw_it.next().unwrap();
      let (kw, vi, val) = FixedFormatRead::split_kw_indicator_value(first_kw.1);
      match kw {
        Xtension::KEYWORD => match Xtension::from_value_comment(val)? {
          Xtension::Image => ImageHeader::from_starting_mandatory_kw_records(
            HDUType::Extension(Xtension::Image),
            &mut kw_it,
          )
          .and_then(|h| B::build_image(h, &mut kw_it).map(HDUHeader::Image)),
          Xtension::AsciiTable => AsciiTableHeader::from_starting_mandatory_kw_records(
            HDUType::Extension(Xtension::AsciiTable),
            &mut kw_it,
          )
          .and_then(|h| B::build_asciitable(h, &mut kw_it).map(HDUHeader::AsciiTable)),
          Xtension::BinTable => BinTableHeader::from_starting_mandatory_kw_records(
            HDUType::Extension(Xtension::BinTable),
            &mut kw_it,
          )
          .and_then(|h| B::build_bintable(h, &mut kw_it).map(HDUHeader::BinTable)),
          Xtension::Unknown(_) => UnknownXtensionHeader::from_starting_mandatory_kw_records(
            HDUType::Extension(Xtension::Unknown(kw.clone())),
            &mut kw_it,
          )
          .and_then(|h| B::build_unknown(h, &mut kw_it).map(HDUHeader::Unknown)),
        },
        _ => todo!(),
      }
    }
  }

  /// Returns the number of blocks (of 2880 bytes) the header contains.
  pub fn n_blocks(&self) -> usize {
    self.blocks.len()
  }

  /// Returns the number of keyword records (of 80 bytes) used in the header,
  /// excluding the last one starting by `END     `.
  pub fn n_kw_records(&self) -> usize {
    self.end_position
  }

  /// Returns the size, in bytes, of the header which us `2880 x n_blocks`
  pub fn byte_size(&self) -> usize {
    2880 * self.blocks.len()
  }

  /// Iterates on all keyword records, i.e. **chunks of 80 bytes**,
  /// From the first one (inclusive) to the one starting by `END` (exclusive).
  pub fn kw_records_iter(&self) -> impl Iterator<Item = &[u8; 80]> {
    // Once stabilized, one could use [array_chunks](https://doc.rust-lang.org/beta/core/primitive.slice.html#method.array_chunks)
    self
      .blocks
      .iter()
      .flat_map(|block| block.as_ref().chunks(80))
      .map(|chunk| chunk.try_into().unwrap())
      .take(self.end_position)
  }

  /// Iterates on header blocks, i.e. **chunks of 2880 bytes**.
  /// # TIP
  /// Made to quickly re-write the header if needed (e.g. if we simply sort a file).
  pub fn blocks_iter(&self) -> impl Iterator<Item = &[u8; 2880]> {
    self.blocks.iter().map(|s| s.as_ref().try_into().unwrap())
  }
}
