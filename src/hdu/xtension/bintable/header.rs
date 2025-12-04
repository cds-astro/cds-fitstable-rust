use log::warn;

#[cfg(feature = "vot")]
use crate::common::keywords::tables::bintable::{
  tdisp::TDispValue,
  tform::{RepeatCountAndExtraChar, VariableLenghtArrayInfo},
};
use crate::{
  common::{
    DynValueKwr, ValueKwr,
    keywords::{
      bitpix::BitPix,
      naxis::{NAxis, NAxis1, NAxis2},
      pgcount::{GCount, PCount},
      tables::{
        bintable::{
          tdim::TDim,
          tdisp::TDispn,
          tform::{TFormValue, TFormn, VariableLenghtArrayDataType},
          theap::THeap,
        },
        tcomm::TComm,
        tdminmax::{TDMax, TDMin},
        tfields::TFields,
        tnull::TNull,
        tscaltzero::{TScal, TZero, UIF64},
        ttype::TType,
        tucd::TUCD,
        tunit::TUnit,
      },
      xtension::Xtension,
    },
    read::{FixedFormatRead, KwrFormatRead, is_value_indicator},
  },
  error::{Error, new_custom},
  hdu::{
    HDUType,
    header::Header,
    xtension::bintable::{
      read::bytes::{to_i8, to_u16, to_u32, to_u64},
      schema::{
        ArrayParam, HeapArrayParam, HeapArraySchema, RowSchema, ScaleOffset32, ScaleOffset64,
        Schema,
      },
    },
  },
};
#[cfg(feature = "vot")]
use votable::{
  Max, Min, Values,
  datatype::Datatype as VOTDatatype,
  field::{ArraySize, Field as VOTField, Precision},
};

pub const XTENSION: Xtension = Xtension::BinTable;
pub const BITPIX: BitPix = BitPix::U8;
pub const NAXIS: NAxis = NAxis::new(2);
pub const GCOUNT: GCount = GCount::new(1);

#[derive(Default, Debug)]
pub struct BinTableColumnHeader {
  /// Column name
  ttype: Option<TType>,
  /// Data type (the only one to be mandatory)
  tform: Option<TFormn>,
  /// Display info
  tdisp: Option<TDispn>,
  /// Unit
  tunit: Option<TUnit>,
  /// UCD
  tucd: Option<TUCD>,
  /// Description
  tcomm: Option<TComm>,
  /// Null value, for types `B`, `I`, `J`, `K`, `P`, `Q` only.
  tnull: Option<TNull>,
  /// To be used with in: `field_value = TZERO + TSCAL * stored_value`
  tscal: Option<TScal>,
  /// To be used with in: `field_value = TZERO + TSCAL * stored_value`
  tzero: Option<TZero>,
  /// Multi-dim columns info (array to be interpreted as a multi-dim array)
  tdim: Option<TDim>,
  /// Min column value
  tdmin: Option<TDMin>,
  /// Max column value
  tdmax: Option<TDMax>,
  // TO be implemented?
  // TLMAX, TLMIN,
}

impl BinTableColumnHeader {
  pub fn colname(&self) -> Option<&str> {
    self.ttype.as_ref().map(|ttype| ttype.col_name())
  }
  pub fn unit(&self) -> Option<&str> {
    self.tunit.as_ref().map(|tunit| tunit.col_unit())
  }
  pub fn ucd(&self) -> Option<&str> {
    self.tucd.as_ref().map(|tucd| tucd.col_ucd())
  }
  pub fn description(&self) -> Option<&str> {
    self.tcomm.as_ref().map(|tcomm| tcomm.col_description())
  }
  pub fn min(&self) -> Option<&str> {
    self.tdmin.as_ref().map(|tdmin| tdmin.min_value())
  }
  pub fn max(&self) -> Option<&str> {
    self.tdmax.as_ref().map(|tdmax| tdmax.max_value())
  }
  pub fn disp(&self) -> Option<&TDispValue> {
    self.tdisp.as_ref().map(|tdisp| tdisp.data_type())
  }

  // format (tdips)

  /// Returns the VOTable arraysize of this field, given the data type length and TDISP
  #[cfg(feature = "vot")]
  pub fn to_arraysize(&self, len: usize) -> ArraySize {
    match &self.tdim {
      None => ArraySize::Fixed1D { size: len as u32 },
      Some(tdim) => {
        assert_eq!(len as u16, tdim.col_nbr());
        ArraySize::FixedND {
          sizes: tdim.dimensions().iter().map(|v| *v as u32).collect(),
        }
      }
    }
  }

  /// Compute the VOTable Field corresponding to this column.
  /// If no name is define in FITS, use the given column index to build name.
  #[cfg(feature = "vot")]
  pub fn to_vot_field(&self, i_col: u16) -> Result<VOTField, Error> {
    self
      .schema()
      .ok_or_else(|| {
        new_custom(format!(
          "Not enough information to build filed schema! {:?}",
          &self
        ))
      })
      .and_then(|schema| self.to_vot_field_with_schema(i_col, &schema))
  }

  /// Compute the VOTable Field corresponding to this column.
  /// If no name is define in FITS, use the given column index to build name.
  /// # Info
  /// Same as `to_vot_field`, except that the `schema` of the field has already been computed.
  #[cfg(feature = "vot")]
  pub fn to_vot_field_with_schema(&self, i_col: u16, schema: &Schema) -> Result<VOTField, Error> {
    let name = self
      .colname()
      .map(String::from)
      .unwrap_or_else(|| format!("col_{}", i_col));
    let mut vot_field = match schema {
      Schema::Empty => Err(new_custom(
        "Empty FITS field, transform into VOTable PARAM?",
      )),
      Schema::NullableBoolean => Ok(VOTField::new(name, VOTDatatype::Logical)),
      Schema::Bits { n_bits } => Ok(VOTField::new(name, VOTDatatype::Bit).set_arraysize(
        ArraySize::Fixed1D {
          size: *n_bits as u32,
        },
      )),
      Schema::Byte => Ok(VOTField::new(name, VOTDatatype::Byte).set_xtype("signed")),
      Schema::Short => Ok(VOTField::new(name, VOTDatatype::ShortInt)),
      Schema::Int => Ok(VOTField::new(name, VOTDatatype::Int)),
      Schema::Long => Ok(VOTField::new(name, VOTDatatype::LongInt)),
      Schema::NullableByte { null } => Ok(
        VOTField::new(name, VOTDatatype::Byte)
          .set_xtype("signed")
          .set_values(Values::new().set_null(to_i8(*null).to_string())),
      ),
      Schema::NullableShort { null } => Ok(
        VOTField::new(name, VOTDatatype::ShortInt)
          .set_values(Values::new().set_null(null.to_string())),
      ),
      Schema::NullableInt { null } => Ok(
        VOTField::new(name, VOTDatatype::Int).set_values(Values::new().set_null(null.to_string())),
      ),
      Schema::NullableLong { null } => Ok(
        VOTField::new(name, VOTDatatype::LongInt)
          .set_values(Values::new().set_null(null.to_string())),
      ),
      Schema::UnsignedByte => Ok(VOTField::new(name, VOTDatatype::Byte)),
      Schema::UnsignedShort => Ok(VOTField::new(name, VOTDatatype::ShortInt).set_xtype("unsigned")),
      Schema::UnsignedInt => Ok(VOTField::new(name, VOTDatatype::Int).set_xtype("unsigned")),
      Schema::UnsignedLong => Ok(VOTField::new(name, VOTDatatype::LongInt).set_xtype("unsigned")),
      Schema::NullableUnsignedByte { null } => Ok(
        VOTField::new(name, VOTDatatype::Byte).set_values(Values::new().set_null(null.to_string())),
      ),
      Schema::NullableUnsignedShort { null } => Ok(
        VOTField::new(name, VOTDatatype::ShortInt)
          .set_xtype("unsigned")
          .set_values(Values::new().set_null(to_u16(*null).to_string())),
      ),
      Schema::NullableUnsignedInt { null } => Ok(
        VOTField::new(name, VOTDatatype::Int)
          .set_xtype("unsigned")
          .set_values(Values::new().set_null(to_u32(*null as i32).to_string())),
      ),
      Schema::NullableUnsignedLong { null } => Ok(
        VOTField::new(name, VOTDatatype::LongInt)
          .set_xtype("unsigned")
          .set_values(Values::new().set_null(to_u64(*null).to_string())),
      ),
      Schema::Float => Ok(VOTField::new(name, VOTDatatype::Float)),
      Schema::FloatFromFloat(_so) | Schema::FloatFromByte(_so) | Schema::FloatFromShort(_so) => {
        Ok(VOTField::new(name, VOTDatatype::Float))
      }
      Schema::Double => Ok(VOTField::new(name, VOTDatatype::Double)),
      Schema::DoubleFromDouble(_so) | Schema::DoubleFromInt(_so) | Schema::DoubleFromLong(_so) => {
        Ok(VOTField::new(name, VOTDatatype::Double))
      }
      Schema::ComplexFloat => Ok(VOTField::new(name, VOTDatatype::ComplexFloat)),
      Schema::ComplexDouble => Ok(VOTField::new(name, VOTDatatype::ComplexDouble)),
      Schema::AsciiChar => Ok(VOTField::new(name, VOTDatatype::CharASCII)),
      Schema::NullableBooleanArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::Logical).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::ByteArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::Byte)
          .set_xtype("signed")
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::ShortArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::ShortInt).set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::IntArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::Int).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::LongArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::LongInt).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::NullableByteArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::Byte)
          .set_values(Values::new().set_null(to_i8(*null).to_string()))
          .set_xtype("signed")
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableShortArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::ShortInt)
          .set_values(Values::new().set_null(null.to_string()))
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableIntArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::Int)
          .set_values(Values::new().set_null(null.to_string()))
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableLongArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::LongInt)
          .set_values(Values::new().set_null(null.to_string()))
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::UnsignedByteArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::Byte).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::UnsignedShortArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::ShortInt)
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::UnsignedIntArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::Int)
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::UnsignedLongArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::LongInt)
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::NullableUnsignedByteArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::Byte)
          .set_values(Values::new().set_null(null.to_string()))
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableUnsignedShortArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::ShortInt)
          .set_values(Values::new().set_null(to_u16(*null).to_string()))
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableUnsignedIntArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::Int)
          .set_values(Values::new().set_null(to_u32(*null).to_string()))
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::NullableUnsignedLongArray { null, p } => Ok(
        VOTField::new(name, VOTDatatype::LongInt)
          .set_values(Values::new().set_null(to_u64(*null).to_string()))
          .set_xtype("unsigned")
          .set_arraysize(self.to_arraysize(p.get_len())),
      ),
      Schema::FloatArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::Float).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::FloatArrayFromFloat(apwso)
      | Schema::FloatArrayFromBytes(apwso)
      | Schema::FloatArrayFromShort(apwso) => Ok(
        VOTField::new(name, VOTDatatype::Float).set_arraysize(ArraySize::Fixed1D {
          size: apwso.get_len() as u32,
        }),
      ),
      Schema::DoubleArray(ap) => {
        Ok(VOTField::new(name, VOTDatatype::Double).set_arraysize(self.to_arraysize(ap.get_len())))
      }
      Schema::DoubleArrayFromDouble(apwso)
      | Schema::DoubleArrayFromInt(apwso)
      | Schema::DoubleArrayFromLong(apwso) => Ok(
        VOTField::new(name, VOTDatatype::Double).set_arraysize(ArraySize::Fixed1D {
          size: apwso.get_len() as u32,
        }),
      ),
      Schema::ComplexFloatArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::ComplexFloat)
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::ComplexDoubleArray(ap) => Ok(
        VOTField::new(name, VOTDatatype::ComplexDouble)
          .set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::AsciiString(ap) => Ok(
        VOTField::new(name, VOTDatatype::CharASCII).set_arraysize(self.to_arraysize(ap.get_len())),
      ),
      Schema::HeapArrayPtr32(_has) => todo!(),
      Schema::HeapArrayPtr64(_has) => todo!(),
    }?;
    if let Some(unit) = self.unit() {
      vot_field.set_unit_by_ref(unit);
    }
    if let Some(ucd) = self.ucd() {
      vot_field.set_ucd_by_ref(ucd);
    }
    if let Some(desc) = self.description() {
      vot_field.set_description_by_ref(desc.into());
    }
    if let Some(min) = self.min() {
      let min = Min::new(min);
      match &mut vot_field.values {
        Some(values) => values.set_min_by_ref(min),
        None => vot_field.set_values_by_ref(Values::new().set_min(min)),
      }
    }
    if let Some(max) = self.max() {
      let max = Max::new(max);
      match &mut vot_field.values {
        Some(values) => values.set_max_by_ref(max),
        None => vot_field.set_values_by_ref(Values::new().set_max(max)),
      }
    }
    if let Some(disp) = self.disp() {
      let (w, p) = disp.get_width_and_prec();
      vot_field.set_width_by_ref(w);
      if let Some(p) = p {
        vot_field.set_width_by_ref(p);
      }
    }
    Ok(vot_field)
  }

  /// Replace the empty elements by the ones provided in the given VOTable field.
  /// If the option `overwrite` is set to `true`, elements are overwritten (except the ones defining the
  /// datatype, i.e. TFORM, TDIM, TNULL, TSCAL and TZERO;  and TDISP).
  #[cfg(feature = "vot")]
  pub fn merge(&mut self, icol: u16, field: &VOTField, overwrite: bool) {
    // Can be used to create a FITS Column header from a VOTable FIELD!
    let n = icol + 1;
    if self.ttype.is_none() || overwrite {
      if let Some(prev) = self.ttype.replace(TType::new(n, field.name.clone())) {
        warn!(
          "Col {}. Name '{}' replaced by '{}'.",
          n,
          prev.col_name(),
          field.name
        );
      }
    }
    // Handles array information
    let compute_size = |elems: &Vec<u32>| elems.iter().fold(1_u32, |acc, n| acc * *n);
    let tdim_value = |elems: &Vec<u32>| {
      TDim::new(
        n,
        elems.iter().map(|v| *v as u16).collect::<Vec<u16>>().into(),
      )
    };
    enum Type {
      Fixed(RepeatCountAndExtraChar),
      Var(u16), // max_len
    }

    // No overwrite for column format
    if self.tform.is_none() {
      let array_type = field
        .arraysize
        .as_ref()
        .map(|array_size| match array_size {
          ArraySize::Fixed1D { size } => {
            Type::Fixed(RepeatCountAndExtraChar::default().with_r(*size as u16))
          }
          ArraySize::FixedND { sizes } => {
            self.tdim = Some(tdim_value(sizes));
            Type::Fixed(RepeatCountAndExtraChar::default().with_r(compute_size(sizes) as u16))
          }
          ArraySize::VariableWithUpperLimit1D { upper_limit } => Type::Var(*upper_limit as u16),
          ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
            let mut sizes = sizes.clone();
            sizes.push(*upper_limit);
            self.tdim = Some(tdim_value(&sizes));
            Type::Var(compute_size(&sizes) as u16)
          }
          ArraySize::Variable1D => {
            warn!("Set variable length array upper size to the arbitrary 16!");
            Type::Var(16_u16)
          }
          ArraySize::VariableND { sizes } => {
            warn!("Set variable length array upper size to the arbitrary 16!");
            let mut sizes = sizes.clone();
            sizes.push(16);
            self.tdim = Some(tdim_value(&sizes));
            Type::Var(compute_size(&sizes) as u16)
          }
        })
        .unwrap_or(Type::Fixed(RepeatCountAndExtraChar::default()));

      // TODO: Add other ArraySize and set TDIM accordingly!
      let tform = match (&field.datatype, array_type) {
        (VOTDatatype::Logical, Type::Fixed(r)) => TFormn::new(n, TFormValue::L(r)),
        (VOTDatatype::Bit, Type::Fixed(r)) => TFormn::new(n, TFormValue::X(r)),
        (VOTDatatype::Byte, Type::Fixed(r)) => TFormn::new(n, TFormValue::B(r)),
        (VOTDatatype::ShortInt, Type::Fixed(r)) => TFormn::new(n, TFormValue::I(r)),
        (VOTDatatype::Int, Type::Fixed(r)) => TFormn::new(n, TFormValue::J(r)),
        (VOTDatatype::LongInt, Type::Fixed(r)) => TFormn::new(n, TFormValue::K(r)),
        (VOTDatatype::CharASCII, Type::Fixed(r)) => TFormn::new(n, TFormValue::A(r)),
        (VOTDatatype::Float, Type::Fixed(r)) => TFormn::new(n, TFormValue::E(r)),
        (VOTDatatype::Double, Type::Fixed(r)) => TFormn::new(n, TFormValue::D(r)),
        (VOTDatatype::ComplexFloat, Type::Fixed(r)) => TFormn::new(n, TFormValue::C(r)),
        (VOTDatatype::ComplexDouble, Type::Fixed(r)) => TFormn::new(n, TFormValue::M(r)),
        (VOTDatatype::CharUnicode, Type::Fixed(r)) => {
          // Return a result instead?
          warn!("FITS not supposed to support UnicodeChar!");
          TFormn::new(n, TFormValue::A(r))
        }
        (dt, Type::Var(max_len)) => TFormn::new(
          n,
          TFormValue::Q(VariableLenghtArrayInfo::new(
            None,
            match dt {
              VOTDatatype::Logical => VariableLenghtArrayDataType::L,
              VOTDatatype::Bit => todo!(),
              VOTDatatype::Byte => VariableLenghtArrayDataType::B,
              VOTDatatype::ShortInt => VariableLenghtArrayDataType::I,
              VOTDatatype::Int => VariableLenghtArrayDataType::J,
              VOTDatatype::LongInt => VariableLenghtArrayDataType::K,
              VOTDatatype::CharASCII => VariableLenghtArrayDataType::A,
              VOTDatatype::Float => VariableLenghtArrayDataType::E,
              VOTDatatype::Double => VariableLenghtArrayDataType::D,
              VOTDatatype::ComplexFloat => VariableLenghtArrayDataType::C,
              VOTDatatype::ComplexDouble => VariableLenghtArrayDataType::M,
              _ => todo!(),
            },
            max_len,
            None,
          )),
        ),
      };
      self.tform = Some(tform);
    }
    if self.tdisp.is_none() {
      match (field.width, &field.precision, &self.tform) {
        (
          Some(w),
          Some(p),
          Some(TFormn {
            value: TFormValue::E(..) | TFormValue::D(..),
            ..
          }),
        ) => {
          self.tdisp = match p {
            Precision::F { n_decimal } => Some(TDispn::new(
              n,
              TDispValue::F {
                w,
                d: *n_decimal as u16,
              },
            )),
            Precision::E { n_significant } => Some(TDispn::new(
              n,
              TDispValue::E {
                w,
                d: *n_significant as u16,
                e: None,
              },
            )),
          }
        }
        (
          Some(w),
          None,
          Some(TFormn {
            value: TFormValue::A(..),
            ..
          }),
        ) => self.tdisp = Some(TDispn::new(n, TDispValue::A { w })), // I
        (
          Some(w),
          None,
          Some(TFormn {
            value: TFormValue::L(..),
            ..
          }),
        ) => self.tdisp = Some(TDispn::new(n, TDispValue::L { w })),
        (
          Some(w),
          None,
          Some(TFormn {
            value: TFormValue::B(..) | TFormValue::I(..) | TFormValue::J(..) | TFormValue::K(..),
            ..
          }),
        ) => self.tdisp = Some(TDispn::new(n, TDispValue::I { w, m: None })),
        _ => {}
      }
    }
    // CHECK TFORM COMPATIBILITY??! + scale offset
    // tdisp?
    // Create:
    // * TLINK ? (keep only the first link of the field)
    // * TUTYP ?
    // * TXTYP ?
    if let Some(unit) = field.unit.as_ref()
      && (self.tunit.is_none() || overwrite)
    {
      if let Some(prev) = self.tunit.replace(TUnit::new(n, unit.clone())) {
        warn!(
          "Col {}. Unit '{}' replaced by '{}'.",
          n,
          prev.col_unit(),
          unit
        );
      }
    }
    if let Some(ucd) = field.ucd.as_ref()
      && (self.tucd.is_none() || overwrite)
    {
      if let Some(prev) = self.tucd.replace(TUCD::new(n, ucd.clone())) {
        warn!("Col {}. UCD '{}' replaced by '{}'.", n, prev.col_ucd(), ucd);
      }
    }
    if let Some(desc) = field.description.as_ref()
      && (self.tcomm.is_none() || overwrite)
    {
      if let Some(prev) = self
        .tcomm
        .replace(TComm::new(n, desc.get_content_unwrapped().into()))
      {
        warn!(
          "Col {}. Description '{}' replaced by '{}'.",
          n,
          prev.col_description(),
          desc.get_content_unwrapped()
        );
      }
    }
    if let Some(values) = &field.values {
      // Null value (nut no overwrite!)
      if let Some(null) = &values.null
        && self.tnull.is_none()
      {
        if let Ok(null) = null.parse::<i64>() {
          self.tnull.replace(TNull::new(n, null));
        } else {
          warn!(
            "Col {}. Impossible to set null value '{}': it is not an integer!",
            n, null
          );
        }
      }
      // Min value
      if let Some(min) = &values.min
        && (self.tdmin.is_none() || overwrite)
      {
        // TODO: do something about inclusive/exclusive ?
        if let Some(prev) = self.tdmin.replace(TDMin::new(n, min.value.clone())) {
          warn!(
            "Col {}. Min value '{}' replaced by '{}'.",
            n,
            prev.min_value(),
            min.value
          );
        }
      }
      // Max value
      if let Some(max) = &values.max
        && (self.tdmax.is_none() || overwrite)
      {
        // TODO: do something about inclusive/exclusive ?
        if let Some(prev) = self.tdmax.replace(TDMax::new(n, max.value.clone())) {
          warn!(
            "Col {}. Max value '{}' replaced by '{}'.",
            n,
            prev.max_value(),
            max.value
          );
        }
      }
    }
  }

  pub fn schema(&self) -> Option<Schema> {
    let scale = self.tscal.as_ref().map(|s| s.scale()).unwrap_or(1.0);
    let offset = self
      .tzero
      .as_ref()
      .map(|z| z.zero())
      .unwrap_or(UIF64::F64(0.0));
    self.tform.as_ref().map(|tform| match tform.tform_type() {
      // Logical (bool)
      TFormValue::L(rc) => {
        if scale != 1.0 || !offset.is_0() {
          warn!("TSCAL/TZERO ignored: not supposed to be used with TFORM 'L'.")
        }
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::NullableBoolean,
          len => Schema::NullableBooleanArray(ArrayParam::new(len as usize)),
        }
      }

      // Bit encoded on bytes
      TFormValue::X(rc) => {
        if scale != 1.0 || !offset.is_0() {
          warn!("TSCAL/TZERO ignored: not supposed to be used with TFORM 'X'.")
        }
        match rc.repeat_count() {
          0 => Schema::Empty,
          len => Schema::Bits {
            n_bits: len as usize,
          },
        }
      }

      // Unsigned Byte (u8)
      // -- normal case
      TFormValue::B(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::UnsignedByte,
          Some(null) => Schema::NullableUnsignedByte {
            null: null.col_null_value() as u8,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::UnsignedByteArray(p),
            Some(null) => Schema::NullableUnsignedByteArray {
              null: null.col_null_value() as u8,
              p,
            },
          }
        }
      },
      // -- signed case
      TFormValue::B(rc) if scale == 1.0 && offset.is_i8_offset() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::Byte,
          Some(null) => Schema::NullableByte {
            null: null.col_null_value() as u8,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::ByteArray(p),
            Some(null) => Schema::NullableByteArray {
              null: null.col_null_value() as u8,
              p,
            },
          }
        }
      },
      // -- float using scale/offset
      TFormValue::B(rc) => {
        let transform = ScaleOffset32::new(scale as f32, offset.as_f32());
        if self.tnull.is_some() {
          todo!("TNULL not yet supported in FloatFromByte / FloatArrayFromBytes");
        }
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::FloatFromByte(transform),
          len => Schema::FloatArrayFromBytes(
            ArrayParam::new(len as usize).with_scale_offset_32(transform),
          ),
        }
      }

      // Short integer (i16)
      // -- normal case
      TFormValue::I(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::Short,
          Some(null) => Schema::NullableShort {
            null: null.col_null_value() as i16,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::ShortArray(p),
            Some(null) => Schema::NullableShortArray {
              null: null.col_null_value() as i16,
              p,
            },
          }
        }
      },
      // -- unsigned case
      TFormValue::I(rc) if scale == 1.0 && offset.is_u16_offset() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::UnsignedShort,
          Some(null) => Schema::NullableUnsignedShort {
            null: null.col_null_value() as i16,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::UnsignedShortArray(p),
            Some(null) => Schema::NullableUnsignedShortArray {
              null: null.col_null_value() as i16,
              p,
            },
          }
        }
      },
      // -- float using scale/offset
      TFormValue::I(rc) => {
        let transform = ScaleOffset32::new(scale as f32, offset.as_f32());
        if self.tnull.is_some() {
          todo!("TNULL not yet supported in FloatFromShort / FloatArrayFromShort");
        }
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::FloatFromShort(transform),
          len => Schema::FloatArrayFromShort(
            ArrayParam::new(len as usize).with_scale_offset_32(transform),
          ),
        }
      }
      // Integer (i32)
      // -- normal case
      TFormValue::J(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::Int,
          Some(null) => Schema::NullableInt {
            null: null.col_null_value() as i32,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::IntArray(p),
            Some(null) => Schema::NullableIntArray {
              null: null.col_null_value() as i32,
              p,
            },
          }
        }
      },
      // -- unsigned case
      TFormValue::J(rc) if scale == 1.0 && offset.is_u32_offset() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::UnsignedInt,
          Some(null) => Schema::NullableUnsignedInt {
            null: null.col_null_value() as i32,
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::UnsignedIntArray(p),
            Some(null) => Schema::NullableUnsignedIntArray {
              null: null.col_null_value() as i32,
              p,
            },
          }
        }
      },
      // -- double using scale/offset
      TFormValue::J(rc) => {
        let transform = ScaleOffset64::new(scale, offset.as_f64());
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::DoubleFromInt(transform),
          len => Schema::DoubleArrayFromInt(
            ArrayParam::new(len as usize).with_scale_offset_64(transform),
          ),
        }
      }
      // Long integer (i64) -> should be float80 i computations.
      TFormValue::K(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::Long,
          Some(null) => Schema::NullableLong {
            null: null.col_null_value(),
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::LongArray(p),
            Some(null) => Schema::NullableLongArray {
              null: null.col_null_value(),
              p,
            },
          }
        }
      },
      // -- unsigned case
      TFormValue::K(rc) if scale == 1.0 && offset.is_u64_offset() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => match &self.tnull {
          None => Schema::UnsignedLong,
          Some(null) => Schema::NullableUnsignedLong {
            null: null.col_null_value(),
          },
        },
        len => {
          let p = ArrayParam::new(len as usize);
          match &self.tnull {
            None => Schema::UnsignedLongArray(p),
            Some(null) => Schema::NullableUnsignedLongArray {
              null: null.col_null_value(),
              p,
            },
          }
        }
      },
      // -- double using scale/offset
      TFormValue::K(rc) => {
        let transform = ScaleOffset64::new(scale, offset.as_f64());
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::DoubleFromLong(transform), // Should be a float with 64 bits mantissa...
          len => Schema::DoubleArrayFromLong(
            ArrayParam::new(len as usize).with_scale_offset_64(transform),
          ),
        }
      }
      // Character ASCII (u8)
      TFormValue::A(rc) => {
        if scale != 1.0 || !matches!(offset, UIF64::F64(0.0)) {
          warn!("TSCAL/TZERO ignored: not supposed to be used with TFORM 'A'.")
        }
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::AsciiChar,
          len => Schema::AsciiString(ArrayParam::new(len as usize)),
        }
      }
      // Float (f32)
      TFormValue::E(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => Schema::Float,
        len => Schema::FloatArray(ArrayParam::new(len as usize)),
      },
      TFormValue::E(rc) => {
        let transform = ScaleOffset32::new(scale as f32, offset.as_f32());
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::FloatFromFloat(transform),
          len => Schema::FloatArrayFromFloat(
            ArrayParam::new(len as usize).with_scale_offset_32(transform),
          ),
        }
      }
      // Double (f64)
      TFormValue::D(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => Schema::Double,
        len => Schema::DoubleArray(ArrayParam::new(len as usize)),
      },
      TFormValue::D(rc) => {
        let transform = ScaleOffset64::new(scale, offset.as_f64());
        match rc.repeat_count() {
          0 => Schema::Empty,
          1 => Schema::DoubleFromDouble(transform),
          len => Schema::DoubleArrayFromDouble(
            ArrayParam::new(len as usize).with_scale_offset_64(transform),
          ),
        }
      }

      // Complex f32 (f32, f32)
      TFormValue::C(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => Schema::ComplexFloat,
        len => Schema::ComplexFloatArray(ArrayParam::new(len as usize)),
      },
      TFormValue::C(_rc) => {
        // scale + offset only on the real part? Why not scale on modulus and offset on angle?
        todo!()
      }
      // Complex f64 (f64, f64)
      TFormValue::M(rc) if scale == 1.0 || offset.is_0() => match rc.repeat_count() {
        0 => Schema::Empty,
        1 => Schema::ComplexDouble,
        len => Schema::ComplexDoubleArray(ArrayParam::new(len as usize)),
      },
      TFormValue::M(_rc) => {
        // scale + offset only on the real part? Why not scale on modulus and offset on angle?
        todo!()
      }
      // Array descriptor 32-bit (u32)
      TFormValue::P(zo) => {
        if self.tnull.is_some() {
          todo!("TNULL not yet supported in Array descriptor 32-bit ");
        }
        if zo.is_repeat_count_eq_1() {
          let hap = HeapArrayParam::new(zo.max_len() as usize);
          Schema::HeapArrayPtr32(heap_array_data_type(zo.data_type(), hap, scale, offset))
        } else {
          Schema::Empty
        }
      }
      // Array descriptor 64-bit (u64)
      TFormValue::Q(zo) => {
        if self.tnull.is_some() {
          todo!("TNULL not yet supported in Array descriptor 64-bit ");
        }
        if zo.is_repeat_count_eq_1() {
          let hap = HeapArrayParam::new(zo.max_len() as usize);
          Schema::HeapArrayPtr64(heap_array_data_type(zo.data_type(), hap, scale, offset))
        } else {
          Schema::Empty
        }
      }
    })
  }
}

fn heap_array_data_type(
  vdt: VariableLenghtArrayDataType,
  hap: HeapArrayParam,
  scale: f64,
  offset: UIF64,
) -> HeapArraySchema {
  match vdt {
    VariableLenghtArrayDataType::L => {
      if scale != 1.0 || !matches!(offset, UIF64::F64(0.0)) {
        warn!("TSCAL/TZERO ignored: not supposed to be used with TFORM '[PQ]L'.")
      }
      HeapArraySchema::HeapNullableBooleanArray(hap)
    }
    VariableLenghtArrayDataType::A => {
      if scale != 1.0 || !matches!(offset, UIF64::F64(0.0)) {
        warn!("TSCAL/TZERO ignored: not supposed to be used with TFORM '[PQ]A'.")
      }
      HeapArraySchema::HeapAsciiString(hap)
    }
    VariableLenghtArrayDataType::B if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapUnsignedByteArray(hap)
    }
    VariableLenghtArrayDataType::B if scale == 1.0 && offset.is_i8_offset() => {
      HeapArraySchema::HeapByteArray(hap)
    }
    VariableLenghtArrayDataType::B => HeapArraySchema::HeapFloatArrayFromByte(
      hap.with_scale_offset_32(ScaleOffset32::new(scale as f32, offset.as_f32())),
    ),
    VariableLenghtArrayDataType::I if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapShortArray(hap)
    }
    VariableLenghtArrayDataType::I if scale == 1.0 && offset.is_u16_offset() => {
      HeapArraySchema::HeapUnsignedShortArray(hap)
    }
    VariableLenghtArrayDataType::I => HeapArraySchema::HeapFloatArrayFromShort(
      hap.with_scale_offset_32(ScaleOffset32::new(scale as f32, offset.as_f32())),
    ),
    VariableLenghtArrayDataType::J if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapIntArray(hap)
    }
    VariableLenghtArrayDataType::J if scale == 1.0 && offset.is_u32_offset() => {
      HeapArraySchema::HeapUnsignedIntArray(hap)
    }
    VariableLenghtArrayDataType::J => HeapArraySchema::HeapDoubleArrayFromInt(
      hap.with_scale_offset_64(ScaleOffset64::new(scale, offset.as_f64())),
    ),
    VariableLenghtArrayDataType::K if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapLongArray(hap)
    }
    VariableLenghtArrayDataType::K if scale == 1.0 && offset.is_u64_offset() => {
      HeapArraySchema::HeapUnsignedLongArray(hap)
    }
    VariableLenghtArrayDataType::K => HeapArraySchema::HeapDoubleArrayFromLong(
      hap.with_scale_offset_64(ScaleOffset64::new(scale, offset.as_f64())),
    ),
    VariableLenghtArrayDataType::E if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapFloatArray(hap)
    }
    VariableLenghtArrayDataType::E => HeapArraySchema::HeapFloatArrayFromFloat(
      hap.with_scale_offset_32(ScaleOffset32::new(scale as f32, offset.as_f32())),
    ),
    VariableLenghtArrayDataType::D if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapDoubleArray(hap)
    }
    VariableLenghtArrayDataType::D => HeapArraySchema::HeapDoubleArrayFromDouble(
      hap.with_scale_offset_64(ScaleOffset64::new(scale, offset.as_f64())),
    ),
    VariableLenghtArrayDataType::C if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapComplexFloatArray(hap)
    }
    VariableLenghtArrayDataType::C => {
      // scale + offset only on the real part? Why not scale on modulus and offset on angle?
      todo!()
    }
    VariableLenghtArrayDataType::M if scale == 1.0 && offset.is_0() => {
      HeapArraySchema::HeapComplexDoubleArray(hap)
    }
    VariableLenghtArrayDataType::M => {
      // scale + offset only on the real part? Why not scale on modulus and offset on angle?
      todo!()
    }
  }
}

// Oher table keywords:
// * look at compression.
// * ...

pub struct BinTableHeader {
  // xtension
  // bitpix
  // naxis
  /// Row byte size
  naxis1: NAxis1,
  /// Number of rows in the table
  naxis2: NAxis2,
  /// Size of the heap (for variable width columns), including the (optional) gap
  pcount: PCount,
  // gcount
  /// Number of columns in the table
  tfield: TFields,
}

impl BinTableHeader {
  /// # Params
  /// * `naxis1`: byte size of a row
  /// * `naxis2`: number of rows in the table
  /// * `pcount`: size of the variable size data (heap), if any
  /// * `tfield`: number of columns in the table
  pub fn new(naxis1: u32, naxis2: u64, pcount: usize, tfield: u16) -> Self {
    Self {
      naxis1: NAxis1::new(naxis1),
      naxis2: NAxis2::new(naxis2),
      pcount: PCount::new(pcount),
      tfield: TFields::new(tfield),
    }
  }

  /// Number of leading mandatory keyword records (from `XTENSION`, inclusive, to `TFIELD`, inclusive).
  pub fn n_kw_records(&self) -> usize {
    8
  }

  pub fn n_cols(&self) -> usize {
    self.tfield.get() as usize
  }

  pub fn n_rows(&self) -> usize {
    self.naxis2.get() as usize
  }

  pub fn row_byte_size(&self) -> usize {
    self.naxis1.get() as usize
  }

  pub fn main_table_byte_size(&self) -> usize {
    self.n_rows() * self.row_byte_size()
  }

  /// Size of the heap, including the size of the gap (if any).
  pub fn heap_byte_size(&self) -> usize {
    self.pcount.get()
  }
}

impl Header for BinTableHeader {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    // We assume XTENSION has already been parsed (needed to decide to enter here!).
    assert_eq!(hdu_type, HDUType::Extension(Xtension::BinTable));
    BITPIX.check_keyword_record_it(kw_records_it)?;
    NAXIS.check_keyword_record_it(kw_records_it)?;
    let naxis1 = NAxis1::from_keyword_record_it(kw_records_it)?;
    let naxis2 = NAxis2::from_keyword_record_it(kw_records_it)?;
    let pcount = PCount::from_keyword_record_it(kw_records_it)?;
    GCOUNT.check_keyword_record_it(kw_records_it)?;
    let tfield = TFields::from_keyword_record_it(kw_records_it)?;
    Ok(Self {
      naxis1,
      naxis2,
      pcount,
      tfield,
    })
  }

  /// Size fo the data part, including the main table and the heap.
  /// # Warning
  /// The value is **not necessarily a multiple of 2880**!
  fn data_byte_size(&self) -> u64 {
    // The BITPIX byte size equals 1, so it is useless to multiply by BITPIX.byte_size() here
    // BITPIX.byte_size() * (...)
    // (self.naxis1.get() as u64) * self.naxis2.get() + self.pcount.byte_size() as u64
    self.main_table_byte_size() as u64 + self.heap_byte_size() as u64
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    XTENSION
      .write_kw_record(dest)
      .and_then(|()| BITPIX.write_kw_record(dest))
      .and_then(|()| NAXIS.write_kw_record(dest))
      .and_then(|()| self.naxis1.write_kw_record(dest))
      .and_then(|()| self.naxis2.write_kw_record(dest))
      .and_then(|()| self.pcount.write_kw_record(dest))
      .and_then(|()| GCOUNT.write_kw_record(dest))
      .and_then(|()| self.tfield.write_kw_record(dest))
  }
}

/// A header storing information to access tables data: rows, columns, fields content, ...
pub struct BinTableHeaderWithColInfo {
  /// Minimal Required Header to be able to skip data
  mrh: BinTableHeader,
  /// Index of the first starting byte of the HEAP, from the starting data byte.
  /// **Cannot** be lower than `naxis1 * naxis2`.
  theap: Option<THeap>,
  /// Columns metadata
  cols: Vec<BinTableColumnHeader>,
}
impl BinTableHeaderWithColInfo {
  fn check_n(&self, n: u16) -> Result<(), Error> {
    if n as usize > self.cols.len() {
      Err(new_custom(format!(
        "Out of bound column number. Expected: max {}. Actual: {}.",
        self.cols.len(),
        n
      )))
    } else {
      Ok(())
    }
  }

  /// Number of leading mandatory keyword records (from `XTENSION`, inclusive, to `TFIELD`, inclusive).
  pub fn n_kw_records(&self) -> usize {
    self.mrh.n_kw_records()
  }

  pub fn n_cols(&self) -> usize {
    self.mrh.n_cols()
  }

  pub fn n_rows(&self) -> usize {
    self.mrh.n_rows()
  }

  pub fn row_byte_size(&self) -> usize {
    self.mrh.row_byte_size()
  }

  pub fn main_table_byte_size(&self) -> usize {
    self.mrh.main_table_byte_size()
  }

  /// Size of the heap, ncluding the size of the gap (if any).
  pub fn heap_byte_size(&self) -> usize {
    self.mrh.heap_byte_size()
  }

  pub fn table(&self) -> &BinTableHeader {
    &self.mrh
  }

  /// Size of the gap between the end of the main table and the heap, in bytes.
  pub fn gap_byte_size(&self) -> usize {
    self
      .theap
      .as_ref()
      .map(|theap| {
        let byte_offset = theap.byte_offset();
        let main_table_byte_size = self.mrh.main_table_byte_size();
        if byte_offset < main_table_byte_size {
          warn!("Heap offset (THEAP) value {} is larger than the main table size {}. THEAP value ignored!", byte_offset, main_table_byte_size);
          0
        } else {
          byte_offset - main_table_byte_size
        }
      })
      .unwrap_or(0)
  }

  pub fn cols(&self) -> &[BinTableColumnHeader] {
    self.cols.as_slice()
  }

  pub fn cols_mut(&mut self) -> &mut [BinTableColumnHeader] {
    self.cols.as_mut_slice()
  }

  pub fn build_row_schema(&self) -> RowSchema {
    self
      .cols()
      .iter()
      .enumerate()
      .map(|(i, col_header)| {
        col_header.schema().expect(&format!(
          "Unable to create schema for column {}: TFORM probably missing!",
          i + 1
        ))
      })
      .collect()
  }
}

impl Header for BinTableHeaderWithColInfo {
  fn from_starting_mandatory_kw_records<'a, I>(
    hdu_type: HDUType,
    kw_records_it: &mut I,
  ) -> Result<Self, Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    BinTableHeader::from_starting_mandatory_kw_records(hdu_type, kw_records_it).map(|v| v.into())
  }

  fn data_byte_size(&self) -> u64 {
    self.mrh.data_byte_size()
  }

  fn write_starting_mandatory_kw_records<'a, I>(&self, dest: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = Result<&'a mut [u8; 80], Error>>,
  {
    self.mrh.write_starting_mandatory_kw_records(dest)
  }

  fn consume_remaining_kw_records<'a, I>(&mut self, kw_records_it: &mut I) -> Result<(), Error>
  where
    I: Iterator<Item = (usize, &'a [u8; 80])>,
  {
    fn get_n(bytes: &[u8]) -> Option<u16> {
      unsafe { str::from_utf8_unchecked(bytes) }
        .trim()
        .parse::<u16>()
        .ok()
    }

    for (_, kwr) in kw_records_it {
      let (kw, ind, kw_value_comment) = FixedFormatRead::split_kw_indicator_value(kwr);
      // Skip keyword if it does not contain a value indicator
      if !is_value_indicator(ind) {
        continue;
      }
      // Analyse keyword
      match kw {
        [b'T', b'H', b'E', b'A', b'P', b' ', b' ', b' '] => {
          THeap::from_value_comment(kw_value_comment).map(|kwo| self.theap.replace(kwo))?;
        }
        [b'T', b'T', b'Y', b'P', b'E', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            // 'kwo' stands for keyword object
            self
              .check_n(n)
              .and_then(|()| TType::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].ttype.replace(kwo))?;
          }
        }
        [b'T', b'F', b'O', b'R', b'M', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TFormn::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tform.replace(kwo))?;
          }
        }
        [b'T', b'D', b'I', b'S', b'P', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TDispn::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tdisp.replace(kwo))?;
          }
        }
        [b'T', b'U', b'C', b'D', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TUCD::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tucd.replace(kwo))?;
          }
        }
        [b'T', b'U', b'N', b'I', b'T', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TUnit::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tunit.replace(kwo))?;
          }
        }
        [b'T', b'C', b'O', b'M', b'M', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TComm::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tcomm.replace(kwo))?;
          }
        }
        [b'T', b'N', b'U', b'L', b'L', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TNull::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tnull.replace(kwo))?;
          }
        }
        [b'T', b'S', b'C', b'A', b'L', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TScal::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tscal.replace(kwo))?;
          }
        }
        [b'T', b'Z', b'E', b'R', b'O', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TZero::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tzero.replace(kwo))?;
          }
        }
        [b'T', b'D', b'I', b'M', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TDim::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tdim.replace(kwo))?;
          }
        }
        [b'T', b'D', b'M', b'I', b'N', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TDMin::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tdmin.replace(kwo))?;
          }
        }
        [b'T', b'D', b'M', b'A', b'X', nbr @ ..] => {
          if let Some(n) = get_n(nbr) {
            self
              .check_n(n)
              .and_then(|()| TDMax::from_value_comment(n, kw_value_comment))
              .map(|kwo| self.cols[(n - 1) as usize].tdmax.replace(kwo))?;
          }
        }
        _ => {}
      }
    }
    Ok(())
  }
}

impl From<BinTableHeader> for BinTableHeaderWithColInfo {
  fn from(mrh: BinTableHeader) -> Self {
    let cols = (0..mrh.n_cols())
      .into_iter()
      .map(|_| BinTableColumnHeader::default())
      .collect();
    Self {
      mrh,
      theap: None,
      cols,
    }
  }
}
