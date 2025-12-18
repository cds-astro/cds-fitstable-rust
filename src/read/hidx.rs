//! Module dedicated to the indexation of HEALPix sorted BINTABLE files.
use std::{
  fs::{self, File},
  io::BufWriter,
  io::{Seek, SeekFrom, Write},
  path::PathBuf,
};

use log::debug;
use memmap2::{Advice, MmapOptions};

use cdshealpix::nested::{
  get, n_hash,
  sort::cindex::{
    FITSCIndex, FitsMMappedCIndex, HCIndex, HCIndexShape, OwnedCIndex, OwnedCIndexExplicit,
  },
};
use skyregion::{SkyRegion, SkyRegionProcess};

use crate::{
  common::{ValueKwr, keywords::naxis::NAxis2},
  error::{Error, new_custom, new_io_err, new_parse_u16_err},
  hdu::{
    header::{HDUHeader, builder::r#impl::bintable::Bintable},
    xtension::bintable::schema::{RowSchema, Schema},
  },
  read::slice::FitsBytes,
};

// ADD https://github.com/cds-astro/cds-bstree-file-readonly-rust INDEX!

/// Create an HEALPix Cumulative Index.
pub fn hcidx(
  input: PathBuf,
  output: PathBuf,
  i_lon: usize,
  i_lat: usize,
  depth: u8,
  in_mem_explicit: bool,
  in_file_implicit_over_explicit_ratio: Option<f64>,
) -> Result<(), Error> {
  // Prepare reading, creating a memory map
  let file = File::open(&input).map_err(new_io_err)?;
  let mmap = unsafe { MmapOptions::new().map(&file) }.map_err(new_io_err)?;
  mmap.advise(Advice::Sequential).map_err(new_io_err)?;

  // Read as a FITS file, prepare iteration on HDUs
  let bytes = mmap.as_ref();
  let fits = FitsBytes::from_slice(bytes);
  let mut is_first_bintable = true;
  // Iterate on all HDUs
  for (i, hdu) in fits.new_iterator::<Bintable>().enumerate() {
    let hdu = hdu?;
    if hdu.is_bintable_hdu() {
      let index_path = if is_first_bintable {
        is_first_bintable = false;
        output.clone()
      } else {
        let mut path = output.clone();
        path.set_extension(format!(".{}.fits", i));
        path
      };

      // * read bintable metadata
      let bintable_header = match &hdu.parsed_header {
        HDUHeader::BinTable(h) => h,
        _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
      };
      let row_byte_size = bintable_header.row_byte_size();
      let n_rows = bintable_header.n_rows();
      let main_table_byte_size = bintable_header.main_table_byte_size();

      assert_eq!(main_table_byte_size, n_rows * row_byte_size);

      // * build the table schema
      let row_schema: RowSchema = bintable_header.build_row_schema();

      // * get RA and Dec columns info, and ensure they are of type Double (no scale/offset allowed here so far)
      let ra_meta = &row_schema.fields_schemas()[i_lon];
      let de_meta = &row_schema.fields_schemas()[i_lat];
      if !matches!(ra_meta.schema, Schema::Double) {
        return Err(new_custom(format!(
          "RA column is not a double. Header: {:?}",
          &bintable_header.cols()[i_lon]
        )));
      }
      if !matches!(de_meta.schema, Schema::Double) {
        return Err(new_custom(format!(
          "Dec column is not a double. Header: {:?}",
          &bintable_header.cols()[i_lat]
        )));
      }

      let layer = get(depth);
      // Position provider
      let hpx = move |row_bytes: &[u8]| {
        let lon = f64::from_be_bytes(
          row_bytes[ra_meta.starting_byte..ra_meta.starting_byte + 8]
            .try_into()
            .unwrap(),
        );
        let lat = f64::from_be_bytes(
          row_bytes[de_meta.starting_byte..de_meta.starting_byte + 8]
            .try_into()
            .unwrap(),
        );
        if lon.is_nan() || lat.is_nan() {
          0
        } else {
          layer.hash(lon.to_radians(), lat.to_radians())
        }
      };

      // Get row reader
      let rows = (&hdu.data()[..main_table_byte_size]).chunks(row_byte_size);

      let mut byte_offset = hdu.data_starting_byte();
      if in_mem_explicit {
        let len = n_hash(depth) + 1;
        let mut entries: Vec<(u64, u64)> = Vec::with_capacity(len.min(1_000_000) as usize);
        // Read line by line
        let mut prev_icell = 0;
        let mut irow = 0;
        for row in rows {
          let icell = hpx(row);
          if icell + 1 < prev_icell {
            return Err(new_custom(format!(
              "HEALPix error at row {}: the file seems not to be sorted!",
              irow
            )));
          }
          // Push only the starting byte of the first row having a given cell number.
          if icell > prev_icell || (icell == 0 && entries.is_empty()) {
            entries.push((icell, byte_offset as u64));
          }
          byte_offset += row_byte_size;
          irow += 1;
          prev_icell = icell;
        }
        entries.push((prev_icell + 1, byte_offset as u64));

        // Write the cumulative map
        let explicit_index = OwnedCIndexExplicit::new_unchecked(depth, entries);
        write_index(
          &input,
          &index_path,
          i_lon,
          i_lat,
          in_mem_explicit,
          in_file_implicit_over_explicit_ratio,
          explicit_index,
        )?;
      } else {
        // Prepare building the map
        let len = n_hash(depth) + 1;
        let mut map: Vec<u64> = Vec::with_capacity(len as usize);
        // Read line by line
        let mut i_row = 0;
        for row in rows {
          let icell = hpx(row);
          if icell + 1 < map.len() as u64 {
            return Err(new_custom(format!(
              "HEALPix error at row {}: the file seems not to be sorted!",
              i_row
            )));
          }
          // Push only the starting byte of the first row having a given cell number.
          // Copy the value for all empty cells between two non-empty cells.
          for _ in map.len() as u64..=icell {
            //info!("Push row: {}; bytes: {:?}", irow, &byte_range);
            map.push(byte_offset as u64);
          }
          // Push only the starting byte of the first row having a given cell number.
          byte_offset += row_byte_size;
          i_row += 1;
        }
        // Complete the map if necessary
        for _ in map.len() as u64..len {
          map.push(byte_offset as u64);
        }

        // Write the cumulative map
        let implicit_index = OwnedCIndex::new_unchecked(depth, map.into_boxed_slice());
        write_index(
          &input,
          &index_path,
          i_lon,
          i_lat,
          in_mem_explicit,
          in_file_implicit_over_explicit_ratio,
          implicit_index,
        )?;
      }
    }
  }
  Ok(())
}

fn write_index<H: HCIndex>(
  input: &PathBuf,
  output: &PathBuf,
  lon: usize,
  lat: usize,
  in_mem_explicit: bool,
  implicit_over_explicit_ratio: Option<f64>,
  cindex: H,
) -> Result<(), Error> {
  // Prepare output
  let fits_file = File::create(output).map_err(new_io_err)?;
  let out_fits_write = BufWriter::new(fits_file);
  let file_metadata = input.metadata().ok();
  let best_repr = implicit_over_explicit_ratio
    .map(|ratio| cindex.best_representation(ratio))
    .unwrap_or(if in_mem_explicit {
      HCIndexShape::Explicit
    } else {
      HCIndexShape::Implicit
    });
  match best_repr {
    HCIndexShape::Implicit => cindex.to_fits_implicit(
      out_fits_write,
      input.file_name().and_then(|name| name.to_str()),
      file_metadata.as_ref().map(|meta| meta.len()),
      None, // So far we do not compute the md5 of the VOTable!
      file_metadata.as_ref().and_then(|meta| meta.modified().ok()),
      Some(format!("#{}", lon).as_str()),
      Some(format!("#{}", lat).as_str()),
    ),
    HCIndexShape::Explicit => cindex.to_fits_explicit(
      out_fits_write,
      input.file_name().and_then(|name| name.to_str()),
      file_metadata.as_ref().map(|meta| meta.len()),
      None, // So far we do not compute the md5 of the VOTable!
      file_metadata.as_ref().and_then(|meta| meta.modified().ok()),
      Some(format!("#{}", lon).as_str()),
      Some(format!("#{}", lat).as_str()),
    ),
  }
  .map_err(|e| new_custom(e.to_string()))
}

/// Query an index created with the method `mkidx`.
// TODO: code redundant with `healpix-cli` file `qhcidx.rs`, to be put in `healpix-lib`!
pub fn qidx<S, W>(idx_file: PathBuf, region: S, limit: Option<usize>, write: W) -> Result<(), Error>
where
  S: SkyRegion,
  W: Write + Seek,
{
  match FITSCIndex::from_fits_file(idx_file).map_err(|e| new_custom(format!("{}", e)))? {
    FITSCIndex::ImplicitU64(fits_hci) => QIdxProcess::new(&fits_hci, write, limit).exec(region),
    FITSCIndex::ExplicitU32U64(fits_hci) => QIdxProcess::new(&fits_hci, write, limit).exec(region),
    FITSCIndex::ExplicitU64U64(fits_hci) => QIdxProcess::new(&fits_hci, write, limit).exec(region),
    _ => Err(new_custom(String::from(
      "Wrong data type in the FITS Healpix Cumulative Index type. Expected: u64.",
    ))),
  }
}

fn check_file_exists_and_check_file_len(
  file_name: &String,
  expected_csv_len: u64,
) -> Result<(), Error> {
  // Check if file exists
  fs::exists(file_name)
    .map_err(new_io_err)
    .and_then(|exists| {
      if exists {
        Ok::<(), Error>(())
      } else {
        Err(new_custom(format!(
          "File `{}` not found in the current directory.",
          file_name
        )))
      }
    })?;
  // Check file len
  let actual_csv_len = fs::metadata(file_name)
    .map(|metadata| metadata.len())
    .map_err(new_io_err)?;
  if actual_csv_len != expected_csv_len {
    Err(new_custom(format!(
      "Local FITS file `{}` len does not match index info. Expected: {}. Actual: {}.",
      file_name, expected_csv_len, actual_csv_len
    )))
  } else {
    Ok(())
  }
}

struct QIdxProcess<'a, H, T, W>
where
  H: HCIndex<V = u64>,
  T: FitsMMappedCIndex<'a, HCIndexType = H> + 'a,
  W: Write + Seek,
{
  fits_idx: &'a T,
  write: W,
  /// Maximum number of output rows (to avoid too large in memory files).
  /// For "unlimited", set to the number of rows in the file.
  limit: Option<usize>,
}
impl<'a, H, T, W> QIdxProcess<'a, H, T, W>
where
  H: HCIndex<V = u64>,
  T: FitsMMappedCIndex<'a, HCIndexType = H> + 'a,
  W: Write + Seek,
{
  pub fn new(fits_idx: &'a T, write: W, limit: Option<usize>) -> Self {
    Self {
      fits_idx,
      write,
      limit,
    }
  }
}

impl<'a, H, T, W> SkyRegionProcess for QIdxProcess<'a, H, T, W>
where
  H: HCIndex<V = u64>,
  T: FitsMMappedCIndex<'a, HCIndexType = H> + 'a,
  W: Write + Seek,
{
  type Output = ();
  type Error = Error;

  fn exec<S: SkyRegion>(mut self, region: S) -> Result<Self::Output, Self::Error> {
    let file_name = self
      .fits_idx
      .get_indexed_file_name()
      .ok_or_else(|| new_custom("No file name found in the FITS HCI file."))?;
    let expected_file_len = self
      .fits_idx
      .get_indexed_file_len()
      .ok_or_else(|| new_custom("No file length found in the FITS HCI file."))?;
    check_file_exists_and_check_file_len(file_name, expected_file_len)?;
    let lon = self
      .fits_idx
      .get_indexed_colname_lon()
      .ok_or_else(|| new_custom("No longitude column index found in the FITS HCI file."))
      .and_then(|s| {
        s.strip_prefix('#')
          .ok_or_else(|| new_custom(format!("{} does not starts with '#'", s)))
      })
      .and_then(|s| s.parse::<usize>().map_err(new_parse_u16_err))?;
    let lat = self
      .fits_idx
      .get_indexed_colname_lat()
      .ok_or_else(|| new_custom("No latitude column index found in the FITS HCI file."))
      .and_then(|s| {
        s.strip_prefix('#')
          .ok_or_else(|| new_custom(format!("{} does not starts with '#'", s)))
      })
      .and_then(|s| s.parse::<usize>().map_err(new_parse_u16_err))?;

    // Ok, load index data...
    let hci = self.fits_idx.get_hcindex();
    let first_byte = hci.get(0);
    debug!("First row starting byte: {}", first_byte);

    // Load FITS file data
    let file = File::open(file_name).map_err(new_io_err)?;
    // Prepare reading, creating a memory map
    let mmap = unsafe { MmapOptions::new().map(&file) }.map_err(new_io_err)?;
    mmap.advise(Advice::Sequential).map_err(new_io_err)?;
    // Read as a FITS file, prepare iteration on HDUs
    let bytes = mmap.as_ref();
    let fits = FitsBytes::from_slice(bytes);
    let mut hdu_it = fits.new_iterator::<Bintable>();
    if let Some(Ok(hdu)) = hdu_it.next() {
      // Copy PrimaryHDU
      hdu.copy_hdu(&mut self.write)?;
    }

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
        // Parse header and get lon/lat column indices.
        // * read bintable metadata
        let bintable_header = match &hdu.parsed_header {
          HDUHeader::BinTable(h) => h,
          _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
        };
        let bintable_header_starting_byte = hdu.starting_byte;
        let row_byte_size = bintable_header.row_byte_size();
        // * build the table schema
        let row_schema: RowSchema = bintable_header.build_row_schema();
        // * get RA and Dec columns info, and ensure they are of type Double (no scale/offset allowed here so far)
        let lon_meta = &row_schema.fields_schemas()[lon];
        let lat_meta = &row_schema.fields_schemas()[lat];
        if !matches!(lon_meta.schema, Schema::Double) {
          return Err(new_custom(format!(
            "RA column is not a double. Header: {:?}",
            &bintable_header.cols()[lon]
          )));
        }
        if !matches!(lat_meta.schema, Schema::Double) {
          return Err(new_custom(format!(
            "Dec column is not a double. Header: {:?}",
            &bintable_header.cols()[lat]
          )));
        }

        debug!("Bintable data starting byte: {}", hdu.data_starting_byte());

        let mut limit = self.limit.unwrap_or(bintable_header.n_rows());

        hdu.copy_header(&mut self.write)?;
        let mut n_data_bytes_written = 0_usize;
        for (range, flag) in region.sorted_hpx_ranges(hci.depth()) {
          let bytes_range = hci.get_with_range_at_index_depth(range);
          let bytes_range = bytes_range.start as usize..bytes_range.end as usize;
          if flag {
            let n_rows = n_data_bytes_written / row_byte_size;
            if n_rows < limit {
              self
                .write
                .write_all(&mmap[bytes_range.start..bytes_range.end])
                .map_err(new_io_err)?;
              limit -= n_rows;
              n_data_bytes_written += bytes_range.end - bytes_range.start;
            } else {
              self
                .write
                .write_all(&mmap[bytes_range.start..bytes_range.start + limit * row_byte_size])
                .map_err(new_io_err)?;
              limit = 0;
              n_data_bytes_written += limit * row_byte_size;
            }
          } else {
            for row in mmap[bytes_range].chunks(row_byte_size) {
              let lon = f64::from_be_bytes(
                row[lon_meta.starting_byte..lon_meta.starting_byte + 8]
                  .try_into()
                  .unwrap(),
              );
              let lat = f64::from_be_bytes(
                row[lat_meta.starting_byte..lat_meta.starting_byte + 8]
                  .try_into()
                  .unwrap(),
              );
              if limit > 0
                && !lon.is_nan()
                && !lat.is_nan()
                && region.contains(lon.to_radians(), lat.to_radians())
              {
                self.write.write_all(row).map_err(new_io_err)?;
                limit -= 1;
                n_data_bytes_written += row_byte_size;
              }
            }
          }
        }

        debug!(
          "Main table number of written bytes: {}",
          n_data_bytes_written
        );

        // Complete bytes if necessary
        if n_data_bytes_written % 2880 != 0 {
          debug!(
            "Pad data adding {} bytes",
            2880 - n_data_bytes_written % 2880
          );

          self
            .write
            .write_all(vec![0_u8; 2880 - n_data_bytes_written % 2880].as_slice())
            .map_err(new_io_err)?;
        }
        // Overwrite number of rows.
        let mut naxis2 = [0_u8; 80];
        self
          .write
          .seek(SeekFrom::Start(
            bintable_header_starting_byte as u64 + 4 * 80,
          ))
          .map_err(new_io_err)?;
        NAxis2::new((n_data_bytes_written / row_byte_size) as u64)
          .write_kw_record(&mut std::iter::once(Ok(&mut naxis2)))?;
        debug!("Rewrite NAXIS2: {}", String::from_utf8_lossy(&naxis2));
        self
          .write
          .write_all(naxis2.as_slice())
          // .and_then(|()| self.write.flush())
          .map_err(new_io_err)
      }
      None => Err(new_custom(format!(
        "No HDU with data starting at byte offset {}",
        first_byte
      ))),
    }
  }
}
