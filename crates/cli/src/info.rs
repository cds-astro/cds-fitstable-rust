use std::{
  error::Error,
  fmt::Debug,
  fs::File,
  io::{Write, stdout},
  path::PathBuf,
};

use clap::Args;
use log::{error, info, warn};
use memmap2::MmapOptions;

use votable::{TableElem, VOTable, VoidTableDataContent};

use fitstable::{
  hdu::{
    header::{
      HDUHeader,
      builder::r#impl::{bintable::Bintable},
    },
    primary::header::{PrimaryHeaderWithVOTable},
    xtension::{
      asciitable::header::AsciiTableHeader,
      bintable::header::{BinTableHeaderWithColInfo},
      image::header::ImageHeader,
      unknown::UnknownXtensionHeader,
    },
  },
  read::slice::{FitsBytes, HDU},
};

#[derive(Debug, Clone, Args)]
pub struct Info {
  /// Path of the input file.
  #[clap(value_name = "FILE")]
  pub input: PathBuf,
  /// Do not print the VOTable of a FITS Plus file
  #[clap(short = 'n', long)]
  pub no_vot: bool,
  /// Only print the VOTable of a FITS Plus file
  #[clap(short = 'o', long, conflicts_with = "no_vot")]
  pub only_vot: bool,
  /// Do not merge VOTable and FITS column metadata in a FITS Plus file
  #[clap(short = 'm', long, conflicts_with = "only_vot")]
  pub no_merge: bool,
  /// Overwrite FITS column attributes with VOTable one's (if any), except for data type.
  #[clap(short = 'w', long, conflicts_with = "no_merge")]
  pub vot_overwrite: bool,
}

impl Info {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    info!("Open file {:?}", &self.input);
    let file = File::open(&self.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    if self.only_vot {
      if let Some(prim_hdu) = fits.new_iterator::<Bintable>().next() {
        let prim_hdu = prim_hdu?;
        if prim_hdu.is_fits_plus_primary_hdu() {
          stdout().write_all(prim_hdu.data)?;
        }
      }
    } else {
      let mut hdu_it = fits.new_iterator::<Bintable>().enumerate();
      if let Some((_, phd)) = hdu_it.next() {
        let phd = phd?;
        let vot = if self.no_merge {
          None
        } else {
          match phd.parse_votable_if_any() {
            Some(Ok(vot)) => {
              if phd.n_bintable_hdu() > 1 {
                warn!("Supports only FITS-plus files having a single BINTABLE HDU, not {}", phd.n_bintable_hdu());
              }
              Some(vot)
            },
            Some(Err(e)) => {
              error!("Error parsing the VOTable header: {:?}", e);
              error!("Use option '-m' not to see this error message.");
              None
            },
            None => None
          }
        };
        for (i, hdu) in  hdu_it {
          hdu
            .map_err(|e| e.into())
            .and_then(|hdu| self.print_hdu_struct(i, hdu, vot.as_ref()))?;
        }
      }
    }
    Ok(())
  }

  fn print_hdu_struct(&self, i_hdu: usize, hdu: HDU<Bintable>, vot: Option<&VOTable<VoidTableDataContent>>) -> Result<(), Box<dyn Error>> {
    print!("HDU[{}]: ", i_hdu);
    match hdu.parsed_header {
      HDUHeader::Primary(h) => print_primhdu_struct(h, hdu.data, !self.no_vot),
      HDUHeader::Image(h) => print_imghdu_struct(h),
      HDUHeader::AsciiTable(h) => print_ascisstablehdu_struct(h),
      HDUHeader::BinTable(h) => print_bintablehdu_struct(h, i_hdu, vot, self.vot_overwrite),
      HDUHeader::Unknown(h) => print_unknownhdu_struct(h),
    }
  }
}

fn print_primhdu_struct(
  header: PrimaryHeaderWithVOTable,
  data: &[u8],
  print_vot: bool,
) -> Result<(), Box<dyn Error>> {
  print_hdu_type("PRIMARY");
  if print_vot && header.is_fits_plus() {
    stdout().write_all(data)?;
  }
  Ok(())
}

fn print_imghdu_struct(_header: ImageHeader) -> Result<(), Box<dyn Error>> {
  print_hdu_type("IMAGE");
  Ok(())
}

fn print_bintablehdu_struct(mut header: BinTableHeaderWithColInfo, i_hdu: usize, vot: Option<&VOTable<VoidTableDataContent>>, vot_overwrite: bool) -> Result<(), Box<dyn Error>> {
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


  // Merge with VOTable elements
  if let Some(vot) = vot {
    if i_hdu == 1 {
      // We are lazy: so far we support only FITS-plus file with a single BINTABLE HDU
      // (single table in the VOTable)
      if let Some(table) = vot.get_first_table() {
        let vot_fields = table.elems.iter().filter_map(|e| match e {
          TableElem::Field(field) => Some(field),
          _ => None
        }).collect::<Vec<_>>();
        if vot_fields.len() == header.n_cols() {
          for (icol, (f, v)) in header.cols_mut().iter_mut().zip(vot_fields.iter()).enumerate() {
            f.merge(icol as u16, v, vot_overwrite);
          }
        } else {
          warn!("Different number of fields in BINTABLE and in VOTable: {} != {}. No merge performed.", header.n_cols(), vot_fields.len());
        }
      }
    } else {
      warn!("So far supports only FITS-plus files having a single BINTABLE HDU after the Primary HDU");
    }
  }

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

fn print_ascisstablehdu_struct(_header: AsciiTableHeader) -> Result<(), Box<dyn Error>> {
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
