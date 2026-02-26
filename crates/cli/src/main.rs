extern crate fitstable_cli;

#[cfg(feature = "cgi")]
use std::collections::HashMap;
use std::error::Error;

use clap::Parser;
#[cfg(feature = "cgi")]
use http::{method::Method, status::StatusCode as Status};
#[cfg(feature = "cgi")]
use serde_qs as qs;

#[cfg(feature = "cgi")]
use fitstable_cli::qhips::Action;
use fitstable_cli::{
  csv::Csv, head::Head, info::Info, mkhips::MkHiPS, mkidx::MkIndex, qhips::QHips, qidx::QIndex,
  sort::Sort, r#struct::Struct,
};

// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(target_env = "musl", target_arch = "x86_64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Perform FITS file related operations on the command line.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
  /// Read and print the structure of a FITS file
  #[clap(name = "struct")]
  Struct(Struct),
  /// Read and print the headers of all the HDU in a FITS file
  #[clap(name = "head")]
  Head(Head),
  /// Print tables information (such as column names, units, ...)
  #[clap(name = "info")]
  Info(Info),
  /// Print tables in CSV format.
  #[clap(name = "csv")]
  Csv(Csv),
  /// Sort a file, or sort and concatenate a set of files, according to HEALPix
  #[clap(name = "sort")]
  Sort(Sort),
  /// Make a positional index for HEALPix sorted files
  #[clap(name = "mkidx")]
  MkIndex(MkIndex),
  /// Query a BINTABLE using to a HEALPix index
  #[clap(name = "qidx")]
  QIndex(QIndex),
  // Add Mk and Q bstree index?
  /// Create a HiPS catalogue from a HEALPix sorted and index BINTABLE
  #[clap(name = "mkhips")]
  MkHips(MkHiPS),
  /// Query a HiPS catalogue
  #[clap(name = "qhips")]
  QHips(QHips),
}

impl Args {
  fn exec(self) -> Result<(), Box<dyn Error>> {
    match self {
      Self::Struct(args) => args.exec(),
      Self::Head(args) => args.exec(),
      Self::Info(args) => args.exec(),
      Self::Csv(args) => args.exec(),
      Self::Sort(args) => args.exec(),
      Self::MkIndex(args) => args.exec(),
      Self::QIndex(args) => args.exec(),
      Self::MkHips(args) => args.exec(),
      Self::QHips(args) => args.exec(false),
    }
  }
}

fn exec_cli() -> Result<(), Box<dyn Error>> {
  env_logger::init();
  let args = Args::parse();
  match args.exec() {
    Ok(()) => Ok(()),
    Err(e) => {
      eprintln!("Error: {}", e);
      Err(e)
    }
  }
}

/// Only for 'QHips'
#[cfg(feature = "cgi")]
fn exec_cgi(env_vars: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
  let qhips = args_from_cgi(env_vars)?;
  qhips.exec(true)
}

/// Retrieve CGI arguments from the given map of environment variables.
#[cfg(feature = "cgi")]
fn args_from_cgi(env_vars: HashMap<String, String>) -> Result<QHips, Box<dyn Error>> {
  match &env_vars["REQUEST_METHOD"].parse::<Method>() {
    Ok(Method::GET) => args_from_cgi_get(env_vars),
    Ok(method) => {
      let error_msg = format!("Method {} not supported.", method);
      println!("Status: {}\n\n{}", Status::NOT_IMPLEMENTED, error_msg);
      Err(error_msg.into())
    }
    Err(_) => {
      let error_msg = String::from("Invalid method.");
      println!("Status: {}\n\n{}", Status::BAD_REQUEST, error_msg);
      Err(error_msg.into())
    }
  }
}

/// Retrieve CGI arguments from the given map of environment variables, assuming GET query.
#[cfg(feature = "cgi")]
fn args_from_cgi_get(env_vars: HashMap<String, String>) -> Result<QHips, Box<dyn Error>> {
  qs::from_str::<QHips>(&env_vars["QUERY_STRING"]).or_else(|e| args_from_cgi_path_info(e, env_vars))

  // In MyDir
  // > python -m http.server --cgi 8000
  //
  // /MyDir
  //  ├── path/to/hips
  //  ├── cgi-bin/
  //  │   └── hips.cgi
  //
  // http://localhost:8000/cgi-bin/hips.cgi/path/to/hips/index.html
}

#[cfg(feature = "cgi")]
fn args_from_cgi_path_info(
  qs_err: qs::Error,
  env_vars: HashMap<String, String>,
) -> Result<QHips, Box<dyn Error>> {
  match (&env_vars["PATH_INFO"]).as_str() {
    "" => {
      let error_msg = format!("No PATH_INFO and wrong QUERY_STRING: {} ", qs_err);
      println!("Status: {}\n\n{}", Status::BAD_REQUEST, &error_msg);
      Err(error_msg.into())
    }
    path => {
      fn parse_depth(norder: &str) -> Result<u8, Box<dyn Error>> {
        norder
          .strip_prefix("Norder")
          .expect("No Norder found!")
          .parse::<u8>()
          .map_err(|e| {
            let error_msg = format!("Unable to parse Norder in {}: {} ", norder, e);
            println!("Status: {}\n\n{}", Status::BAD_REQUEST, error_msg);
            error_msg.into()
          })
      }
      // check if path is from python server dir or from cgi-bin dir...
      let (dir, action) = path.rsplit_once('/').unwrap_or_else(|| ("", path));
      let dir = dir.strip_prefix('/').unwrap_or(dir);
      match action {
        "" | "index.html" | "info" => Ok(QHips::new(dir, Action::IndexHTML)),
        "Properties" | "properties" => Ok(QHips::new(dir, Action::Properties)),
        "Metadata.xml" | "metadata.xml" => Ok(QHips::new(dir, Action::Metadata)),
        "Moc.fits" | "moc.fits" => Ok(QHips::new(dir, Action::Moc)),
        "Tiles.csv" | "tiles.csv" => Ok(QHips::new(dir, Action::TileList)), // Not in the standard
        "Allsky1.tsv" | "allsky1.tsv" => Ok(QHips::new(dir, Action::Allsky { depth: 1 })),
        "Allsky2.tsv" | "allsky2.tsv" => Ok(QHips::new(dir, Action::Allsky { depth: 2 })),
        "Allsky.tsv" | "allsky.tsv" => {
          let (dir, norder) = dir.rsplit_once('/').unwrap_or_else(|| ("", dir));
          parse_depth(norder).map(|depth| QHips::new(dir, Action::Allsky { depth }))
        }
        tile_name if action.ends_with(".tsv") => {
          let hash = tile_name
            .strip_prefix("Npix")
            .expect("No Npix found!")
            .strip_suffix(".tsv")
            .expect("No .tsv found!")
            .parse::<u64>()
            .map_err(|e| {
              let error_msg = format!("Unable to parse Npix in {}: {} ", tile_name, e);
              println!("Status: {}\n\n{}", Status::BAD_REQUEST, error_msg);
              error_msg
            })?;
          let (dir, _tile_dir) = dir.rsplit_once('/').unwrap_or_else(|| ("", dir));
          // TODO: ensure tile_dir.start_with("Dir"); ?
          let (dir, norder) = dir.rsplit_once('/').unwrap_or_else(|| ("", dir));
          parse_depth(norder).map(|depth| QHips::new(dir, Action::Tile { depth, hash }))
        }
        unknown => {
          /*let error_msg = format!("Action \"{}\" not recognized.", unknown);
          println!("Status: {}\n\n{}", Status::BAD_REQUEST, error_msg);
          Err(error_msg.into())*/
          let dir = format!("{}/{}", dir, unknown);
          Ok(QHips::new(&dir, Action::IndexHTML))
        }
      }
    }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  #[cfg(feature = "cgi")]
  {
    let env_vars: HashMap<String, String> = std::env::vars().collect();
    if env_vars.get("REQUEST_METHOD").is_none() {
      exec_cli()
    } else {
      exec_cgi(env_vars)
    }
  }
  #[cfg(not(feature = "cgi"))]
  exec_cli()
}
