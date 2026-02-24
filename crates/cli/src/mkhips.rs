//! In VizieR, 90% of all tables have a number of source lower than 12_000.
//! We fixed, as default values:
//! * n1: number of sources in allsky 1 (order 1)
//! * n2: number of sources in allsky 2 (order 2)
//! * n2 = 4 * n1 (because 4x more cells at level 2 than at level 1)
//! * n1 + n2 = 15000
//! * => n1 = 3000 and n2 = 12000
//! If VizieR tables where all alsky, more than 90% of the table would only need allsky 1 and 2.

// For qhips, keep a file with the indices of the columns to be kept by default.

use std::{
  borrow::Borrow,
  collections::BTreeSet,
  default::Default,
  error::Error,
  fmt::Display,
  fs,
  fs::File,
  io::{BufWriter, Error as IoErr, Seek, SeekFrom, Write},
  ops::Range,
  path::{Path, PathBuf},
};

use clap::Args;
use jiff::Timestamp;
use log::{debug, info, trace};
use memmap2::MmapOptions;
use serde::{Deserialize, Serialize};

use bstree_file_readonly::{
  IdType, IdVal, ValType,
  cliargs::{memsize::MemSizeArgs, mkargs::MkAlgoArgs},
  mk::BSTreeFileBuilder,
  rw::U64RW,
};
use cdshealpix::{
  n_hash,
  nested::{
    get,
    sort::cindex::{FITSCIndex, FitsMMappedCIndex, HCIndex, OwnedCIndex, OwnedCIndexExplicit},
    to_zuniq,
  },
};
use fitstable::{
  common::{ValueKwr, keywords::naxis::NAxis2},
  hdu::{
    header::{HDUHeader, builder::r#impl::bintable::Bintable},
    xtension::bintable::{
      read::expreval::{ExprEvalRow, TableSchema},
      schema::{FieldSchema, RowSchema, Schema},
    },
  },
  read::{hidx::check_file_exists_and_check_file_len, slice::FitsBytes},
};
use moc::{
  moc::{
    CellMOCIterator, RangeMOCIntoIterator, RangeMOCIterator,
    builder::maxdepth_range::RangeMocBuilder, range::RangeMOC,
  },
  qty::Hpx,
};

/// Make an HiPS from an HEALPix NESTED sorted and indexed BINTABLE FITS file.
/// The output MOC correspond to the footprint of the leaf tiles.
///
/// For large files, you probably want to create a slim version (few rows) for the progressive
/// view, and put fir each row a link pointing to the full content of the row.
#[derive(Debug, Args)]
pub struct MkHiPS {
  /// Path of the FITS file index (not the file containing the data)
  #[clap(value_name = "FILE")]
  input: PathBuf,
  /// Output directory containing the HiPS.
  #[clap(value_name = "DIR")]
  output: PathBuf,
  /// Number of sources at level 1 (if allsky).
  #[clap(short = 'n', long, value_name = "N", default_value_t = 3000)]
  n1: u16, // l1, 48 cells, 62.5 source per cell (if homogeneous and allsky)
  /// Ratio between the number of source in level 2 and level 1
  #[clap(short = 'r', long, default_value_t = 3)]
  r21: u8, // l2 = 4 * l1 = 12000 => 15_000 src in allsky1 + allsky2
  /// From level 3, number of cell per tile
  #[clap(short = 'm', long, default_value_t = 500)]
  n_tot: u16,
  /// Score, if any: sources with the highest score appear first in the hierarchy.
  #[clap(short = 'm', long, allow_hyphen_values = true)]
  score: Option<String>,
  #[command(flatten)]
  /// Set properties
  properties: Properties,
}

impl MkHiPS {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    let idx_file = self.input.clone();
    info!("Open index file...");
    match FITSCIndex::from_fits_file(idx_file).map_err(|e| format!("{}", e))? {
      FITSCIndex::ImplicitU64(hci) => self.exec_gen(&hci),
      FITSCIndex::ExplicitU32U64(hci) => self.exec_gen(&hci),
      FITSCIndex::ExplicitU64U64(hci) => self.exec_gen(&hci),
      _ => Err(
        String::from("Wrong data type in the FITS Healpix Cumulative Index type. Expected: u64.")
          .into(),
      ),
    }
    .map(|n_tiles| {
      info!("N tiles: {}", n_tiles);
      ()
    })
  }

  /// Returns the total number of non-emtpy tiles
  pub fn exec_gen<'a, H, T>(mut self, hcidx: &'a T) -> Result<usize, Box<dyn Error>>
  where
    H: HCIndex<V = u64>,
    T: FitsMMappedCIndex<HCIndexType<'a> = H> + 'a,
  {
    info!("Get metadata (file name and size, ra and dec columns) in index file...");
    let file_name = hcidx
      .get_indexed_file_name()
      .ok_or_else(|| String::from("No file name found in the FITS HCI file."))?;
    let expected_file_len = hcidx
      .get_indexed_file_len()
      .ok_or_else(|| String::from("No file length found in the FITS HCI file."))?;
    check_file_exists_and_check_file_len(file_name, expected_file_len)?;
    let lon = hcidx
      .get_indexed_colname_lon()
      .ok_or_else(|| String::from("No longitude column index found in the FITS HCI file."))
      .and_then(|s| {
        s.strip_prefix('#')
          .ok_or_else(|| format!("{} does not starts with '#'", s))
      })
      .and_then(|s| {
        s.parse::<usize>()
          .map_err(|e| format!("Error parsing {}: {:?}", s, e))
      })?;
    let lat = hcidx
      .get_indexed_colname_lat()
      .ok_or_else(|| String::from("No latitude column index found in the FITS HCI file."))
      .and_then(|s| {
        s.strip_prefix('#')
          .ok_or_else(|| format!("{} does not starts with '#'", s))
      })
      .and_then(|s| {
        s.parse::<usize>()
          .map_err(|e| format!("Error parsing {}: {:?}", s, e))
      })?;

    info!("Load index data...");
    let hci = hcidx.get_hcindex();
    let first_byte = hci.get(0);

    info!("Load indexed file data...");
    let file =
      File::open(file_name).map_err(|e| format!("Error opening file '{}': {:?}", file_name, e))?;
    // Prepare reading, creating a memory map
    let mmap =
      unsafe { MmapOptions::new().map(&file) }.map_err(|e| format!("Mmap error: {:?}", e))?;
    // Read as a FITS file, prepare iteration on HDUs
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    let mut hdu_it = fits.new_iterator::<Bintable>();

    info!(" * read primary HDU...");
    let prim_hdu_bytes = match hdu_it.next() {
      Some(Ok(hdu)) => {
        // Copy PrimaryHDU
        let mut prim_hdu_bytes = Vec::<u8>::new();
        hdu
          .copy_hdu(&mut prim_hdu_bytes)
          .map_err(|e| e.to_string())
          .map(|()| prim_hdu_bytes)
      }
      Some(Err(e)) => Err(e.to_string()),
      None => Err(String::from("No HDU found")),
    }?;

    info!(" * iterate on HDUs, looking for the first BINTABLE...");
    let hdu = loop {
      if let Some(hdu) = hdu_it.next() {
        let hdu = hdu?;
        if hdu.is_bintable_hdu() && first_byte == hdu.data_starting_byte() as u64 {
          break Some(hdu);
        }
      } else {
        break None;
      }
    };

    match hdu {
      Some(hdu) => {
        let mut bintable_header_bytes = Vec::<u8>::new();
        hdu.copy_header(&mut bintable_header_bytes)?;

        info!("Parse header and get lon/lat column indices...");
        info!(" * read BINTABLE metadata...");
        let bintable_header = match &hdu.parsed_header {
          HDUHeader::BinTable(h) => h,
          _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
        };

        info!(" * build table schema...");
        let row_schema: RowSchema = bintable_header.build_row_schema();
        let col_names: Vec<String> = bintable_header.build_col_names();

        info!(
          " * get RA and Dec columns info, and ensure they are of type Double (no scale/offset allowed here so far)..."
        );
        let lon_meta = &row_schema.fields_schemas()[lon];
        let lat_meta = &row_schema.fields_schemas()[lat];
        if !matches!(lon_meta.schema, Schema::Double) {
          return Err(
            format!(
              "RA column is not a double. Header: {:?}",
              &bintable_header.cols()[lon]
            )
            .into(),
          );
        }
        if !matches!(lat_meta.schema, Schema::Double) {
          return Err(
            format!(
              "Dec column is not a double. Header: {:?}",
              &bintable_header.cols()[lat]
            )
            .into(),
          );
        }

        info!(" * define hpx29 method...");
        let layer29 = get(29);
        let hpx29 = move |row_bytes: &[u8]| {
          let lon = f64::from_be_bytes(
            row_bytes[lon_meta.starting_byte..lon_meta.starting_byte + 8]
              .try_into()
              .unwrap(),
          );
          let lat = f64::from_be_bytes(
            row_bytes[lat_meta.starting_byte..lat_meta.starting_byte + 8]
              .try_into()
              .unwrap(),
          );
          if lon.is_nan() || lat.is_nan() {
            0
          } else {
            layer29.hash(lon.to_radians(), lat.to_radians())
          }
        };

        info!("Compute coverage of order 1 and 2...");
        info!(" * count number of non-empty cell at level 2...");
        let mut nc2 = 0_u64; // number of non-empty cells at depth 2, in [0..192]
        for h in 0..192 {
          if hci.get_cell_noncumulative(2, h) > 0 {
            nc2 += 1;
          }
        }

        info!(" * deduce n1 and n2...");
        let bintable_data_starting_byte = hdu.data_starting_byte();
        let row_byte_size = bintable_header.row_byte_size();
        let nrows = bintable_header.n_rows() as u64;

        // n2 / n1 = r21
        // n12 = n1 + n2 = n1 * (1 + r21)
        // => n1 = n12 / (1 + r21) AND n2 = n12 - n1
        let one_plus_r21 = 1_u64 + self.r21 as u64;
        let n12_allsky = (self.n1 as u64) * one_plus_r21;
        let n12 = (n12_allsky * nc2) / 192;
        let n1 = n12 / one_plus_r21;
        let n2 = n12 - n1;
        info!("   + n1: {}; n2: {}; n1+2: {}.", n1, n2, n12);

        // Create the destination directory if it does not exists.
        if !fs::exists(&self.output).unwrap_or(false) {
          fs::create_dir(&self.output)?;
        } // else ensure the directory is empty?

        info!("Prepare inputs (parameters, I/Os, MOC builder...");
        let algo_params = AlgoParams::new(
          self.output.clone(),
          n12,
          n1,
          n2,
          one_plus_r21,
          self.n_tot as u64,
        );

        // Get score! compile_f64_expr
        let expr_table_schema = TableSchema::new(&col_names, row_schema.fields_schemas());
        let score = self
          .score
          .clone()
          .map(|expr| {
            expr_table_schema.compile_f64_expr(expr) // .map(|f| {
            // Box::new(f) as Box<dyn for<'b> Fn(&ExprEvalRow<'b>) -> f64 + Sync + Send + 'b>
            //})
          })
          .transpose()?;

        let input_data = InputData::new(
          bytes,
          bintable_data_starting_byte,
          row_byte_size,
          prim_hdu_bytes,
          bintable_header_bytes,
          nrows,
          hci,
          lon,
          lat,
          hpx29,
          row_schema.fields_schemas(),
          score,
        );

        let mut moc_builder = RangeMocBuilder::<u64, Hpx<u64>>::new(29, None);
        let mut stat_writer = TilesStatWriter::new(self.tmp_bstree_path(), self.bstree_path())?;

        info!("Start processing...");
        let depth_max = Layer1and2.exec(
          &algo_params,
          &input_data,
          &mut moc_builder,
          &mut stat_writer,
        )?;

        info!("Set proper MOC depth...");
        let moc = RangeMOC::new(depth_max, moc_builder.into_moc().into_moc_ranges());

        info!("Compute properties values...");
        let sky_fraction = moc.coverage_percentage();
        let (lon_rad, lat_rad) = (&moc).into_range_moc_iter().cells().mean_center();
        let r_max_rad = (&moc)
          .into_range_moc_iter()
          .cells()
          .max_distance_from(lon_rad, lat_rad);
        self.properties.set_fixed_values();
        self.properties.set_computed_values(
          depth_max,
          nrows,
          (nrows * (row_byte_size as u64)) / 1024,
          sky_fraction,
          lon_rad.to_degrees(),
          lat_rad.to_degrees(),
          r_max_rad.to_degrees(),
        );

        info!("Write moc and properties files...");
        self
          .write_moc(moc)
          .and_then(|()| self.write_properties(&self.properties))?;

        info!("Write tiles stats in BSTree file...");
        stat_writer.build_bstree().map_err(|e| e.into())
      }
      None => Err(format!("No HDU with data starting at byte offset {}", first_byte).into()),
    }
  }

  fn write_moc(&self, moc: RangeMOC<u64, Hpx<u64>>) -> Result<(), Box<dyn Error>> {
    moc
      .to_fits_file_ivoa(None, None, self.moc_path())
      .map_err(|e| e.into())
  }

  fn write_properties(&self, properties: &Properties) -> Result<(), Box<dyn Error>> {
    toml::to_string_pretty(properties)
      .map_err(|e| e.into())
      .and_then(|content| fs::write(self.properties_path(), content).map_err(|e| e.into()))
  }

  fn moc_path(&self) -> PathBuf {
    let mut path = self.output.clone();
    path.push("moc.fits");
    path
  }

  fn properties_path(&self) -> PathBuf {
    let mut path = self.output.clone();
    path.push("properties.toml");
    path
  }

  fn tmp_bstree_path(&self) -> PathBuf {
    let mut path = self.output.clone();
    path.push("tiles.sorted.tmp");
    path
  }

  fn bstree_path(&self) -> PathBuf {
    let mut path = self.output.clone();
    path.push("tiles.bstree");
    path
  }
}

/// See [HiPS stnadard, p. 17-19](http://www.ivoa.net/documents/HiPS/20170406/PR-HIPS-1.0-20170406.pdf)
#[derive(Default, Clone, Debug, Args, Serialize, Deserialize)]
pub struct Properties {
  #[clap(long, default_value = "ivo://${PUBLISHER}/${HIPS_NAME}")]
  /// Unique identifier of the HiPS, e.g. `ivo://CDS/I/355/gaiadr3`
  creator_did: String,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Unique ID of the HiPS publisher, e.g. `ivo://CDS`
  publisher_id: Option<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Short name of original data set, e.g `Gaia`
  obs_collection: Option<String>,
  #[clap(long, default_value = "${TITLE}")]
  /// Data set title, e.g. `Gaia DR3 Main source`
  obs_title: String,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Data set description
  obs_description: Option<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Acknowledgment mention
  obs_ack: Option<String>,
  #[clap(long)]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  /// Provenance of the original data
  prov_progenitor: Vec<String>,
  #[clap(long)]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  /// Bibliographic reference
  bib_reference: Vec<String>,
  #[clap(long)]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  /// URL to bibliographic reference
  bib_reference_url: Vec<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Copyright mention
  obs_copyright: Option<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// URL to a copyright mention
  obs_copyright_url: Option<String>,
  #[clap(long)]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  /// General wavelength, e.g. "Radio", "Optical", "UV", "X-ray"
  obs_regime: Vec<String>,
  #[clap(long)]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  /// Data UCDs
  data_ucd: Vec<String>,
  #[clap(skip)] // = "0.1"
  /// HiPS version, default value should not be changed!
  hips_version: String,
  #[clap(skip)] // Set by default to: concat!(clap::crate_name!(), "_v", clap::crate_version!()))
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Name and version of the tool used for building the HiPS
  hips_builder: Option<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Institute or person who built the HiPS, e.g. `CDS (N. Surname)`
  hips_publisher: Option<String>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  // value_parser = clap::builder::StringValueParser::new().try_map(parse_datetime), DateTime<Local> with Chrono (with serde??)
  /// HiPS first creation date, format: `YYYY-mm-ddTHH:MMZ`
  hips_creation_date: Option<Timestamp>,
  #[clap(skip)] // Computed in default
  /// Release date of the HiPS, we use the creation date (computed automatically)
  hips_release_date: Timestamp, // = 2023-11-16T22:11Z, DateTime<Local> with Chrono( with serde??)?
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// HiPS public URLs, e.g. `https://hipscat.cds.unistra.fr/HiPSCatService/I/255/gaiadr3`
  hips_service_url: Option<String>,
  #[clap(long, default_value = "public master clonableOnce")]
  /// Status when shared in a HiPS node
  hips_status: String,
  #[clap(skip)] // Computed
  #[serde(skip_serializing_if = "Option::is_none")]
  /// HiPS size estimation, in kB
  hips_estsize: Option<u64>,
  #[clap(long, default_value = "equatorial")]
  /// Positions frame, default value should not be changed!
  hips_frame: String,
  #[clap(skip)]
  // = 1,
  // But, it would be nice to be able to change this for a better algorithm, e.g.
  // aply a bottom-up (instead of up-bottom) approach (also making the difference between an empty
  // tile and a tile not in the hierarchy: if only a small part of a large tile contains data, the
  // tile could be empty at low resolution, but with sources appearing at high resolution.
  /// HiPS starting order (always 1)
  hips_order_min: u8,
  #[clap(skip)] // Computed
  /// HEALPix order of the deepest tile(s), computed automatically
  hips_order: u8,
  #[clap(skip)] // = "tsv"
  /// Tile format, default value should not be changed!
  hips_tile_format: String,
  #[clap(skip)] // = "catalog"
  /// HiPS type, default value should not be changed!
  dataproduct_type: String,
  #[clap(skip)] // Computed later
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Total number fo rows in the HiPS, computed automatically
  hips_cat_nrows: Option<u64>,
  #[clap(skip)] // Computed later
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Longitude of the initial position when visualize in a tool like Aladin, computed automatically
  hips_initial_ra: Option<f64>,
  #[clap(skip)] // Computed later
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Longitude of the initial position when visualize in a tool like Aladin, computed automatically
  hips_initial_dec: Option<f64>,
  #[clap(skip)] // Computed later
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Default display size, in degrees
  hips_initial_fov: Option<f64>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Start time of the observations, in MJD
  t_min: Option<f64>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Stop time of the observations, in MJD
  t_max: Option<f64>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Start in spectral coordinates, in meters
  em_min: Option<f64>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Stop in spectral coordinates, in meters
  em_max: Option<f64>,
  #[clap(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// '/' separated keywords suggesting a display hierarchy to the client, e.g. catalog/vizier/I
  client_category: Option<String>,
  #[clap(skip)] // Computed later
  #[serde(skip_serializing_if = "Option::is_none")]
  /// Fraction of the sky covers by the MOC associated to the HiPS, in `[0.0, 1.0]`
  moc_sky_fraction: Option<f64>,
}

impl Properties {
  fn set_fixed_values(&mut self) {
    if self.hips_version.is_empty() {
      self.hips_version = String::from("1.0");
    }
    if self
      .hips_builder
      .as_ref()
      .map(|s| s.is_empty())
      .unwrap_or(true)
    {
      let creator = concat!(env!("CARGO_PKG_NAME"), "_v", env!("CARGO_PKG_VERSION"));
      self.hips_builder = Some(String::from(creator));
    }
    if self.hips_release_date == Timestamp::default() {
      self.hips_release_date = Timestamp::now();
      if self.hips_creation_date.is_none() {
        self.hips_creation_date = Some(self.hips_release_date.clone());
      }
    }
    if self.hips_status.is_empty() {
      self.hips_status = String::from("public master clonableOnce");
    }
    if self.hips_frame.is_empty() {
      self.hips_frame = String::from("equatorial");
    }
    if self.hips_order_min == 0 {
      self.hips_order_min = 1;
    }
    if self.hips_tile_format.is_empty() {
      self.hips_tile_format = String::from("tsv");
    }
    if self.dataproduct_type.is_empty() {
      self.dataproduct_type = String::from("catalog");
    }
  }

  fn set_computed_values(
    &mut self,
    depth_max: u8,
    nrows: u64,
    approx_size: u64,
    sky_fraction: f64,
    ra_deg: f64,
    de_deg: f64,
    radius_deg: f64,
  ) {
    self.hips_cat_nrows = Some(nrows);
    self.hips_estsize = Some(approx_size);
    self.hips_initial_ra = Some(ra_deg);
    self.hips_initial_dec = Some(de_deg);
    self.hips_initial_fov = Some(radius_deg);
    self.hips_order = depth_max;
    self.moc_sky_fraction = Some(sky_fraction);
  }
}

impl Display for Properties {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let width = 20;
    writeln!(f, "{:<width$} = {}", "creator_did", &self.creator_did)?;
    if let Some(publisher_id) = &self.publisher_id {
      writeln!(f, "{:<width$} = {}", "publisher_id", publisher_id)?;
    }
    if let Some(obs_collection) = &self.obs_collection {
      writeln!(f, "{:<width$} = {}", "obs_collection", obs_collection)?;
    }
    writeln!(f, "{:<width$} = {}", "obs_title", &self.obs_title)?;
    if let Some(obs_description) = &self.obs_description {
      writeln!(f, "{:<width$} = {}", "obs_description", obs_description)?;
    }
    if let Some(obs_ack) = &self.obs_ack {
      writeln!(f, "{:<width$} = {}", "obs_ack", obs_ack)?;
    }
    for prov_progenitor in &self.prov_progenitor {
      writeln!(f, "{:<width$} = {}", "prov_progenitor", prov_progenitor)?;
    }
    for bib_reference in &self.bib_reference {
      writeln!(f, "{:<width$} = {}", "bib_reference", bib_reference)?;
    }
    for bib_reference_url in &self.bib_reference_url {
      writeln!(f, "{:<width$} = {}", "bib_reference_url", bib_reference_url)?;
    }
    if let Some(obs_copyright) = &self.obs_copyright {
      writeln!(f, "{:<width$} = {}", "obs_copyright", obs_copyright)?;
    }
    if let Some(obs_copyright_url) = &self.obs_copyright_url {
      writeln!(f, "{:<width$} = {}", "obs_copyright_url", obs_copyright_url)?;
    }
    for obs_regime in &self.obs_regime {
      writeln!(f, "{:<width$} = {}", "obs_regime", obs_regime)?;
    }
    for data_ucd in &self.data_ucd {
      writeln!(f, "{:<width$} = {}", "data_ucd", data_ucd)?;
    }
    writeln!(f, "{:<width$} = {}", "hips_version", &self.hips_version)?;
    if let Some(hips_builder) = &self.hips_builder {
      writeln!(f, "{:<width$} = {}", "hips_builder", hips_builder)?;
    }
    if let Some(hips_publisher) = &self.hips_publisher {
      writeln!(f, "{:<width$} = {}", "hips_publisher", hips_publisher)?;
    }
    if let Some(hips_creation_date) = &self.hips_creation_date {
      writeln!(
        f,
        "{:<width$} = {}",
        "hips_creation_date", hips_creation_date
      )?;
    }
    writeln!(
      f,
      "{:<width$} = {}",
      "hips_release_date", &self.hips_release_date
    )?;
    if let Some(hips_service_url) = &self.hips_service_url {
      writeln!(f, "{:<width$} = {}", "hips_service_url", hips_service_url)?;
    }
    writeln!(f, "{:<width$} = {}", "hips_status", &self.hips_status)?;
    if let Some(hips_estsize) = &self.hips_estsize {
      writeln!(f, "{:<width$} = {}", "hips_estsize", hips_estsize)?;
    }
    writeln!(f, "{:<width$} = {}", "hips_frame", &self.hips_frame)?;
    writeln!(f, "{:<width$} = {}", "hips_order_min", &self.hips_order_min)?;
    writeln!(f, "{:<width$} = {}", "hips_order", &self.hips_order)?;
    writeln!(
      f,
      "{:<width$} = {}",
      "hips_tile_format", &self.hips_tile_format
    )?;
    writeln!(
      f,
      "{:<width$} = {}",
      "dataproduct_type", &self.dataproduct_type
    )?;
    if let Some(hips_cat_nrows) = &self.hips_cat_nrows {
      writeln!(f, "{:<width$} = {}", "hips_cat_nrows", hips_cat_nrows)?;
    }
    if let Some(hips_initial_ra) = &self.hips_initial_ra {
      writeln!(f, "{:<width$} = {}", "hips_initial_ra", hips_initial_ra)?;
    }
    if let Some(hips_initial_dec) = &self.hips_initial_dec {
      writeln!(f, "{:<width$} = {}", "hips_initial_dec", hips_initial_dec)?;
    }
    if let Some(hips_initial_fov) = &self.hips_initial_fov {
      writeln!(f, "{:<width$} = {}", "hips_initial_fov", hips_initial_fov)?;
    }
    if let Some(t_min) = &self.t_min {
      writeln!(f, "{:<width$} = {}", "t_min", t_min)?;
    }
    if let Some(t_max) = &self.t_max {
      writeln!(f, "{:<width$} = {}", "t_max", t_max)?;
    }
    if let Some(em_min) = &self.em_min {
      writeln!(f, "{:<width$} = {}", "em_min", em_min)?;
    }
    if let Some(em_max) = &self.em_max {
      writeln!(f, "{:<width$} = {}", "em_max", em_max)?;
    }
    if let Some(client_category) = &self.client_category {
      writeln!(f, "{:<width$} = {}", "client_category", client_category)?;
    }
    if let Some(moc_sky_fraction) = &self.moc_sky_fraction {
      writeln!(f, "{:<width$} = {}", "moc_sky_fraction", moc_sky_fraction)?;
    }
    Ok(())
  }
}

// Derived values comming from:
// (1): n12 = n1 + n2
// (2): r21 = n2 / n1
// => n12 = n1 (1 + r21)
// => n1 = n12 / (1 + r21)
// => n2 = n12 - n1
struct AlgoParams {
  /// Output dir
  dir: PathBuf,
  /// Number of sources to keep at level 1 plus 2 (already accounting for survey coverage)
  n12: u64,
  /// Number of sources in level 1 (already accounting for survey coverage)
  n1: u64,
  /// Number of sources in level 2 (already accounting for survey coverage)
  n2: u64,
  /// 1 + r21
  one_plus_r21: u64,
  /// Number of sources in each tile (of depth >= 3)
  nt: u64,
}
impl AlgoParams {
  fn new(dir: PathBuf, n12: u64, n1: u64, n2: u64, one_plus_r21: u64, nt: u64) -> Self {
    Self {
      dir,
      n12,
      n1,
      n2,
      one_plus_r21,
      nt,
    }
  }
}

/// # Generics parameters:
/// * `I`: the type of input fits file HEALPix Cumulative index
/// * `H`: closure computing the hpx29 index from a row bytes
/// * `E`: closure computing a score for a row bytes
struct InputData<'a, I, H, E>
where
  I: HCIndex<V = u64>,
  H: Fn(&'a [u8]) -> u64,
  E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
{
  /// All FITS file bytes.
  bytes: &'a [u8],
  /// Index of the first byte of the first row from the beginning of the file
  data_starting_byte: u64,
  /// Fixed length of a raw row, in byte.
  row_byte_size: u64,
  /// Copy of the primary HDU
  primary_hdu: Vec<u8>,
  /// Copy of the BINTABLE header
  bintable_header: Vec<u8>,
  /// Nuber of rows in the data
  nrows: u64,
  /// Healpix cumulative index on the FITS file.
  hcidx: I,
  /// Index of the longitude
  icol_lon: usize,
  /// Index of the latitude
  icol_lat: usize,
  /// Function to compute the HEALPix hash value at depth 29 of a row.
  hpx29: H,
  /// Field schema (used for deserialization when computing the score)
  schema: &'a [FieldSchema],
  /// Function computing the score of a row
  score: Option<E>,
}

impl<'a, I, H, E> InputData<'a, I, H, E>
where
  I: HCIndex<V = u64>,
  H: Fn(&'a [u8]) -> u64,
  E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
{
  fn new(
    bytes: &'a [u8], // Mmap,
    data_starting_byte: usize,
    row_byte_size: usize,
    primary_hdu: Vec<u8>,
    bintable_header: Vec<u8>,
    nrows: u64,
    hcidx: I,
    icol_lon: usize,
    icol_lat: usize,
    hpx29: H,
    schema: &'a [FieldSchema],
    score: Option<E>,
  ) -> Self {
    Self {
      bytes,
      data_starting_byte: data_starting_byte as u64,
      row_byte_size: row_byte_size as u64,
      primary_hdu,
      bintable_header,
      nrows,
      hcidx,
      icol_lon,
      icol_lat,
      hpx29,
      schema,
      score,
    }
  }

  fn has_score(&self) -> bool {
    self.score.is_some()
  }

  fn primary_hdu(&self) -> &[u8] {
    self.primary_hdu.as_slice()
  }

  fn bintable_header(&self) -> &[u8] {
    self.bintable_header.as_slice()
  }

  fn row_from_recno(&self, recno: u64) -> &'a [u8] {
    let from_byte = self.data_starting_byte + recno * self.row_byte_size;
    let to_byte = from_byte + self.row_byte_size;
    &self.bytes[from_byte as usize..to_byte as usize]
  }

  fn row_with_hpx29_from_recno(&self, recno: u64) -> (u64, &[u8]) {
    let from_byte = self.data_starting_byte + recno * self.row_byte_size;
    let to_byte = from_byte + self.row_byte_size;
    let row = &self.bytes[from_byte as usize..to_byte as usize];
    ((self.hpx29)(row), row)
  }

  /*
  fn all_rows(&self) -> impl Iterator<Item = &'a [u8]> {
    let from_byte = self.data_starting_byte;
    let to_byte = self.data_starting_byte + self.nrows * self.row_byte_size;
    self.bytes[from_byte as usize..to_byte as usize].chunks(self.row_byte_size as usize)
  }*/
  fn all_rows_with_hpx29(&self) -> impl Iterator<Item = (u64, &'a [u8])> {
    let from_byte = self.data_starting_byte;
    let to_byte = self.data_starting_byte + self.nrows * self.row_byte_size;
    self.bytes[from_byte as usize..to_byte as usize]
      .chunks(self.row_byte_size as usize)
      .map(|row| ((self.hpx29)(row), row))
  }

  /*
  fn all_rows_with_recno(&self) -> impl Iterator<Item = (usize, &'a [u8])> {
    self.all_rows().enumerate()
  }*/
  fn all_rows_with_hpx29_and_recno(&self) -> impl Iterator<Item = (usize, (u64, &'a [u8]))> {
    self.all_rows_with_hpx29().enumerate()
  }

  fn rows_in_recno_range<R: Borrow<Range<u64>>>(
    &self,
    recno_range: R,
  ) -> impl Iterator<Item = &'a [u8]> {
    let from_byte = self.data_starting_byte + self.row_byte_size * recno_range.borrow().start;
    let to_byte = self.data_starting_byte + self.row_byte_size * recno_range.borrow().end;
    self.bytes[from_byte as usize..to_byte as usize].chunks(self.row_byte_size as usize)
  }

  fn rows_in_recno_range_with_hpx29<R: Borrow<Range<u64>>>(
    &self,
    recno_range: R,
  ) -> impl Iterator<Item = (u64, &'a [u8])> {
    let from_byte = self.data_starting_byte + self.row_byte_size * recno_range.borrow().start;
    let to_byte = self.data_starting_byte + self.row_byte_size * recno_range.borrow().end;
    self.bytes[from_byte as usize..to_byte as usize]
      .chunks(self.row_byte_size as usize)
      .map(|row| ((self.hpx29)(row), row))
  }

  fn rows_in_recno_range_with_recno<R: Borrow<Range<u64>>>(
    &self,
    recnos: R,
  ) -> impl Iterator<Item = (usize, &'a [u8])> {
    (recnos.borrow().start as usize..recnos.borrow().end as usize)
      .into_iter()
      .zip(self.rows_in_recno_range(recnos))
  }

  fn rows_in_recno_range_with_hpx29_and_recno<R: Borrow<Range<u64>>>(
    &self,
    recnos: R,
  ) -> impl Iterator<Item = (usize, (u64, &'a [u8]))> {
    (recnos.borrow().start as usize..recnos.borrow().end as usize)
      .into_iter()
      .zip(self.rows_in_recno_range_with_hpx29(recnos))
  }

  fn rows_in_recno_range_except<R: Borrow<Range<u64>>>(
    &self,
    recnos: R,
    except: &BTreeSet<u64>,
  ) -> impl Iterator<Item = &'a [u8]> {
    self
      .rows_in_recno_range_with_recno(recnos)
      .filter_map(|(recno, row)| {
        if except.contains(&(recno as u64)) {
          None
        } else {
          Some(row)
        }
      })
  }

  fn hpx29_in_recno_range<R: Borrow<Range<u64>>>(&self, recnos: R) -> impl Iterator<Item = u64> {
    let from_byte = self.data_starting_byte + self.row_byte_size * recnos.borrow().start;
    let to_byte = self.data_starting_byte + self.row_byte_size * recnos.borrow().end;
    self.bytes[from_byte as usize..to_byte as usize]
      .chunks(self.row_byte_size as usize)
      .map(&self.hpx29)
  }

  fn hpx29_in_recno_range_with_recno<R: Borrow<Range<u64>>>(
    &self,
    recnos: R,
  ) -> impl Iterator<Item = (usize, u64)> {
    (recnos.borrow().start as usize..recnos.borrow().end as usize)
      .into_iter()
      .zip(self.hpx29_in_recno_range(recnos))
  }

  /*
  /// Warning: if depth > index depth, returns row in parent cell(s)
  fn rows_in_cell(&self, depth: u8, hash: u64) -> impl Iterator<Item = &'a [u8]> {
    trace!("Method 'rows_in_cell'. depth: {}; hash: {}.", depth, hash,);
    let byte_range = self.hcidx.get_cell(depth, hash);
    self.bytes[byte_range.start as usize..byte_range.end as usize]
      .chunks(self.row_byte_size as usize)
  } // with recno: build recno (from byte_index - data_starting_byte) / row_byte_size

  /// Warning: if depth > index depth, returns row in parent cell(s)
  fn hpx29_in_cell(&self, depth: u8, hash: u64) -> impl Iterator<Item = u64> {
    trace!("Method 'hpx29_in_cell'. depth: {}; hash: {}.", depth, hash,);
    let byte_range = self.hcidx.get_cell(depth, hash);
    self.bytes[byte_range.start as usize..byte_range.end as usize]
      .chunks(self.row_byte_size as usize)
      .map(&self.hpx29)
  }*/

  /// Returns the number of sub-cells (in `[0, 64[`) at the given depth + 3 containing at least one source.
  fn n_cells_at_depth_plus_3<R: Borrow<Range<u64>>>(&self, depth: u8, hash: u64, recnos: R) -> u8 {
    let depth_plus_3 = depth + 3;
    let count = if depth_plus_3 <= self.hcidx.depth() {
      let hash_plus_3 = hash << 6;
      (0..64)
        .into_iter()
        .filter(|i| {
          self
            .hcidx
            .get_cell_noncumulative(depth_plus_3, hash_plus_3 | i)
            > 0
        })
        .count() as u8
    } else if depth <= self.hcidx.depth() {
      let twice_dd = (29 - depth_plus_3) << 1;
      // Count the number of distinct sub-cell (0-64) in the current cell.
      self
        .hpx29_in_recno_range(recnos)
        .fold(0_u64, |c, hpx29| {
          let n64 = 1_u64 << ((hpx29 >> twice_dd) & 63);
          c | n64
        })
        .count_ones() as u8
    } else {
      let mut twice_dd = (29 - depth) << 1;
      let h29_range = hash << twice_dd..((hash + 1) << twice_dd);
      twice_dd -= 6;
      self
        .hpx29_in_recno_range(recnos)
        .filter(|h29| h29_range.contains(&h29))
        .fold(0_u64, |c, hpx29| {
          let n64 = 1_u64 << ((hpx29 >> twice_dd) & 63);
          c | n64
        })
        .count_ones() as u8
    };
    assert!((0..=64).contains(&count));
    count
  }

  fn has_index(&self, depth: u8) -> bool {
    depth <= self.hcidx.depth()
  }

  /// Returns recnos belonging to the given hash.
  /// Returns Ok is depth <= index depth else return Err.
  /// # Params
  /// * `depth`: depth of thc cell
  /// * `hash`: hash value of the cell
  /// * `conservative_recno_rangeconservative_recnos` a range containing (but possibly larger than) the cell of given depth and hash
  fn recno_range(&self, depth: u8, hash: u64, mut conservative_recnos: Range<u64>) -> Range<u64> {
    trace!(
      "Method 'recno_range'. depth: {}; hash: {}; recnos: {:?}",
      depth, hash, &conservative_recnos
    );
    let Range { start, end } = self.hcidx.get_cell(depth, hash);
    let recnos = (start - self.data_starting_byte) / self.row_byte_size
      ..(end - self.data_starting_byte) / self.row_byte_size;
    if self.has_index(depth) {
      recnos
    } else {
      if conservative_recnos.start < recnos.start {
        conservative_recnos.start = recnos.start;
      }
      if recnos.end < conservative_recnos.end {
        conservative_recnos.end = recnos.end;
      }
      self.compute_recno_range(depth, hash, conservative_recnos)
    }
  }

  /// Look for the range of records corresponding to the given healpix cell.
  ///
  /// In addition to `recno_range` when the result is an `Err`, compute the exact value
  /// by reading and filtering the rows in the file
  ///
  /// # Params
  /// * `depth`: depth of thc cell
  /// * `hash`: hash value of the cell
  /// * `conservative_recno_range` a range containing (but possibly larger than) the cell of given depth and hash
  fn compute_recno_range(
    &self,
    depth: u8,
    hash: u64,
    conservative_recno_range: Range<u64>,
  ) -> Range<u64> {
    let twice_dd = (29 - depth) << 1;
    let mut first = conservative_recno_range.end;
    for (recno, hpx29) in self.hpx29_in_recno_range_with_recno(&conservative_recno_range) {
      let recno = recno as u64;
      let hpx = hpx29 >> twice_dd;
      if hpx == hash {
        if recno < first {
          first = recno;
        }
      } else if hpx > hash {
        return first.min(recno)..recno;
      }
    }
    first..conservative_recno_range.end
  }

  fn recno_having_highest_score(&self, recnos: Range<u64>) -> u64 {
    match &self.score {
      None => (recnos.start + recnos.end) / 2, // point in the middle of the chunk
      Some(score) => recnos
        .into_iter()
        .min_by(|recno_l, recno_r| {
          let heap = &[0_u8; 0]; // So far e ignore data on the heap!
          let expr_eval_row_l = ExprEvalRow::new(self.schema, self.row_from_recno(*recno_l), heap);
          let expr_eval_row_r = ExprEvalRow::new(self.schema, self.row_from_recno(*recno_r), heap);
          score(&expr_eval_row_l).total_cmp(&score(&expr_eval_row_r))
        })
        .unwrap(),
    }
  }

  fn recno_having_highest_score_except(
    &self,
    recnos: Range<u64>,
    already_selected: &BTreeSet<u64>,
  ) -> Option<u64> {
    match &self.score {
      None => {
        // We do so instead of taking the first recno to avoid having several row at almost
        // the same location for sub-cells of index 0.
        let middle = (recnos.start + recnos.end) / 2;
        let upward = || {
          (middle..recnos.end)
            .into_iter()
            .find(|recno| !already_selected.contains(&recno))
        };
        (recnos.start..middle)
          .into_iter()
          .rev()
          .find(|recno| !already_selected.contains(&recno))
          .or_else(upward)
      }
      Some(score) => recnos
        .into_iter()
        .filter(|recno| !already_selected.contains(&recno))
        .min_by(|recno_l, recno_r| {
          let heap = &[0_u8; 0]; // So far e ignore data on the heap!
          let expr_eval_row_l = ExprEvalRow::new(self.schema, self.row_from_recno(*recno_l), heap);
          let expr_eval_row_r = ExprEvalRow::new(self.schema, self.row_from_recno(*recno_r), heap);
          score(&expr_eval_row_l).total_cmp(&score(&expr_eval_row_r))
        }),
    }
  }
}

struct TilesStatWriter {
  builder: BSTreeFileBuilder<u64, u64, U64RW, U64RW>,
}
impl TilesStatWriter {
  fn new(tmp_path: PathBuf, output_file_path: PathBuf) -> Result<Self, IoErr> {
    let chunk_size = 500_000; // => file of 7.63 MB
    let args = MkAlgoArgs::new(Some(chunk_size), Some(7), Some(tmp_path), output_file_path);
    let mem_size_args = MemSizeArgs {
      l1: 32,           // 32 kB
      disk: 8192,       // 8192 kB, i.e. 8MB
      fill_factor: 0.8, // 80%
    };
    BSTreeFileBuilder::<u64, u64, U64RW, U64RW>::new(
      args,
      mem_size_args,
      IdVal::new(IdType::U64, ValType::U64),
      U64RW,
      U64RW,
    )
    .map(|builder| Self { builder })
  }

  fn append(&mut self, depth: u8, hash: u64, cell_info: &CellInfo) -> Result<(), IoErr> {
    // At order 14 with all tiles containing 10_000 rows, we get n_cumul=140_000 rows < 16_777_216
    let n_cumul = cell_info.cumul_count;
    let n_tot = cell_info.tot_count;
    if n_cumul > 0 || n_tot > 0 {
      assert!(n_cumul < (1_u64 << 24));
      assert!(n_tot < (1_u64 << 40));
      // The value associated to the quantity that is indexed
      let val = n_tot | (n_cumul << 40);
      //  The indexed value (queries are performed on indexed values)
      let z = to_zuniq(depth, hash);
      /*trace!(
        "Append tile stat. depth: {:>2}; hash: {:>12}; z: {:>12}; zbinary: {:064b}.",
        depth,
        hash,
        z,
        z
      );*/
      self.builder.append(val, z)
    } else {
      Ok(())
    }
  }

  /// Returns the number of entries in the BSTree
  fn build_bstree(self) -> Result<usize, IoErr> {
    self.builder.build_index()
  }
}

struct Cell {
  hash: u64,
  info: CellInfo,
}
impl Cell {
  fn new(hash: u64) -> Self {
    Self {
      hash,
      info: Default::default(),
    }
  }
}
#[derive(Clone, Copy, Default)]
struct CellInfo {
  /// Byte of the first row in the cell, in the file of this layer.
  // In a first step, serve only to count the number of source in this layer
  from_byte: u64,
  /// Number of source in the cell up to this layer.
  cumul_count: u64,
  /// Number of sources in the cell, for all layers
  tot_count: u64,
}
impl CellInfo {
  /// Add a source in the cell but belonging to a lower resolution cell
  fn add_at_lower_res(&mut self) {
    self.cumul_count += 1;
    self.tot_count += 1;
  }
  /// Add a source in the cell, at the cell resolution
  fn add_at_cell_res(&mut self) {
    self.from_byte += 1;
    self.add_at_lower_res();
  }
  /// Add a source in the cell, but belonging to a higher resolution cell
  fn add_at_higher_res(&mut self) {
    self.tot_count += 1;
  }
}

/// `N` must be the number of cells at the given depth plus one (the last cell simply contains in
/// `from_byte` the last byte of the FITS data block).
struct CellLayerImplicit<const N: usize> {
  layer: [CellInfo; N],
}

impl<const N: usize> CellLayerImplicit<N> {
  fn new(depth: u8) -> Self {
    assert_eq!(N as u64, n_hash(depth) + 1);
    Self {
      layer: [Default::default(); N],
    }
  }
  /// Once the layer has been filled, transform simple counts into byte offsets.
  /// To be called internally only, when writing?
  fn finalize(mut self, byte_offset: u64, row_byte_size: u64) -> Self {
    // Compute from_byte (so far, they are juste counts) from raw count
    // to cumulative counts with starting offset.
    let mut next_byte = byte_offset;
    for cell in &mut self.layer {
      let add = cell.from_byte * row_byte_size;
      cell.from_byte = next_byte;
      next_byte += add;
    }
    self
  }
}

/// Both layers 1 and 2 are built at the same time, in the same 'virtual' struct.
struct Layer1and2;

impl Layer1and2 {
  /// Returns the deepest depth.
  fn exec<'a, I, H, E>(
    self,
    algo: &AlgoParams,
    input: &InputData<'a, I, H, E>,
    moc_builder: &mut RangeMocBuilder<u64, Hpx<u64>>,
    stat_writer: &mut TilesStatWriter,
  ) -> Result<u8, Box<dyn Error>>
  where
    I: HCIndex<V = u64>,
    H: Fn(&'a [u8]) -> u64,
    E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
  {
    let mut layer1 = Layer1or2::<49>::new(1, algo, input)?;

    let deepest_depth;
    if input.nrows <= (3 * algo.n1) / 2 {
      trace!(
        "All sources in level1 ({} < 1.5 * {})",
        input.nrows, algo.n1
      );
      // If nrows <= 1.5 * n1 => build layer 1 only
      for (hpx29, row) in input.all_rows_with_hpx29() {
        layer1.add_row(hpx29, row)?;
      }
      layer1.build_moc(moc_builder);

      deepest_depth = 1;
      Ok(())
    } else if input.nrows <= (3 * algo.n12) / 2 {
      trace!(
        "All sources in level either 1 or 2 ({} < 1.5 * {})",
        input.nrows, algo.n12
      );
      let mut layer2 = Layer1or2::<193>::new(2, algo, input)?;

      if !input.has_score() {
        // Specific algo without score
        for (recno, (hpx29, row)) in input.all_rows_with_hpx29_and_recno() {
          if (recno as u64) % algo.one_plus_r21 == 0 {
            layer2.add_at_lower_res(hpx29);
            layer1.add_row(hpx29, row)
          } else {
            layer1.add_at_hihger_res(hpx29);
            layer2.add_row(hpx29, row)
          }?;
          // Add cell at layer 2 to the moc
          let hpx2 = hpx29 >> 54;
          moc_builder.push(hpx2 << 54..(hpx2 + 1) << 54);
        }
      } else {
        // Specific algo with a score
        for recno_start in (0..input.nrows).step_by(algo.one_plus_r21 as usize) {
          let recnos = recno_start..(recno_start + algo.one_plus_r21).min(input.nrows);
          // compute highest score
          let recno_l1 = input.recno_having_highest_score(recnos.clone());
          // re-read, writing
          for (recno, (hpx29, row)) in input.rows_in_recno_range_with_hpx29_and_recno(&recnos) {
            if recno as u64 == recno_l1 {
              layer2.add_at_lower_res(hpx29);
              layer1.add_row(hpx29, row)
            } else {
              layer1.add_at_hihger_res(hpx29);
              layer2.add_row(hpx29, row)
            }?;
            // Add cell at layer 2 to the moc
            let hpx2 = hpx29 >> 54;
            moc_builder.push(hpx2 << 54..(hpx2 + 1) << 54);
          }
        }
      }
      layer2.build_moc(moc_builder);

      deepest_depth = 2;
      layer2.finalize(input.icol_lon, input.icol_lat, stat_writer)
    } else {
      trace!(
        "Number of sources in layer 1: {}; layer 2 {}. Number tot: {}",
        algo.n1, algo.n2, input.nrows
      );
      let mut layer2 = Layer1or2::<193>::new(2, algo, input)?;
      let mut layer3 = LayerExpl::new(3, algo, input)?;

      let chunk_size = (input.nrows + algo.n12 - 1) / algo.n12; // (floor + 1), except if input.nrows % algo.n12 == 0
      assert!(chunk_size >= 2); // Because nrows <= n12 is caught by a condition before reaching this points.

      // Prepare variables used in the loop
      // * current range of recnos to be read (or re-read)
      let mut recno_cursors = 0_u64..0_64;
      // * list of already selected recnos in recno_cursors
      let mut selected_recnos: BTreeSet<u64> = Default::default();
      // * next hash of depth 3 to be processed
      let mut next_h3 = 0;
      let mut h3_recnos = input.recno_range(3, next_h3, 0..input.nrows);
      while h3_recnos.start == h3_recnos.end {
        next_h3 += 1;
        h3_recnos = input.recno_range(3, next_h3, 0..input.nrows);
      }

      for (i12, recno_start) in (0..input.nrows).step_by(chunk_size as usize).enumerate() {
        // Select one source in the chunk for level 1 or 2.
        let recnos = recno_start..(recno_start + chunk_size).min(input.nrows);
        recno_cursors.end = recnos.end;
        trace!(
          "Chunk allsky number {}, recnos: {:?}; recno_cursor: {:?}; h3_recnos: {:?}",
          i12, &recnos, &recno_cursors, &h3_recnos
        );

        let selected_recno = input.recno_having_highest_score(recnos.clone());
        let (hpx29, row) = input.row_with_hpx29_from_recno(selected_recno);

        if i12 as u64 % algo.one_plus_r21 == 0 {
          layer2.add_at_lower_res(hpx29);
          layer1.add_row(hpx29, row)
        } else {
          layer1.add_at_hihger_res(hpx29);
          layer2.add_row(hpx29, row)
        }?;
        selected_recnos.insert(selected_recno);

        // Deal with level 3 cells fully covered by 'recnos.end'
        while h3_recnos.end <= recno_cursors.end {
          layer3.exec(
            next_h3,
            h3_recnos.clone(),
            &mut selected_recnos,
            algo,
            input,
            moc_builder,
            stat_writer,
          )?;
          recno_cursors.start = h3_recnos.end;
          selected_recnos
            .extract_if(h3_recnos.clone(), |_v| true)
            .for_each(drop);

          if h3_recnos.end < input.nrows {
            assert!(next_h3 < 767); // Else it means there is a problem in the index (not all depth 3 cells cover the full file!)
            next_h3 += 1;
            h3_recnos = input.recno_range(3, next_h3, recno_cursors.start..input.nrows);
            while h3_recnos.start == h3_recnos.end {
              assert!(next_h3 < 767); // Else it means there is a problem in the index (not all depth 3 cells cover the full file!)
              next_h3 += 1;
              h3_recnos = input.recno_range(3, next_h3, 0..input.nrows);
            }
          } else {
            break;
          }
        }
      }

      deepest_depth = layer3.get_deepest_depth();

      layer3
        .finalize(input.icol_lon, input.icol_lat)
        .and_then(|()| layer2.finalize(input.icol_lon, input.icol_lat, stat_writer))
    }
    .and_then(|()| layer1.finalize(input.icol_lon, input.icol_lat, stat_writer))
    .map(|()| deepest_depth)
  }
}

struct Layer1or2<const N: usize> {
  depth: u8,
  twice_dd: u64,
  cells: CellLayerImplicit<N>,
  fitsw: FitsHiPSLayerWriter,
}
impl<const N: usize> Layer1or2<N> {
  fn new<'a, I, H, E>(
    depth: u8,
    algo: &AlgoParams,
    input: &InputData<'a, I, H, E>,
  ) -> Result<Self, IoErr>
  where
    I: HCIndex<V = u64>,
    H: Fn(&'a [u8]) -> u64,
    E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
  {
    assert_eq!(N as u64, n_hash(depth) + 1);
    let twice_dd = ((29 - depth) << 1) as u64;
    let cells = CellLayerImplicit::<N>::new(depth);
    FitsHiPSLayerWriter::new(
      depth,
      algo.dir.clone(),
      input.primary_hdu(),
      input.bintable_header(),
      input.row_byte_size,
    )
    .map(|fitsw| Self {
      depth,
      twice_dd,
      cells,
      fitsw,
    })
  }

  fn add_at_lower_res(&mut self, hpx29: u64) {
    self.cells.layer[(hpx29 >> self.twice_dd) as usize].add_at_lower_res()
  }

  fn add_at_cell_res(&mut self, hpx29: u64) {
    let h = hpx29 >> self.twice_dd;
    self.cells.layer[h as usize].add_at_cell_res();
  }

  fn add_at_hihger_res(&mut self, hpx29: u64) {
    self.cells.layer[(hpx29 >> self.twice_dd) as usize].add_at_higher_res()
  }

  fn add_row(&mut self, hpx29: u64, row: &[u8]) -> Result<(), IoErr> {
    self.add_at_cell_res(hpx29);
    self.fitsw.write_row(row)
  }

  fn build_moc(&self, moc_builder: &mut RangeMocBuilder<u64, Hpx<u64>>) {
    for h in self.cells.layer.iter().enumerate().filter_map(|(h, c)| {
      if c.tot_count > 0 {
        Some(h as u64)
      } else {
        None
      }
    }) {
      moc_builder.push(h << self.twice_dd..(h + 1) << self.twice_dd);
    }
  }

  /// # Params
  /// * `icol_lon`: index of the longitude column
  /// * `icol_lat`: index of the latitude colutilesmn
  fn finalize(
    self,
    icol_lon: usize,
    icol_lat: usize,
    stat_writer: &mut TilesStatWriter,
  ) -> Result<(), Box<dyn Error>> {
    let n_hash = self.cells.layer.len() - 1;
    debug!("Write tiles stats for layer {}...", self.depth);
    for (hash, cell_info) in self.cells.layer[..n_hash].iter().enumerate() {
      stat_writer.append(self.depth, hash as u64, cell_info)?;
    }
    debug!("Write FITS index for layer {}...", self.depth);
    let entries: Vec<u64> = self
      .cells
      .finalize(
        self.fitsw.bintable_data_starting_byte,
        self.fitsw.row_byte_size,
      )
      .layer
      .iter()
      .map(|cell| cell.from_byte)
      .collect();
    let hcidx = OwnedCIndex::new_unsafe(self.depth, entries.into_boxed_slice());
    self.fitsw.finalize(hcidx, icol_lon, icol_lat)
  }
}

/// Layer with explicit index, from depth 3
struct LayerExpl {
  depth: u8,
  cells: Vec<Cell>,
  fitsw: FitsHiPSLayerWriter,
  sublayer: Option<Box<LayerExpl>>,
}
impl LayerExpl {
  fn new<'a, I, H, E>(
    depth: u8,
    algo: &AlgoParams,
    input: &InputData<'a, I, H, E>,
  ) -> Result<Self, IoErr>
  where
    I: HCIndex<V = u64>,
    H: Fn(&'a [u8]) -> u64,
    E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
  {
    FitsHiPSLayerWriter::new(
      depth,
      algo.dir.clone(),
      input.primary_hdu(),
      input.bintable_header(),
      input.row_byte_size,
    )
    .map(|fitsw| Self {
      depth,
      cells: Default::default(),
      fitsw,
      sublayer: None,
    })
  }

  fn get_deepest_depth(&self) -> u8 {
    match &self.sublayer {
      None => self.depth,
      Some(subl) => subl.get_deepest_depth(),
    }
  }

  // hash at the layer depth
  // recnos = the range of recnos covered by the hash
  fn exec<'a, I, H, E>(
    &mut self,
    hash: u64,
    recnos: Range<u64>,
    selected_recnos: &mut BTreeSet<u64>,
    algo: &AlgoParams,
    input: &InputData<'a, I, H, E>,
    moc_builder: &mut RangeMocBuilder<u64, Hpx<u64>>,
    stat_writer: &mut TilesStatWriter,
  ) -> Result<(), Box<dyn Error>>
  where
    I: HCIndex<V = u64>,
    H: Fn(&'a [u8]) -> u64,
    E: Fn(&ExprEvalRow<'a>) -> f64 + Sync + Send + 'a,
  {
    assert!(hash < n_hash(self.depth));

    // Coverage at depth + 3 (=> value in [0, 64])
    let cov3 = input.n_cells_at_depth_plus_3(self.depth, hash, &recnos) as u64;
    assert!((0..=64).contains(&cov3), "cov3: {}", cov3);

    // Number of rows in this cell
    let nrows_tot_in_cell = recnos.end - recnos.start;
    // Number of rows already selected in lower resolution layers
    let nrows_selected = selected_recnos.range(recnos.clone()).count() as u64;
    // Number of rows to be selected for this layer
    let nrows_to_select = (algo.nt * cov3) / 64;
    // Number of rows no yet selected
    let nrows_available = nrows_tot_in_cell - nrows_selected;

    let mut cell = Cell::new(hash);
    cell.info.from_byte = self.fitsw.position()?;
    cell.info.cumul_count = nrows_selected;
    cell.info.tot_count = nrows_tot_in_cell;

    /*trace!(
      " * exec layer depth: {}; cell: {}; recnos: {:?}; cov: {}, ntot: {}; nsel: {}; ntosel: {}; navai: {}",
      self.depth,
      hash,
      recnos,
      cov3,
      nrows_tot_in_cell,
      nrows_selected,
      nrows_to_select,
      nrows_available
    );*/

    if nrows_available <= (3 * nrows_to_select) / 2 {
      // trace!("   * put all remaining rows: {}", nrows_available);
      // Tolerance factor of 3/2 = 1.5
      for row in input.rows_in_recno_range_except(&recnos, &selected_recnos) {
        self.fitsw.write_row(row)?;
        cell.info.cumul_count += 1;
      }
      let tdd = (29 - self.depth) << 1;
      moc_builder.push(hash << tdd..(hash + 1) << tdd)
    } else {
      let next_depth = self.depth + 1;
      let mut next_subh = hash << 2; // hash value at depth + 1
      let next_endh = (hash + 1) << 2;
      let mut subh_recnos = if input.has_index(next_depth) {
        input.recno_range(next_depth, next_subh, recnos.clone())
      } else {
        input.compute_recno_range(next_depth, next_subh, recnos.clone())
      };
      /*trace!(
        " * subh. depth: {}; cell: {}, recnos: {:?}.",
        next_depth,
        next_subh,
        &subh_recnos
      );*/
      // If empty sub-cell, go to first non-empty sub-cell
      while subh_recnos.start == subh_recnos.end {
        next_subh += 1;
        assert!(next_subh < next_endh); // else pb in the index!
        subh_recnos = if input.has_index(next_depth) {
          input.recno_range(next_depth, next_subh, recnos.clone())
        } else {
          input.compute_recno_range(next_depth, next_subh, recnos.clone())
        };
        /*trace!(
          " * subh. depth: {}; cell: {}, recnos: {:?}.",
          next_depth,
          next_subh,
          &subh_recnos
        );*/
      }
      assert!(subh_recnos.start < subh_recnos.end); // Else it means no sub-cell covering this cell!

      let mut from_recno = recnos.start;
      for k in 1..=nrows_to_select {
        let to_recno = recnos.start + (k * nrows_tot_in_cell) / nrows_to_select;
        assert!(to_recno <= recnos.end);

        /*trace!(
          "   * chunk {}; chunk_recno: {:?}; recnos: {:?}; subh_recnos: {:?}",
          k,
          from_recno..to_recno,
          &recnos,
          &subh_recnos
        );*/

        if let Some(selected_recno) =
          input.recno_having_highest_score_except(from_recno..to_recno, &selected_recnos)
        {
          /*trace!(
            "recno_selected: {:>10}; depth: {}",
            selected_recno, self.depth
          );*/
          let row = input.row_from_recno(selected_recno);
          self.fitsw.write_row(row)?;
          cell.info.cumul_count += 1;
          // add count to cell!!
          selected_recnos.insert(selected_recno);
        }

        // Deal with level +1 cells fully covered by 'recnos.end'
        while subh_recnos.end <= to_recno {
          if let Some(sublayer) = self.sublayer.as_mut() {
            sublayer.exec(
              next_subh,
              subh_recnos.clone(),
              selected_recnos,
              algo,
              input,
              moc_builder,
              stat_writer,
            )?;
          } else {
            let mut new_sublayer = LayerExpl::new(next_depth, algo, input)?;
            new_sublayer.exec(
              next_subh,
              subh_recnos.clone(),
              selected_recnos,
              algo,
              input,
              moc_builder,
              stat_writer,
            )?;
            self.sublayer = Some(Box::new(new_sublayer));
          }
          // Remove recnos which are not going to be used any more
          selected_recnos
            .extract_if(subh_recnos.clone(), |_v| true)
            .for_each(drop);

          if subh_recnos.end < recnos.end {
            next_subh += 1;
            subh_recnos = if input.has_index(next_depth) {
              input.recno_range(next_depth, next_subh, subh_recnos.end..recnos.end)
            } else {
              input.compute_recno_range(next_depth, next_subh, subh_recnos.end..recnos.end)
            };
            /*trace!(
              " * subh. depth: {}; cell: {}, recnos: {:?}.",
              next_depth,
              next_subh,
              &subh_recnos
            );*/
            while subh_recnos.start == subh_recnos.end {
              next_subh += 1;
              assert!(next_subh < next_endh); // else pb in the index!
              subh_recnos = if input.has_index(next_depth) {
                input.recno_range(next_depth, next_subh, recnos.clone())
              } else {
                input.compute_recno_range(next_depth, next_subh, recnos.clone())
              };
              /*trace!(
                " * subh. depth: {}; cell: {}, recnos: {:?}.",
                next_depth,
                next_subh,
                &subh_recnos
              );*/
            }
          } else {
            break;
          }
        }

        from_recno = to_recno;
      }
    }
    // Remove recno range in selected_recnos
    selected_recnos
      .extract_if(recnos.clone(), |_v| true)
      .for_each(drop);
    // Write the tile stats (we do this here instead of in 'finalize' to get files more or less already sorted
    stat_writer.append(self.depth, hash, &cell.info)?;
    // Add the index element
    self.cells.push(cell);
    Ok(())
  }

  /// # Params
  /// * `icol_lon`: index of the longitude column
  /// * `icol_lat`: index of the latitude column
  fn finalize(mut self, icol_lon: usize, icol_lat: usize) -> Result<(), Box<dyn Error>> {
    // Bottom-up finalize
    if let Some(sub) = self.sublayer {
      sub.finalize(icol_lon, icol_lat)?;
    }
    // Build the last element of the index
    // * unwrap is ok here, because we build the object when we have at least one cell
    let mut cell = Cell::new(self.cells.last().unwrap().hash + 1);
    self.fitsw.position().map(|pos| {
      cell.info.from_byte = pos;
      self.cells.push(cell);
      ()
    })?;
    // Build HCIndex and finalize
    if self.depth <= 13 {
      let entries: Vec<(u32, u64)> = self
        .cells
        .iter()
        .map(|cell| (cell.hash as u32, cell.info.from_byte))
        .collect();
      let hcidx = OwnedCIndexExplicit::new_unchecked(self.depth, entries);
      self.fitsw.finalize(hcidx, icol_lon, icol_lat)
    } else {
      let entries: Vec<(u64, u64)> = self
        .cells
        .iter()
        .map(|cell| (cell.hash, cell.info.from_byte))
        .collect();
      let hcidx = OwnedCIndexExplicit::new_unchecked(self.depth, entries);
      self.fitsw.finalize(hcidx, icol_lon, icol_lat)
    }
  }
}

/// Structure used to write a HiPS FITS file associated to a HEALPix layer of given depth.
struct FitsHiPSLayerWriter {
  depth: u8,
  dir: PathBuf,
  writer: BufWriter<File>,
  row_byte_size: u64,
  bintable_header_starting_byte: u64,
  bintable_data_starting_byte: u64,
  n_written_rows: u64,
}

impl FitsHiPSLayerWriter {
  fn new<P: AsRef<Path>>(
    depth: u8,
    dir: P,
    prim_hdu_bytes: &[u8],
    bintable_header: &[u8],
    row_byte_size: u64,
  ) -> Result<Self, IoErr> {
    let prim_hdu_len = prim_hdu_bytes.len() as u64;
    let bintable_header_len = bintable_header.len() as u64;
    assert_eq!(prim_hdu_len % 2880, 0);
    assert_eq!(bintable_header_len % 2880, 0);
    let dir = dir.as_ref().to_path_buf();
    let mut path = dir.clone();
    path.push(Self::filename(depth));
    File::create(&path)
      .map(BufWriter::new)
      .and_then(|mut writer| {
        writer
          .write_all(prim_hdu_bytes)
          .and_then(|()| writer.write_all(bintable_header))
          .and_then(|()| {
            writer.stream_position().map(|position| {
              let bintable_header_starting_byte = prim_hdu_len;
              let bintable_data_starting_byte = position as u64;
              assert_eq!(
                prim_hdu_len + bintable_header_len,
                bintable_data_starting_byte
              );
              FitsHiPSLayerWriter {
                depth,
                dir,
                writer,
                row_byte_size,
                bintable_header_starting_byte,
                bintable_data_starting_byte,
                n_written_rows: 0,
              }
            })
          })
      })
  }

  fn filename(depth: u8) -> String {
    format!("hips.cat.layer{}.fits", depth)
  }
  fn filename_hcidx(depth: u8) -> String {
    format!("hips.cat.layer{}.hcidx.fits", depth)
  }

  fn position(&mut self) -> Result<u64, IoErr> {
    self.writer.stream_position()
  }

  fn write_row(&mut self, row: &[u8]) -> Result<(), IoErr> {
    assert_eq!(row.len(), self.row_byte_size as usize);
    self.n_written_rows += 1;
    self.writer.write_all(row)
  }

  /// Write the FITS BINTABLE file, ensuring its size if  a mutliple of 2880 byte
  /// and overwriting the number of rows, and returns (if Ok):
  /// * the HEALPix depth associated to the file
  /// * the directory containing the file
  /// * the length of the file, in bytes
  /// # Warning
  /// * this method is called internnaly (by Sefl) and should not be called explicitly.
  /// * => it would be better to put the structure in its own module to ensure the privacy of the method
  fn finalize_bintable(mut self) -> Result<(u8, PathBuf, u64), Box<dyn Error>> {
    debug!("Write bintable for layer {}...", self.depth);

    let expected_pos = self.bintable_data_starting_byte + self.n_written_rows * self.row_byte_size;

    // Complete bytes if necessary
    let byte_len = self.writer.stream_position().and_then(|actual_pos| {
      assert_eq!(expected_pos, actual_pos);
      if actual_pos % 2880 != 0 {
        let pad_length = 2880 - actual_pos % 2880;
        self
          .writer
          .write_all(vec![0_u8; pad_length as usize].as_slice())
          .map(|()| actual_pos + pad_length)
      } else {
        Ok(actual_pos)
      }
    })?;

    // Overwrite number of rows.
    let mut naxis2 = [0_u8; 80];
    self
      .writer
      .seek(SeekFrom::Start(self.bintable_header_starting_byte + 4 * 80))?;
    NAxis2::new(self.n_written_rows).write_kw_record(&mut std::iter::once(Ok(&mut naxis2)))?;
    debug!("Rewrite NAXIS2: {}", String::from_utf8_lossy(&naxis2));
    self
      .writer
      .write_all(naxis2.as_slice())
      .map_err(|e| e.into())
      .map(|()| (self.depth, self.dir, byte_len))
  }

  fn finalize<H: HCIndex>(
    self,
    hcidx: H,
    icol_lon: usize,
    icol_lat: usize,
  ) -> Result<(), Box<dyn Error>> {
    self
      .finalize_bintable()
      .and_then(|(depth, mut dir, file_len)| {
        dir.push(Self::filename_hcidx(depth));
        let indexed_file_name_owned = Self::filename(depth);
        let best_repr = hcidx.best_representation(4.0);
        let indexed_file_name = Some(indexed_file_name_owned.as_str());
        let indexed_file_len = Some(file_len);
        let indexed_file_mdfy_date = None;
        let indexed_file_md5 = None;
        let index_colname_lon = format!("#{}", icol_lon);
        let index_colname_lat = format!("#{}", icol_lat);
        debug!("Write bintable hcidx for layer {}...", depth);
        hcidx
          .to_fits_file(
            dir,
            best_repr,
            indexed_file_name,
            indexed_file_len,
            indexed_file_md5,
            indexed_file_mdfy_date,
            Some(index_colname_lon.as_str()),
            Some(index_colname_lat.as_str()),
          )
          .map_err(|e| e.into())
      })
  }
}
