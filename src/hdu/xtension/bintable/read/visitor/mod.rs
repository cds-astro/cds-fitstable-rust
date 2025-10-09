use crate::{
  error::{new_unsupported_by_visitor, Error},
  hdu::xtension::bintable::field::{ComplexF32, ComplexF64},
};

pub mod csv;
pub mod field;
pub mod primitive;

pub trait FieldVisitorProvider {
  type FieldValue;
  type FieldVisitor<'v>: Visitor<Value = Self::FieldValue>
  where
    Self: 'v;

  fn field_visitor(&mut self) -> Self::FieldVisitor<'_>;
}

pub trait RowVisitor {
  type Value;
  type FieldValue;

  fn visit_row<I>(self, fields_it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Result<Self::FieldValue, Error>>;
}

/// Largely inspired from [serde.rs](https://docs.rs/serde/latest/serde/de/trait.Visitor.html),
/// but simplified and adapted for FITS datatypes.
/// Unfortunately, `serde` has no "Complex" types.
/// Also, we added `visit_opt_xx` methods for various `xx` types, so that confronted to a `NULL`
/// value we still known what the type was supposed to be (see if this could be removed or not).
/// Same thing for array and arrays of nullable elements: we removed the genericity.
pub trait Visitor: Sized {
  /// The value produced by this visitor.
  type Value;

  fn expecting(&self) -> &str {
    "nothing (default impl)"
  }

  fn visit_empty(self) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Empty column"))
  }

  fn visit_opt_bool(self, _v: Option<bool>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Boolean",
    ))
  }

  fn visit_ascii_char(self, _v: u8) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "ASCII char"))
  }

  fn visit_i8(self, _v: i8) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Byte"))
  }
  fn visit_i16(self, _v: i16) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Short"))
  }
  fn visit_i32(self, _v: i32) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Int"))
  }
  fn visit_i64(self, _v: i64) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Long"))
  }

  fn visit_opt_i8(self, _v: Option<i8>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Byte",
    ))
  }
  fn visit_opt_i16(self, _v: Option<i16>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Short",
    ))
  }
  fn visit_opt_i32(self, _v: Option<i32>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Nullable Int"))
  }
  fn visit_opt_i64(self, _v: Option<i64>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Long",
    ))
  }

  fn visit_u8(self, _v: u8) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Byte",
    ))
  }
  fn visit_u16(self, _v: u16) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Short",
    ))
  }
  fn visit_u32(self, _v: u32) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Unsigned Int"))
  }
  fn visit_u64(self, _v: u64) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Long",
    ))
  }

  fn visit_opt_u8(self, _v: Option<u8>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Byte",
    ))
  }
  fn visit_opt_u16(self, _v: Option<u16>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Short",
    ))
  }
  fn visit_opt_u32(self, _v: Option<u32>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Int",
    ))
  }
  fn visit_opt_u64(self, _v: Option<u64>) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Long",
    ))
  }

  fn visit_f32(self, _v: f32) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Float"))
  }

  fn visit_f64(self, _v: f64) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Double"))
  }

  fn visit_cf32(self, _v: ComplexF32) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Cmoplex Float",
    ))
  }

  fn visit_cf64(self, _v: ComplexF64) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Complex Double",
    ))
  }

  // Provide the number of bits to be read in the n bytes?
  fn visit_bit_array(self, _v: &[u8]) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "Bit array"))
  }

  fn visit_ascii_string(self, _v: &str) -> Result<Self::Value, Error> {
    Err(new_unsupported_by_visitor(self.expecting(), "ASCII String"))
  }

  fn visit_opt_bool_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<bool>>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Bool array"))
  }

  fn visit_i8_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i8>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Byte array"))
  }

  fn visit_i16_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i16>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Short array"))
  }

  fn visit_i32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i32>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Int array"))
  }

  fn visit_i64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i64>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Long array"))
  }

  fn visit_opt_i8_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i8>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Byte array",
    ))
  }

  fn visit_opt_i16_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i16>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Short array",
    ))
  }

  fn visit_opt_i32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i32>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Int array",
    ))
  }

  fn visit_opt_i64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i64>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Long array",
    ))
  }

  fn visit_u8_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u8>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Byte array",
    ))
  }

  fn visit_u16_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u16>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Short array",
    ))
  }

  fn visit_u32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u32>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Int array",
    ))
  }

  fn visit_u64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u64>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Unsigned Long array",
    ))
  }

  fn visit_opt_u8_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u8>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Byte array",
    ))
  }

  fn visit_opt_u16_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u16>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Short array",
    ))
  }

  fn visit_opt_u32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u32>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Int array",
    ))
  }

  fn visit_opt_u64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u64>>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Nullable Unsigned Long array",
    ))
  }

  fn visit_f32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f32>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Float array"))
  }

  fn visit_f64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f64>,
  {
    Err(new_unsupported_by_visitor(self.expecting(), "Double array"))
  }

  fn visit_cf32_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF32>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Complex float array",
    ))
  }

  fn visit_cf64_array<I>(self, _it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF64>,
  {
    Err(new_unsupported_by_visitor(
      self.expecting(),
      "Complex double array",
    ))
  }
}
