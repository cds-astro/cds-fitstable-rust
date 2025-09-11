use crate::{
  error::Error,
  hdu::xtension::bintable::{
    field::{ComplexF32Iterator, ComplexF64Iterator},
    read::bytes::to_opt_bool,
  },
};

use super::{
  super::{
    super::field::{ComplexF32, ComplexF64},
    bytes::{to_i8, to_u16, to_u32, to_u64, Bytes},
    visitor::Visitor,
  },
  DeserializeSeed, Deserializer,
};

pub struct DeserializerWithHeap<'a> {
  row: Bytes<'a>,
  heap: Bytes<'a>,
}
impl<'a> DeserializerWithHeap<'a> {
  fn get_len_offset_ptr32(&self, from: usize) -> (usize, usize) {
    let len = self.row.read_i32(from) as usize;
    let heap_byte_offset = self.row.read_i32(from + size_of::<i32>()) as usize;
    (len, heap_byte_offset)
  }

  fn get_len_offset_ptr64(&self, from: usize) -> (usize, usize) {
    let len = self.row.read_i64(from) as usize;
    let heap_byte_offset = self.row.read_i64(from + size_of::<i64>()) as usize;
    (len, heap_byte_offset)
  }
}

impl<'de> Deserializer<'de> for DeserializerWithHeap<'de> {
  /*fn deserialize_row<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    // Here, the visitor know the row schema, and it knows what type of value it returns.
    // It (the visitor) deserialize fields one by one thanks to the RowAccess.

    struct Access<'a> {
      deserializer: &'a mut DeserializerWithHeap<'a>,
      len: usize,
    }

    impl<'de, 'a, 'b: 'a> RowAccess<'de> for Access<'a> {
      fn next_field_seed<T, V>(&mut self, seed: T, visitor: V) -> Result<Option<V::Value>, Error>
      where
        T: DeserializeSeed<'de>,
        V: Visitor,
      {
        if self.len > 0 {
          self.len -= 1;
          seed
            .deserialize(&mut *self.deserializer, visitor)
            .map(|v| Some(v))
        } else {
          Ok(None)
        }
      }

      fn size_hint(&self) -> usize {
        self.len
      }
    }

    visitor.visit_row(Access {
      deserializer: self,
      len,
    })
  }*/

  fn deserialize_empty<V>(&mut self, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_empty()
  }

  fn deserialize_opt_bool<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_bool(self.row.read_bool(from))
  }

  fn deserialize_bits<V>(
    &mut self,
    from: usize,
    n_bits: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    todo!()
  }

  fn deserialize_byte<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i8(to_i8(self.row.read_u8(from)))
  }

  fn deserialize_short<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i16(self.row.read_i16(from))
  }

  fn deserialize_int<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i32(self.row.read_i32(from))
  }

  fn deserialize_long<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i64(self.row.read_i64(from))
  }

  fn deserialize_opt_byte<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_u8(from);
    visitor.visit_opt_i8(if v != null { Some(to_i8(v)) } else { None })
  }

  fn deserialize_opt_short<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i16(from);
    visitor.visit_opt_i16(if v != null { Some(v) } else { None })
  }

  fn deserialize_opt_int<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i32(from);
    visitor.visit_opt_i32(if v != null { Some(v) } else { None })
  }

  fn deserialize_opt_long<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i64(from);
    visitor.visit_opt_i64(if v != null { Some(v) } else { None })
  }

  fn deserialize_unsigned_byte<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u8(self.row.read_u8(from))
  }

  fn deserialize_unsigned_short<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u16(to_u16(self.row.read_i16(from)))
  }

  fn deserialize_unsigned_int<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u32(to_u32(self.row.read_i32(from)))
  }

  fn deserialize_unsigned_long<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u64(to_u64(self.row.read_i64(from)))
  }

  fn deserialize_opt_unsigned_byte<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_u8(from);
    visitor.visit_opt_u8(if v != null { Some(v) } else { None })
  }

  fn deserialize_opt_unsigned_short<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i16(from);
    visitor.visit_opt_u16(if v != null { Some(to_u16(v)) } else { None })
  }

  fn deserialize_opt_unsigned_int<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i32(from);
    visitor.visit_opt_u32(if v != null { Some(to_u32(v)) } else { None })
  }

  fn deserialize_opt_unsigned_long<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let v = self.row.read_i64(from);
    visitor.visit_opt_u64(if v != null { Some(to_u64(v)) } else { None })
  }

  fn deserialize_float<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32(self.row.read_f32(from))
  }

  fn deserialize_float_with_scale_offset<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32(self.row.read_f32(from) * scale + offset)
  }

  fn deserialize_float_with_scale_offset_from_byte<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32(self.row.read_u8(from) as f32 * scale + offset)
  }

  fn deserialize_float_with_scale_offset_from_short<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32(self.row.read_i16(from) as f32 * scale + offset)
  }

  fn deserialize_double<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64(self.row.read_f64(from))
  }

  fn deserialize_double_with_scale_offset<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64(self.row.read_f64(from) * scale + offset)
  }

  fn deserialize_double_with_scale_offset_from_int<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64(self.row.read_i32(from) as f64 * scale + offset)
  }

  fn deserialize_double_with_scale_offset_from_long<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64(self.row.read_i64(from) as f64 * scale + offset)
  }

  fn deserialize_complex_float<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let real = self.row.read_f32(from);
    let img = self.row.read_f32(from + size_of::<f32>());
    visitor.visit_cf32(ComplexF32::new(real, img))
  }

  fn deserialize_complex_double<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let real = self.row.read_f64(from);
    let img = self.row.read_f64(from + size_of::<f64>());
    visitor.visit_cf64(ComplexF64::new(real, img))
  }

  fn deserialize_ascii_char<V>(&mut self, from: usize, visitor: V) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_ascii_char(self.row.read_u8(from))
  }

  fn deserialize_opt_bool_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_bool_array(
      self
        .row
        .read_n_bytes(from, len)
        .iter()
        .cloned()
        .map(to_opt_bool),
    )
  }

  fn deserialize_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i8_array(self.row.read_n_bytes(from, len).iter().cloned().map(to_i8))
  }

  fn deserialize_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i16_array(self.row.read_i16_array(from, len))
  }

  fn deserialize_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i32_array(self.row.read_i32_array(from, len))
  }

  fn deserialize_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_i64_array(self.row.read_i64_array(from, len))
  }

  fn deserialize_opt_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_i8_array(self.row.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(to_i8(*v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_i16_array(self.row.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_i32_array(self.row.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_i64_array(self.row.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_unsigned_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u8_array(self.row.read_n_bytes(from, len).iter().cloned())
  }

  fn deserialize_unsigned_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u16_array(self.row.read_i16_array(from, len).map(to_u16))
  }

  fn deserialize_unsigned_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u32_array(self.row.read_i32_array(from, len).map(to_u32))
  }

  fn deserialize_unsigned_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_u64_array(self.row.read_i64_array(from, len).map(to_u64))
  }

  fn deserialize_opt_unsigned_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_u8_array(self.row.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(*v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_u16_array(self.row.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(to_u16(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_u32_array(self.row.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(to_u32(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_opt_u64_array(self.row.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(to_u64(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_float_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32_array(self.row.read_f32_array(from, len))
  }

  fn deserialize_float_with_scale_offset_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32_array(
      self
        .row
        .read_f32_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_byte_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32_array(
      self
        .row
        .read_n_bytes(from, len)
        .iter()
        .map(|v| *v as f32 * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_short_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f32_array(
      self
        .row
        .read_i16_array(from, len)
        .map(|v| v as f32 * scale + offset),
    )
  }

  fn deserialize_double_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64_array(self.row.read_f64_array(from, len))
  }

  fn deserialize_double_with_scale_offset_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64_array(
      self
        .row
        .read_f64_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_int_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64_array(
      self
        .row
        .read_i32_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_long_array<V>(
    &mut self,
    len: usize,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_f64_array(
      self
        .row
        .read_i64_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_complex_float_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_cf32_array(ComplexF32Iterator::new(
      self.row.read_f32_array(from, len << 1),
    ))
  }

  fn deserialize_complex_double_array<V>(
    &mut self,
    len: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_cf64_array(ComplexF64Iterator::new(
      self.row.read_f64_array(from, len << 1),
    ))
  }

  fn deserialize_ascii_string_fixed_length<V>(
    &mut self,
    n_chars: usize,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    visitor.visit_ascii_string(self.row.read_string(from, n_chars))
  }

  fn deserialize_opt_bool_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_bool_array(
      self
        .heap
        .read_n_bytes(from, len)
        .iter()
        .cloned()
        .map(to_opt_bool),
    )
  }

  fn deserialize_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_i8_array(self.heap.read_n_bytes(from, len).iter().cloned().map(to_i8))
  }

  fn deserialize_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_i16_array(self.heap.read_i16_array(from, len))
  }

  fn deserialize_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_i32_array(self.heap.read_i32_array(from, len))
  }

  fn deserialize_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_i64_array(self.heap.read_i64_array(from, len))
  }

  fn deserialize_opt_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_i8_array(self.heap.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(to_i8(*v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_i16_array(self.heap.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_i32_array(self.heap.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_i64_array(self.heap.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_unsigned_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_u8_array(self.heap.read_n_bytes(from, len).iter().cloned())
  }

  fn deserialize_unsigned_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_u16_array(self.heap.read_i16_array(from, len).map(to_u16))
  }

  fn deserialize_unsigned_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_u32_array(self.heap.read_i32_array(from, len).map(to_u32))
  }

  fn deserialize_unsigned_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_u64_array(self.heap.read_i64_array(from, len).map(to_u64))
  }

  fn deserialize_opt_unsigned_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_u8_array(self.heap.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(*v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_u16_array(self.heap.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(to_u16(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_u32_array(self.heap.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(to_u32(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_opt_u64_array(self.heap.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(to_u64(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_float_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f32_array(self.heap.read_f32_array(from, len))
  }

  fn deserialize_float_with_scale_offset_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_f32_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_byte_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_n_bytes(from, len)
        .iter()
        .map(|v| *v as f32 * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_short_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_i16_array(from, len)
        .map(|v| v as f32 * scale + offset),
    )
  }

  fn deserialize_double_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f64_array(self.heap.read_f64_array(from, len))
  }

  fn deserialize_double_with_scale_offset_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_f64_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_int_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_i32_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_long_vararray_ptr32<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_i64_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_complex_float_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_cf32_array(ComplexF32Iterator::new(
      self.heap.read_f32_array(from, len << 1),
    ))
  }

  fn deserialize_complex_double_vararray_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_cf64_array(ComplexF64Iterator::new(
      self.heap.read_f64_array(from, len << 1),
    ))
  }

  fn deserialize_ascii_string_var_ptr32<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr32(from);
    visitor.visit_ascii_string(self.heap.read_string(from, len))
  }

  fn deserialize_opt_bool_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_bool_array(
      self
        .heap
        .read_n_bytes(from, len)
        .iter()
        .cloned()
        .map(to_opt_bool),
    )
  }

  fn deserialize_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_i8_array(self.heap.read_n_bytes(from, len).iter().cloned().map(to_i8))
  }

  fn deserialize_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_i16_array(self.heap.read_i16_array(from, len))
  }

  fn deserialize_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_i32_array(self.heap.read_i32_array(from, len))
  }

  fn deserialize_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_i64_array(self.heap.read_i64_array(from, len))
  }

  fn deserialize_opt_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_i8_array(self.heap.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(to_i8(*v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_i16_array(self.heap.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_i32_array(self.heap.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_i64_array(self.heap.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(v)
      } else {
        None
      }
    }))
  }

  fn deserialize_unsigned_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_u8_array(self.heap.read_n_bytes(from, len).iter().cloned())
  }

  fn deserialize_unsigned_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_u16_array(self.heap.read_i16_array(from, len).map(to_u16))
  }

  fn deserialize_unsigned_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_u32_array(self.heap.read_i32_array(from, len).map(to_u32))
  }

  fn deserialize_unsigned_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_u64_array(self.heap.read_i64_array(from, len).map(to_u64))
  }

  fn deserialize_opt_unsigned_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: u8,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_u8_array(self.heap.read_n_bytes(from, len).iter().map(|v| {
      if *v != null {
        Some(*v)
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i16,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_u16_array(self.heap.read_i16_array(from, len).map(|v| {
      if v != null {
        Some(to_u16(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_u32_array(self.heap.read_i32_array(from, len).map(|v| {
      if v != null {
        Some(to_u32(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_opt_unsigned_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    null: i64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_opt_u64_array(self.heap.read_i64_array(from, len).map(|v| {
      if v != null {
        Some(to_u64(v))
      } else {
        None
      }
    }))
  }

  fn deserialize_float_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f32_array(self.heap.read_f32_array(from, len))
  }

  fn deserialize_float_with_scale_offset_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_f32_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_byte_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_n_bytes(from, len)
        .iter()
        .map(|v| *v as f32 * scale + offset),
    )
  }

  fn deserialize_float_with_scale_offset_from_short_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f32,
    offset: f32,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f32_array(
      self
        .heap
        .read_i16_array(from, len)
        .map(|v| v as f32 * scale + offset),
    )
  }

  fn deserialize_double_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f64_array(self.heap.read_f64_array(from, len))
  }

  fn deserialize_double_with_scale_offset_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_f64_array(from, len)
        .map(|v| v * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_int_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_i32_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_double_with_scale_offset_from_long_vararray_ptr64<V>(
    &mut self,
    from: usize,
    scale: f64,
    offset: f64,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_f64_array(
      self
        .heap
        .read_i64_array(from, len)
        .map(|v| v as f64 * scale + offset),
    )
  }

  fn deserialize_complex_float_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_cf32_array(ComplexF32Iterator::new(
      self.heap.read_f32_array(from, len << 1),
    ))
  }

  fn deserialize_complex_double_vararray_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_cf64_array(ComplexF64Iterator::new(
      self.heap.read_f64_array(from, len << 1),
    ))
  }

  fn deserialize_ascii_string_var_ptr64<V>(
    &mut self,
    from: usize,
    visitor: V,
  ) -> Result<V::Value, Error>
  where
    V: Visitor,
  {
    let (len, from) = self.get_len_offset_ptr64(from);
    visitor.visit_ascii_string(self.heap.read_string(from, len))
  }
}
