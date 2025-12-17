use std::{fmt::Display, io::Write};

use crate::{
  error::{new_io_err, new_unsupported_by_visitor, Error},
  hdu::xtension::bintable::field::{ComplexF32, ComplexF64},
};

use super::{FieldVisitorProvider, RowVisitor, Visitor};

pub struct CSVRowVisitor;

impl RowVisitor for CSVRowVisitor {
  type Value = ();
  type FieldValue = ();

  fn visit_row<I>(self, fields_it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Result<Self::FieldValue, Error>>,
  {
    for res in fields_it {
      res?;
    }
    Ok(())
  }
}

pub struct CSVVisitor<'a, W: Write> {
  writer: &'a mut W,
  sep: u8,
}
impl<'a, W: Write> CSVVisitor<'a, W> {
  pub fn new(writer: &'a mut W) -> Self {
    Self { writer, sep: b'\n' }
  }

  pub fn starts_new_line(&mut self) {
    self.sep = b'\n';
  }

  fn write_sep(&mut self) -> Result<(), std::io::Error> {
    let res = self.writer.write_all(&[self.sep]);
    self.sep = b',';
    res
  }

  fn write<V: Display>(&mut self, v: V) -> Result<(), Error> {
    self
      .write_sep()
      .and_then(|()| write!(self.writer, "{}", v))
      .map_err(new_io_err)
  }

  fn write_opt<V: Display>(&mut self, v: Option<V>) -> Result<(), Error> {
    self
      .write_sep()
      .and_then(|()| match v {
        Some(v) => write!(self.writer, "{}", v),
        None => Ok(()),
      })
      .map_err(new_io_err)
  }

  fn write_array<V, I>(&mut self, mut it: I) -> Result<(), Error>
  where
    V: Display,
    I: Iterator<Item = V>,
  {
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          write!(self.writer, "\"[{}", v)?;
          for v in it {
            write!(self.writer, ", {}", v)?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn write_opt_array<V, I>(&mut self, mut it: I) -> Result<(), Error>
  where
    V: Display,
    I: Iterator<Item = Option<V>>,
  {
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          // We do no write firts "[ and then the possible the value to use ? once instead of twice.
          match v {
            Some(v) => write!(self.writer, "\"[{}", v),
            None => self.writer.write_all(b"\"["),
          }?;
          for v in it {
            // We do no write firts "[ and then the possible the value to use ? once instead of twice.
            match v {
              Some(v) => write!(self.writer, ", {}", v),
              None => self.writer.write_all(b", "),
            }?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }
}

impl<'a, W: Write> FieldVisitorProvider for CSVVisitor<'a, W> {
  type FieldValue = ();
  type FieldVisitor<'v>
    = &'v mut CSVVisitor<'a, W>
  where
    Self: 'v;

  fn field_visitor(&mut self) -> Self::FieldVisitor<'_> {
    self
  }
}

impl<'a, W: Write> RowVisitor for &mut CSVVisitor<'a, W> {
  type Value = ();
  type FieldValue = ();

  fn visit_row<I>(self, fields_it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Result<Self::FieldValue, Error>>,
  {
    for res in fields_it {
      res?;
    }
    Ok(())
  }
}

impl<'a, W: Write> Visitor for &mut CSVVisitor<'a, W> {
  type Value = ();

  fn expecting(&self) -> &str {
    "Unreachable for CSV visitor"
  }

  fn visit_empty(self) -> Result<Self::Value, Error> {
    self.write_sep().map_err(new_io_err)
  }

  fn visit_opt_bool(self, v: Option<bool>) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| match v {
        Some(true) => self.writer.write_all(b"true"),
        Some(false) => self.writer.write_all(b"false"),
        None => Ok(()),
      })
      .map_err(new_io_err)
  }

  fn visit_ascii_char(self, v: u8) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| match v {
        b'\0' => Ok(()),
        b'"' => self.writer.write_all(b"\"\"\"\""), // """", see https://www.ietf.org/rfc/rfc4180.txt
        _ => {
          if v != self.sep {
            self.writer.write_all(&[v])
          } else {
            self.writer.write_all(&[b'"', v, b'"'])
          }
        }
      })
      .map_err(new_io_err)
  }

  fn visit_i8(self, v: i8) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_i16(self, v: i16) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_i32(self, v: i32) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_i64(self, v: i64) -> Result<Self::Value, Error> {
    self.write(v)
  }

  fn visit_opt_i8(self, v: Option<i8>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_i16(self, v: Option<i16>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_i32(self, v: Option<i32>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_i64(self, v: Option<i64>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }

  fn visit_u8(self, v: u8) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_u16(self, v: u16) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_u32(self, v: u32) -> Result<Self::Value, Error> {
    self.write(v)
  }
  fn visit_u64(self, v: u64) -> Result<Self::Value, Error> {
    self.write(v)
  }

  fn visit_opt_u8(self, v: Option<u8>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_u16(self, v: Option<u16>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_u32(self, v: Option<u32>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }
  fn visit_opt_u64(self, v: Option<u64>) -> Result<Self::Value, Error> {
    self.write_opt(v)
  }

  fn visit_f32(self, v: f32) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| {
        if !v.is_nan() {
          write!(self.writer, "{:?}", v)
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_f64(self, v: f64) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| {
        if !v.is_nan() {
          write!(self.writer, "{:?}", v)
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_cf32(self, v: ComplexF32) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| write!(self.writer, "\"({}, {})\"", v.real(), v.img()))
      .map_err(new_io_err)
  }

  fn visit_cf64(self, v: ComplexF64) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| write!(self.writer, "\"({}, {})\"", v.real(), v.img()))
      .map_err(new_io_err)
  }

  // Provide the number of bits to be read in the n bytes?
  fn visit_bit_array(self, _v: &[u8]) -> Result<Self::Value, Error> {
    // TODO!
    Err(new_unsupported_by_visitor(self.expecting(), "Bit array"))
  }

  fn visit_ascii_string(self, v: &str) -> Result<Self::Value, Error> {
    self
      .write_sep()
      .and_then(|()| {
        if !(v.contains('"') || v.contains(self.sep as char)) {
          self.writer.write_all(v.trim_end().as_bytes())
        } else {
          // See https://www.ietf.org/rfc/rfc4180.txt : " are replaced by ""
          write!(self.writer, "\"{}\"", v.replace('"', "\"\""))
        }
      })
      .map_err(new_io_err)
  }

  fn visit_opt_bool_array<I>(self, mut it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<bool>>,
  {
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          // We do no write firts "[ and then the possible the value to use ? once instead of twice.
          match v {
            Some(true) => self.writer.write_all(b"\"[T"),
            Some(false) => self.writer.write_all(b"\"[F"),
            None => self.writer.write_all(b"\"["),
          }?;
          for v in it {
            // We do no write firts ", " and then the possible the value to use ? once instead of twice.
            match v {
              Some(true) => self.writer.write_all(b", T"),
              Some(false) => self.writer.write_all(b", F"),
              None => self.writer.write_all(b", "),
            }?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_i8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i8>,
  {
    self.write_array(it)
  }

  fn visit_i16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i16>,
  {
    self.write_array(it)
  }

  fn visit_i32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i32>,
  {
    self.write_array(it)
  }

  fn visit_i64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = i64>,
  {
    self.write_array(it)
  }

  fn visit_opt_i8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i8>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_i16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i16>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_i32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i32>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_i64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<i64>>,
  {
    self.write_opt_array(it)
  }

  fn visit_u8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u8>,
  {
    self.write_array(it)
  }

  fn visit_u16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u16>,
  {
    self.write_array(it)
  }

  fn visit_u32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u32>,
  {
    self.write_array(it)
  }

  fn visit_u64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = u64>,
  {
    self.write_array(it)
  }

  fn visit_opt_u8_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u8>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_u16_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u16>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_u32_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u32>>,
  {
    self.write_opt_array(it)
  }

  fn visit_opt_u64_array<I>(self, it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = Option<u64>>,
  {
    self.write_opt_array(it)
  }

  fn visit_f32_array<I>(self, mut it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f32>,
  {
    // Simple but not optimal since we have to test on for NaN and once the Option
    // self.write_opt_array(it.map(|v| if !v.is_nan() { Some(v) } else { None }))
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          // We do no write firts "[ and then the possible the value to use ? once instead of twice.
          if !v.is_nan() {
            write!(self.writer, "\"[{}", v)
          } else {
            self.writer.write_all(b"\"[")
          }?;
          for v in it {
            // We do no write firts ", " and then the possible the value to use ? once instead of twice.
            if !v.is_nan() {
              write!(self.writer, ", {}", v)
            } else {
              self.writer.write_all(b", ")
            }?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_f64_array<I>(self, mut it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = f64>,
  {
    // Simple but not optimal since we have to test on for NaN and once the Option
    // self.write_opt_array(it.map(|v| if !v.is_nan() { Some(v) } else { None }))
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          // We do no write firts "[ and then the possible the value to use ? once instead of twice.
          if !v.is_nan() {
            write!(self.writer, "\"[{}", v)
          } else {
            self.writer.write_all(b"\"[")
          }?;
          for v in it {
            // We do no write firts ", " and then the possible the value to use ? once instead of twice.
            if !v.is_nan() {
              write!(self.writer, ", {}", v)
            } else {
              self.writer.write_all(b", ")
            }?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_cf32_array<I>(self, mut it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF32>,
  {
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          write!(self.writer, "\"[({}, {})", v.real(), v.img())?;
          for v in it {
            write!(self.writer, ", ({}, {})", v.real(), v.img())?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }

  fn visit_cf64_array<I>(self, mut it: I) -> Result<Self::Value, Error>
  where
    I: Iterator<Item = ComplexF64>,
  {
    self
      .write_sep()
      .and_then(|()| {
        if let Some(v) = it.next() {
          write!(self.writer, "\"[({}, {})", v.real(), v.img())?;
          for v in it {
            write!(self.writer, ", ({}, {})", v.real(), v.img())?;
          }
          self.writer.write_all(b"]\"")
        } else {
          Ok(())
        }
      })
      .map_err(new_io_err)
  }
}
