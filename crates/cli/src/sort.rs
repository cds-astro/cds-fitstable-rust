use std::{error::Error, path::PathBuf};

use clap::Args;

use fitstable::read::hsort::hsort;

/// Sorts a file (or sort and concatenate a list fo fles) by order 29 HEALPix NESTED indices,
/// uses external sort to support huge files.
#[derive(Debug, Clone, Args)]
pub struct Sort {
  /// Input file or directory containing FITS files
  #[clap(value_name = "FILE")]
  input: PathBuf,
  /// Field number of the longitude(in degrees) used to compute the HEALPix number, starting from 1.
  #[clap(short = 'l', long, value_name = "FIELD")]
  lon: usize,
  /// Field number of the latitude (in degrees) used to compute the HEALPix number, starting from 1.
  #[clap(short = 'b', long, value_name = "FIELD")]
  lat: usize,
  /// Path of the output file
  #[clap(value_name = "FILE")]
  output: PathBuf,
  /// Set the number of threads used [default: use all available threads]
  #[arg(long, value_name = "N")]
  parallel: Option<usize>,
  /*/// Do not use external sort. Faster, but use only with table holding in RAM.
  #[arg(short = 'f', long = "full-in-mem")]
  fully_in_memory: bool,*/
  /// Directory containing the temporary directories/files for external sort.
  #[arg(long, default_value = ".sort_tmp/")]
  tmp_dir: PathBuf,
  /// Size, in bytes, per external sort chunk (2 chunks are simultaneously loaded in memory, if the table
  /// is smaller than the chunk_size, an internal sort is performed).
  #[arg(long, default_value_t = 209_715_200_usize)]
  chunk_size: usize,
  /// Depth of the computed HEALPix count map for the external sort. Should be deep enough so that
  /// the largest count map value is smaller than `chunk-size`.
  #[arg(short = 'd', long, default_value_t = 9_u8)]
  depth: u8,
  /*/// Save the computed count map in the given FITS file path.
  #[arg(long)]
  count_map_path: Option<PathBuf>,*/
}

impl Sort {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    hsort(
      self.input,
      self.lon - 1,
      self.lat - 1,
      self.output,
      self.chunk_size,
      self.depth,
      Some(self.tmp_dir),
      self.parallel,
    )
  }
}
