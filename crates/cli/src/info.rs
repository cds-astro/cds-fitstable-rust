use std::{error::Error, fmt::Debug, fs::File, io::BufReader, path::PathBuf};

use clap::Args;
use log::info;
use memmap2::{Mmap, MmapOptions};

use fitstable::hdu::header::builder::r#impl::bintable::Bintable;
use fitstable::hdu::xtension::bintable::header::BinTableHeaderWithColInfo;
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
pub struct Info {
  /// Path of the input file.
  #[clap(value_name = "FILE")]
  pub input: PathBuf,
}

impl Info {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    info!("Open file {:?}", &self.input);
    let file = File::open(&self.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    for (i, hdu) in fits.new_iterator::<Bintable>().enumerate() {
      hdu
        .map_err(|e| e.into())
        .and_then(|hdu| print_hdu_struct(i, hdu))?;
    }
    Ok(())
  }
}

fn print_hdu_struct(i: usize, hdu: HDU<Bintable>) -> Result<(), Box<dyn Error>> {
  print!("HDU[{}]: ", i);
  match hdu.parsed_header {
    HDUHeader::Primary(h) => print_primhdu_struct(h),
    HDUHeader::Image(h) => print_imghdu_struct(h),
    HDUHeader::AsciiTable(h) => print_ascisstablehdu_struct(h),
    HDUHeader::BinTable(h) => print_bintablehdu_struct(h),
    HDUHeader::Unknown(h) => print_unknownhdu_struct(h),
  }
}

fn print_primhdu_struct(header: PrimaryHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("PRIMARY");
  Ok(())
}

fn print_imghdu_struct(header: ImageHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("IMAGE");
  Ok(())
}

fn print_bintablehdu_struct(header: BinTableHeaderWithColInfo) -> Result<(), Box<dyn Error>> {
  print_hdu_type(
    format!(
      "BINTABLE  n_cols: {}; n_rows : {}",
      header.table().n_cols(),
      header.table().n_rows()
    )
    .as_str(),
  );

  // num / name / dt / unit / desc
  let mut width_num = 4_usize;
  let mut width_name = 4_usize;
  let mut width_dt = 4_usize;
  let mut width_unit = 4_usize;
  let mut width_ucd = 3_usize;
  let mut width_desc = 4_usize;

  // Compute sizes
  for (i, field) in header.cols().iter().enumerate() {
    let num_str = i.to_string();
    if num_str.len() > width_num {
      width_num = num_str.len();
    }

    if let Some(name) = field.colname()
      && name.len() > width_name
    {
      width_name = name.len();
    }

    if let Some(dt) = field.schema() {
      let dt_str = dt.to_string();
      if dt_str.len() > width_dt {
        width_dt = dt_str.len();
      }
    }

    if let Some(unit) = field.unit()
      && unit.len() > width_unit
    {
      width_unit = unit.len();
    }

    if let Some(ucd) = field.ucd()
      && ucd.len() > width_ucd
    {
      width_ucd = ucd.len();
    }

    if let Some(desc) = field.description()
      && desc.len() > width_desc
    {
      width_desc = desc.len();
    }
  }

  // Print
  println!(
    "{:>width_num$} {:>width_name$} {:>width_dt$} {:>width_unit$} {:>width_ucd$} {:<width_desc$}",
    "#", "name", "type", "unit", "ucd", "desc"
  );
  for (i, field) in header.cols().iter().enumerate() {
    let name = field.colname().unwrap_or("");
    let dt = field
      .schema()
      .map(|dt| dt.to_string())
      .unwrap_or(String::from(""));
    let unit = field.unit().unwrap_or("---");
    let ucd = field.ucd().unwrap_or("---");
    let desc = field.description().unwrap_or("");

    println!(
      "{:>width_num$} {:>width_name$} {:>width_dt$} {:>width_unit$} {:>width_ucd$} {:<width_desc$}",
      i, name, dt, unit, ucd, desc
    );
  }

  Ok(())
}

fn print_ascisstablehdu_struct(header: AsciiTableHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("ASCIITABLE");
  Ok(())
}

fn print_unknownhdu_struct(header: UnknownXtensionHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type(
    format!("UNKNOWN({})", unsafe {
      str::from_utf8_unchecked(header.xtension.value())
    })
    .as_str(),
  );
  Ok(())
}

fn print_hdu_type(hdu_type: &str) {
  println!(" {}", hdu_type);
}
