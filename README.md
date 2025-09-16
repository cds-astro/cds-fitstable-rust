# `fitstable`

A native, full Rust, library to read tabular data in FITS files.

About
-----

The library is dedicate to read table (ASCII or BINARY) in FITS file, in pure Rust.
For a similar library, to read images, see [fitsrs](https://github.com/cds-astro/fitsrs).
This work is exploratory, testing design choices, and could be merged in
[fitsrs](https://github.com/cds-astro/fitsrs) in the future.

It is used in:

* the [fitstable](crates/cli) command line, possibly reaching more than **1.2 GB/s** in **BINTABLE to CSV** conversion
  (depending on the level of parallelism and I/O capabilities of the machine)

Standalone
----------

See [fitstable-cli](crates/cli) for a standalone command line tool.
Among the features, you get a multi-threaded FITS to CSV conversion.


ToDo
----

* [ ] Implement a Reader for streamed data (heap ignored)
    + Remark: for stream reading, it would have been better to put the BINTABLE heap before the main table so that
      one could have kept it in memory (or write it in a temporary file) to access the data when reading pointers
      pointing to it reading the main table.
* [ ] Implement writers
    + Remark: stream writing is not possible in FITS size the size of the result must be known in advance (the number of
      rows is writen in the header.
* [ ] Add test with a great variety of FITS file
* [ ] Implement ASCIITABLE

Disclaimer
----------

This library is very young and requires testing on various FITS files!
If you have exotic files, please send them to us!
We are looking for BINTABLE FITS files:

* using SCALE and OFFSET;
* using ARRAYS;
* using the HEAP.

Acknowledgements
----------------

If you use this code and work in a scientific public domain
(especially astronomy), please acknowledge its usage and the
[CDS](https://en.wikipedia.org/wiki/Centre_de_donn%C3%A9es_astronomiques_de_Strasbourg)
who developed it.
It may help us in promoting our work to our financiers.


License
-------

Like most projects in Rust, this project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.


Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

