use crate::error::Error;
use std::fmt::{Display, Formatter};

use super::read::{
  deser::{DeserializeSeed, Deserializer},
  visitor::{FieldVisitorProvider, RowVisitor, Visitor},
};

#[derive(Debug, Clone)]
pub struct RowSchema {
  fields_schemas: Vec<FieldSchema>,
}
impl RowSchema {
  pub fn n_cols(&self) -> usize {
    self.fields_schemas.len()
  }

  pub fn fields_schemas(&self) -> &[FieldSchema] {
    self.fields_schemas.as_slice()
  }

  pub fn deserialize<'de, D, F, R>(
    &self,
    deserializer: &mut D,
    field_visitor_provider: &mut F,
    row_visitor: R,
  ) -> Result<R::Value, Error>
  where
    D: Deserializer<'de>,
    F: FieldVisitorProvider,
    R: RowVisitor<FieldValue = F::FieldValue>,
  {
    row_visitor.visit_row(self.fields_schemas.iter().map(|field_schema| {
      field_schema.deserialize(deserializer, field_visitor_provider.field_visitor())
    }))
  }
}
/// So we can use `collect()` to build a `RowSchema`!
impl FromIterator<Schema> for RowSchema {
  fn from_iter<T: IntoIterator<Item = Schema>>(iter: T) -> Self {
    let mut byte_offset = 0_usize;
    let fields_schemas = iter
      .into_iter()
      .map(|schema| {
        let field_schema = FieldSchema::new(byte_offset, schema);
        byte_offset += field_schema.schema.stored_byte_len();
        field_schema
      })
      .collect();
    Self { fields_schemas }
  }
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
  /// The starting byte of the field in the stored row bytes.
  starting_byte: usize,
  /// The schema of the field.
  schema: Schema,
}
impl FieldSchema {
  pub fn new(byte_offset: usize, field_schema: Schema) -> Self {
    Self {
      starting_byte: byte_offset,
      schema: field_schema,
    }
  }
}
impl<'de, 'a> DeserializeSeed<'de> for FieldSchema {
  fn deserialize<D, V>(&self, deserializer: &mut D, visitor: V) -> Result<V::Value, Error>
  where
    D: Deserializer<'de>,
    V: Visitor,
  {
    let de = deserializer;
    let v = visitor;
    let from = self.starting_byte;
    match &self.schema {
      Schema::Empty => de.deserialize_empty(v),
      Schema::NullableBoolean => de.deserialize_opt_bool(from, v),
      Schema::Bits { n_bits } => de.deserialize_bits(from, *n_bits, v),
      Schema::Byte => de.deserialize_byte(from, v),
      Schema::Short => de.deserialize_short(from, v),
      Schema::Int => de.deserialize_int(from, v),
      Schema::Long => de.deserialize_long(from, v),
      Schema::NullableByte { null } => de.deserialize_opt_byte(from, *null, v),
      Schema::NullableShort { null } => de.deserialize_opt_short(from, *null, v),
      Schema::NullableInt { null } => de.deserialize_opt_int(from, *null, v),
      Schema::NullableLong { null } => de.deserialize_opt_long(from, *null, v),
      Schema::UnsignedByte => de.deserialize_unsigned_byte(from, v),
      Schema::UnsignedShort => de.deserialize_unsigned_short(from, v),
      Schema::UnsignedInt => de.deserialize_unsigned_int(from, v),
      Schema::UnsignedLong => de.deserialize_unsigned_long(from, v),
      Schema::NullableUnsignedByte { null } => de.deserialize_opt_unsigned_byte(from, *null, v),
      Schema::NullableUnsignedShort { null } => de.deserialize_opt_unsigned_short(from, *null, v),
      Schema::NullableUnsignedInt { null } => de.deserialize_opt_unsigned_int(from, *null, v),
      Schema::NullableUnsignedLong { null } => de.deserialize_opt_unsigned_long(from, *null, v),
      Schema::Float => de.deserialize_float(from, v),
      Schema::FloatFromFloat(p) => {
        de.deserialize_float_with_scale_offset(from, p.scale, p.offset, v)
      }
      Schema::FloatFromByte(p) => {
        de.deserialize_float_with_scale_offset_from_byte(from, p.scale, p.offset, v)
      }
      Schema::FloatFromShort(p) => {
        de.deserialize_float_with_scale_offset_from_short(from, p.scale, p.offset, v)
      }
      Schema::Double => de.deserialize_double(from, v),
      Schema::DoubleFromDouble(p) => {
        de.deserialize_double_with_scale_offset(from, p.scale, p.offset, v)
      }
      Schema::DoubleFromInt(p) => {
        de.deserialize_double_with_scale_offset_from_int(from, p.scale, p.offset, v)
      }
      Schema::DoubleFromLong(p) => {
        de.deserialize_double_with_scale_offset_from_long(from, p.scale, p.offset, v)
      }
      Schema::ComplexFloat => de.deserialize_complex_float(from, v),
      Schema::ComplexDouble => de.deserialize_complex_double(from, v),
      Schema::AsciiChar => de.deserialize_ascii_char(from, v),
      Schema::NullableBooleanArray(p) => de.deserialize_opt_bool_array(p.len, from, v),
      Schema::ByteArray(p) => de.deserialize_byte_array(p.len, from, v),
      Schema::ShortArray(p) => de.deserialize_short_array(p.len, from, v),
      Schema::IntArray(p) => de.deserialize_int_array(p.len, from, v),
      Schema::LongArray(p) => de.deserialize_long_array(p.len, from, v),
      Schema::NullableByteArray { null, p } => de.deserialize_opt_byte_array(p.len, from, *null, v),
      Schema::NullableShortArray { null, p } => {
        de.deserialize_opt_short_array(p.len, from, *null, v)
      }
      Schema::NullableIntArray { null, p } => de.deserialize_opt_int_array(p.len, from, *null, v),
      Schema::NullableLongArray { null, p } => de.deserialize_opt_long_array(p.len, from, *null, v),
      Schema::UnsignedByteArray(p) => de.deserialize_unsigned_byte_array(p.len, from, v),
      Schema::UnsignedShortArray(p) => de.deserialize_unsigned_short_array(p.len, from, v),
      Schema::UnsignedIntArray(p) => de.deserialize_unsigned_int_array(p.len, from, v),
      Schema::UnsignedLongArray(p) => de.deserialize_unsigned_long_array(p.len, from, v),
      Schema::NullableUnsignedByteArray { null, p } => {
        de.deserialize_opt_unsigned_byte_array(p.len, from, *null, v)
      }
      Schema::NullableUnsignedShortArray { null, p } => {
        de.deserialize_opt_unsigned_short_array(p.len, from, *null, v)
      }
      Schema::NullableUnsignedIntArray { null, p } => {
        de.deserialize_opt_unsigned_int_array(p.len, from, *null, v)
      }
      Schema::NullableUnsignedLongArray { null, p } => {
        de.deserialize_opt_unsigned_long_array(p.len, from, *null, v)
      }
      Schema::FloatArray(p) => de.deserialize_float_array(p.len, from, v),
      Schema::FloatArrayFromFloat(p) => de.deserialize_float_with_scale_offset_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::FloatArrayFromBytes(p) => de.deserialize_float_with_scale_offset_from_byte_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::FloatArrayFromShort(p) => de.deserialize_float_with_scale_offset_from_short_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::DoubleArray(p) => de.deserialize_double_array(p.len, from, v),
      Schema::DoubleArrayFromDouble(p) => de.deserialize_double_with_scale_offset_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::DoubleArrayFromInt(p) => de.deserialize_double_with_scale_offset_from_int_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::DoubleArrayFromLong(p) => de.deserialize_double_with_scale_offset_from_long_array(
        p.array_params.len,
        from,
        p.scale_offset.scale,
        p.scale_offset.offset,
        v,
      ),
      Schema::ComplexFloatArray(p) => de.deserialize_complex_float_array(p.len, from, v),
      Schema::ComplexDoubleArray(p) => de.deserialize_complex_double_array(p.len, from, v),
      Schema::AsciiString(p) => de.deserialize_ascii_string_fixed_length(p.len, from, v),
      Schema::HeapArrayPtr32(has) => match has {
        HeapArraySchema::HeapNullableBooleanArray(p) => {
          de.deserialize_opt_bool_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapByteArray(_) => de.deserialize_byte_vararray_ptr32(from, v),
        HeapArraySchema::HeapShortArray(_) => de.deserialize_short_vararray_ptr32(from, v),
        HeapArraySchema::HeapIntArray(_) => de.deserialize_int_vararray_ptr32(from, v),
        HeapArraySchema::HeapLongArray(_) => de.deserialize_long_vararray_ptr32(from, v),
        HeapArraySchema::HeapNullableByteArray { null, hap: _ } => {
          de.deserialize_opt_byte_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableShortArray { null, hap: _ } => {
          de.deserialize_opt_short_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableIntArray { null, hap: _ } => {
          de.deserialize_opt_int_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableLongArray { null, hap: _ } => {
          de.deserialize_opt_long_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapUnsignedByteArray(_) => {
          de.deserialize_unsigned_byte_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapUnsignedShortArray(_) => {
          de.deserialize_unsigned_short_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapUnsignedIntArray(_) => {
          de.deserialize_unsigned_int_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapUnsignedLongArray(_) => {
          de.deserialize_unsigned_long_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapNullableUnsignedByteArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_byte_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedShortArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_short_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedIntArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_int_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedLongArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_long_vararray_ptr32(from, *null, v)
        }
        HeapArraySchema::HeapFloatArray(_p) => de.deserialize_float_vararray_ptr32(from, v),
        HeapArraySchema::HeapFloatArrayFromFloat(p) => de
          .deserialize_float_with_scale_offset_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapFloatArrayFromByte(p) => de
          .deserialize_float_with_scale_offset_from_byte_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapFloatArrayFromShort(p) => de
          .deserialize_float_with_scale_offset_from_short_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArray(_p) => de.deserialize_double_vararray_ptr32(from, v),
        HeapArraySchema::HeapDoubleArrayFromDouble(p) => de
          .deserialize_double_with_scale_offset_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArrayFromInt(p) => de
          .deserialize_double_with_scale_offset_from_int_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArrayFromLong(p) => de
          .deserialize_double_with_scale_offset_from_long_vararray_ptr32(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapComplexFloatArray(_p) => {
          de.deserialize_complex_float_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapComplexDoubleArray(_p) => {
          de.deserialize_complex_double_vararray_ptr32(from, v)
        }
        HeapArraySchema::HeapAsciiString(_p) => de.deserialize_ascii_string_var_ptr32(from, v),
      },
      Schema::HeapArrayPtr64(has) => match has {
        HeapArraySchema::HeapNullableBooleanArray(_p) => {
          de.deserialize_opt_bool_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapByteArray(_) => de.deserialize_byte_vararray_ptr64(from, v),
        HeapArraySchema::HeapShortArray(_) => de.deserialize_short_vararray_ptr64(from, v),
        HeapArraySchema::HeapIntArray(_) => de.deserialize_int_vararray_ptr64(from, v),
        HeapArraySchema::HeapLongArray(_) => de.deserialize_long_vararray_ptr64(from, v),
        HeapArraySchema::HeapNullableByteArray { null, hap: _ } => {
          de.deserialize_opt_byte_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableShortArray { null, hap: _ } => {
          de.deserialize_opt_short_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableIntArray { null, hap: _ } => {
          de.deserialize_opt_int_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableLongArray { null, hap: _ } => {
          de.deserialize_opt_long_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapUnsignedByteArray(_) => {
          de.deserialize_unsigned_byte_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapUnsignedShortArray(_) => {
          de.deserialize_unsigned_short_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapUnsignedIntArray(_) => {
          de.deserialize_unsigned_int_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapUnsignedLongArray(_) => {
          de.deserialize_unsigned_long_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapNullableUnsignedByteArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_byte_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedShortArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_short_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedIntArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_int_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapNullableUnsignedLongArray { null, hap: _ } => {
          de.deserialize_opt_unsigned_long_vararray_ptr64(from, *null, v)
        }
        HeapArraySchema::HeapFloatArray(_p) => de.deserialize_float_vararray_ptr64(from, v),
        HeapArraySchema::HeapFloatArrayFromFloat(p) => de
          .deserialize_float_with_scale_offset_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapFloatArrayFromByte(p) => de
          .deserialize_float_with_scale_offset_from_byte_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapFloatArrayFromShort(p) => de
          .deserialize_float_with_scale_offset_from_short_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArray(_p) => de.deserialize_double_vararray_ptr64(from, v),
        HeapArraySchema::HeapDoubleArrayFromDouble(p) => de
          .deserialize_double_with_scale_offset_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArrayFromInt(p) => de
          .deserialize_double_with_scale_offset_from_int_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapDoubleArrayFromLong(p) => de
          .deserialize_double_with_scale_offset_from_long_vararray_ptr64(
            from,
            p.scale_offset.scale,
            p.scale_offset.offset,
            v,
          ),
        HeapArraySchema::HeapComplexFloatArray(_p) => {
          de.deserialize_complex_float_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapComplexDoubleArray(_p) => {
          de.deserialize_complex_double_vararray_ptr64(from, v)
        }
        HeapArraySchema::HeapAsciiString(_p) => de.deserialize_ascii_string_var_ptr64(from, v),
      },
    }
  }
}

/// Scale and offset to be used in the transformation:
/// > output_value = scale * stored_value + offset
/// for a 32-bit float
#[derive(Debug, Clone)]
pub struct ScaleOffset32 {
  scale: f32,
  offset: f32,
}
impl ScaleOffset32 {
  pub fn new(scale: f32, offset: f32) -> Self {
    Self { scale, offset }
  }
}

/// Scale and offset to be used in the transformation:
/// > output_value = scale * stored_value + offset
/// for a 64-bit float
#[derive(Debug, Clone)]
pub struct ScaleOffset64 {
  scale: f64,
  offset: f64,
}
impl ScaleOffset64 {
  pub fn new(scale: f64, offset: f64) -> Self {
    Self { scale, offset }
  }
}

/// Regular array parameter (only the length so far).
#[derive(Debug, Clone)]
pub struct ArrayParam {
  len: usize,
}
impl ArrayParam {
  pub fn new(len: usize) -> Self {
    Self { len }
  }
  pub fn with_scale_offset_32(self, scale_offset: ScaleOffset32) -> ArrayParamWithScaleOffset32 {
    ArrayParamWithScaleOffset32::new(self, scale_offset)
  }
  pub fn with_scale_offset_64(self, scale_offset: ScaleOffset64) -> ArrayParamWithScaleOffset64 {
    ArrayParamWithScaleOffset64::new(self, scale_offset)
  }
}
impl From<&HeapArrayParam> for ArrayParam {
  fn from(p: &HeapArrayParam) -> Self {
    Self::new(p.max_len)
  }
}

/// Array parameter with scaled and offset for 32-bit floats.
#[derive(Debug, Clone)]
pub struct ArrayParamWithScaleOffset32 {
  array_params: ArrayParam,
  scale_offset: ScaleOffset32,
}
impl ArrayParamWithScaleOffset32 {
  pub fn new(array_params: ArrayParam, scale_offset: ScaleOffset32) -> Self {
    Self {
      array_params,
      scale_offset,
    }
  }
}
impl From<&HeapArrayParamWithScaleOffset32> for ArrayParamWithScaleOffset32 {
  fn from(p: &HeapArrayParamWithScaleOffset32) -> Self {
    Self::new((&p.heap_params).into(), p.scale_offset.clone())
  }
}

/// Array parameter with scaled and offset for 64-bit floats.
#[derive(Debug, Clone)]
pub struct ArrayParamWithScaleOffset64 {
  array_params: ArrayParam,
  scale_offset: ScaleOffset64,
}
impl ArrayParamWithScaleOffset64 {
  pub fn new(array_params: ArrayParam, scale_offset: ScaleOffset64) -> Self {
    Self {
      array_params,
      scale_offset,
    }
  }
}
impl From<&HeapArrayParamWithScaleOffset64> for ArrayParamWithScaleOffset64 {
  fn from(p: &HeapArrayParamWithScaleOffset64) -> Self {
    Self::new((&p.heap_params).into(), p.scale_offset.clone())
  }
}

/// Variable length array parameter (only the length so far).
#[derive(Debug, Clone)]
pub struct HeapArrayParam {
  /// Upper bound on the stored array size.
  max_len: usize,
}
impl HeapArrayParam {
  pub fn new(max_len: usize) -> Self {
    Self { max_len }
  }
  pub fn with_scale_offset_32(
    self,
    scale_offset: ScaleOffset32,
  ) -> HeapArrayParamWithScaleOffset32 {
    HeapArrayParamWithScaleOffset32::new(self, scale_offset)
  }
  pub fn with_scale_offset_64(
    self,
    scale_offset: ScaleOffset64,
  ) -> HeapArrayParamWithScaleOffset64 {
    HeapArrayParamWithScaleOffset64::new(self, scale_offset)
  }
}
impl From<&ArrayParam> for HeapArrayParam {
  fn from(array_param: &ArrayParam) -> Self {
    Self::new(array_param.len)
  }
}

/// Variable length array parameter with scaled and offset for 32-bit floats.
#[derive(Debug, Clone)]
pub struct HeapArrayParamWithScaleOffset32 {
  heap_params: HeapArrayParam,
  scale_offset: ScaleOffset32,
}
impl HeapArrayParamWithScaleOffset32 {
  pub fn new(heap_params: HeapArrayParam, scale_offset: ScaleOffset32) -> Self {
    Self {
      heap_params,
      scale_offset,
    }
  }
}

/// Variable length array parameter with scaled and offset for 64-bit floats.
#[derive(Debug, Clone)]
pub struct HeapArrayParamWithScaleOffset64 {
  heap_params: HeapArrayParam,
  scale_offset: ScaleOffset64,
}
impl HeapArrayParamWithScaleOffset64 {
  pub fn new(heap_params: HeapArrayParam, scale_offset: ScaleOffset64) -> Self {
    Self {
      heap_params,
      scale_offset,
    }
  }
}

/// Represent both the logical and the storage data type information
/// (and how to convert the storage type to the logical type and conversely).
#[derive(Debug, Clone)]
pub enum Schema {
  // When repeat count = 0
  Empty,

  // Bool
  NullableBoolean,
  Bits { n_bits: usize },
  // Signed integer
  Byte,
  Short,
  Int,
  Long,
  NullableByte { null: u8 },
  NullableShort { null: i16 },
  NullableInt { null: i32 },
  NullableLong { null: i64 },
  // Unsigned Integer
  UnsignedByte,
  UnsignedShort,
  UnsignedInt,
  UnsignedLong,
  NullableUnsignedByte { null: u8 },
  NullableUnsignedShort { null: i16 },
  NullableUnsignedInt { null: i32 },
  NullableUnsignedLong { null: i64 },
  // Real
  Float,
  FloatFromFloat(ScaleOffset32),
  FloatFromByte(ScaleOffset32),  // with null value
  FloatFromShort(ScaleOffset32), // with null value
  Double,
  DoubleFromDouble(ScaleOffset64),
  DoubleFromInt(ScaleOffset64),  // with null value
  DoubleFromLong(ScaleOffset64), // with null value
  // Complex
  ComplexFloat,
  // TODO: ComplexFloatFromFloat(ScaleOffset32),
  ComplexDouble,
  // TODO: ComplexDoubleFromDouble(ScaleOffset64),
  // AsciiChar
  AsciiChar,

  // Fixed length ARRAYS //

  // Bool
  NullableBooleanArray(ArrayParam),
  // Signed integer
  ByteArray(ArrayParam),
  ShortArray(ArrayParam),
  IntArray(ArrayParam),
  LongArray(ArrayParam),
  NullableByteArray { null: u8, p: ArrayParam },
  NullableShortArray { null: i16, p: ArrayParam },
  NullableIntArray { null: i32, p: ArrayParam },
  NullableLongArray { null: i64, p: ArrayParam },
  // Unsigned Integer
  UnsignedByteArray(ArrayParam),
  UnsignedShortArray(ArrayParam),
  UnsignedIntArray(ArrayParam),
  UnsignedLongArray(ArrayParam),
  NullableUnsignedByteArray { null: u8, p: ArrayParam },
  NullableUnsignedShortArray { null: i16, p: ArrayParam },
  NullableUnsignedIntArray { null: i32, p: ArrayParam },
  NullableUnsignedLongArray { null: i64, p: ArrayParam },
  // Real
  FloatArray(ArrayParam),
  FloatArrayFromFloat(ArrayParamWithScaleOffset32),
  FloatArrayFromBytes(ArrayParamWithScaleOffset32),
  FloatArrayFromShort(ArrayParamWithScaleOffset32),
  DoubleArray(ArrayParam),
  DoubleArrayFromDouble(ArrayParamWithScaleOffset64),
  DoubleArrayFromInt(ArrayParamWithScaleOffset64),
  DoubleArrayFromLong(ArrayParamWithScaleOffset64),
  // Complex
  ComplexFloatArray(ArrayParam),
  // TODO: ComplexFloatArrayFromFloat(ArrayParamWithScaleOffset32),
  ComplexDoubleArray(ArrayParam),
  // TODO: ComplexDoubleArrayFromDouble(ArrayParamWithScaleOffset64),
  // String
  AsciiString(ArrayParam),

  // Variable length ARRAYS //
  HeapArrayPtr32(HeapArraySchema),
  HeapArrayPtr64(HeapArraySchema),
  // multi-dimension array with TDIM ?? Add TDIM to ArrayParam?
}

impl Schema {
  /// Number of byte required to store a value in the main bintable. Thus this is not:
  /// * the size in memory (e.g. a float or a double may be store on a byte or an integer).
  /// * for variable length array, this return the size of the pointer and not the (variable) size in the BINTABLE heap.
  pub fn stored_byte_len(&self) -> usize {
    match self {
      Self::Empty => 0,
      Self::NullableBoolean
      | Self::Byte
      | Self::NullableByte { .. }
      | Self::UnsignedByte
      | Self::NullableUnsignedByte { .. }
      | Self::FloatFromByte(_)
      | Self::AsciiChar => 1,
      Self::Bits { n_bits } => (n_bits + 7) / 8,
      Self::Short
      | Self::NullableShort { .. }
      | Self::UnsignedShort
      | Self::NullableUnsignedShort { .. }
      | Self::FloatFromShort(_) => 2,
      Self::Int
      | Self::NullableInt { .. }
      | Self::UnsignedInt
      | Self::NullableUnsignedInt { .. }
      | Self::Float
      | Self::FloatFromFloat(_)
      | Self::DoubleFromInt(_) => 4,
      Self::Long
      | Self::NullableLong { .. }
      | Self::UnsignedLong
      | Self::NullableUnsignedLong { .. }
      | Self::Double
      | Self::ComplexFloat
      | Self::DoubleFromDouble(_)
      | Self::DoubleFromLong(_)
      | Self::HeapArrayPtr32(_) => 8, // + all array descriptor 32bits
      Self::ComplexDouble | Self::HeapArrayPtr64(_) => 16, // + all array descriptor 64bits
      // Arrays
      Self::NullableBooleanArray(ArrayParam { len })
      | Self::ByteArray(ArrayParam { len })
      | Self::NullableByteArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::UnsignedByteArray(ArrayParam { len })
      | Self::NullableUnsignedByteArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::AsciiString(ArrayParam { len })
      | Self::FloatArrayFromBytes(ArrayParamWithScaleOffset32 {
        array_params: ArrayParam { len },
        ..
      }) => *len,
      Self::ShortArray(ArrayParam { len })
      | Self::NullableShortArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::UnsignedShortArray(ArrayParam { len })
      | Self::NullableUnsignedShortArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::FloatArrayFromShort(ArrayParamWithScaleOffset32 {
        array_params: ArrayParam { len },
        ..
      }) => 2 * *len,
      Self::IntArray(ArrayParam { len })
      | Self::NullableIntArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::UnsignedIntArray(ArrayParam { len })
      | Self::NullableUnsignedIntArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::FloatArray(ArrayParam { len })
      | Self::FloatArrayFromFloat(ArrayParamWithScaleOffset32 {
        array_params: ArrayParam { len },
        ..
      })
      | Self::DoubleArrayFromInt(ArrayParamWithScaleOffset64 {
        array_params: ArrayParam { len },
        ..
      }) => 4 * *len,
      Self::LongArray(ArrayParam { len })
      | Self::NullableLongArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::UnsignedLongArray(ArrayParam { len })
      | Self::NullableUnsignedLongArray {
        null: _,
        p: ArrayParam { len },
      }
      | Self::DoubleArray(ArrayParam { len })
      | Self::ComplexFloatArray(ArrayParam { len })
      | Self::DoubleArrayFromDouble(ArrayParamWithScaleOffset64 {
        array_params: ArrayParam { len },
        ..
      })
      | Self::DoubleArrayFromLong(ArrayParamWithScaleOffset64 {
        array_params: ArrayParam { len },
        ..
      }) => 8 * *len,
      Self::ComplexDoubleArray(ArrayParam { len }) => 16 * *len,
    }
  }
}

impl Display for Schema {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Empty => write!(f, "void"),
      Self::NullableBoolean => write!(f, "bool?",),
      Self::Bits { n_bits } => write!(f, "bits[{}]", n_bits),
      Self::Byte => write!(f, "i8"),
      Self::Short => write!(f, "i16"),
      Self::Int => write!(f, "i32"),
      Self::Long => write!(f, "i64"),
      Self::NullableByte { null: _ } => write!(f, "i8?"),
      Self::NullableShort { null: _ } => write!(f, "i16?"),
      Self::NullableInt { null: _ } => write!(f, "i32?"),
      Self::NullableLong { null: _ } => write!(f, "i64?"),
      Self::UnsignedByte => write!(f, "u8"),
      Self::UnsignedShort => write!(f, "u16"),
      Self::UnsignedInt => write!(f, "u32"),
      Self::UnsignedLong => write!(f, "u64"),
      Self::NullableUnsignedByte { null: _ } => write!(f, "u8?"),
      Self::NullableUnsignedShort { null: _ } => write!(f, "u16?"),
      Self::NullableUnsignedInt { null: _ } => write!(f, "u32?"),
      Self::NullableUnsignedLong { null: _ } => write!(f, "u64?"),
      Self::Float => write!(f, "f32"),
      Self::FloatFromFloat(_) => write!(f, "f32(f32)"),
      Self::FloatFromByte(_) => write!(f, "f32(i8)"),
      Self::FloatFromShort(_) => write!(f, "f32(i16)"),
      Self::Double => write!(f, "f64"),
      Self::DoubleFromDouble(_) => write!(f, "f64(f64)"),
      Self::DoubleFromInt(_) => write!(f, "f64(i32)"),
      Self::DoubleFromLong(_) => write!(f, "f64(i64)"),
      Self::ComplexFloat => write!(f, "C32"),
      Self::ComplexDouble => write!(f, "C64"),
      Self::AsciiChar => write!(f, "c"),
      Self::NullableBooleanArray(p) => write!(f, "bool?[{}]", p.len),
      Self::ByteArray(p) => write!(f, "i8[{}]", p.len),
      Self::ShortArray(p) => write!(f, "i16[{}]", p.len),
      Self::IntArray(p) => write!(f, "i32[{}]", p.len),
      Self::LongArray(p) => write!(f, "i64[{}]", p.len),
      Self::NullableByteArray { null: _, p } => write!(f, "i8?[{}]", p.len),
      Self::NullableShortArray { null: _, p } => write!(f, "i16?[{}]", p.len),
      Self::NullableIntArray { null: _, p } => write!(f, "i32?[{}]", p.len),
      Self::NullableLongArray { null: _, p } => write!(f, "i64?[{}]", p.len),
      Self::UnsignedByteArray(p) => write!(f, "u8[{}]", p.len),
      Self::UnsignedShortArray(p) => write!(f, "u16[{}]", p.len),
      Self::UnsignedIntArray(p) => write!(f, "u32[{}]", p.len),
      Self::UnsignedLongArray(p) => write!(f, "u64[{}]", p.len),
      Self::NullableUnsignedByteArray { null: _, p } => write!(f, "u8?[{}]", p.len),
      Self::NullableUnsignedShortArray { null: _, p } => write!(f, "u16?[{}]", p.len),
      Self::NullableUnsignedIntArray { null: _, p } => write!(f, "u32?[{}]", p.len),
      Self::NullableUnsignedLongArray { null: _, p } => write!(f, "u64?[{}]", p.len),
      Self::FloatArray(p) => write!(f, "f32[{}]", p.len),
      Self::FloatArrayFromFloat(p) => write!(f, "f32(f32)[{}]", p.array_params.len),
      Self::FloatArrayFromBytes(p) => write!(f, "f32(i8)[{}]", p.array_params.len),
      Self::FloatArrayFromShort(p) => write!(f, "f32(i16)[{}]", p.array_params.len),
      Self::DoubleArray(p) => write!(f, "f64[{}]", p.len),
      Self::DoubleArrayFromDouble(p) => write!(f, "f64(f64)[{}]", p.array_params.len),
      Self::DoubleArrayFromInt(p) => write!(f, "f64(i32)[{}]", p.array_params.len),
      Self::DoubleArrayFromLong(p) => write!(f, "f64(i64)[{}]", p.array_params.len),
      Self::ComplexFloatArray(p) => write!(f, "C32[{}]", p.len),
      Self::ComplexDoubleArray(p) => write!(f, "C64[{}]", p.len),
      Self::AsciiString(p) => write!(f, "s[{}]", p.len),
      Self::HeapArrayPtr32(hp) => write!(f, "h32({})", hp.to_string()),
      Self::HeapArrayPtr64(hp) => write!(f, "h64({})", hp.to_string()),
    }
  }
}

#[derive(Debug, Clone)]
pub enum HeapArraySchema {
  // Bool
  HeapNullableBooleanArray(HeapArrayParam),
  // Signed integer
  HeapByteArray(HeapArrayParam),
  HeapShortArray(HeapArrayParam),
  HeapIntArray(HeapArrayParam),
  HeapLongArray(HeapArrayParam),
  HeapNullableByteArray { null: u8, hap: HeapArrayParam },
  HeapNullableShortArray { null: i16, hap: HeapArrayParam },
  HeapNullableIntArray { null: i32, hap: HeapArrayParam },
  HeapNullableLongArray { null: i64, hap: HeapArrayParam },
  // Unsigned Integer
  HeapUnsignedByteArray(HeapArrayParam),
  HeapUnsignedShortArray(HeapArrayParam),
  HeapUnsignedIntArray(HeapArrayParam),
  HeapUnsignedLongArray(HeapArrayParam),
  HeapNullableUnsignedByteArray { null: u8, hap: HeapArrayParam },
  HeapNullableUnsignedShortArray { null: i16, hap: HeapArrayParam },
  HeapNullableUnsignedIntArray { null: i32, hap: HeapArrayParam },
  HeapNullableUnsignedLongArray { null: i64, hap: HeapArrayParam },
  // Real
  HeapFloatArray(HeapArrayParam),
  HeapFloatArrayFromFloat(HeapArrayParamWithScaleOffset32),
  HeapFloatArrayFromByte(HeapArrayParamWithScaleOffset32),
  HeapFloatArrayFromShort(HeapArrayParamWithScaleOffset32),
  HeapDoubleArray(HeapArrayParam),
  HeapDoubleArrayFromDouble(HeapArrayParamWithScaleOffset64),
  HeapDoubleArrayFromInt(HeapArrayParamWithScaleOffset64),
  HeapDoubleArrayFromLong(HeapArrayParamWithScaleOffset64),
  // Complex
  HeapComplexFloatArray(HeapArrayParam),
  // TODO: HeapComplexFloatArrayFromFloat(HeapArrayParamWithScaleOffset32),
  HeapComplexDoubleArray(HeapArrayParam),
  // TODO: HeapComplexDoubleArrayFromDouble(HeapArrayParamWithScaleOffset64),
  // String
  HeapAsciiString(HeapArrayParam),
}
impl Display for HeapArraySchema {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::HeapNullableBooleanArray(p) => Schema::NullableBooleanArray(p.into()).fmt(f),
      Self::HeapByteArray(p) => Schema::ByteArray(p.into()).fmt(f),
      Self::HeapShortArray(p) => Schema::ShortArray(p.into()).fmt(f),
      Self::HeapIntArray(p) => Schema::IntArray(p.into()).fmt(f),
      Self::HeapLongArray(p) => Schema::LongArray(p.into()).fmt(f),
      Self::HeapNullableByteArray { null, hap } => Schema::NullableByteArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableShortArray { null, hap } => Schema::NullableShortArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableIntArray { null, hap } => Schema::NullableIntArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableLongArray { null, hap } => Schema::NullableLongArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapUnsignedByteArray(p) => Schema::UnsignedByteArray(p.into()).fmt(f),
      Self::HeapUnsignedShortArray(p) => Schema::UnsignedShortArray(p.into()).fmt(f),
      Self::HeapUnsignedIntArray(p) => Schema::UnsignedIntArray(p.into()).fmt(f),
      Self::HeapUnsignedLongArray(p) => Schema::UnsignedLongArray(p.into()).fmt(f),
      Self::HeapNullableUnsignedByteArray { null, hap } => Schema::NullableUnsignedByteArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableUnsignedShortArray { null, hap } => Schema::NullableUnsignedShortArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableUnsignedIntArray { null, hap } => Schema::NullableUnsignedIntArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapNullableUnsignedLongArray { null, hap } => Schema::NullableUnsignedLongArray {
        null: *null,
        p: hap.into(),
      }
      .fmt(f),
      Self::HeapFloatArray(p) => Schema::FloatArray(p.into()).fmt(f),
      Self::HeapFloatArrayFromFloat(p) => Schema::FloatArrayFromFloat(p.into()).fmt(f),
      Self::HeapFloatArrayFromByte(p) => Schema::FloatArrayFromBytes(p.into()).fmt(f),
      Self::HeapFloatArrayFromShort(p) => Schema::FloatArrayFromShort(p.into()).fmt(f),
      Self::HeapDoubleArray(p) => Schema::DoubleArray(p.into()).fmt(f),
      Self::HeapDoubleArrayFromDouble(p) => Schema::DoubleArrayFromDouble(p.into()).fmt(f),
      Self::HeapDoubleArrayFromInt(p) => Schema::DoubleArrayFromInt(p.into()).fmt(f),
      Self::HeapDoubleArrayFromLong(p) => Schema::DoubleArrayFromLong(p.into()).fmt(f),
      Self::HeapComplexFloatArray(p) => Schema::ComplexFloatArray(p.into()).fmt(f),
      Self::HeapComplexDoubleArray(p) => Schema::ComplexDoubleArray(p.into()).fmt(f),
      Self::HeapAsciiString(p) => Schema::AsciiString(p.into()).fmt(f),
    }
  }
}
