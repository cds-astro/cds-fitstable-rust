use std::{
  error::Error,
  fmt::Debug,
  fs::{File, metadata, read_to_string},
  io::{BufReader, Cursor, Read, Write, stdout},
  path::PathBuf,
};

use clap::{Args, Subcommand};
#[cfg(feature = "cgi")]
use http::status::StatusCode as Status;
use log::{error, warn};
#[cfg(not(windows))]
use memmap2::Advice;
use memmap2::MmapOptions;
use serde;

use bstree_file_readonly::{
  Entry,
  bstree::{SubTreeR, read_meta},
  rw::{ReadWrite, U64RW},
  visitors::VisitorExact,
};
use cdshealpix::nested::{
  from_zuniq,
  sort::cindex::{FITSCIndex, FitsMMappedCIndex, HCIndex},
  to_zuniq,
};
use fitstable::{
  hdu::{
    header::{HDUHeader, Header, builder::r#impl::bintable::Bintable},
    xtension::bintable::{
      read::{
        deser::sliceheap::DeserializerWithHeap,
        visitor::csv::{CSVRowVisitor, CSVVisitor},
      },
      schema::RowSchema,
    },
  },
  read::slice::{FitsBytes, HDU},
};
use votable::{Resource, Table, VOTable, VoidTableDataContent, votable::Version};

use crate::mkhips::Properties;

/// Perform all possible operations on a HiPS catalogue.
#[derive(Debug, Args, serde::Serialize, serde::Deserialize)]
#[clap(author, version, name = "table.cgi", about = "")]
pub struct QHips {
  /// Path of the HiPS directory
  #[clap(value_name = "DIR")]
  pub input: PathBuf,
  #[clap(subcommand)]
  /// Action to be performed
  pub action: Action,
}

impl QHips {
  pub fn new(path: &str, action: Action) -> Self {
    Self {
      input: path.into(),
      action,
    }
  }
  pub fn exec(self, is_cgi: bool) -> Result<(), Box<dyn Error>> {
    self.action.exec(self.input, is_cgi)
  }
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize)]
#[clap(name = "action", about = "Action to be performed on the table")]
pub enum Action {
  #[serde(rename = "properties")]
  #[clap(name = "properties")]
  /// Get the `properties` file
  Properties,
  #[serde(rename = "metadata")]
  #[clap(name = "metadata")]
  /// Get the `metadata.xml` file
  Metadata,
  #[serde(rename = "moc")]
  #[clap(name = "moc")]
  /// Get the `Moc.fits` file (WARNING: binary data in stdout!)
  Moc,
  #[serde(rename = "allsky")]
  #[clap(name = "allsky")]
  /// Get the `Norder${depth}/allsky.tsv` file
  Allsky { depth: u8 },
  #[serde(rename = "tile")]
  #[clap(name = "tile")]
  /// Get the `Norder${depth}/Dir[0-9]*/Npix${hash}.tsv` file
  Tile { depth: u8, hash: u64 },
  #[serde(rename = "list")]
  #[clap(name = "list")]
  /// Get the list of all tiles, together with their statistics
  TileList,
  #[serde(rename = "info")]
  #[clap(name = "info")]
  /// Print the landing page (i.e. the `index.html` page)
  IndexHTML,
}

impl Action {
  pub fn exec(self, input: PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
    match self {
      Self::Properties => print_properties(input, is_cgi),
      Self::Metadata => print_metadata(input, is_cgi),
      Self::Moc => print_moc(input, is_cgi),
      Self::Allsky { depth } => print_allsky(input, depth, is_cgi),
      Self::Tile { depth, hash } => print_tile(input, depth, hash, is_cgi),
      Self::TileList => print_tiles_stats(input, is_cgi),
      Self::IndexHTML => print_landing_page(is_cgi),
    }
  }
}

fn check_file_exists(path: &PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  if !path.is_file() {
    #[cfg(feature = "cgi")]
    if is_cgi {
      println!(
        "Status: {}\n\nFile not found. Filename: {:?}.",
        Status::BAD_REQUEST,
        path.file_name()
      );
    }
    Err(format!("File not found. Filename: {:?}.", path.file_name()).into())
  } else {
    Ok(())
  }
}

fn print_properties(mut input: PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  input.push("properties.toml");
  check_file_exists(&input, is_cgi)?;
  read_to_string(input)
    .map_err(|e| e.into())
    .and_then(|s| {
      toml::from_str::<Properties>(s.as_str()).map_err(|e| {
        #[cfg(feature = "cgi")]
        if is_cgi {
          println!(
            "Status: {}\n\nUnable to deserialize properties.toml: {}",
            Status::INTERNAL_SERVER_ERROR,
            e.to_string()
          );
        }
        e.into()
      })
    })
    .map(|prop| {
      #[cfg(feature = "cgi")]
      if is_cgi {
        println!("Content-Type: text/plain\n");
      }
      println!("{}", prop)
    })
}

fn print_metadata(mut input: PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  input.push("hips.cat.layer1.fits");
  check_file_exists(&input, is_cgi)?;
  let file = File::open(&input)?;
  let mmap = unsafe { MmapOptions::new().map(&file)? };
  let bytes = mmap.as_ref();
  let fits = FitsBytes::from_slice(bytes);
  let mut hdu_it = fits.new_iterator::<Bintable>().enumerate();
  if let Some((_, phd)) = hdu_it.next() {
    let vot = if let Some(vot) = match phd?.parse_votable_if_any() {
      Some(Ok(vot)) => Some(vot),
      Some(Err(e)) => {
        error!("Error parsing the VOTable header: {:?}", e);
        None
      }
      None => None,
    } {
      Ok::<VOTable<VoidTableDataContent>, Box<dyn Error>>(vot)
    } else {
      // Else build a VOTable header from BINTABLE header
      match hdu_it.next() {
        Some((_, Ok(hdu))) => match hdu.parsed_header {
          HDUHeader::BinTable(header) => {
            let mut vot_table = Table::<VoidTableDataContent>::new();
            for field in header
              .cols()
              .iter()
              .enumerate()
              .map(|(i, field)| field.to_vot_field(i as u16))
            {
              let field = field?;
              vot_table.push_field_by_ref(field);
            }
            let vot = VOTable::new(Version::V1_5, Resource::new().push_table(vot_table));
            Ok(vot)
          }
          _ => Err(format!("BINTABLE not found in second HDU of file \"{:?}\".", &input).into()),
        },
        _ => Err(format!("No valid second HDU found in file \"{:?}\".", &input).into()),
      }
    }?;
    #[cfg(feature = "cgi")]
    if is_cgi {
      println!("Content-Type: application/xml\n");
    }
    // Write the VOTable header
    let write = stdout().lock();
    vot.wrap().to_ivoa_xml_writer(write).map_err(|e| e.into())
  } else {
    Err(format!("No primary HDU found in file \"{:?}\".", &input).into())
  }
}

fn print_moc(mut input: PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  input.push("moc.fits");
  check_file_exists(&input, is_cgi)?;
  #[cfg(feature = "cgi")]
  if is_cgi {
    let len = metadata(&input).map(|metadata| metadata.len())?;
    println!("Content-Type: application/fits");
    println!("Content-Disposition: attachment; filename=\"moc.fits\";");
    println!("Content-Transfer-Encoding: binary");
    println!("Content-Length: {}", &len);
    println!();
  }
  let mut stdout = stdout().lock();
  File::open(input)
    .and_then(|file| {
      let mut read = BufReader::new(file);
      let mut bytes = [0u8; 8192]; // 8kB chunks
      let mut n = read.read(&mut bytes)?;
      while n != 0 {
        n = stdout
          .write_all(&bytes[..n])
          .and_then(|()| read.read(&mut bytes))?;
      }
      Ok(())
    })
    .map_err(|e| e.into())
}

fn print_allsky(mut input: PathBuf, depth: u8, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  // Prepare input
  input.push(format!("hips.cat.layer{}.fits", depth));
  check_file_exists(&input, is_cgi)?;
  #[cfg(feature = "cgi")]
  if is_cgi {
    if depth > 2 {
      println!(
        "Status: {}\n\nAllsky with order > 2 not allowed.",
        Status::BAD_REQUEST,
      );
      return Err("Allsky with order > 2 not allowed.".into());
    }
    println!("Content-Type: text/plain\n");
  }
  let file = File::open(&input)?;
  let mmap = unsafe { MmapOptions::new().map(&file)? };
  #[cfg(not(windows))]
  if let Err(e) = mmap.advise(Advice::Sequential) {
    warn!(
      "Error advising for sequential read on file '{:?}': {}",
      file, e
    );
  }

  // Set output
  let mut write = stdout().lock();

  // Iterate over HDUs
  for hdu in FitsBytes::from_slice(mmap.as_ref()).new_iterator::<Bintable>() {
    let HDU {
      starting_byte: _,
      raw_header: _,
      parsed_header,
      data,
    } = hdu?;
    match parsed_header {
      HDUHeader::BinTable(bintable_header_full) => {
        // Get all variable to know where is and how to interpret the dat
        let table_header = bintable_header_full.table();
        let row_byte_size = table_header.row_byte_size();
        let data_byte_size = table_header.data_byte_size();
        let table_byte_size = table_header.main_table_byte_size();
        let heap_byte_size = table_header.heap_byte_size();
        let gap_byte_size = bintable_header_full.gap_byte_size();
        let n_cols = table_header.n_cols();
        let n_rows = table_header.n_rows();
        assert_eq!(data_byte_size as usize, data.len());
        assert_eq!(data_byte_size as usize, table_byte_size + heap_byte_size);

        // Get table schema
        let row_schema: RowSchema = bintable_header_full
          .cols()
          .iter()
          .enumerate()
          .map(|(i, col_header)| {
            col_header.schema().expect(&format!(
              "Unable to create schema for column {}: TFORM probably missing!",
              i + 1
            ))
          })
          .collect();
        assert_eq!(row_schema.n_cols(), n_cols);

        // Separate main table data and heap data
        let (main, rem) = data.split_at(table_byte_size);
        let (_gap, heap) = rem.split_at(gap_byte_size);
        assert_eq!(n_rows * row_byte_size, table_byte_size);

        // Print header
        let mut first = true;
        for (i, field) in bintable_header_full.cols().iter().enumerate() {
          if first {
            first = false;
          } else {
            write!(write, "\t")?;
          }
          match field.colname() {
            Some(name) => write!(write, "{}", name),
            None => write!(write, "col_{}", i),
          }?;
        }

        // Print data
        let mut visitor = CSVVisitor::new_custom(&mut write, b'\t');
        for raw_row in main.chunks(row_byte_size) {
          let mut de = DeserializerWithHeap::new(raw_row, heap);
          visitor.starts_new_line();
          row_schema.deserialize(&mut de, &mut visitor, CSVRowVisitor)?;
        }
        write!(write, "\n",)?;
      }
      _ => {}
    }
  }
  Ok(())
}

fn print_tile(input: PathBuf, depth: u8, hash: u64, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  let mut bstree_path = input.clone();
  bstree_path.push("tiles.bstree");
  check_file_exists(&bstree_path, is_cgi)?;

  let bstree_file = File::open(&bstree_path)?;
  let mmap = unsafe { MmapOptions::new().map(&bstree_file)? };
  let (_version, data_starting_byte, bstree_meta) = read_meta(&mmap)?;
  let visitor = bstree_meta.get_root().visit(
    VisitorExact::new(to_zuniq(depth, hash)),
    &mmap[data_starting_byte..],
    &U64RW,
    &U64RW,
  )?;
  if let Some(Entry { id, val: _ }) = visitor.entry {
    #[cfg(feature = "cgi")]
    if is_cgi {
      println!("Content-Type: text/plain\n");
    }
    let mut write = stdout().lock();
    writeln!(
      write,
      "# Completeness = {}/{}",
      id >> 40,
      id & 0x000000FFFFFFFFFF
    )?;
    // we do not used 'is_cgi' after here (tile found, we assume the file will be found too).
    print_tile_data(input, depth, hash, &mut write)
  } else {
    #[cfg(feature = "cgi")]
    if is_cgi {
      println!(
        "Status: {}\n\nNo cell {}/{} in the HiPS.",
        Status::NOT_FOUND,
        depth,
        hash
      );
    }
    Ok(())
  }
}

fn print_tiles_stats(mut input: PathBuf, is_cgi: bool) -> Result<(), Box<dyn Error>> {
  input.push("tiles.bstree");
  check_file_exists(&input, is_cgi)?;
  #[cfg(feature = "cgi")]
  if is_cgi {
    println!("Content-Type: text/plain\n");
  }
  // Open file and read metadata
  let file = File::open(&input)?;
  let mmap = unsafe { MmapOptions::new().map(&file)? };
  let (_version, data_starting_byte, _) = read_meta(&mmap)?;
  let mut write = stdout().lock();
  writeln!(&mut write, "depth,cell,cumul_count,tot_count")?;
  for kv in mmap[data_starting_byte..].chunks_exact(16) {
    let mut cursor = Cursor::new(kv);
    let c = U64RW.read(&mut cursor)?;
    let z = U64RW.read(&mut cursor)?;
    let (depth, hash) = from_zuniq(z);
    writeln!(
      &mut write,
      "{},{},{},{}",
      depth,
      hash,
      c >> 40,
      c & 0x000000FFFFFFFFFF
    )?;
  }
  Ok(())
}

fn print_tile_data<W: Write>(
  input: PathBuf,
  depth: u8,
  hash: u64,
  write: &mut W,
) -> Result<(), Box<dyn Error>> {
  let mut path = input.clone();
  path.push(format!("hips.cat.layer{}.hcidx.fits", depth));
  match FITSCIndex::from_fits_file(path)? {
    FITSCIndex::ImplicitU64(fits_hci) => {
      print_tile_data_from_idx(input, depth, hash, &fits_hci, write)
    }
    FITSCIndex::ExplicitU32U64(fits_hci) => {
      print_tile_data_from_idx(input, depth, hash, &fits_hci, write)
    }
    FITSCIndex::ExplicitU64U64(fits_hci) => {
      print_tile_data_from_idx(input, depth, hash, &fits_hci, write)
    }
    _ => Err(
      String::from("Wrong data type in the FITS Healpix Cumulative Index type. Expected: u64.")
        .into(),
    ),
  }
}
fn print_tile_data_from_idx<'a, H, T, W>(
  mut dir: PathBuf,
  depth: u8,
  hash: u64,
  fits_idx: &'a T,
  write: &mut W,
) -> Result<(), Box<dyn Error>>
where
  H: HCIndex<V = u64>,
  T: FitsMMappedCIndex<HCIndexType<'a> = H> + 'a,
  W: Write,
{
  /*let file_name = fits_idx
    .get_indexed_file_name()
    .ok_or_else(|| String::from("No file name found in the FITS HCI file."))?;
  let expected_file_len = fits_idx
    .get_indexed_file_len()
    .ok_or_else(|| String::from("No file length found in the FITS HCI file."))?;
  check_file_exists_and_check_file_len(file_name, expected_file_len)?;*/

  // Ok, load index data...
  let hci = fits_idx.get_hcindex();
  let bytes_range = hci.get_cell(depth, hash);
  let bytes_range = bytes_range.start as usize..bytes_range.end as usize;

  // Load FITS file data
  dir.push(format!("hips.cat.layer{}.fits", depth));
  let file = File::open(dir)?;
  // Prepare reading, creating a memory map
  let mmap = unsafe { MmapOptions::new().map(&file) }?;
  #[cfg(not(windows))]
  if let Err(e) = mmap.advise(Advice::Sequential) {
    warn!(
      "Error advising for sequential read on file '{:?}': {}",
      file, e
    );
  }
  // Read as a FITS file, prepare iteration on HDUs
  let bytes = mmap.as_ref();
  let fits = FitsBytes::from_slice(bytes);
  let mut hdu_it = fits.new_iterator::<Bintable>();
  let _prim_hdu = hdu_it
    .next()
    .ok_or_else(|| String::from("No primary HDU found"))?;
  let bint_hdu = hdu_it
    .next()
    .ok_or_else(|| String::from("No secondary HDU found"))??;
  assert!(bint_hdu.is_bintable_hdu());
  match bint_hdu.parsed_header {
    HDUHeader::BinTable(bintable_header_full) => {
      let row_byte_size = bintable_header_full.row_byte_size();
      // Get schema
      let row_schema: RowSchema = bintable_header_full
        .cols()
        .iter()
        .enumerate()
        .map(|(i, col_header)| {
          col_header.schema().expect(&format!(
            "Unable to create schema for column {}: TFORM probably missing!",
            i + 1
          ))
        })
        .collect();
      // Print header
      let mut first = true;
      for (i, field) in bintable_header_full.cols().iter().enumerate() {
        if first {
          first = false;
        } else {
          write!(write, "\t")?;
        }
        match field.colname() {
          Some(name) => write!(write, "{}", name),
          None => write!(write, "col_{}", i),
        }?;
      }
      // Print data
      let mut visitor = CSVVisitor::new_custom(write, b'\t');
      for raw_row in (&mmap[bytes_range]).chunks(row_byte_size) {
        let mut de = DeserializerWithHeap::new(raw_row, &[0_u8; 0]);
        visitor.starts_new_line();
        row_schema.deserialize(&mut de, &mut visitor, CSVRowVisitor)?;
      }
      write!(write, "\n",).map_err(|e| e.into())
    }
    _ => Err(String::from("Secondary HDU not a BINTABLE!").into()),
  }
}

// Print the index.html page
fn print_landing_page(is_cgi: bool) -> Result<(), Box<dyn Error>> {
  #[cfg(feature = "cgi")]
  if is_cgi {
    println!("Content-Type: text/html\n");
  }
  let html = r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1.0, user-scalable=no">

        <script src="https://aladin.cds.unistra.fr/hips-templates/hips-landing-page.js" type="text/javascript"></script>
        <noscript>Please enable Javascript to view this page.</noscript>
    </head>

    <body></body>

    <script type="text/javascript">
      let root = new URL(window.location.href).pathname;
      if (root.endsWith("/") || root.endsWith("index.html")) {
        root = root.substring(0, root.lastIndexOf("/", root.length) + 1);
      } else {
        root = root + '/'
      }
      buildLandingPage({url: root});
    </script>
</html>
    "#;
  println!("{}", html);
  Ok(())
}
