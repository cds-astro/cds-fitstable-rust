# `fitstable-cli`

A command line tool to explore FITS file, with advance features for
FITS files containing tables.

## About

This **C**ommand **L**ine **I**nterface (CLI) is made from the
[CDS FITS Table Rust library](https://github.com/cds-astro/cds-fitstable-rust).

## Motivations

The motivations for the cli are:

* to be able to test the [fitstable](https://github.com/cds-astro/cds-fitstable-rust?) library
* to quickly get the structure of a FITS file
* to have the equivalent of the command `fold -80 myfile.fits | more` without the binary data
* to have a quick tool converting possible large FITS files (especially when dealing with the ingestion of large table,
  such as ESO tables, in VizieR)

## Usage

```bash
> fitstable
Command-line tool for fitstable

Usage: fitstable <COMMAND>

Commands:
  struct  Read and print the structure of a FITS file
  head    Read and print the headers of all the HDU in a FITS file
  csv     Print found table in CSV
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


>  fitstable csv --help
Print found table in CSV

Usage: fitstable csv [OPTIONS] <FILE>

Arguments:
  <FILE>  Path of the input file

Options:
  -o, --out <FILE>                     Path of the output file [default: write to stdout]
      --parallel <N>                   Exec concurrently using N threads [default: all possible threads]
      --chunk-size-mb <CHUNK_SIZE_MB>  Number of FITS table bytes process by each `parallel` thread [default: 10]
  -s, --ssd                            Data on a SSD instead of on an HDD (avoid a copy enforcing sequential reading of the data)
  -q, --seq                            Use 'sequential' mmap advice: to be used on Unix, with cold HDD cache, no SSD option and no HEAP (i.e. variable length columns) in FITS tables
  -h, --help                           Print help
```

## Performances

We tested the FITS BINTABLE to CSV conversion speed using the 1.3 GB unzipped
file [4XMM DR11](http://xmmssc.irap.omp.eu/Catalogue/4XMM-DR11/4XMM_DR11cat_v1.0.fits.gz) available
from [this page](http://xmmssc.irap.omp.eu/Catalogue/4XMM-DR11/4XMM_DR11.html).
The output CSV file is 2.7 GB large.

We tested the performances on a recent server (SSDs, 64 threads).

To also assess the pure conversion speed (without actually writing the file), we tested
both writing in a file and in `/dev/null`. Here the results:

```bash
> time fitstable csv 4XMM_DR11cat_v1.0.fits -o 4XMM_DR11cat_v1.0.csv --parallel 32

real	0m3,758s
user	0m42,544s
sys	0m7,346s

> time fitstable csv 4XMM_DR11cat_v1.0.fits -o /dev/null --parallel 32

real	0m2,226s
user	0m40,174s
sys	0m3,483s

> time fitstable csv 4XMM_DR11cat_v1.0.fits -o /dev/null --parallel 32 -s

real	0m1,984s
user	0m38,573s
sys	0m2,640s


```

For the first command, we achieve conversion + writing speed of more than **700 MB/s**
(probably limited by SSDs writing speed) while the pure conversion (writing in `/dev/null`)
reaches **1.2 GB/s**. Those measures were made with a hot disk cache.

Conclusion: with enough CPUs, IOs, in particular the writing speed, seem to be the main limiting factor.

## License

Like most projects in Rust, this project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.


