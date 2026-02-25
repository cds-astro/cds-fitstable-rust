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
* to create HEALPix sorted and indexed FITS file for:
    + fast positional queries (especially with `qat2s`)
    + on-the-fly HATS products generation
    + source to build HiPS catalogues

## `hipsgen-cat` with `fitstable`

See [this specific page](doc/hipsgen.md)

## Install

### From pypi for python users

fitstable-cli is available in [pypi](https://pypi.org/project/fitstable-cli),
you can thus install the `fitstable` executable using `pip`:
```bash
pip install -U fitstable-cli
fitstable --help
```

### Debian package

Download the last `fitstable-cli_vxx_yyy.deb` corresponding to your architecture
(`x86_64_musl` has the most chances to fit your needs)
from the [github release page](https://github.com/cds-astro/cds-fitstable-rust/releases).

Install the `.deb` by clicking on it or using the command line:
```bash
sudo dpkg -i fitstable-cli_vxx_yyy.deb
sudo apt-get install -f
```

Then you can use the tool:
```bash
fitstable-cli
man fitstable-cli
```

You can uninstall using, e.g.:
```bash
sudo dpkg -r $(dpkg -f fitstable-cli_vxx_yyy.deb Package)
```

WARNING: using this method, the command line name is `fitstable-cli` instead of `fitstable` due to a conflict with an existing debian `fitstable` package.


### Pre-compile binaries for MacOS, Linux and Windows

Download the last `fitstable-cli-vxx_yyy.tar.gz` corresponding to your architecture
from the [github release page](https://github.com/cds-astro/cds-fitstable-rust/releases).
You probably want ot use:
* Linux: `fitstable-cli-vxx-x86_64-unknown-linux-musl.tar.gz`
* MacOS: `fitstable-cli-vxx-x86_64-apple-darwin.tar.gz`
* Windows: `fitstable-cli-vxx-windows.zip`

WARNING: for linux, use [`musl`](https://en.wikipedia.org/wiki/Musl) instead of `gnu` (high chances of uncompatibility in the latter case)

The tar contains a single executable binary file.
```bash
tar xzvf fitstable-cli-vxx-yyy.tar.gz
./fitstable
```


### From source code

1 - Install or update Rust, see [this page](https://www.rust-lang.org/tools/install). For macOS or Linux:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Update it if already installed
rustup update
```

2 - clone the git repository containing the source code:

```bash
git clone https://github.com/cds-astro/cds-fitstable-rust.git
```

3 - compile and create the executable:

```bash
cd cds-fitstable-rust
cargo install --path crates/cli
```

## Usage

```bash
> fitstable

Command-line tool for fitstable, including hipsgen-cat features

Usage: fitstable <COMMAND>

Commands:
  struct  Read and print the structure of a FITS file
  head    Read and print the headers of all the HDU in a FITS file
  info    Print tables information (such as column names, units, ...)
  csv     Print tables in CSV format
  sort    Sort a file, or sort and concatenate a set of files, according to HEALPix
  mkidx   Make a positional index for HEALPix sorted files
  qidx    Query a BINTABLE using to a HEALPix index
  mkhips  Create a HiPS catalogue from a HEALPix sorted and index BINTABLE
  qhips   Query a HiPS catalogue
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

You can use the `--help` option in any sub-command, e.g.: 

```bash
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

### FITS to CSV conversion

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
sys 	0m7,346s

> time fitstable csv 4XMM_DR11cat_v1.0.fits -o /dev/null --parallel 32

real	0m2,226s
user	0m40,174s
sys 	0m3,483s

> time fitstable csv 4XMM_DR11cat_v1.0.fits -o /dev/null --parallel 32 -s

real	0m1,984s
user	0m38,573s
sys 	0m2,640s
```

For the first command, we achieve conversion + writing speed of more than **700 MB/s**
(probably limited by SSDs writing speed) while the pure conversion (writing in `/dev/null`)
reaches **1.2 GB/s**. Those measures were made with a hot disk cache.

Conclusion: with enough CPUs, IOs, in particular the writing speed, seem to be the main limiting factor.

### Sort and query 4XMM-DR14

```bash
# Donwload and uncompress the 4XMM-DR14 catalogue (1.4GB)
> wget 'https://nxsa.esac.esa.int/catalogues/4xmmdr14_240411.fits.gz'
> gzip -d 4xmmdr14_240411.fits.gz

# Look a the RA/DEC column indices
> time fitstable info 4xmmdr14_240411.fits
HDU[1]:  BINTABLE  n_cols: 336; n_rows : 1035832
   #             name  type        unit ucd desc                                                                
   0            DETID  i64?         --- --- A unique number which identifies each entry (detection) in the catal
...
  25               RA   f64         deg --- Corrected Right Ascension of the detection in degrees (J2000) after 
  26              DEC   f64         deg --- Corrected Declination of the detection in degrees (J2000) after stat
...

real	0m0,031s
user	0m0,002s
sys 	0m0,003s

# Sort the file
> time fitstable sort 4xmmdr14_240411.fits 4xmmdr14_240411.sorted.fits --lon 26 --lat 27 --chunk-size 10485760000

real	0m22,906s
user	0m1,467s
sys 	0m4,136s

# Create an external HEALPix Index on file (default depth = 9)
> time fitstable mkidx 4xmmdr14_240411.sorted.fits 4xmmdr14_240411.sorted.hidx.fits --lon 26 --lat 27

real	0m2,343s
user	0m0,159s
sys 	0m0,942s

# Get all sources in a HEALPix cell 4/1
> time fitstable qidx 4xmmdr14_240411.sorted.hidx.fits q.fits hpx 4 1

real	0m0,009s
user	0m0,003s
sys 	0m0,005s

# Look at the file structure to know the number of rows
# WARNING: so far, reading with STITS is ok, but not with TOPCAT
# WARNING: due to 'nrows' in FITS-plus VOTable left unchanged.
> fitstable struct q.fits

HDU[0]:
 * HEAD starting byte: 0; n_blocks: 1; byte size: 2880
 * DATA starting byte: 2880; byte size: 88061.
 * TYPE: PRIMARY
   + simple: true; naxis: 1; bitpix : 8; dimensions: 88061.
HDU[1]:
 * HEAD starting byte: 92160; n_blocks: 35; byte size: 100800
 * DATA starting byte: 192960; byte size: 47968.
 * TYPE: BINTABLE
   + n_cols: 336; n_rows : 32; row_byte_size: 1499; heap_byte_size: 0.

# Perform a cone search query on the file
> time fitstable qidx 4xmmdr14_240411.sorted.hidx.fits q.fits cone 123.45 +67.89 4.0

real	0m0,024s
user	0m0,003s
sys 	0m0,009s

> fitstable struct q.fits

HDU[0]:
 * HEAD starting byte: 0; n_blocks: 1; byte size: 2880
 * DATA starting byte: 2880; byte size: 88061.
 * TYPE: PRIMARY
   + simple: true; naxis: 1; bitpix : 8; dimensions: 88061.
HDU[1]:
 * HEAD starting byte: 92160; n_blocks: 35; byte size: 100800
 * DATA starting byte: 192960; byte size: 3278313.
 * TYPE: BINTABLE
```

Remark 1: use `RUST_LOG=debug` to activate loging, e.g.:

```bash
RUST_LOG=debug fitstable ...
```

Remark 2: from source code

```bash
RUST_LOG=debug cargo run --release -- ...
```

### Sort and query a set of ESO catalogue FITS files

Disclaimer: the following commands have been executed on a server with disks accessed through a local network.
Better results are expected on SSDs.

```bash
# Go to https://www.eso.org/qi/ 
# Look at VMD DR6 info https://www.eso.org/qi/catalog/show/396
# And donwload all FITS files (login requested) in a 'orgdata' diretory
# We get 113 FITS files, from 56 MB to 586 MB, and for a total of 30 GB

# Look a RA/Dec columns indices using a random table
> fitstable info orgdata/ADP.2022-07-28T13:55:38.537.fits

HDU[1]:  BINTABLE  n_cols: 96; n_rows : 196582
   #                 name  type   unit                             ucd desc                                                                
   0              IAUNAME s[29]                                meta.id IAU Name (not unique)                                               
...
   4               RA2000   f64    deg             pos.eq.ra;meta.main Celestial Right Ascension                                           
   5              DEC2000   f64    deg            pos.eq.dec;meta.main Celestial Declination
...


# Merge and sort all FITS files in a vmc_dr6.fits file (output size: 30 GB)
> time fitstable sort orgdata vmc_dr6.fits --lon 5 --lat 6 --depth 10 --chunk-size 1073741824

real    11m51,598s
user    10m3,839s
sys      2m14,942s

# Look at the total number of rows
> fitstable struct vmc_dr6.fits

HDU[0]:
 * HEAD starting byte: 0; n_blocks: 12; byte size: 34560
 * DATA starting byte: 34560; byte size: 0.
 * TYPE: PRIMARY
   + simple: true; naxis: 0; bitpix : 8; dimensions: 0.
HDU[1]:
 * HEAD starting byte: 34560; n_blocks: 28; byte size: 80640
 * DATA starting byte: 115200; byte size: 31285155084.
 * TYPE: BINTABLE
   + n_cols: 96; n_rows : 70462061; row_byte_size: 444; heap_byte_size: 0.

# Create an HEALPix index
> fitstable mkidx vmc_dr6.fits vmc_dr6.hidx.fits --lon 5 --lat 6 --depth 9

real    1m39,131s
user    0m10,111s
sys     0m16,943s


# Perform a positional query around the LMC
> time ./fitstable qidx vmc_dr6.hidx.fits res.fits cone 80.8942 -69.75 0.1

real    0m0,038s
user    0m0,014s
sys     0m0,022s

# Look at the result file, it contains 32454 rows
> ./fitstable struct res.fits 

HDU[0]:
 * HEAD starting byte: 0; n_blocks: 12; byte size: 34560
 * DATA starting byte: 34560; byte size: 0.
 * TYPE: PRIMARY
   + simple: true; naxis: 0; bitpix : 8; dimensions: 0.
HDU[1]:
 * HEAD starting byte: 34560; n_blocks: 28; byte size: 80640
 * DATA starting byte: 115200; byte size: 14409576.
 * TYPE: BINTABLE
   + n_cols: 96; n_rows : 32454; row_byte_size: 444; heap_byte_size: 0.
  
# Bonus: if you install hpx-cli https://github.com/cds-astro/cds-healpix-rust/tree/master/crates/cli
# to create a density map from the generated 'vmc_dr6.countmap.fits' file
hpx map view vmc_dr6.countmap.fits vmc_dr6.png allsky 300

```

## ToDo

* [X] add HEALPix sort, index and query commands

## Further work

This code could be a base for:

* a new HiPS Catalogue generation tool replacing the Java one
* a HATS products generator, working on a single machine
* producing ExXMatch indexes for fast XMatches
* ...

<!--
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
-->

License
-------

So far, we adopt the conservative GPL license, but hope to change it in favor
of a dual Apache/MIT license in the future.
Please contact us if you wish to use this code.

Contribution
------------

No direct contribution accepted so far.

