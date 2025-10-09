//! Module dedicated to sorting FITS BINTABLEs according to an HEALPix index computed from
//! a pair of coordinates.

use std::{
  convert::Infallible,
  convert::TryInto,
  error::Error,
  fs::{File, read_dir},
  io::{BufWriter, Seek, SeekFrom, Write},
  path::PathBuf,
};

use log::{debug, warn};
use memmap2::{Advice, Mmap, MmapOptions};

use crate::{
  common::{ValueKwr, keywords::naxis::NAxis2},
  error::new_custom,
  hdu::{
    header::{HDUHeader, builder::r#impl::bintable::Bintable},
    xtension::bintable::schema::{RowSchema, Schema},
  },
  read::slice::FitsBytes,
};

use crate::error::new_io_err;
use cdshealpix::nested::{
  get,
  map::skymap::CountMapU32,
  sort::{
    SimpleExtSortParams, hpx_external_sort_stream, hpx_external_sort_with_knowledge,
    hpx_internal_sort,
  },
};

/// Creates a single FITS file containing an HEALPix sorted BINTABLE in the first extension.
/// In input, takes either the path of a FITS file or a directory containing FITS files
/// having the exact same structure (the file contents are concatenated).
/// * a single FITS file may contain additional HDUs (that are copied without modification)
/// * each FITS file of a directory **must** have a single extension (a BINTABLE)
pub fn hsort(
  input: PathBuf,
  i_ra: usize,
  i_dec: usize,
  output: PathBuf,
  internal_threshold: usize,
  depth: u8,
  tmp: Option<PathBuf>,
  parallel: Option<usize>,
) -> Result<(), Box<dyn Error>> {
  if input.is_file() {
    File::open(input).map_err(|e| e.into()).and_then(|file| {
      hsort_file(
        file,
        i_ra,
        i_dec,
        output,
        internal_threshold,
        depth,
        tmp,
        parallel,
      )
    })
  } else {
    hsort_files(
      input,
      i_ra,
      i_dec,
      output,
      internal_threshold,
      depth,
      tmp,
      parallel,
    )
  }
}

/// Copies the input file, sorting the first extension (a BINTABLE) according to HEALPix indices at order 29.
///
/// # Params
/// * `file`: the file to be copied, sorting the BINTABLE in the first extension
/// * `i_ra`: index of the column containing the Right Ascension (in degrees)
/// * `i_dec`: index of the column containing the Declination (in degrees)
/// * `output`: path of the output file, containing the sorted version of the input file
/// * `depth`: for external sort, depth used to compute the count map for temporary files ranges
/// * `internal_threshold`: maximum size, in bytes, of the main table to perform an internal sort
/// * `tmp`: temporary directory to be used in case of external sort
/// * `parallel`: number of threads to be used (all available thread if `None`)
/// # Warning
/// * RA and Dec columns units are assumed to be degrees (without any checks)
/// * the first extension **must be** a BINTABLE, other BINTABLEs are copied without being sorted
pub fn hsort_file(
  file: File,
  i_ra: usize,
  i_dec: usize,
  output: PathBuf,
  internal_threshold: usize,
  depth: u8,
  tmp: Option<PathBuf>,
  parallel: Option<usize>,
) -> Result<(), Box<dyn Error>> {
  debug!("Start hsort procedure...");
  // Prepare writer
  let output_file = File::create(output)?;
  let mut writer = BufWriter::new(output_file);

  // Prepare reading, creating a memory map
  let mmap = unsafe { MmapOptions::new().map(&file)? };
  mmap.advise(Advice::Sequential)?;

  // Read as a FITS file, prepare iteration on HDUs
  let bytes = mmap.as_ref();
  let fits = FitsBytes::from_slice(bytes);
  let mut hdu_it = fits.new_iterator::<Bintable>();

  // Read/write Primary HDU
  // * read
  let primary_hdu = hdu_it
    .next()
    .ok_or_else(|| new_custom("No HDU found!"))
    .and_then(|r| r)?;
  if !primary_hdu.is_primary_hdu() {
    return Err(String::from("First HDU is not a primary HDU!").into());
  }
  // * write
  primary_hdu.copy_hdu(&mut writer)?;
  debug!("Primary HDU copied");

  // Read/write BINTABLE in the first Extension HDU
  let bintable_hdu = hdu_it
    .next()
    .ok_or_else(|| new_custom("No second HDU found!"))
    .and_then(|r| r)?;
  if !primary_hdu.is_primary_hdu() {
    return Err(String::from("First HDU is not a primary HDU!").into());
  }

  // * read bintable metadata
  let bintable_header = match &bintable_hdu.parsed_header {
    HDUHeader::BinTable(h) => h,
    _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
  };
  let row_byte_size = bintable_header.row_byte_size();
  let n_rows = bintable_header.n_rows();
  let main_table_byte_size = bintable_header.main_table_byte_size();
  assert_eq!(main_table_byte_size, n_rows * row_byte_size);

  // * build the table schema
  let row_schema: RowSchema = bintable_header.buld_row_schema();

  // * get RA and Dec columns info, and ensure they are of type Double (no scale/offset allowed here so far)
  let ra_meta = &row_schema.fields_schemas()[i_ra];
  let de_meta = &row_schema.fields_schemas()[i_dec];
  if !matches!(ra_meta.schema, Schema::Double) {
    return Err(new_custom(format!(
      "RA column is not a double. Header: {:?}",
      &bintable_header.cols()[i_ra]
    )));
  }
  if !matches!(de_meta.schema, Schema::Double) {
    return Err(new_custom(format!(
      "Dec column is not a double. Header: {:?}",
      &bintable_header.cols()[i_dec]
    )));
  }

  // * copy bintable header. What about adding 3 keywords?
  //     + 1 stating that the file is HPX sorted
  //     + 1 one providing the index of the RA  column used in the HPX sort
  //     + 1 one providing the index of the Dec column used in the HPX sort
  bintable_hdu.copy_header(&mut writer)?;
  debug!("BINTABLE header copied");

  // * sort maintable rows
  let layer29 = get(29);
  // Position provider
  let hpx29v = move |row_bytes: &Vec<u8>| {
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
      layer29.hash(lon.to_radians(), lat.to_radians())
    }
  };
  //   + performs either an internal or an external sort
  if main_table_byte_size <= internal_threshold {
    debug!("Retrieve (and copy) BINTABLE rows...");
    // internal sort
    /*let mut rows: Vec<&[u8]> = (&bintable_hdu.data()[..main_table_byte_size])
    .chunks(row_byte_size)
    .collect();*/
    let mut rows: Vec<Vec<u8>> = (&bintable_hdu.data()[..main_table_byte_size])
      .chunks(row_byte_size)
      .map(|slice| slice.to_vec()) // make a copy in memory
      .collect();
    debug!("Start internal sort...");
    hpx_internal_sort(rows.as_mut_slice(), hpx29v, parallel);
    //debug!("Start mem copy of sorted rows...");
    /*let bytes: Vec<u8> = Vec::with_capacity(main_table_byte_size);
    for row in rows {
     bytes.
    }*/
    // It seems that making a copy in memory and then writting everything at once is
    // way faster that iterating on each row to write them...
    //let bytes = rows.concat();
    debug!("Start writing sorted rows...");
    //writer.write_all(&bytes)?;
    for row in rows {
      // writer.write_all(row)?;
      writer.write_all(row.as_slice())?;
      // see write_all_vectored once stabilized?!
    }
  } else {
    // Position provider
    let hpx29 = move |row_bytes: &&[u8]| {
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
        layer29.hash(lon.to_radians(), lat.to_radians())
      }
    };
    // external sort
    // - read once to get healpix distribution
    let twice_dd = (29 - depth) << 1;
    debug!("Compute count map...");
    let count_map = CountMapU32::from_hash_values(
      depth,
      (&bintable_hdu.data()[..main_table_byte_size])
        .chunks(row_byte_size)
        .map(|row| (hpx29(&row) >> twice_dd) as u32),
    );

    let mut sort_params: SimpleExtSortParams = Default::default();
    sort_params = sort_params.set_n_elems_per_chunk((internal_threshold / row_byte_size) as u32);
    if let Some(tmp_dir) = tmp {
      sort_params = sort_params.set_tmp_dir(tmp_dir);
    }
    if let Some(n_threads) = parallel {
      sort_params = sort_params.set_n_threads(n_threads);
    }

    // save_countmap_in_file: Option<P>, // Save a copy of the computed count map in the given Path

    debug!("Start external sort...");
    // - sort
    let sorted_row_it = hpx_external_sort_with_knowledge(
      (&bintable_hdu.data()[..main_table_byte_size])
        .chunks(row_byte_size)
        .map(|slice| Ok::<_, Infallible>(slice.to_vec())), // Copy here because of serde!!
      &count_map,
      hpx29v,
      Some(sort_params),
    )?;
    debug!("Start writting data...");
    for row_res in sorted_row_it {
      row_res.and_then(|row| writer.write_all(row.as_ref()).map_err(|e| e.into()))?;
    }
  }
  // * copy heap part if any (sizes have not changed, so no need to ckeck for 2880 byte blocks)
  debug!("Copy BINTABLE heap and padding bytes...");
  writer
    .write_all(&bintable_hdu.data()[main_table_byte_size..])
    .map_err(new_io_err)
    .and_then(|()| bintable_hdu.copy_blanks(&mut writer))?;
  // Copy other HDUs (if any)
  debug!("Copy other HDUs (if any)...");
  for other_hdu in hdu_it {
    other_hdu.and_then(|hdu| hdu.copy_hdu(&mut writer))?;
  }
  debug!("Done!");
  Ok(())
}

/// # Warning
/// * stops after the first encountered BINTABLE in each file
/// * all BINTABLEs must have the exact same structure
pub fn hsort_files(
  dir: PathBuf,
  i_ra: usize,
  i_dec: usize,
  output: PathBuf,
  internal_threshold: usize,
  depth: u8,
  tmp: Option<PathBuf>,
  parallel: Option<usize>,
) -> Result<(), Box<dyn Error>> {
  let mut fits_files_it = read_dir(&dir)?
    .filter_map(|res| res.ok())
    .filter(|entry| {
      entry.metadata().ok().map(|e| e.is_file()).unwrap_or(false)
        && entry
          .file_name()
          .to_str()
          .map(|name| name.ends_with(".fits"))
          .unwrap_or(false)
    })
    .map(|entry| entry.path());

  let first_file = fits_files_it
    .next()
    .ok_or_else(|| new_custom(format!("No '.fits' file found in directory {:?}", dir)))?;
  debug!("Reference file: {:?}", first_file);

  // Prepare writer
  let output_file = File::create(&output)?;
  let mut writer = BufWriter::new(output_file);

  // Prepare reading, creating a memory map
  let first_file = File::open(&first_file)?;
  let mmap = unsafe { MmapOptions::new().map(&first_file)? };
  mmap.advise(Advice::Sequential)?;

  // Read as a FITS file, prepare iteration on HDUs
  let bytes = mmap.as_ref();
  let fits = FitsBytes::from_slice(bytes);
  let mut hdu_it = fits.new_iterator::<Bintable>();

  // Read/write Primary HDU
  // * read
  let primary_hdu = hdu_it
    .next()
    .ok_or_else(|| new_custom("No HDU found!"))
    .and_then(|r| r)?;
  if !primary_hdu.is_primary_hdu() {
    return Err(String::from("First HDU is not a primary HDU!").into());
  }
  // * write
  primary_hdu.copy_hdu(&mut writer)?;

  // Read/write BINTABLE in the first Extension HDU
  let bintable_hdu = hdu_it
    .next()
    .ok_or_else(|| new_custom("No second HDU found!"))
    .and_then(|r| r)?;
  if !bintable_hdu.is_bintable_hdu() {
    return Err(String::from("Second HDU not a BINTABLE HDU!").into());
  }

  // * read bintable metadata
  let bintable_header = match &bintable_hdu.parsed_header {
    HDUHeader::BinTable(h) => h,
    _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
  };
  let row_byte_size = bintable_header.row_byte_size();
  let n_rows = bintable_header.n_rows();
  let main_table_byte_size = bintable_header.main_table_byte_size();
  debug_assert_eq!(main_table_byte_size, n_rows * row_byte_size);

  if bintable_header.heap_byte_size() != 0 {
    return Err(String::from("BINTABLEs having a HEAP are not supported!").into());
  }

  // * build the table schema
  let row_schema: RowSchema = bintable_header.buld_row_schema();
  // * get RA and Dec columns info, and ensure they are of type Double (no scale/offset allowed here so far)
  let ra_meta = row_schema.fields_schemas()[i_ra].clone();
  let de_meta = row_schema.fields_schemas()[i_dec].clone();
  if !matches!(ra_meta.schema, Schema::Double) {
    return Err(new_custom(format!(
      "RA column is not a double. Header: {:?}",
      &bintable_header.cols()[i_ra]
    )));
  }
  if !matches!(de_meta.schema, Schema::Double) {
    return Err(new_custom(format!(
      "Dec column is not a double. Header: {:?}",
      &bintable_header.cols()[i_dec]
    )));
  }

  let bintable_header_starting_byte = bintable_hdu.starting_byte;
  // * copy bintable header. What about adding 3 keywords?
  //     + 1 stating that the file is HPX sorted
  //     + 1 one providing the index of the RA  column used in the HPX sort
  //     + 1 one providing the index of the Dec column used in the HPX sort
  bintable_hdu.copy_header(&mut writer)?;

  // * sort maintable rows
  let layer29 = get(29);
  // Position provider
  let hpx29 = move |row_bytes: &Vec<u8>| {
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
      layer29.hash(lon.to_radians(), lat.to_radians())
    }
  };

  // Now, creates an iterator iterating over the first BINTABLE of all files!!
  let from = bintable_hdu.data_starting_byte();
  let first_file_rows_it = RowFileIt::new(mmap, from, from + main_table_byte_size, row_byte_size);

  let rows_it = FirstBintableRowsIt::new(fits_files_it, row_schema, first_file_rows_it);

  let mut sort_params: SimpleExtSortParams = Default::default();
  sort_params = sort_params.set_n_elems_per_chunk((internal_threshold / row_byte_size) as u32);
  if let Some(tmp_dir) = tmp {
    sort_params = sort_params.set_tmp_dir(tmp_dir);
  }
  if let Some(n_threads) = parallel {
    sort_params = sort_params.set_n_threads(n_threads);
  }

  let mut count_map_path = output.clone();
  count_map_path.set_extension("countmap.fits");
  let sorted_row_it = hpx_external_sort_stream(
    rows_it,
    hpx29,
    depth,
    Some(count_map_path),
    Some(sort_params),
  )?;
  let mut n = 0_u64;
  for row_res in sorted_row_it {
    row_res.and_then(|row| writer.write_all(row.as_ref()).map_err(|e| e.into()))?;
    n += 1;
  }
  // blanck padding if necessary
  let mod2880 = writer.stream_position()? % 2880;
  if mod2880 != 0 {
    writer.write_all(vec![8_u8; 2880 - mod2880 as usize].as_slice())?;
  }
  // Re-write number of rows
  let mut naxis2 = [0_u8; 80];
  writer.seek(SeekFrom::Start(
    bintable_header_starting_byte as u64 + 4 * 80,
  ))?;
  NAxis2::new(n).write_kw_record(&mut std::iter::once(Ok(&mut naxis2)))?;
  writer
    .write_all(naxis2.as_slice())
    .and_then(|()| writer.flush())
    .map_err(|e| e.into())
}

struct FirstBintableRowsIt<I>
where
  I: Iterator<Item = PathBuf>,
{
  file_it: I,
  row_schema: RowSchema,
  curr_row_it: RowFileIt,
}
impl<I> FirstBintableRowsIt<I>
where
  I: Iterator<Item = PathBuf>,
{
  pub fn new(file_it: I, row_schema: RowSchema, first_file_row_it: RowFileIt) -> Self {
    Self {
      file_it,
      row_schema,
      curr_row_it: first_file_row_it,
    }
  }
}

impl<I> Iterator for FirstBintableRowsIt<I>
where
  I: Iterator<Item = PathBuf>,
{
  type Item = Result<Vec<u8>, crate::error::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.curr_row_it.next() {
      Some(e) => Some(Ok(e)),
      None => match self.file_it.next() {
        Some(f) => {
          // Prepare reading file, creating a memory map
          let file = match File::open(&f)
            .map_err(|e| new_custom(format!("Error opening file '{:?}': {:?}", f, e)))
          {
            Ok(file) => file,
            Err(e) => return Some(Err(e)),
          };
          let mmap = match unsafe { MmapOptions::new().map(&file) }.map_err(|e| {
            new_custom(format!(
              "Error creating the mmap on file '{:?}': {}",
              file, e
            ))
          }) {
            Ok(mmap) => mmap,
            Err(e) => return Some(Err(e)),
          };
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

          // Read Primary HDU
          if let Err(primary_hdu_err) = hdu_it
            .next()
            .ok_or_else(|| new_custom(format!("No HDU found in file '{:?}'!", file)))
            .and_then(|r| r)
            .map_err(|e| {
              new_custom(format!(
                "Error reading primary HDU in file {:?}: {:?}",
                file, e
              ))
            })
          {
            return Some(Err(primary_hdu_err));
          }

          // Read BINTABLE in the first Extension HDU
          let bintable_hdu = match hdu_it
            .next()
            .ok_or_else(|| new_custom(format!("No secondary HDU found in file '{:?}'!", file)))
            .and_then(|r| r)
            .map_err(|e| {
              new_custom(format!(
                "Error reading BINTABLE HDU in file {:?}: {:?}",
                file, e
              ))
            }) {
            Ok(hdu) => hdu,
            Err(e) => return Some(Err(e)),
          };
          if !bintable_hdu.is_bintable_hdu() {
            return Some(Err(new_custom(format!(
              "2nd HDU in file '{:?}' is not a BINTABLE.",
              file
            ))));
          }

          // * read bintable metadata
          let bintable_header = match &bintable_hdu.parsed_header {
            HDUHeader::BinTable(h) => h,
            _ => unreachable!(), // since we already tested with 'is_bintable_hdu'
          };
          let row_byte_size = bintable_header.row_byte_size();
          let n_rows = bintable_header.n_rows();
          let main_table_byte_size = bintable_header.main_table_byte_size();
          assert_eq!(main_table_byte_size, n_rows * row_byte_size);

          // * build the table schema
          let row_schema: RowSchema = bintable_header.buld_row_schema();
          if row_schema != self.row_schema {
            return Some(Err(new_custom(format!(
              "Incompatible BINTABLE HDU schema in file {:?}.",
              file
            ))));
          }
          if bintable_header.heap_byte_size() > 0 {
            return Some(Err(
              new_custom("BINTABLEs having a HEAP are not supported!").into(),
            ));
          }

          let from = bintable_hdu.data_starting_byte();
          self.curr_row_it = RowFileIt::new(mmap, from, from + main_table_byte_size, row_byte_size);
          self.next()
        }
        None => None,
      },
    }
  }
}

struct RowFileIt {
  mmap: Mmap,
  from_byte: usize,
  to_byte: usize,
  row_byte_size: usize,
}
impl RowFileIt {
  fn new(mmap: Mmap, from_byte: usize, to_byte: usize, row_byte_size: usize) -> Self {
    Self {
      mmap,
      from_byte,
      to_byte,
      row_byte_size,
    }
  }
}

impl Iterator for RowFileIt {
  type Item = Vec<u8>;

  fn next(&mut self) -> Option<Self::Item> {
    let to = self.from_byte + self.row_byte_size;
    if to <= self.to_byte {
      let bytes = self.mmap[self.from_byte..to].to_vec();
      self.from_byte = to;
      Some(bytes)
    } else {
      None
    }
  }
}
