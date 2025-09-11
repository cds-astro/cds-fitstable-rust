//! `ASCIITABLE` and/or `BINTABLE` specific keywords.

/*
TBCOLn (mandatory) = starting col, i.e. byte index (from 1)
TFORMn (mandatory)
TTYPEn (recommanded)
TSCALn (optional)
TZEROn (optional)
TNULLn (optional)
TDISPn (optional)
*/

/*
TFORMn (mandatory),
TTYPEn (recommanded),
TSCALn
TZEROn
TNULLn
TDISPn

// For multidim arrays:
THEAP  /  Byte offset of the heap area
TDIMn
 */

/* COMMON
TUNITn (optional)

 */

pub mod asciitable;
pub mod bintable;
pub mod tdminmax;
pub mod tfields;
pub mod tnull;
pub mod tscaltzero;
pub mod ttype;
pub mod tunit;
