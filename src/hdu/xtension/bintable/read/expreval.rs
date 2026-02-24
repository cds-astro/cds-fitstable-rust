//! Module implementing evaluation expression to compute columns on-the-fly or
//! filter rows.

use log::warn;
use std::marker::PhantomData;

use expreval::{
  compile_expression,
  compiled::Node,
  fieldtype::FieldType,
  literal::Literal,
  table::{Row, Table},
};

use crate::hdu::xtension::bintable::{
  read::{
    deser::{DeserializeSeed, sliceheap::DeserializerWithHeap},
    visitor::primitive::get_visitor,
  },
  schema::{FieldSchema, Schema},
};

pub struct TableSchema<'a, 'b> {
  names: &'a [String],
  schema: &'a [FieldSchema],
  _phantom: PhantomData<&'b ()>,
}

impl<'a, 'b> TableSchema<'a, 'b> {
  pub fn new(field_names: &'a [String], field_schemas: &'a [FieldSchema]) -> Self {
    Self {
      names: field_names,
      schema: field_schemas,
      _phantom: PhantomData,
    }
  }

  pub fn compile_f64_expr(
    &self,
    expression: String,
  ) -> Result<impl Fn(&ExprEvalRow<'b>) -> f64 + Sync + Send + 'b, String> {
    compile_f64_expr(expression, &self)
  }

  pub fn compile_bool_expr(
    &self,
    expression: String,
  ) -> Result<impl Fn(&ExprEvalRow<'b>) -> bool + Sync + Send + 'b, String> {
    compile_bool_expr(expression, &self)
  }
}

impl<'a, 'b> Table for TableSchema<'a, 'b> {
  type RowType = ExprEvalRow<'b>;

  fn col_index(&self, col_name: &str) -> Result<u16, String> {
    for (i, name) in self.names.iter().enumerate() {
      if col_name == name.as_str() {
        return Ok(i as u16);
      }
    }
    Err(format!("Column name '{}' not found!", col_name))
  }

  fn datatype(&self, col_index: u16) -> Result<FieldType, String> {
    match self.schema[col_index as usize].schema {
      Schema::Empty => Ok(FieldType::Null),
      Schema::NullableBoolean => Ok(FieldType::Opt(Box::new(FieldType::Bool))),
      Schema::Byte => Ok(FieldType::I8),
      Schema::Short => Ok(FieldType::I16),
      Schema::Int => Ok(FieldType::I32),
      Schema::Long => Ok(FieldType::I64),
      Schema::NullableByte { .. } => Ok(FieldType::Opt(Box::new(FieldType::I8))),
      Schema::NullableShort { .. } => Ok(FieldType::Opt(Box::new(FieldType::I16))),
      Schema::NullableInt { .. } => Ok(FieldType::Opt(Box::new(FieldType::I32))),
      Schema::NullableLong { .. } => Ok(FieldType::Opt(Box::new(FieldType::I64))),
      Schema::UnsignedByte => Ok(FieldType::U8),
      Schema::UnsignedShort => Ok(FieldType::U16),
      Schema::UnsignedInt => Ok(FieldType::U32),
      Schema::UnsignedLong => Ok(FieldType::U64),
      Schema::NullableUnsignedByte { .. } => Ok(FieldType::Opt(Box::new(FieldType::U8))),
      Schema::NullableUnsignedShort { .. } => Ok(FieldType::Opt(Box::new(FieldType::U16))),
      Schema::NullableUnsignedInt { .. } => Ok(FieldType::Opt(Box::new(FieldType::U32))),
      Schema::NullableUnsignedLong { .. } => Ok(FieldType::Opt(Box::new(FieldType::U64))),
      Schema::Float
      | Schema::FloatFromFloat(_)
      | Schema::FloatFromByte(_)
      | Schema::FloatFromShort(_) => Ok(FieldType::F32),
      Schema::Double
      | Schema::DoubleFromDouble(_)
      | Schema::DoubleFromInt(_)
      | Schema::DoubleFromLong(_) => Ok(FieldType::F64),
      // Schema::ComplexFloat => ,
      // Schema::ComplexDouble => ,
      Schema::AsciiChar => Ok(FieldType::Char),
      _ => Err(format!(
        "FITS field schema {} not implemented or not usable with ExprEval!",
        self.schema[col_index as usize].schema,
      )),
    }
  }
}

// also make the method returning the FITS type from the ExprEval FieldType

pub struct ExprEvalRow<'a> {
  schema: &'a [FieldSchema],
  row: &'a [u8],
  heap: &'a [u8],
}
impl<'a> ExprEvalRow<'a> {
  /// # Params
  /// * `schema`: the BINTABLE fields schema
  /// * `row`: the bytes of a single row in the main table
  /// * `heap`: all byte of the heap, if any.
  pub fn new(schema: &'a [FieldSchema], row: &'a [u8], heap: &'a [u8]) -> Self {
    Self { schema, row, heap }
  }
}

impl<'a> Row for ExprEvalRow<'a> {
  fn field(&self, _col_index: u16) -> Literal {
    unreachable!()
  }

  fn field_bool(&self, col_index: u16) -> bool {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<bool>>(),
      )
      .map(|opt_bool| match opt_bool {
        Some(b) => b,
        None => {
          // No 'bool' implemented in FITS, only Option<bool>.
          warn!("Optional boolean None forced to 'false'. Use Option<bool> instead of bool in expression evaluation!");
          false
        },
      })
      .unwrap()
  }

  fn field_char(&self, col_index: u16) -> char {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<u8>(),
      )
      .unwrap() as char
  }

  fn field_u8(&self, col_index: u16) -> u8 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<u8>(),
      )
      .unwrap()
  }

  fn field_u16(&self, col_index: u16) -> u16 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<u16>(),
      )
      .unwrap()
  }

  fn field_u32(&self, col_index: u16) -> u32 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<u32>(),
      )
      .unwrap()
  }

  fn field_u64(&self, col_index: u16) -> u64 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<u64>(),
      )
      .unwrap()
  }

  fn field_i8(&self, col_index: u16) -> i8 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<i8>(),
      )
      .unwrap()
  }

  fn field_i16(&self, col_index: u16) -> i16 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<i16>(),
      )
      .unwrap()
  }

  fn field_i32(&self, col_index: u16) -> i32 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<i32>(),
      )
      .unwrap()
  }

  fn field_i64(&self, col_index: u16) -> i64 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<i64>(),
      )
      .unwrap()
  }

  fn field_f32(&self, col_index: u16) -> f32 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<f32>(),
      )
      .unwrap()
  }

  fn field_f64(&self, col_index: u16) -> f64 {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<f64>(),
      )
      .unwrap()
  }

  fn field_str(&self, col_index: u16) -> String {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<String>(),
      )
      .unwrap()
  }

  fn field_opt_bool(&self, col_index: u16) -> Option<bool> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<bool>>(),
      )
      .unwrap()
  }

  fn field_opt_char(&self, col_index: u16) -> Option<char> {
    Some(
      self.schema[col_index as usize]
        .deserialize(
          &mut DeserializerWithHeap::new(self.row, self.heap),
          get_visitor::<u8>(),
        )
        .unwrap() as char,
    )
  }

  fn field_opt_u8(&self, col_index: u16) -> Option<u8> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<u8>>(),
      )
      .unwrap()
  }

  fn field_opt_u16(&self, col_index: u16) -> Option<u16> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<u16>>(),
      )
      .unwrap()
  }

  fn field_opt_u32(&self, col_index: u16) -> Option<u32> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<u32>>(),
      )
      .unwrap()
  }

  fn field_opt_u64(&self, col_index: u16) -> Option<u64> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<u64>>(),
      )
      .unwrap()
  }

  fn field_opt_i8(&self, col_index: u16) -> Option<i8> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<i8>>(),
      )
      .unwrap()
  }

  fn field_opt_i16(&self, col_index: u16) -> Option<i16> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<i16>>(),
      )
      .unwrap()
  }

  fn field_opt_i32(&self, col_index: u16) -> Option<i32> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<i32>>(),
      )
      .unwrap()
  }

  fn field_opt_i64(&self, col_index: u16) -> Option<i64> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<Option<i64>>(),
      )
      .unwrap()
  }

  fn field_opt_f32(&self, col_index: u16) -> Option<f32> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<f32>(),
      )
      .map(|val| if val.is_finite() { Some(val) } else { None })
      .unwrap()
  }

  fn field_opt_f64(&self, col_index: u16) -> Option<f64> {
    self.schema[col_index as usize]
      .deserialize(
        &mut DeserializerWithHeap::new(self.row, self.heap),
        get_visitor::<f64>(),
      )
      .map(|val| if val.is_finite() { Some(val) } else { None })
      .unwrap()
  }

  fn field_opt_str(&self, col_index: u16) -> Option<String> {
    Some(
      self.schema[col_index as usize]
        .deserialize(
          &mut DeserializerWithHeap::new(self.row, self.heap),
          get_visitor::<String>(),
        )
        .unwrap(),
    )
  }
}

/// "Compiled" a boolean expression.
fn compile_bool_expr<'a, 'b>(
  expression: String,
  table_schema: &TableSchema<'a, 'b>,
) -> Result<impl Fn(&ExprEvalRow<'b>) -> bool + Sync + Send + 'b, String> {
  let node = if expression.starts_with('\"') && expression.ends_with('\"') {
    compile_expression(&expression[1..expression.len() - 1], table_schema)
  } else {
    compile_expression(&expression, table_schema)
  }?;
  match node {
    Node::Bool(f) => Ok(move |row: &ExprEvalRow<'b>| f(row)),
    _ => Err(format!(
      "Filter expression '{}' must return a boolean.",
      &expression
    )),
  }
}

/*
/// Decorate the given row iterator with a filter based on a boolean expression.
fn filter<'a, 'b, T>(
  expression: &String,
  table_schema: &TableSchema<'a, 'b>,
  it: T,
) -> Result<std::iter::Filter<T, impl Fn(&ExprEvalRow<'b>) -> bool + 'b>, String>
where
  T: Iterator<Item = ExprEvalRow<'b>>,
{
  compile_bool_expr(expression, table_schema).map(|fn_filter| it.filter(fn_filter))
}

/// Decorate the given row parallel iterator with a filter based on a boolean expression.
pub fn par_filter<'a, T>(
  expression: &String,
  table_schema: &TableSchema<'a>,
  it: T,
) -> Result<rayon::iter::Filter<T, impl Fn(&ExprEvalRow<'a>) -> bool + Sync + Send + 'a>, String>
where
  T: ParallelIterator<Item = ExprEvalRow<'a>>,
{
  compile_bool_expr(expression, table_schema).map(|fn_filter| it.filter(fn_filter))
}
*/

/// "Compiled" an expression returning a f64 value.
fn compile_f64_expr<'a, 'b>(
  expression: String,
  table_schema: &TableSchema<'a, 'b>,
) -> Result<impl Fn(&ExprEvalRow<'b>) -> f64 + Sync + Send + 'b, String> {
  let node = if expression.as_str().starts_with('\"') && expression.as_str().ends_with('\"') {
    compile_expression(&expression[1..expression.len() - 1], table_schema)
  } else {
    compile_expression(&expression, table_schema)
  }?;
  match node {
    Node::F64(f) => Ok(move |row: &ExprEvalRow<'b>| f(row)),
    node => Err(format!(
      "Wrong type returned by expression: {}. Expected: f64. Actual: {:?}.",
      &expression,
      node.get_type()
    )),
  }
}
