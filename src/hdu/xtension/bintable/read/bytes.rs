//! Read primitives (or iterator of primitive) from raw bytes.

use std::{mem::size_of, ptr::copy_nonoverlapping};

use log::error;

pub const fn to_i8(v: u8) -> i8 {
  // We use wrapping_add not to panic in debug mode
  (-128_i8).wrapping_add(v as i8)
}
pub const fn to_u16(v: i16) -> u16 {
  // We use wrapping_add not to panic in debug mode
  32_768_u16.wrapping_add(v as u16)
}
pub const fn to_u32(v: i32) -> u32 {
  // We use wrapping_add not to panic in debug mode
  2_147_483_648_u32.wrapping_add(v as u32)
}
pub const fn to_u64(v: i64) -> u64 {
  // We use wrapping_add not to panic in debug mode
  9_223_372_036_854_775_808_u64.wrapping_add(v as u64)
}

pub fn to_opt_bool(v: u8) -> Option<bool> {
  match v {
    b'T' => Some(true),
    b'F' => Some(false),
    b'0' => None,
    o => {
      error!(
        "Wrong boolean value. Expected: T, F or 0. Found: {}. Value set to NULL.",
        o as char
      );
      None
    }
  }
}

/// Simple tuple struct to provide methods to read stored (not logical) primitives from a byte slice.
/// This is low level and we do not implement methods such as `read_u32` because
/// of the possible null value which must be tested on the `i32`, thus before the transformation
/// into a `u32`.
/// Conversion and operation assigning NULL value are made at a higher level.
pub struct Bytes<'a>(&'a [u8]);

/// Implementation made to read storage types only.
/// Possible conversions are made at the row deserializer level.
impl<'a> Bytes<'a> {
  pub fn new(slice: &'a [u8]) -> Self {
    Self(slice)
  }

  pub fn read_bool(&self, from: usize) -> Option<bool> {
    to_opt_bool(self.read_u8(from))
  }

  pub fn read_u8(&self, from: usize) -> u8 {
    self.0[from]
  }
  /*pub fn read_i8(&self, from: usize) -> i8 {
    to_i8(self.read_u8(from))
  }*/
  pub fn read_i16(&self, from: usize) -> i16 {
    i16::from_be_bytes(self.read_bytes(from))
  }
  /*pub fn read_u16(&self, from: usize) -> u16 {
    // We use wrapping_add not to panic in debug mode
    to_u16(self.read_i16(from))
  }*/
  pub fn read_i32(&self, from: usize) -> i32 {
    i32::from_be_bytes(self.read_bytes(from))
  }
  /*pub fn read_u32(&self, from: usize) -> u32 {
    to_u32(self.read_i32(from))
  }*/
  pub fn read_i64(&self, from: usize) -> i64 {
    i64::from_be_bytes(self.read_bytes(from))
  }
  /*pub fn read_u64(&self, from: usize) -> u64 {
    to_u64(self.read_i64(from))
  }*/
  pub fn read_f32(&self, from: usize) -> f32 {
    f32::from_be_bytes(self.read_bytes(from))
  }
  pub fn read_f64(&self, from: usize) -> f64 {
    f64::from_be_bytes(self.read_bytes(from))
  }
  // An other option here may be to return an array of Char enum (ASCII chars, one u8 each).
  // But this is a nightly-only experimental API.
  pub fn read_string(&self, from: usize, n_ascii_chars: usize) -> &str {
    let bytes = &self.0[from..from + n_ascii_chars];
    let bytes = match bytes.iter().position(|b| *b == b'\0') {
      None => bytes,
      Some(index) => {
        // We are safe here since we are sure that 'index' <= slice.len()
        unsafe { std::slice::from_raw_parts(bytes.as_ptr(), index) }
      }
    };
    // Should we check that every single byte is an ASCII character?
    unsafe { str::from_utf8_unchecked(bytes) }
  }

  pub fn read_n_bytes(&'a self, from: usize, n: usize) -> &'a [u8] {
    &self.0[from..from + n]
  }
  pub fn read_i16_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = i16> + 'a {
    // To avoid a copy, we could have copied all bytes and swap them in-place.
    self
      .read_chunks(from, n)
      .map(|bytes| i16::from_be_bytes(bytes))
  }
  /*pub fn read_u16_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = u16> + 'a {
    self.read_i16_array(from, n).map(to_u16)
  }*/
  pub fn read_i32_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = i32> + 'a {
    // To avoid a copy, we could have copied all bytes and swap them in-place.
    self
      .read_chunks(from, n)
      .map(|bytes| i32::from_be_bytes(bytes))
  }
  /*pub fn read_u32_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = u32> + 'a {
    self.read_i32_array(from, n).map(to_u32)
  }*/
  pub fn read_i64_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = i64> + 'a {
    // To avoid a copy, we could have copied all bytes and swap them in-place.
    self
      .read_chunks(from, n)
      .map(|bytes| i64::from_be_bytes(bytes))
  }
  /*pub fn read_u64_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = u64> + 'a {
    self.read_i64_array(from, n).map(to_u64)
  }*/
  pub fn read_f32_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = f32> + 'a {
    // To avoid a copy, we could have copied all bytes and swap them in-place.
    self
      .read_chunks(from, n)
      .map(|bytes| f32::from_be_bytes(bytes))
  }
  pub fn read_f64_array(&'a self, from: usize, n: usize) -> impl Iterator<Item = f64> + 'a {
    // To avoid a copy, we could have copied all bytes and swap them in-place.
    self
      .read_chunks(from, n)
      .map(|bytes| f64::from_be_bytes(bytes))
  }

  pub fn read_bytes<const N: usize>(&self, from: usize) -> [u8; N] {
    /*let mut dest = [0_u8; N];
    let src = &self.0[from..from + N];
    unsafe {
      ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), N);
    }*/
    // Should we go unsafe?
    // Not sure, this seems to be well optimized:
    // https://doc.rust-lang.org/std/primitive.array.html
    (&self.0[from..from + N]).try_into().unwrap()
  }

  pub fn read_chunks<const N: usize>(
    &'a self,
    from: usize,
    n: usize,
  ) -> impl Iterator<Item = [u8; N]> + 'a {
    let chunks = unsafe { (&self.0[from..from + n * N]).as_chunks_unchecked::<N>() };
    chunks.iter().map(|slice| {
      // We do a copy here!
      // Another solution could have been to copy all bytes at once, on then swap them on place
      // using first "from_raw_parts" to get the right vec type. But this is highly unsafe.
      // To be done only performances really needs to be improved.
      let mut dest = [0_u8; N];
      unsafe {
        copy_nonoverlapping(slice.as_ptr(), dest.as_mut_ptr(), N);
      }
      dest
    })
  }
}
