use std::{error::Error, fmt::Debug, fs::File, io::BufReader, path::PathBuf};

use clap::Args;
use memmap2::{Mmap, MmapOptions};

use fitstable::{
  common::{DynValueKwr, keywords::naxis::NAxisn},
  hdu::{
    header::{HDUHeader, builder::r#impl::minimal::Minimalist},
    primary::header::PrimaryHeader,
    xtension::{
      asciitable::header::AsciiTableHeader, bintable::header::BinTableHeader,
      image::header::ImageHeader, unknown::UnknownXtensionHeader,
    },
  },
  read::slice::{FitsBytes, HDU},
};

#[derive(Debug, Clone, Args)]
pub struct Head {
  /// Path of the input file.
  #[clap(value_name = "FILE")]
  pub input: PathBuf,
}

impl Head {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    let file = File::open(&self.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    for (i, hdu) in fits.new_iterator::<Minimalist>().enumerate() {
      hdu
        .map_err(|e| e.into())
        .and_then(|hdu| print_hdu_header(i, hdu))?;
    }
    Ok(())
  }
}

fn print_hdu_header(i: usize, hdu: HDU<Minimalist>) -> Result<(), Box<dyn Error>> {
  println!("HDU[{}]:", i);
  for kwr in hdu.raw_header.kw_records_iter() {
    println!("{}", unsafe { str::from_utf8_unchecked(kwr) });
  }
  println!();
  Ok(())
}
