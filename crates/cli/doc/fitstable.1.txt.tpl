fitstable(1)
============

Name
----
fitstable - Command-line tool for fitstable, including hipsgen-cat features


Synopsis
--------

*fitstable* _SUBCMD_ _SUBCMDPARAMS_

*fitstable* *--version*

*fitstable* *--help*

*fitstable* _SUBCMD_ *--help*

*fitstable* _SUBCMD_ _SUBCMDPARAMS_ *--help*


SUBCMD
------

_struct_::
  Read and print the structure of a FITS file

_head_::
  Read and print the headers of all the HDU in a FITS file 
 
_info_::
  Print tables information (such as column names, units, ...)

_csv_::
  Print tables in CSV format  

_sort_::
  Sort a file, or sort and concatenate a set of files, according to HEALPix

_mkidx_::
  Make a positional index for HEALPix sorted files

_qidx_::
  Query a HEALPix sorted and indexed BINTABLE

_mkhips_::
  Create a HiPS catalogue from a HEALPix sorted and index BINTABLE

_qhips_::
  Query a HiPS catalogue


Examples
--------

hpx head myfile.fits


DESCRIPTION
-----------

FITS BINTABLE operations on the command line.


VERSION
-------
{VERSION}


HOMEPAGE
--------
https://github.com/cds-astro/cds-fitstable-rust

Please report bugs and feature requests in the issue tracker.


AUTHORS
-------
F.-X. Pineau <francois-xavier.pineau@astro.unistra.fr>


