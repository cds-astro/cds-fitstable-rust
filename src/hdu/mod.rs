use crate::common::keywords::xtension::Xtension;

pub mod header;
pub mod primary;
pub mod xtension;

#[derive(Debug, PartialEq, Eq)]
pub enum HDUType {
  Primary,
  Extension(Xtension),
}
