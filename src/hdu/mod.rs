use std::io::Write;

use crate::common::keywords::xtension::Xtension;

pub mod header;
pub mod primary;
pub mod xtension;

struct HDU {
  // header
  // header_starting_byte
  // data_starting_byte
  // data_byte_size
}

#[derive(Debug, PartialEq, Eq)]
pub enum HDUType {
  Primary,
  Extension(Xtension),
}
