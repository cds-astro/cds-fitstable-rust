// Nullable type requires the `TNULLn` keyword.
pub enum Field {
  // For empty columns
  Empty,

  // Primitive types
  // + bool
  NullableBoolean(Option<bool>),
  BitArray(Vec<u8>),
  // - signed integer
  Byte(i8),
  Short(i16),
  Int(i32),
  Long(i64),
  NullableByte(Option<i8>),
  NullableShort(Option<i16>),
  NullableInt(Option<i32>),
  NullableLong(Option<i64>),
  // - unsigned Integer
  UnsignedByte(u8),
  UnsignedShort(u16),
  UnsignedInt(u32),
  UnsignedLong(u64),
  NullableUnsignedByte(Option<u8>),
  NullableUnsignedShort(Option<u16>),
  NullableUnsignedInt(Option<u32>),
  NullableUnsignedLong(Option<u64>),
  // - real
  Float(f32),  // NaN = null
  Double(f64), // NaN = null
  // - complex
  ComplexFloat(ComplexF32),
  ComplexDouble(ComplexF64),
  // - char
  AsciiChar(u8),

  // Arrays
  // - bool
  NullableBooleanArray(Vec<Option<bool>>),
  // - signed integer
  ByteArray(Vec<i8>),
  ShortArray(Vec<i16>),
  IntArray(Vec<i32>),
  LongArray(Vec<i64>),
  NullableByteArray(Vec<Option<i8>>),
  NullableShortArray(Vec<Option<i16>>),
  NullableIntArray(Vec<Option<i32>>),
  NullableLongArray(Vec<Option<i64>>),
  // - unsigned Integer
  UnsignedByteArray(Vec<u8>),
  UnsignedShortArray(Vec<u16>),
  UnsignedIntArray(Vec<u32>),
  UnsignedLongArray(Vec<u64>),
  NullableUnsignedByteArray(Vec<Option<u8>>),
  NullableUnsignedShortArray(Vec<Option<u16>>),
  NullableUnsignedIntArray(Vec<Option<u32>>),
  NullableUnsignedLongArray(Vec<Option<u64>>),
  // - real
  FloatArray(Vec<f32>),  // NaN = null
  DoubleArray(Vec<f64>), // NaN = null
  // - complex
  ComplexFloatArray(Vec<ComplexF32>),
  ComplexDoubleArray(Vec<ComplexF64>),
  // - String
  AsciiString(String), // Empty string = null. We could express ASCII string as a Vec of Char (ascii char), but it is a nightly features
}

pub struct ComplexF32 {
  real: f32,
  img: f32,
}
impl ComplexF32 {
  pub fn new(real: f32, img: f32) -> Self {
    Self { real, img }
  }
  pub fn real(&self) -> f32 {
    self.real
  }
  pub fn img(&self) -> f32 {
    self.img
  }
}

pub struct ComplexF64 {
  real: f64,
  img: f64,
}
impl ComplexF64 {
  pub fn new(real: f64, img: f64) -> Self {
    Self { real, img }
  }
  pub fn real(&self) -> f64 {
    self.real
  }
  pub fn img(&self) -> f64 {
    self.img
  }
}

pub struct ComplexF32Iterator<I: Iterator<Item = f32>> {
  it: I,
}
impl<I: Iterator<Item = f32>> ComplexF32Iterator<I> {
  pub fn new(it: I) -> Self {
    Self { it }
  }
}
impl<I: Iterator<Item = f32>> Iterator for ComplexF32Iterator<I> {
  type Item = ComplexF32;

  fn next(&mut self) -> Option<Self::Item> {
    match (self.it.next(), self.it.next()) {
      (Some(real), Some(img)) => Some(ComplexF32::new(real, img)),
      _ => None,
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (low, high) = self.it.size_hint();
    (low >> 1, high.map(|v| v >> 1))
  }
}

pub struct ComplexF64Iterator<I: Iterator<Item = f64>> {
  it: I,
}
impl<I: Iterator<Item = f64>> ComplexF64Iterator<I> {
  pub fn new(it: I) -> Self {
    Self { it }
  }
}
impl<I: Iterator<Item = f64>> Iterator for ComplexF64Iterator<I> {
  type Item = ComplexF64;

  fn next(&mut self) -> Option<Self::Item> {
    match (self.it.next(), self.it.next()) {
      (Some(real), Some(img)) => Some(ComplexF64::new(real, img)),
      _ => None,
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (low, high) = self.it.size_hint();
    (low >> 1, high.map(|v| v >> 1))
  }
}
