//! Read a FITS from a reader, i.e. in streaming mode.
//! This mode e.g. **does not supprt** BINTABLE columns having data stored in the HEAP (or, at least,
//! data on the HEAP is available once all rows have been read).
