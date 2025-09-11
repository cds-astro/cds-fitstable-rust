use crate::{
  error::{new_unsupported_by_visitor, Error},
  hdu::xtension::bintable::field::{ComplexF32, ComplexF64, Field},
};

use super::{FieldVisitorProvider, RowVisitor, Visitor};

pub struct FieldVisitor {}

impl FieldVisitorProvider for FieldVisitor {
  type FieldValue = Field;
  type FieldVisitor<'v> = Self;

  fn field_visitor(&mut self) -> Self::FieldVisitor<'_> {
    FieldVisitor {}
  }
}

impl RowVisitor for FieldVisitor {
  type Value = Vec<Field>;
  type FieldValue = Field;

  fn visit_row<I>(self, fields_it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Result<Self::FieldValue, Error>>,
  {
    fields_it.collect()
  }
}

impl Visitor for FieldVisitor {
  type Value = Field;

  fn expecting(&self) -> &str {
    "Unreachable for Field visitor"
  }

  fn visit_empty(self) -> Result<Self::Value, Error> {
    Ok(Field::Empty)
  }

  fn visit_opt_bool(self, v: Option<bool>) -> Result<Self::Value, Error> {
    Ok(Field::NullableBoolean(v))
  }

  fn visit_ascii_char(self, v: u8) -> Result<Self::Value, Error> {
    Ok(Field::AsciiChar(v))
  }

  fn visit_i8(self, v: i8) -> Result<Self::Value, Error> {
    Ok(Field::Byte(v))
  }
  fn visit_i16(self, v: i16) -> Result<Self::Value, Error> {
    Ok(Field::Short(v))
  }
  fn visit_i32(self, v: i32) -> Result<Self::Value, Error> {
    Ok(Field::Int(v))
  }
  fn visit_i64(self, v: i64) -> Result<Self::Value, Error> {
    Ok(Field::Long(v))
  }

  fn visit_opt_i8(self, v: Option<i8>) -> Result<Self::Value, Error> {
    Ok(Field::NullableByte(v))
  }
  fn visit_opt_i16(self, v: Option<i16>) -> Result<Self::Value, Error> {
    Ok(Field::NullableShort(v))
  }
  fn visit_opt_i32(self, v: Option<i32>) -> Result<Self::Value, Error> {
    Ok(Field::NullableInt(v))
  }
  fn visit_opt_i64(self, v: Option<i64>) -> Result<Self::Value, Error> {
    Ok(Field::NullableLong(v))
  }

  fn visit_u8(self, v: u8) -> Result<Self::Value, Error> {
    Ok(Field::UnsignedByte(v))
  }
  fn visit_u16(self, v: u16) -> Result<Self::Value, Error> {
    Ok(Field::UnsignedShort(v))
  }
  fn visit_u32(self, v: u32) -> Result<Self::Value, Error> {
    Ok(Field::UnsignedInt(v))
  }
  fn visit_u64(self, v: u64) -> Result<Self::Value, Error> {
    Ok(Field::UnsignedLong(v))
  }

  fn visit_opt_u8(self, v: Option<u8>) -> Result<Self::Value, Error> {
    Ok(Field::NullableUnsignedByte(v))
  }
  fn visit_opt_u16(self, v: Option<u16>) -> Result<Self::Value, Error> {
    Ok(Field::NullableUnsignedShort(v))
  }
  fn visit_opt_u32(self, v: Option<u32>) -> Result<Self::Value, Error> {
    Ok(Field::NullableUnsignedInt(v))
  }
  fn visit_opt_u64(self, v: Option<u64>) -> Result<Self::Value, Error> {
    Ok(Field::NullableUnsignedLong(v))
  }

  fn visit_f32(self, v: f32) -> Result<Self::Value, Error> {
    Ok(Field::Float(v))
  }

  fn visit_f64(self, v: f64) -> Result<Self::Value, Error> {
    Ok(Field::Double(v))
  }

  fn visit_cf32(self, v: ComplexF32) -> Result<Self::Value, Error> {
    Ok(Field::ComplexFloat(v))
  }

  fn visit_cf64(self, v: ComplexF64) -> Result<Self::Value, Error> {
    Ok(Field::ComplexDouble(v))
  }

  // Provide the number of bits to be read in the n bytes?
  fn visit_bit_array(self, v: &[u8]) -> Result<Self::Value, Error> {
    // TODO!
    Err(new_unsupported_by_visitor(self.expecting(), "Bit array"))
  }

  fn visit_ascii_string(self, v: &str) -> Result<Self::Value, Error> {
    Ok(Field::AsciiString(v.to_owned()))
  }

  fn visit_opt_bool_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<bool>>,
  {
    Ok(Field::NullableBooleanArray(it.collect()))
  }

  fn visit_i8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i8>,
  {
    Ok(Field::ByteArray(it.collect()))
  }

  fn visit_i16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i16>,
  {
    Ok(Field::ShortArray(it.collect()))
  }

  fn visit_i32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i32>,
  {
    Ok(Field::IntArray(it.collect()))
  }

  fn visit_i64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i64>,
  {
    Ok(Field::LongArray(it.collect()))
  }

  fn visit_opt_i8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i8>>,
  {
    Ok(Field::NullableByteArray(it.collect()))
  }

  fn visit_opt_i16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i16>>,
  {
    Ok(Field::NullableShortArray(it.collect()))
  }

  fn visit_opt_i32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i32>>,
  {
    Ok(Field::NullableIntArray(it.collect()))
  }

  fn visit_opt_i64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i64>>,
  {
    Ok(Field::NullableLongArray(it.collect()))
  }

  fn visit_u8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u8>,
  {
    Ok(Field::UnsignedByteArray(it.collect()))
  }

  fn visit_u16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u16>,
  {
    Ok(Field::UnsignedShortArray(it.collect()))
  }

  fn visit_u32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u32>,
  {
    Ok(Field::UnsignedIntArray(it.collect()))
  }

  fn visit_u64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u64>,
  {
    Ok(Field::UnsignedLongArray(it.collect()))
  }

  fn visit_opt_u8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u8>>,
  {
    Ok(Field::NullableUnsignedByteArray(it.collect()))
  }

  fn visit_opt_u16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u16>>,
  {
    Ok(Field::NullableUnsignedShortArray(it.collect()))
  }

  fn visit_opt_u32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u32>>,
  {
    Ok(Field::NullableUnsignedIntArray(it.collect()))
  }

  fn visit_opt_u64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u64>>,
  {
    Ok(Field::NullableUnsignedLongArray(it.collect()))
  }

  fn visit_f32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f32>,
  {
    Ok(Field::FloatArray(it.collect()))
  }

  fn visit_f64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f64>,
  {
    Ok(Field::DoubleArray(it.collect()))
  }

  fn visit_cf32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF32>,
  {
    Ok(Field::ComplexFloatArray(it.collect()))
  }

  fn visit_cf64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF64>,
  {
    Ok(Field::ComplexDoubleArray(it.collect()))
  }
}
