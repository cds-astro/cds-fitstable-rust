use std::marker::PhantomData;

use crate::error::Error;

use super::Visitor;

/// Structure made to visit a primitive or an optional primitive.
/// Attempts to visit a primitive different from the one it as been made for will fail.
pub struct VisitorPrim<E> {
  _marker: PhantomData<E>,
}

pub fn get_visitor<E>() -> VisitorPrim<E> {
  VisitorPrim {
    _marker: PhantomData,
  }
}

macro_rules! primitive_visitor {
  ($ty:ty, $doc:tt, $method:ident) => {
    impl Visitor for VisitorPrim<$ty> {
      type Value = $ty;

      fn expecting(&self) -> &str {
        $doc
      }

      fn $method(self, v: $ty) -> Result<Self::Value, Error> {
        Ok(v)
      }
    }
  };
}

macro_rules! primitive_vec_visitor {
  (Vec<$ty:ty>, $doc:tt, $method:ident) => {
    impl Visitor for VisitorPrim<Vec<$ty>> {
      type Value = Vec<$ty>;

      fn expecting(&self) -> &str {
        $doc
      }

      fn $method<I>(self, it: I) -> Result<Self::Value, Error>
      where
        I: Iterator<Item = $ty>,
      {
        Ok(it.collect())
      }
    }
  };
}

impl Visitor for VisitorPrim<Vec<char>> {
  type Value = char;

  fn expecting(&self) -> &str {
    "u8 (char)"
  }

  fn visit_ascii_char(self, v: u8) -> Result<Self::Value, Error> {
    Ok(v as char)
  }
}

impl Visitor for VisitorPrim<String> {
  type Value = String;

  fn expecting(&self) -> &str {
    "u8 (char)"
  }

  fn visit_ascii_string(self, v: &str) -> Result<Self::Value, Error> {
    Ok(v.to_owned())
  }
}

// TODO: Create struct BitArray(Vec<u8>) with method to get bit individually...
// TODO: implement the method 'visit_bit_array' of Visitor

primitive_visitor!(u8, "u8", visit_u8);
primitive_visitor!(u16, "u16", visit_u16);
primitive_visitor!(u32, "u32", visit_u32);
primitive_visitor!(u64, "u64", visit_u64);
primitive_visitor!(i8, "i8", visit_i8);
primitive_visitor!(i16, "i16", visit_i16);
primitive_visitor!(i32, "i32", visit_i32);
primitive_visitor!(i64, "i64", visit_i64);
primitive_visitor!(f32, "f32", visit_f32);
primitive_visitor!(f64, "f64", visit_f64);

primitive_visitor!(Option<bool>, "Option<boolean>", visit_opt_bool);
primitive_visitor!(Option<u8>, "Option<u8>", visit_opt_u8);
primitive_visitor!(Option<u16>, "Option<u16>", visit_opt_u16);
primitive_visitor!(Option<u32>, "Option<u32>", visit_opt_u32);
primitive_visitor!(Option<u64>, "Option<u64>", visit_opt_u64);
primitive_visitor!(Option<i8>, "Option<i8>", visit_opt_i8);
primitive_visitor!(Option<i16>, "Option<i16>", visit_opt_i16);
primitive_visitor!(Option<i32>, "Option<i32>", visit_opt_i32);
primitive_visitor!(Option<i64>, "Option<i64>", visit_opt_i64);

primitive_vec_visitor!(Vec<u8>, "Vec<u8>", visit_u8_array);
primitive_vec_visitor!(Vec<u16>, "Vec<u16>", visit_u16_array);
primitive_vec_visitor!(Vec<u32>, "Vec<u32>", visit_u32_array);
primitive_vec_visitor!(Vec<u64>, "Vec<u64>", visit_u64_array);
primitive_vec_visitor!(Vec<i8>, "Vec<i8>", visit_i8_array);
primitive_vec_visitor!(Vec<i16>, "Vec<i16>", visit_i16_array);
primitive_vec_visitor!(Vec<i32>, "Vec<i32>", visit_i32_array);
primitive_vec_visitor!(Vec<i64>, "Vec<i64>", visit_i64_array);
primitive_vec_visitor!(Vec<f32>, "Vec<f32>", visit_f32_array);
primitive_vec_visitor!(Vec<f64>, "Vec<f64>", visit_f64_array);

primitive_vec_visitor!(Vec<Option<bool>>, "Vec<Option<bool>>", visit_opt_bool_array);
primitive_vec_visitor!(Vec<Option<u8>>, "Vec<Option<u8>>", visit_opt_u8_array);
primitive_vec_visitor!(Vec<Option<u16>>, "Vec<Option<u16>>", visit_opt_u16_array);
primitive_vec_visitor!(Vec<Option<u32>>, "Vec<Option<u32>>", visit_opt_u32_array);
primitive_vec_visitor!(Vec<Option<u64>>, "Vec<Option<u64>>", visit_opt_u64_array);
primitive_vec_visitor!(Vec<Option<i8>>, "Vec<Option<i8>>", visit_opt_i8_array);
primitive_vec_visitor!(Vec<Option<i16>>, "Vec<Option<i16>>", visit_opt_i16_array);
primitive_vec_visitor!(Vec<Option<i32>>, "Vec<Option<i32>>", visit_opt_i32_array);
primitive_vec_visitor!(Vec<Option<i64>>, "Vec<Option<i64>>", visit_opt_i64_array);
