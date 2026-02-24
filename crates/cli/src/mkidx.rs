use std::{error::Error, path::PathBuf};

use clap::Args;

use fitstable::read::hidx::hcidx;

/// Make an index on an HEALPix NESTED sorted BINTABLE FITS file,
/// to then quickly retrieve rows in a given HEALPix cell.
#[derive(Debug, Args)]
pub struct MkIndex {
  /// Path of the FITS file to be indexed
  #[clap(value_name = "FILE")]
  input: PathBuf,
  /// Path of the output FITS file containing the HEALPix Cumulative Index.
  #[clap(value_name = "FILE")]
  output: PathBuf,
  /// Field number of the longitude(in degrees) used to compute the HEALPix number, starting from 1.
  #[clap(short = 'l', long, value_name = "FIELD")]
  lon: usize,
  /// Field number of the latitude (in degrees) used to compute the HEALPix number, starting from 1.
  #[clap(short = 'b', long, value_name = "FIELD")]
  lat: usize,
  /// Depth of the HEALPix cumulative index (around 6 to 10, then output file will be large).
  #[arg(short, long, default_value_t = 9_u8)]
  depth: u8,
  #[arg(short = 'e', long)]
  /// Use in-memory `explicit` representation instead of `implicit` (for table covering a small fraction of the sky).
  explicit: bool,
  #[arg(short = 'r', long = "ratio")]
  /// Limit on the ratio of the implicit over the explicit byte sizes for the FITS serialisation.
  /// Above the limit, the explicit representation is chosen.
  /// Below the limit, the implicit representation is chosen.
  /// If unset, use the in-memory representation.
  implicit_over_explicit_ratio: Option<f64>,
}

impl MkIndex {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    hcidx(
      self.input,
      self.output,
      self.lon - 1,
      self.lat - 1,
      self.depth,
      self.explicit,
      self.implicit_over_explicit_ratio,
    )
    .map_err(|e| e.into())
  }
}
