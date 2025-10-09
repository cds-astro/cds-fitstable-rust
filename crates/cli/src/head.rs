use std::{error::Error, fmt::Debug, fs::File, path::PathBuf};

use clap::Args;
use memmap2::MmapOptions;

use fitstable::{
  hdu::header::builder::r#impl::minimal::Minimalist,
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
    for (i, hdu) in FitsBytes::from_slice(mmap.as_ref())
      .new_iterator::<Minimalist>()
      .enumerate()
    {
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
