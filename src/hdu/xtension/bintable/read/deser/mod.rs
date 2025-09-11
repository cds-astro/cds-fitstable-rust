use crate::error::Error;

use super::visitor::Visitor;
pub mod sliceheap;

pub trait Deserialize<'de>: Sized {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, Error>
  where
    D: Deserializer<'de>;
}

/// Similar to Serde DeserializeSeed, except that here wa pass a visitor and the returned
/// type depends on the Visitor.
pub trait DeserializeSeed<'de>: Sized {
  /// Equivalent to the more common `Deserialize::deserialize` method, except
  /// with some initial piece of data (the seed) passed in.
  /// Also, we pass here a Visitor to take in charge array iterators.
  fn deserialize<D, V>(&self, deserializer: &mut D, visitor: V) -> Result<V::Value, Error>
  where
    D: Deserializer<'de>,
    V: Visitor;
}

/*

pub fn deserialize<'de, D, V>(&self, row_deserializer: D, visitor: V) -> Result<V::Value, Error>
  where
    D: Deserializer<'de>,
    V: Visitor,

*/
pub trait Deserializer<'de> {
  // Row deserialization

  /* fn deserialize_row<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;*/

  // Field deserialization
  fn deserialize_empty<V>(&mut self, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_bool<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_bits<V>(
    &mut self,
    from: usize,
    n_bits: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_byte<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_short<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_int<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_long<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_byte<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_short<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_int<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_long<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_byte<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_short<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_int<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_long<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_byte<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_short<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_int<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_long<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_byte<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_short<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_int<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_long<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_float<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_double<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_ascii_char<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_bool_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_float_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_double_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_ascii_string_fixed_length<V>(
    &mut self,
    n_chars: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_bool_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_float_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_double_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_ascii_string_var_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_bool_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_unsigned_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_opt_unsigned_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_float_with_scale_offset_from_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_double_with_scale_offset_from_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_float_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_complex_double_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;

  fn deserialize_ascii_string_var_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor;
}
