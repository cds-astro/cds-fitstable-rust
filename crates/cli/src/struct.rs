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
pub struct Struct {
  /// Path of the input file.
  #[clap(value_name = "FILE")]
  pub input: PathBuf,
}

impl Struct {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    let file = File::open(&self.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    for (i, hdu) in fits.new_iterator::<Minimalist>().enumerate() {
      hdu
        .map_err(|e| e.into())
        .and_then(|hdu| print_hdu_struct(i, hdu))?;
    }
    Ok(())
  }
}

fn print_hdu_struct(i: usize, hdu: HDU<Minimalist>) -> Result<(), Box<dyn Error>> {
  println!("HDU[{}]:", i);
  let HDU {
    starting_byte,
    raw_header,
    parsed_header,
    data,
  } = hdu;
  println!(
    " * HEAD starting byte: {}; n_blocks: {}; byte size: {}",
    starting_byte,
    raw_header.n_blocks(),
    raw_header.byte_size()
  );
  println!(
    " * DATA starting byte: {}; byte size: {}.",
    starting_byte + raw_header.byte_size(),
    data.len()
  );
  match parsed_header {
    HDUHeader::Primary(h) => print_primhdu_struct(h),
    HDUHeader::Image(h) => print_imghdu_struct(h),
    HDUHeader::AsciiTable(h) => print_ascisstablehdu_struct(h),
    HDUHeader::BinTable(h) => print_bintablehdu_struct(h),
    HDUHeader::Unknown(h) => print_unknownhdu_struct(h),
  }
}

fn print_primhdu_struct(header: PrimaryHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("PRIMARY");
  println!(
    "   + simple: {}; naxis: {}; bitpix : {}; dimensions: {}.",
    header.simple.get(),
    header.naxis.get(),
    header.bitpix.i16_value(),
    get_dims(header.naxisn.as_slice())
  );
  Ok(())
}

fn print_imghdu_struct(header: ImageHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("IMAGE");
  println!(
    "   + naxis: {}; bitpix : {}; dimensions: {}.",
    header.naxis.get(),
    header.bitpix.i16_value(),
    get_dims(header.naxisn.as_slice())
  );
  Ok(())
}

fn print_bintablehdu_struct(header: BinTableHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("BINTABLE");
  println!(
    "   + n_cols: {}; n_rows : {}; row_byte_size: {}; heap_byte_size: {}.",
    header.n_cols(),
    header.n_rows(),
    header.row_byte_size(),
    header.heap_byte_size()
  );
  Ok(())
}

fn print_ascisstablehdu_struct(header: AsciiTableHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("ASCIITABLE");
  println!(
    "   + n_cols: {}; n_rows : {}; row_byte_size: {}.",
    header.n_cols(),
    header.n_rows(),
    header.row_byte_size(),
  );
  Ok(())
}

fn print_unknownhdu_struct(header: UnknownXtensionHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type(
    format!("UNKNOWN({})", unsafe {
      str::from_utf8_unchecked(header.xtension.value())
    })
    .as_str(),
  );
  println!(
    "   + naxis: {}; bitpix : {}; dimensions: {}; pcount: {}; gcount: {}.",
    header.naxis.get(),
    header.bitpix.i16_value(),
    get_dims(header.naxisn.as_slice()),
    header.pcount.get(),
    header.gcount.get(),
  );
  Ok(())
}

fn print_hdu_type(hdu_type: &str) {
  println!(" * TYPE: {}", hdu_type);
}

fn get_dims(naxisn: &[NAxisn]) -> String {
  naxisn
    .iter()
    .map(|v| v.axis_len().to_string())
    .reduce(|mut s, d| {
      s.push('x');
      s.push_str(&d);
      s
    })
    .unwrap_or_else(|| String::from("0"))
}
