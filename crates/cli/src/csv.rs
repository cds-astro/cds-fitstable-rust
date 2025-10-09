use std::{
  error::Error,
  fmt::Debug,
  fs::File,
  io::BufWriter,
  io::{stdout, Write},
  path::PathBuf,
  thread::scope,
};

use clap::Args;
use crossbeam::channel::{bounded, Receiver, Sender};
use log::{error, info};
use memmap2::{Advice, MmapOptions};

use fitstable::{
  hdu::{
    header::{builder::r#impl::bintable::Bintable, HDUHeader, Header},
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

#[derive(Debug, Clone, Args)]
pub struct Csv {
  /// Path of the input file.
  #[clap(value_name = "FILE")]
  pub input: PathBuf,
  /// Path of the output file [default: write to stdout]
  #[clap(short = 'o', long = "out", value_name = "FILE")]
  output: Option<PathBuf>,
  /// Do not print the header line (useful when concatenating a set a FITS file of same structure).
  #[clap(short, long)]
  no_header: bool,
  /// Exec concurrently using N threads [default: all possible threads]
  #[arg(long, value_name = "N")]
  parallel: Option<usize>,
  /// Number of FITS table bytes process by each `parallel` thread
  #[arg(long, default_value_t = 10.0_f32)]
  chunk_size_mb: f32,
  /// Data on a SSD instead of on an HDD (avoid a copy enforcing sequential reading of the data).
  #[clap(short = 's', long)]
  ssd: bool,
  /// Use 'sequential' mmap advice: to be used on Unix, with cold HDD cache, no SSD option and no HEAP
  /// (i.e. variable length columns) in FITS tables.
  #[clap(short = 'q', long, conflicts_with = "ssd")]
  seq: bool,
}

impl Csv {
  pub fn exec(self) -> Result<(), Box<dyn Error>> {
    let n_threads = self.parallel.unwrap_or_else(|| num_cpus::get()).max(1);
    let file = File::open(&self.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    if self.seq {
      mmap.advise(Advice::Sequential)?;
    }
    let mut first_table = true;
    for (i, hdu) in FitsBytes::from_slice(mmap.as_ref())
      .new_iterator::<Bintable>()
      .enumerate()
    {
      let hdu = hdu?;
      // Choose between stdout or file
      let is_a_table = match &self.output {
        Some(path) => {
          // Add the hdu number to the extension from the second table
          let file = if first_table {
            File::create(path)
          } else {
            let mut new_path = path.clone();
            match path.extension().and_then(|ext| ext.to_str()) {
              Some(ext) => new_path.set_extension(&format!("{}.{}", i, ext)),
              None => new_path.set_extension(&format!("{}.csv", i)),
            };
            File::create(new_path)
          }?;
          let mut write = BufWriter::new(file);
          convert_to_csv(
            hdu,
            &mut write,
            self.no_header,
            n_threads,
            self.chunk_size_mb,
            !self.ssd,
          )
        }
        None => {
          let stdout = stdout();
          let mut handle = stdout.lock();
          convert_to_csv(
            hdu,
            &mut handle,
            self.no_header,
            n_threads,
            self.chunk_size_mb,
            !self.ssd,
          )
        }
      }?;
      if is_a_table {
        first_table = false;
      }
    }
    Ok(())
  }
}

// Returns true if the HDU was a table written in output.
fn convert_to_csv<W: Write>(
  hdu: HDU<Bintable>,
  write: &mut W,
  no_header: bool,
  n_threads: usize,
  chunk_size_mb: f32,
  hdd: bool,
) -> Result<bool, Box<dyn Error>> {
  let HDU {
    starting_byte: _,
    raw_header: _,
    parsed_header,
    data,
  } = hdu;
  match parsed_header {
    HDUHeader::Primary(_) => Ok(false),
    HDUHeader::Image(_) => Ok(false),
    HDUHeader::AsciiTable(_) => todo!(),
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
      if !no_header {
        let mut first = true;
        for (i, field) in bintable_header_full.cols().iter().enumerate() {
          if first {
            first = false;
          } else {
            write!(write, ",")?;
          }
          match field.colname() {
            Some(name) => write!(write, "{}", name),
            None => write!(write, "col_{}", i),
          }?;
        }
      }

      // Print data
      if n_threads == 1 {
        info!("Exec with a single thread");

        let mut visitor = CSVVisitor::new(write);
        for raw_row in main.chunks(row_byte_size) {
          let mut de = DeserializerWithHeap::new(raw_row, heap);
          visitor.starts_new_line();
          row_schema.deserialize(&mut de, &mut visitor, CSVRowVisitor)?;
        }
        write!(write, "\n",).map_err(|e| e.into())
      } else {
        info!("Exec with {} threads", n_threads);

        // Convert chunk size in MB in a number of rows.
        let chunk_size = 1 + ((chunk_size_mb * 1048576.0_f32) as usize / row_byte_size);
        info!("Number of rows per chunk: {}", chunk_size);

        // Multithreaded code
        // Here, we create one (sender, receiver) pairs per thread and iterate on the
        // ordered sender/receiver to preserve the original row order.
        // One thread, the producer (sender1), read the data and send it to multithreaded processors.
        // One thread, the consumer (receivers2), retrieve the data from producers and write them in the output.
        //
        // The difference (an extra copy) with and without the 'hdd' option seems very small.
        // we let the option for extra tests, but we could remove it for simplicity.
        if hdd {
          // HDD mode: make a copy to be sure to read in sequencial mode.
          // The only difference with the SSD mode is the '.map(Vec::from)' in the chunk reader.
          let (mut senders1, receivers1): (Vec<Sender<_>>, Vec<Receiver<_>>) =
            (0..n_threads).map(|_| bounded(1)).unzip();
          let (mut senders2, receivers2): (Vec<Sender<_>>, Vec<Receiver<_>>) =
            (0..n_threads).map(|_| bounded(1)).unzip();
          scope(|s| {
            // Producer thread
            s.spawn(|| {
              let mut senders_it = senders1.iter().cycle();
              for rows_chunk in main.chunks(row_byte_size * chunk_size).map(Vec::from) {
                senders_it
                  .next()
                  .unwrap()
                  .send(rows_chunk)
                  .expect("Unexpected error sending raw rows");
              }
              // Close the channels, otherwise sink will never exit the for-loop
              senders1.drain(..).for_each(drop);
            });
            // Parallel processing by n_threads
            for (sendr2, recvr1) in senders2.iter().cloned().zip(receivers1.iter().cloned()) {
              // Send to sink, receive from producer
              let row_schema = row_schema.clone();
              // Spawn workers in separate threads
              s.spawn(move || {
                // Receive until channel closes
                for raw_rows_chunk in recvr1.iter() {
                  // We estimate CSV size = 3x binary size
                  let mut buff = Vec::<u8>::with_capacity(3 * row_byte_size * chunk_size);
                  let mut visitor = CSVVisitor::new(&mut buff);
                  for raw_row in raw_rows_chunk.chunks(row_byte_size) {
                    let mut de = DeserializerWithHeap::new(raw_row, heap);
                    visitor.starts_new_line();
                    if let Err(e) = row_schema.deserialize(&mut de, &mut visitor, CSVRowVisitor) {
                      error!("Error deserializing row: {:?}", e);
                    }
                  }
                  sendr2
                    .send(buff)
                    .expect("Unexpected error sending converted rows");
                }
              });
            }
            // Close the channel, otherwise sink will never exit the for-loop
            senders2.drain(..).for_each(drop);
            // Sink in the current thread
            for recvr2 in receivers2.iter().cycle() {
              match recvr2.recv() {
                Ok(raw_rows) => match write.write_all(&raw_rows) {
                  Ok(()) => (),
                  Err(e) => panic!("Error writing in parallel: {:?}", e),
                },
                Err(_) => {
                  // No more data to be written
                  break;
                }
              }
            }
          });
        } else {
          // Directly pass mmap bytes (read when accessed).
          let (mut senders1, receivers1): (Vec<Sender<_>>, Vec<Receiver<_>>) =
            (0..n_threads).map(|_| bounded(1)).unzip();
          let (mut senders2, receivers2): (Vec<Sender<_>>, Vec<Receiver<_>>) =
            (0..n_threads).map(|_| bounded(1)).unzip();
          scope(|s| {
            // Producer thread
            s.spawn(|| {
              let mut senders_it = senders1.iter().cycle();
              for rows_chunk in main.chunks(row_byte_size * chunk_size) {
                senders_it
                  .next()
                  .unwrap()
                  .send(rows_chunk)
                  .expect("Unexpected error sending raw rows");
              }
              // Close the channels, otherwise sink will never exit the for-loop
              senders1.drain(..).for_each(drop);
            });
            // Parallel processing by n_threads
            for (sendr2, recvr1) in senders2.iter().cloned().zip(receivers1.iter().cloned()) {
              // Send to sink, receive from producer
              let row_schema = row_schema.clone();
              // Spawn workers in separate threads
              s.spawn(move || {
                // Receive until channel closes
                for raw_rows_chunk in recvr1.iter() {
                  // We estimate CSV size = 3x binary size
                  let mut buff = Vec::<u8>::with_capacity(3 * row_byte_size * chunk_size);
                  let mut visitor = CSVVisitor::new(&mut buff);
                  for raw_row in raw_rows_chunk.chunks(row_byte_size) {
                    let mut de = DeserializerWithHeap::new(raw_row, heap);
                    visitor.starts_new_line();
                    if let Err(e) = row_schema.deserialize(&mut de, &mut visitor, CSVRowVisitor) {
                      error!("Error deserializing row: {:?}", e);
                    }
                  }
                  sendr2
                    .send(buff)
                    .expect("Unexpected error sending converted rows");
                }
              });
            }
            // Close the channel, otherwise sink will never exit the for-loop
            senders2.drain(..).for_each(drop);
            // Sink in the current thread
            for recvr2 in receivers2.iter().cycle() {
              match recvr2.recv() {
                Ok(raw_rows) => match write.write_all(&raw_rows) {
                  Ok(()) => (),
                  Err(e) => panic!("Error writing in parallel: {:?}", e),
                },
                Err(_) => {
                  // No more data to be written
                  break;
                }
              }
            }
          });
        }
        write!(write, "\n",).map_err(|e| e.into())
      }
      .map(|()| true)
    }
    HDUHeader::Unknown(_) => Ok(false),
  }
}
