use std::{
  error::Error,
  f64::consts::PI,
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter},
  num::{ParseFloatError, ParseIntError},
  ops,
  path::{Path, PathBuf},
  str::FromStr,
};

use clap::{Args, Subcommand};

use cdshealpix::TWICE_PI;
use fitstable::read::hidx::qidx;
use moc::{
  deser::{
    ascii::from_ascii_ivoa,
    fits::{MocIdxType, MocQtyType, MocType, from_fits_ivoa},
    json::from_json_aladin,
  },
  idx::Idx,
  moc::{
    CellMOCIntoIterator, CellMOCIterator, CellOrCellRangeMOCIntoIterator,
    CellOrCellRangeMOCIterator, RangeMOCIntoIterator, RangeMOCIterator,
    range::{RangeMOC, op::convert::convert_to_u64},
  },
  qty::{Hpx, MocQty},
};
use skyregion::{
  SkyRegion, SkyRegionProcess,
  common::math::HALF_PI,
  regions::{
    cone::Cone, ellipse::EllipticalCone, hpx::HpxCell, hpxrange::HpxRange, hpxranges::HpxRanges,
    multicone::MultiCone, polygon::Polygon, ring::Ring, stcs::Stcs, zone::Zone,
  },
};

/// Perform a spatial query on a HEALPix NESTED sorted and indexed BINTABLE FITS file,
#[derive(Debug, Args)]
pub struct QIndex {
  /// Path of the FITS file containing the HEALPix index
  #[clap(value_name = "FILE")]
  input: PathBuf,
  /// Path of the output FITS file, containing the query result
  #[clap(value_name = "FILE")]
  output: PathBuf,
  /// Put a limit on the number of tuples returned
  #[clap(short = 'l', long = "limit")]
  limit: Option<usize>,
  /// Sky region constraint
  #[command(subcommand)]
  region: SkyRegionEnum,
}
impl QIndex {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    self.region.clone().exec(self)
  }
}
impl SkyRegionProcess for QIndex {
  type Output = ();
  type Error = Box<dyn Error>;

  fn exec<S: SkyRegion>(self, region: S) -> Result<Self::Output, Self::Error> {
    let dest_file = File::create(self.output)?;
    let write = BufWriter::new(dest_file);
    qidx(self.input, region, self.limit, write).map_err(|e| e.into())
  }
}

#[derive(Debug, Clone, Subcommand)]
pub enum SkyRegionEnum {
  #[clap(name = "cone")]
  /// Retrieve data in the given cone
  Cone {
    /// Longitude of the cone center (in degrees)
    lon_deg: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude of the cone center (in degrees)
    lat_deg: f64,
    /// Radius of the cone (in degrees)
    r_deg: f64,
  },
  #[clap(name = "ellipse")]
  /// Retrieve data in the given elliptical cone
  EllipticalCone {
    /// Longitude of the elliptical cone center (in degrees)
    lon_deg: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude of the elliptical cone center (in degrees)
    lat_deg: f64,
    /// Elliptical cone semi-major axis (in degrees)
    a_deg: f64,
    /// Elliptical cone semi-minor axis (in degrees)
    b_deg: f64,
    /// Elliptical cone position angle (in degrees)
    pa_deg: f64,
  },
  #[clap(name = "ring")]
  /// Retrieve data in the given ring
  Ring {
    /// Longitude of the ring center (in degrees)
    lon_deg: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude of the ring center (in degrees)
    lat_deg: f64,
    /// Internal radius of the ring (in degrees)
    r_min_deg: f64,
    /// External radius of the ring (in degrees)
    r_max_deg: f64,
  },
  #[clap(name = "jname")]
  /// Retrieve data in a zone defined by the given JNAME
  JName {
    // transform into a zone!
    /// JNAME defining a zone
    jname: String,
    #[clap(short = 'e', long)]
    /// Extra margin, in degrees (e.g. if JNAME computed from %10.6f formatted position, use 1e-6)
    epsilon: Option<f64>,
    #[clap(short = 'r', long)]
    /// JNAME incorrectly computed by rounding instead of truncating positions
    round: bool,
  },
  #[clap(name = "zone")]
  /// Retrieve data in the given zone
  Zone {
    /// Longitude min, in degrees
    lon_deg_min: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude min, in degrees
    lat_deg_min: f64,
    /// Longitude max, in degrees
    lon_deg_max: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude max, in degrees
    lat_deg_max: f64,
  },
  #[clap(name = "box")]
  /// Retrieve data in the given box
  Box {
    // transform into a polygon!
    /// Longitude of the box center, in degrees
    lon_deg: f64,
    #[clap(allow_hyphen_values = true)]
    /// Latitude of the box center, in degrees
    lat_deg: f64,
    /// Semi-major axis of the box, in degrees
    a_deg: f64,
    /// Semi-minor axis of the box, in degrees
    b_deg: f64,
    /// Position angle of the box, in degrees
    pa_deg: f64,
  },
  #[clap(name = "polygon")]
  /// Retrieve data in the given polygon
  Polygon {
    /// List of vertices: "(lon,lat),(lon,lat),...,(lon,lat)" in degrees
    vertices_deg: Vertices, // (ra0,dec0),(ra1,dec1),...,(ran,decn)
    #[clap(short = 'c', long)]
    /// Gravity center of the polygon out of the polygon (in by default)
    complement: bool,
  },
  #[clap(name = "hpx")]
  /// Retrieve data in the given HEALPix cell
  Hpx {
    /// HEALPix depth
    depth: u8,
    /// HEALPix cell hash value (i.e. cell number at the given depth)
    hash: u64,
    #[clap(short = 'b', long = "add-ext-border")]
    /// Optionally add the external border of given depth
    external_border_depth: Option<u8>,
  },
  #[clap(name = "hpxrange")]
  /// Retrieve data in the given HEALPix range
  HpxRange {
    /// HEALPix depth
    depth: u8,
    /// HEALPix range (i.e. from cell (inclusive) to cell (exclusive) at the given depth)
    range: Range,
  },
  #[clap(name = "hpxranges")]
  /// Retrieve data in the given HEALPix ranges (**MUST** be sorted and non-overlaping)
  HpxRanges {
    /// HEALPix depth
    depth: u8,
    /// HEALPix ranges (i.e. from cell (inclusive) to cell (exclusive) at the given depth)
    ranges: Ranges,
  },
  #[clap(name = "hpxmoc")]
  /// Retrieve data in the given HEALPix MOC (Process Substitution can be used, e.g. <(echo "2/4-8"))
  HpxMOC {
    #[clap(long = "moc-format")]
    /// Format of the input MOC ('ascii', 'json', 'fits') [default: guess from the file extension]
    moc_input_fmt: Option<MocInputFormat>,
    /// Path of the input MOC file
    moc_file_path: PathBuf,
  },
  #[clap(name = "multicone")]
  /// Retrieve data in the given, possibly overlapping, cones with no duplicates in output.
  MultiCone {
    /// Radius of the cone (in degrees)
    r_deg: f64,
    /// Path of the file containing the cones centres (one center per line, ra,dec in decimal degrees)
    file_path: PathBuf,
    #[clap(short = 's', long = "separator", default_value = " ")]
    /// Separator between both coordinates (default = ' ')
    separator: String,
    #[clap(short = 'p', long = "parallel")]
    /// Number of threads to be used to compute the cone elements, preparing for the query
    n_threads: Option<u16>,
  },
  #[clap(name = "stcs")]
  /// Retrieve data in the given STC-S region.
  Stcs {
    /// STC-S region
    stcs: String,
  },
  #[clap(name = "stcsfile")]
  /// Retrieve data in the STC-S region describe in the given file.
  StcsFile {
    /// Path of the file containing the STC-S region
    file_path: PathBuf,
  },
}
impl SkyRegionEnum {
  pub fn exec<P>(self, process: P) -> Result<(), Box<dyn Error>>
  where
    P: SkyRegionProcess<Output = (), Error = Box<dyn Error>>,
  {
    match self {
      Self::Cone {
        lon_deg,
        lat_deg,
        r_deg,
      } => {
        let lon = lon_deg2rad(lon_deg)?;
        let lat = lat_deg2rad(lat_deg)?;
        let r = r_deg.to_radians();
        if r <= 0.0 || PI <= r {
          Err("Radius must be in ]0, pi[".to_string().into())
        } else {
          process.exec(Cone::new(lon, lat, r))
        }
      }
      Self::EllipticalCone {
        lon_deg,
        lat_deg,
        a_deg,
        b_deg,
        pa_deg,
      } => {
        let lon = lon_deg2rad(lon_deg)?;
        let lat = lat_deg2rad(lat_deg)?;
        let a = a_deg.to_radians();
        let b = b_deg.to_radians();
        let pa = pa_deg.to_radians();
        if a <= 0.0 || HALF_PI <= a {
          Err("Semi-major axis must be in ]0, pi/2]".to_string().into())
        } else if b <= 0.0 || a <= b {
          Err("Semi-minor axis must be in ]0, a[".to_string().into())
        } else if pa <= 0.0 || PI <= pa {
          Err("Position angle must be in [0, pi[".to_string().into())
        } else {
          process.exec(EllipticalCone::new(lon, lat, a, b, pa))
        }
      }
      Self::Ring {
        lon_deg,
        lat_deg,
        r_min_deg,
        r_max_deg,
      } => {
        let lon = lon_deg2rad(lon_deg)?;
        let lat = lat_deg2rad(lat_deg)?;
        let r_min = r_min_deg.to_radians();
        let r_max = r_max_deg.to_radians();
        if r_min <= 0.0 || r_max <= 0.0 || PI <= r_min || PI <= r_max {
          Err("Radius must be in ]0, pi[".to_string().into())
        } else {
          process.exec(Ring::new(lon, lat, r_min, r_max))
        }
      }
      Self::JName {
        jname,
        epsilon,
        round,
      } => process.exec(Zone::from_jname(jname.as_str(), round, &epsilon)?),
      Self::Zone {
        lon_deg_min,
        lat_deg_min,
        lon_deg_max,
        lat_deg_max,
      } => {
        let lon_min = lon_deg2rad(lon_deg_min)?;
        let lat_min = lat_deg2rad(lat_deg_min)?;
        let lon_max = lon_deg2rad(lon_deg_max)?;
        let lat_max = lat_deg2rad(lat_deg_max)?;
        process.exec(Zone::new(lon_min, lat_min, lon_max, lat_max))
      }
      Self::Box {
        lon_deg,
        lat_deg,
        a_deg,
        b_deg,
        pa_deg,
      } => {
        let lon = lon_deg2rad(lon_deg)?;
        let lat = lat_deg2rad(lat_deg)?;
        let a = a_deg.to_radians();
        let b = b_deg.to_radians();
        let pa = pa_deg.to_radians();
        if a <= 0.0 || HALF_PI <= a {
          Err("Semi-major axis must be in ]0, pi/2]".to_string().into())
        } else if b <= 0.0 || a <= b {
          Err("Semi-minor axis must be in ]0, a[".to_string().into())
        } else if pa <= 0.0 || HALF_PI <= pa {
          Err("Position angle must be in [0, pi[".to_string().into())
        } else {
          process.exec(Polygon::from_box(lon, lat, a, b, pa))
        }
      }
      Self::Polygon {
        vertices_deg,
        complement,
      } => process.exec(Polygon::new(
        vertices_deg
          .list
          .iter()
          .map(|(lon_deg, lat_deg)| {
            let lon = lon_deg2rad(*lon_deg)?;
            let lat = lat_deg2rad(*lat_deg)?;
            Ok((lon, lat))
          })
          .collect::<Result<Vec<(f64, f64)>, String>>()?,
        complement,
      )),
      Self::Hpx {
        depth,
        hash,
        external_border_depth,
      } => match external_border_depth {
        None => process.exec(HpxCell::new(depth, hash)),
        Some(external_border_depth) => process.exec(HpxRanges::from_cell_with_border(
          depth,
          hash,
          external_border_depth,
        )),
      },
      Self::HpxRange { depth, range } => process.exec(HpxRange::new(depth, range.to_range())),
      Self::HpxRanges { depth, ranges } => process.exec(HpxRanges::new(depth, ranges.to_ranges())),
      Self::HpxMOC {
        moc_input_fmt,
        moc_file_path,
      } => {
        // Load MOC
        let moc_input_fmt = match moc_input_fmt {
          Some(moc_input_fmt) => moc_input_fmt.clone(),
          None => moc_fmt_from_extension(&moc_file_path)?,
        };
        let moc: RangeMOC<u64, Hpx<u64>> = match moc_input_fmt {
          MocInputFormat::Ascii => fs::read_to_string(&moc_file_path)
            .map_err(|e| format!("Error opening file '{:?}': {:?}", moc_file_path, e))
            .and_then(|s| {
              from_ascii_ivoa::<u64, Hpx<u64>>(s.as_str())
                .map_err(|e| e.to_string())
                .map(|cellcellranges| {
                  cellcellranges
                    .into_cellcellrange_moc_iter()
                    .ranges()
                    .into_range_moc()
                })
            }),
          MocInputFormat::Json => fs::read_to_string(&moc_file_path)
            .map_err(|e| format!("Error opening file '{:?}': {:?}", moc_file_path, e))
            .and_then(|s| {
              from_json_aladin::<u64, Hpx<u64>>(s.as_str())
                .map_err(|e| e.to_string())
                .map(|cellrange| cellrange.into_cell_moc_iter().ranges().into_range_moc())
            }),
          MocInputFormat::Fits => {
            fn from_fits_hpx<T: Idx, R: BufRead>(
              moc: MocType<T, Hpx<T>, R>,
            ) -> RangeMOC<u64, Hpx<u64>> {
              match moc {
                MocType::Ranges(moc) => {
                  convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc).into_range_moc()
                }
                MocType::Cells(moc) => {
                  convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc.into_cell_moc_iter().ranges())
                    .into_range_moc()
                }
              }
            }
            fn smoc_from_fits_gen<T: Idx, R: BufRead>(
              moc: MocQtyType<T, R>,
            ) -> Result<RangeMOC<u64, Hpx<u64>>, Box<dyn Error>> {
              match moc {
                MocQtyType::Hpx(moc) => Ok(from_fits_hpx(moc)),
                _ => {
                  Err(String::from("Wrong MOC type. Expected: S-MOCs. Actual: Not S-MOC").into())
                }
              }
            }
            let file = File::open(&moc_file_path)
              .map_err(|e| format!("Error opening file '{:?}': {:?}", moc_file_path, e))?;
            let reader = BufReader::new(file);
            from_fits_ivoa(reader)
              .map_err(|e| e.to_string())
              .and_then(|moc| {
                match moc {
                  MocIdxType::U16(moc) => smoc_from_fits_gen(moc),
                  MocIdxType::U32(moc) => smoc_from_fits_gen(moc),
                  MocIdxType::U64(moc) => smoc_from_fits_gen(moc),
                }
                .map_err(|e| e.to_string())
              })
          }
        }?;
        // Transform ranges into ranges at the MOC depth
        let depth = moc.depth_max();
        let shift = Hpx::<u64>::shift_from_depth_max(depth);
        let ranges: Vec<ops::Range<u64>> = moc
          .into_range_moc_iter()
          .map(|range| range.start >> shift..range.end >> shift)
          .collect();
        process.exec(HpxRanges::new(depth, ranges))
      }
      Self::MultiCone {
        r_deg,
        file_path,
        separator,
        n_threads,
      } => {
        // Convert the radius in radians and fail is not in porper bounds
        let r = r_deg.to_radians();
        if r <= 0.0 || PI <= r {
          return Err("Radius must be in ]0, pi[".to_string().into());
        }
        // Define functions to parse and test position from input file
        fn line2coos(
          separator: &str,
          line: std::io::Result<String>,
        ) -> Result<Option<(f64, f64)>, String> {
          let line = line.map_err(|e| e.to_string())?;
          if line.trim().is_empty() || line.starts_with('#') {
            Ok(None)
          } else {
            let (lon_deg, lat_deg) = line
              .trim()
              .split_once(separator)
              .ok_or_else(|| format!("split on {} failed for row: {}.", separator, &line))?;
            let lon_deg = lon_deg.parse::<f64>().map_err(|e| e.to_string())?;
            let lat_deg = lat_deg.parse::<f64>().map_err(|e| e.to_string())?;
            let lon = lon_deg2rad(lon_deg)?;
            let lat = lat_deg2rad(lat_deg)?;
            Ok(Some((lon, lat)))
          }
        }
        // Parse input file
        let f = File::open(&file_path)
          .map_err(|e| format!("Error opening file '{:?}': {:?}", file_path, e))?;
        let positions: Vec<(f64, f64)> = BufReader::new(f)
          .lines()
          .filter_map(move |line| line2coos(&separator, line).transpose())
          .collect::<Result<_, _>>()?;
        // Exec
        process.exec(MultiCone::new(positions, r, n_threads)?)
      }
      Self::Stcs { stcs } => {
        let stcs = Stcs::new(stcs.as_str()).map_err(|e| e.to_string())?;
        process.exec(stcs)
      }
      Self::StcsFile { file_path } => {
        let stcs_query = fs::read_to_string(&file_path)
          .map_err(|e| format!("Error reading file '{:?}': {:?}", file_path, e))
          .and_then(|stcs| Stcs::new(stcs.as_str()).map_err(|e| e.to_string()))?;
        process.exec(stcs_query)
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Vertices {
  // (ra0,dec0),(ra1,dec1),...,(ran,decn)
  list: Vec<(f64, f64)>,
}

impl FromStr for Vertices {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let list: Vec<f64> = s
      .replace(['(', ')'], "")
      .split(',')
      .map(|t| str::parse::<f64>(t.trim()))
      .collect::<Result<Vec<f64>, _>>()?;
    Ok(Vertices {
      list: list
        .iter()
        .step_by(2)
        .zip(list.iter().skip(1).step_by(2))
        .map(|(lon, lat)| (*lon, *lat))
        .collect(),
    })
  }
}

#[derive(Debug, Clone)]
pub struct Range {
  pub from: u64,
  pub n: u64,
}

impl Range {
  pub fn from_range(range: &ops::Range<u64>) -> Range {
    Range {
      from: range.start,
      n: range.end - range.start,
    }
  }

  pub fn to_range(&self) -> ops::Range<u64> {
    ops::Range {
      start: self.from,
      end: self.from + self.n,
    }
  }
}

impl FromStr for Range {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains("-") {
      let v = s
        .split("-")
        .map(|t| str::parse::<u64>(t.trim()))
        .collect::<Result<Vec<u64>, _>>()
        .map_err(|e| format!("Wrong range '{}'. Error: {:?}", s, e))?;
      if v.len() == 2 {
        let from = v[0];
        return Ok(Range {
          from,
          n: v[1] - from,
        });
      }
    } else if s.contains("+") {
      let v = s
        .split("-")
        .map(|t| str::parse::<u64>(t.trim()))
        .collect::<Result<Vec<u64>, _>>()
        .map_err(|e| format!("Wrong range '{}'. Error: {:?}", s, e))?;
      if v.len() == 2 {
        return Ok(Range {
          from: v[0],
          n: v[1],
        });
      }
    }
    Err(format!(
      "Wrong range '{}'. Expected: 'FROM-TO' or 'FROM+N'.",
      s
    ))
  }
}

#[derive(Debug, Clone)]
pub struct Ranges {
  // from-to,from-to from+n,...,from-to
  pub list: Vec<Range>,
}

impl Ranges {
  pub fn to_ranges(&self) -> Vec<ops::Range<u64>> {
    self
      .list
      .iter()
      .map(|r| ops::Range {
        start: r.from,
        end: r.from + r.n,
      })
      .collect()
  }
}

impl FromStr for Ranges {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let list: Vec<i64> = s
      .replace("-", ",-")
      .replace("+", ",+")
      .split(",")
      .map(|t| str::parse::<i64>(t.trim()))
      .collect::<Result<Vec<i64>, _>>()?;
    Ok(Ranges {
      list: list
        .iter()
        .step_by(2)
        .zip(list.iter().skip(1).step_by(2))
        .map(|(from, to_or_n)| {
          if *to_or_n < 0 {
            Range {
              from: *from as u64,
              n: (*to_or_n - *from) as u64,
            }
          } else {
            Range {
              from: *from as u64,
              n: *to_or_n as u64,
            }
          }
        })
        .collect(),
    })
  }
}

#[derive(Clone, Debug)]
pub enum MocInputFormat {
  Ascii,
  Json,
  Fits,
}
impl FromStr for MocInputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "ascii" | "Ascii" | "ASCII" => Ok(MocInputFormat::Ascii),
      "json" | "Json" | "JSON" => Ok(MocInputFormat::Json),
      "fits" | "Fits" | "FITS" => Ok(MocInputFormat::Fits),
      _ => Err(format!(
        "Unrecognized MOC format '{}'. Expected: 'ascii, 'json' or 'fits'",
        s
      )),
    }
  }
}

/// Guess the MOC file format from the extension.
pub fn moc_fmt_from_extension(path: &Path) -> Result<MocInputFormat, String> {
  match path.extension().and_then(|e| e.to_str()) {
    Some("fits") => Ok(MocInputFormat::Fits),
    Some("json") => Ok(MocInputFormat::Json),
    Some("ascii") | Some("txt") => Ok(MocInputFormat::Ascii),
    _ => Err(String::from(
      "Unable to guess the MOC format from the file extension, see options.",
    )),
  }
}

fn lon_deg2rad(lon_deg: f64) -> Result<f64, String> {
  let lon = lon_deg.to_radians();
  if !(0.0..TWICE_PI).contains(&lon) {
    Err(String::from("Longitude must be in [0, 2pi["))
  } else {
    Ok(lon)
  }
}

fn lat_deg2rad(lat_deg: f64) -> Result<f64, String> {
  let lat = lat_deg.to_radians();
  if !(-HALF_PI..=HALF_PI).contains(&lat) {
    Err(String::from("Latitude must be in [-pi/2, pi/2]"))
  } else {
    Ok(lat)
  }
}
